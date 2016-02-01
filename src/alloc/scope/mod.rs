use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::mem;
use std::rc::Rc;

use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::js_type::{JsPtrEnum, JsType, JsVar, Binding};

// Tunable GC parameter. Probably should not be a constant, but good enough for now.
const GC_THRESHOLD: usize = 64;


pub struct Scope {
    pub parent: Option<Box<Scope>>,
    alloc_box: Rc<RefCell<AllocBox>>,
    stack: HashMap<Binding, JsVar>,
    pub get_roots: Box<Fn() -> HashSet<Binding>>,
}

impl Scope {
    pub fn new<F>(alloc_box: &Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Binding> + 'static {
        Scope {
            parent: None,
            alloc_box: alloc_box.clone(),
            stack: HashMap::new(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn as_child<F>(parent: Scope, alloc_box: &Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Binding> + 'static {
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

    fn alloc(&mut self, binding: Binding, ptr: JsPtrEnum) -> Result<()> {
        self.alloc_box.borrow_mut().alloc(binding, ptr)
    }

    pub fn push(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        let res = match &var.t {
            &JsType::JsPtr =>
                if let Some(ptr) = ptr {
                    self.alloc(var.binding.clone(), ptr)
                } else {
                    return Err(GcError::PtrError);
                },
            _ => Ok(()),
        };
        self.stack.insert(var.binding.clone(), var);
        res
    }

    pub fn own(&mut self, var: JsVar) {
        self.stack.insert(var.binding.clone(), var);
    }

    pub fn get_var_copy(&self, bnd: &Binding) -> (Option<JsVar>, Option<JsPtrEnum>) {
        if let Some(var) = self.stack.get(bnd) {
            match var.t {
                JsType::JsPtr => {
                    if let Some(alloc) = self.alloc_box.borrow().find_id(bnd) {
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

    pub fn update_var(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        match var.t {
            JsType::JsPtr =>
                if let Some(ptr) = ptr {
                    self.alloc_box.borrow_mut().update_ptr(&var.binding, ptr)
                } else {
                    Err(GcError::PtrError)
                },
            _ => {
                if let Entry::Occupied(mut view) = self.stack.entry(var.binding.clone()) {
                    *view.get_mut() = var;
                    Ok(())
                } else {
                    Err(GcError::StoreError)
                }
            },
        }
    }

    pub fn transfer_stack(&mut self) -> Option<Box<Scope>> {
        if self.alloc_box.borrow().len() > GC_THRESHOLD {
            self.alloc_box.borrow_mut().mark_roots((self.get_roots)());
            self.alloc_box.borrow_mut().mark_ptrs();
            self.alloc_box.borrow_mut().sweep_ptrs();
        }
        if let Some(ref mut parent) = self.parent {
            for (_, var) in self.stack.drain() {
                match var.t {
                    JsType::JsPtr => {
                        // Mangle each binding before giving it to the parent
                        // scope. This avoids binding collisions, and helps
                        // identify to a human observer which bindings are
                        // not from the current scope.
                        let mut mangled_var = var.clone();
                        mangled_var.binding = Binding::mangle(var.binding);
                        parent.own(mangled_var);
                    }
                    _ => (),
                }
            }
        }
        mem::replace(&mut self.parent, None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    use js_types::js_type::{JsVar, JsType, JsPtrEnum, JsKey, JsKeyEnum};
    use js_types::js_str::JsStrStruct;
    use utils;

    #[test]
    fn test_new_scope() {
        let alloc_box = utils::make_alloc_box();
        let test_scope = Scope::new(&alloc_box, utils::dummy_callback);
        assert!(test_scope.parent.is_none());
    }

    #[test]
    fn test_as_child_scope() {
        let alloc_box = utils::make_alloc_box();
        let parent_scope = Scope::new(&alloc_box, utils::dummy_callback);
        let test_scope = Scope::as_child(parent_scope, &alloc_box, utils::dummy_callback);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_set_parent() {
        let alloc_box = utils::make_alloc_box();
        let parent_scope = Scope::new(&alloc_box, utils::dummy_callback);
        let mut test_scope = Scope::new(&alloc_box, utils::dummy_callback);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(parent_scope);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_alloc() {
        let alloc_box = utils::make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, utils::dummy_callback);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.alloc(test_var.uuid, JsPtrEnum::JsSym(String::from("test"))).unwrap();
        assert_eq!(test_id, test_var.uuid);
    }

    #[test]
    fn test_get_var_copy() {
        let alloc_box = utils::make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, utils::dummy_callback);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.push(test_var, Some(JsPtrEnum::JsSym(String::from("test")))).unwrap();
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
        let alloc_box = utils::make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, utils::dummy_callback);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.push(test_var, Some(JsPtrEnum::JsSym(String::from("test")))).unwrap();
        let (update, _) = test_scope.get_var_copy(&test_id);
        let update_ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new("test")));
        assert!(test_scope.update_var(update.unwrap(), update_ptr).is_ok());

        let (_, update_ptr) = test_scope.get_var_copy(&test_id);
        match update_ptr.clone().unwrap() {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => ()
        }
    }

    #[test]
    fn test_transfer_stack() {
        let alloc_box = utils::make_alloc_box();
        let mut parent_scope = Scope::new(&alloc_box, utils::dummy_callback);
        {
            let mut test_scope = Scope::as_child(parent_scope, &alloc_box, utils::dummy_callback);
            test_scope.push(utils::make_num(0.), None).unwrap();
            test_scope.push(utils::make_num(1.), None).unwrap();
            test_scope.push(utils::make_num(2.), None).unwrap();
            let kvs = vec![(JsKey::new(JsKeyEnum::JsBool(true)),
                            utils::make_num(1.))];
            let (var, ptr) = utils::make_obj(kvs);
            test_scope.push(var, Some(ptr)).unwrap();
            parent_scope = *test_scope.transfer_stack().unwrap();
        }
        assert_eq!(parent_scope.stack.len(), 1);
    }
}
