use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use uuid::Uuid;

use alloc::AllocBox;
use js_types::js_type::{JsPtrEnum, JsType, JsVar};

// Tunable GC parameter. Probably should not be a constant, but good enough for now.
const GC_THRESHOLD: usize = 64;

pub struct Scope {
    pub parent: Option<Box<Scope>>,
    alloc_box: Rc<RefCell<AllocBox>>,
    stack: HashMap<Uuid, JsVar>,
    pub get_roots: Box<Fn() -> HashSet<Uuid>>,
}

impl Scope {
    pub fn new<F>(alloc_box: &Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: None,
            alloc_box: alloc_box.clone(),
            stack: HashMap::new(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn as_child<F>(parent: Scope, alloc_box: &Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: Some(Box::new(parent)),
            alloc_box: alloc_box.clone(),
            stack: HashMap::new(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn set_parent(&mut self, parent: Scope) {
        self.parent = Some(Box::new(parent));
    }

    fn alloc(&mut self, uuid: Uuid, ptr: JsPtrEnum) -> Uuid {
        self.alloc_box.borrow_mut().alloc(uuid, ptr)
    }

    pub fn push(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Uuid {
        let uuid = match &var.t {
            &JsType::JsPtr => self.alloc(var.uuid, ptr.unwrap()),
                //.expect("ERR: Attempted allocation of heap pointer, but pointer contents were invalid!"));
            _ => var.uuid,
        };
        self.stack.insert(var.uuid, var);
        uuid
    }

    pub fn own(&mut self, var: JsVar) {
        self.stack.insert(var.uuid, var);
    }

    pub fn get_var_copy(&self, uuid: &Uuid) -> (Option<JsVar>, Option<JsPtrEnum>) {
        if let Some(var) = self.stack.get(uuid) {
            match var.t {
                JsType::JsPtr => {
                    if let Some(alloc) = self.alloc_box.borrow().find_id(uuid) {
                        (Some(var.clone()), Some(alloc.borrow().clone()))
                    } else {
                        // This case should be impossible unless you have an
                        // invalid ptr, which should also be impossible.
                        (None, None)
                    }
                },
                _ => (Some(var.clone()), None),
            }
        } else { (None, None) }
    }

    // TODO is there a better way to encode ptr than as an option that is only
    // ever used when it is `Some`? Default argument?
    pub fn update_var(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> bool {
        match var.t {
            JsType::JsPtr => self.alloc_box.borrow_mut()
                                           .update_ptr(&var.uuid,
                                                       ptr.expect("Pointer was None in Scope::update_var!")),
            _ => {
                if let Entry::Occupied(mut view) = self.stack.entry(var.uuid) {
                    *view.get_mut() = var;
                    true
                } else { false }
            },
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.alloc_box.borrow().len() > GC_THRESHOLD {
            self.alloc_box.borrow_mut().mark_roots((self.get_roots)());
            self.alloc_box.borrow_mut().mark_ptrs();
            self.alloc_box.borrow_mut().sweep_ptrs();
        }
        if let Some(ref mut parent) = self.parent {
            for (_, var) in self.stack.drain() {
                match var.t {
                    JsType::JsPtr => parent.own(var),
                    _ => (),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::hash_set::HashSet;
    use std::rc::{Rc, Weak};
    use uuid::Uuid;

    use alloc::{Alloc, AllocBox};
    use js_types::js_type::{JsVar, JsType, JsPtrEnum, JsKey, JsKeyEnum};
    use js_types::js_obj::JsObjStruct;
    use js_types::js_str::JsStrStruct;

    fn dummy_get_roots() -> HashSet<Uuid> {
        HashSet::new()
    }

    fn make_alloc_box() -> Rc<RefCell<AllocBox>> {
        Rc::new(RefCell::new(AllocBox::new()))
    }

    fn make_num(i: f64) -> JsVar {
        JsVar::new(JsType::JsNum(i))
    }

    fn make_obj(alloc_box: Rc<RefCell<AllocBox>>, kvs: Vec<(JsKey, JsVar)>) -> JsVar {
        let var = JsVar::new(JsType::JsPtr);
        alloc_box.borrow_mut()
            .alloc(var.uuid, JsPtrEnum::JsObj(JsObjStruct::new(None, "test", kvs)));
        var
    }

    #[test]
    fn test_new_scope() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        assert!(test_scope.parent.is_none());
    }

    #[test]
    fn test_as_child_scope() {
        let alloc_box = make_alloc_box();
        let parent_scope = Scope::new(&alloc_box, dummy_get_roots);
        let mut test_scope = Scope::as_child(parent_scope, &alloc_box, dummy_get_roots);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_set_parent() {
        let alloc_box = make_alloc_box();
        let parent_scope = Scope::new(&alloc_box, dummy_get_roots);
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(parent_scope);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_alloc() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.alloc(test_var.uuid, JsPtrEnum::JsSym(String::from("test")));
        assert_eq!(test_id, test_var.uuid);
    }

    #[test]
    fn test_get_var_copy() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.push(test_var, Some(JsPtrEnum::JsSym(String::from("test"))));
        let bad_uuid = Uuid::new_v4();

        let (var_copy, ptr_copy) = test_scope.get_var_copy(&test_id);
        assert!(var_copy.is_some());
        assert!(ptr_copy.is_some());

        let (bad_copy, ptr_copy) = test_scope.get_var_copy(&bad_uuid);
        assert!(bad_copy.is_none());
        assert!(ptr_copy.is_none());
    }

    #[test]
    fn test_update_var() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.push(test_var, Some(JsPtrEnum::JsSym(String::from("test"))));
        let (update, mut update_ptr) = test_scope.get_var_copy(&test_id);
        update_ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new("test")));
        assert!(test_scope.update_var(update.unwrap(), update_ptr));

        let (update, update_ptr) = test_scope.get_var_copy(&test_id);
        match update_ptr.clone().unwrap() {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => ()
        }
    }
}
