use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::{Rc, Weak};
use std::cmp;
use std::mem;
use uuid::Uuid;

use alloc::{Alloc, AllocBox};
use js_types::js_type::{JsVar, JsType, JsPtrEnum};

pub struct Scope {
    pub parent: Option<Rc<Scope>>,
    alloc_box: Rc<RefCell<AllocBox>>,
    get_roots: Box<Fn() -> HashSet<Uuid>>,
}

impl Scope {
    pub fn new<F>(alloc_box: &Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: None,
            alloc_box: alloc_box.clone(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn as_child<F>(parent: &Rc<Scope>, alloc_box: &Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: Some(parent.clone()),
            alloc_box: alloc_box.clone(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn set_parent(&mut self, parent: &Rc<Scope>) {
        self.parent = Some(parent.clone());
    }

    pub fn alloc(&mut self, uuid: Uuid, ptr: JsPtrEnum) -> Uuid {
        self.alloc_box.borrow_mut().alloc(uuid, ptr)
    }

    pub fn dealloc(&mut self, uuid: &Uuid) -> bool {
        self.alloc_box.borrow_mut().dealloc(uuid)
    }

    pub fn get_ptr_copy(&self, uuid: &Uuid) -> Option<JsPtrEnum> {
        self.alloc_box.borrow().find_id(uuid).map(|var| var.borrow().clone())
    }

    pub fn update_ptr(&mut self, uuid: &Uuid, ptr: JsPtrEnum) -> bool {
        self.alloc_box.borrow_mut().update_var(uuid, ptr)
    }


    /// TODO Compute the roots of the current scope-- any variable that is
    /// directly referenced or declared within the scope. This might just be the
    /// key set of the uuid map(?) Not necessarily, I think. What if you do
    /// something like this:
    /// var x = {}
    /// var y = { 1: x }
    /// y = x
    /// y would be a root by the definition above, but is no longer reachable at
    /// the end of the scope because it now aliases x. A better definition would
    /// be "Any variable that is declared or referenced directly, but a direct
    /// reference (variable usage) supercedes a declaration." The above example
    /// demonstrates why this is necessary.
    /// This should come from the interpreter, so I shouldn't actually have to
    /// care about getting the root set myself.

    /// Roots always get marked as Black, since they're always reachable from
    /// the current scope. NB that this assumes all root references are actually
    /// valid reference types, i.e. they're not numbers, etc.
    pub fn mark_roots(&mut self) {
        let marks = (self.get_roots)();
        self.alloc_box.borrow_mut().mark_roots(marks);
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
        let parent_scope = Rc::new(Scope::new(&alloc_box, dummy_get_roots));
        let mut test_scope = Scope::as_child(&parent_scope, &alloc_box, dummy_get_roots);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_set_parent() {
        let alloc_box = make_alloc_box();
        let parent_scope = Rc::new(Scope::new(&alloc_box, dummy_get_roots));
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(&parent_scope);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_alloc() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.alloc(test_var.uuid, JsPtrEnum::JsNull);
        assert_eq!(test_id, test_var.uuid);
    }

    #[test]
    fn test_dealloc() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.alloc(test_var.uuid, JsPtrEnum::JsNull);
        let bad_uuid = Uuid::new_v4();
        assert!(test_scope.dealloc(&test_id));
        assert!(!test_scope.dealloc(&bad_uuid));
    }

    #[test]
    fn test_get_ptr_copy() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.alloc(test_var.uuid, JsPtrEnum::JsNull);
        let bad_uuid = Uuid::new_v4();

        let ptr_copy = test_scope.get_ptr_copy(&test_id);
        assert!(ptr_copy.is_some());

        // FIXME it's kind of problematic that UUIDs are separated
        // from ptr types
        //let ptr = ptr_copy.unwrap();
        //assert_eq!(ptr.uuid, test_id);

        let bad_copy = test_scope.get_ptr_copy(&bad_uuid);
        assert!(bad_copy.is_none());
    }

    #[test]
    fn test_update_ptr() {
        let alloc_box = make_alloc_box();
        let mut test_scope = Scope::new(&alloc_box, dummy_get_roots);
        let test_var = JsVar::new(JsType::JsPtr);
        let test_id = test_scope.alloc(test_var.uuid, JsPtrEnum::JsNull);
        let mut update = test_scope.get_ptr_copy(&test_id).unwrap();
        update = JsPtrEnum::JsStr(JsStrStruct::new("test"));
        assert!(test_scope.update_ptr(&test_id, update));

        let update = test_scope.get_ptr_copy(&test_id).unwrap();
        match update {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => ()
        }
        test_scope.dealloc(&test_id);
        assert!(!test_scope.update_ptr(&test_id, update));
    }
}
