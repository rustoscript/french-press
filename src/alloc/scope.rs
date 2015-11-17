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
    pub fn new<F>(alloc_box: Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: None,
            alloc_box: alloc_box,
            get_roots: Box::new(get_roots),
        }
    }

    pub fn as_child<F>(parent: Rc<Scope>, alloc_box: Rc<RefCell<AllocBox>>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: Some(parent),
            alloc_box: alloc_box,
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

    pub fn get_var_copy(&self, uuid: &Uuid) -> Option<JsPtrEnum> {
        self.alloc_box.borrow().find_id(uuid).map(|var| var.borrow().clone())
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
    use std::collections::hash_set::HashSet;
    use std::rc::{Rc, Weak};
    use js_types::js_type::{JsVar, JsType, JsPtrEnum, JsKey, JsKeyEnum};
    use js_types::js_obj::{JsObjStruct};
    use uuid::Uuid;

    fn dummy_get_roots() -> HashSet<Uuid> {
        HashSet::new()
    }

    fn make_num(i: f64) -> JsVar {
        JsVar::new(JsType::JsNum(i))
    }

    fn make_obj(kvs: Vec<(JsKey, Alloc<JsVar>)>) -> JsVar {
        JsVar::new(JsType::JsPtr(JsPtrEnum::JsObj(JsObjStruct::new(None, "test", kvs))))
    }

    #[test]
    fn test_new_scope() {
        let mut test_scope = Scope::new(dummy_get_roots);
        assert!(test_scope.parent.is_none());
        assert!(test_scope.black_set.is_empty());
        assert!(test_scope.grey_set.is_empty());
        assert!(test_scope.white_set.is_empty());
        assert_eq!(test_scope.children.len(), 0);
    }

    #[test]
    fn test_as_child_scope() {
        let parent_scope = Rc::new(Scope::new(dummy_get_roots));
        let mut test_scope = Scope::as_child(Rc::downgrade(&parent_scope.clone()), dummy_get_roots);

        assert!(test_scope.parent.is_some());
        assert!(test_scope.black_set.is_empty());
        assert!(test_scope.grey_set.is_empty());
        assert!(test_scope.white_set.is_empty());
        assert_eq!(test_scope.children.len(), 0);
    }

    #[test]
    fn test_set_parent() {
        let parent_scope = Rc::new(Scope::new(dummy_get_roots));
        let mut test_scope = Scope::new(dummy_get_roots);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(&parent_scope);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_add_child() {
        let mut test_scope = Scope::new(dummy_get_roots);
        let child_scope1 = Scope::new(dummy_get_roots);
        let child_scope2 = Scope::new(dummy_get_roots);
        assert_eq!(test_scope.children.len(), 0);
        test_scope.add_child(child_scope1);
        assert_eq!(test_scope.children.len(), 1);
        test_scope.add_child(child_scope2);
        assert_eq!(test_scope.children.len(), 2);
    }

    #[test]
    fn test_alloc() {
        let mut test_scope = Scope::new(dummy_get_roots);
        let test_var = make_num(1.0);
        let test_uuid = test_var.uuid.clone();
        let uuid = test_scope.alloc(test_var);
        assert_eq!(test_uuid, uuid);
        assert!(test_scope.white_set.contains_key(&uuid));
        assert_eq!(test_scope.white_set.len(), 1);
        assert_eq!(test_scope.grey_set.len(), 0);
        assert_eq!(test_scope.black_set.len(), 0);
        let test_var2 = make_num(2.0);
        let uuid = test_scope.alloc(test_var2);
        assert!(test_scope.white_set.contains_key(&uuid));
        assert_eq!(test_scope.white_set.len(), 2);
        assert_eq!(test_scope.grey_set.len(), 0);
        assert_eq!(test_scope.black_set.len(), 0);
    }

    #[test]
    fn test_dealloc() {
        let mut test_scope = Scope::new(dummy_get_roots);
        let test_var = make_num(1.0);
        let uuid = test_scope.alloc(test_var);
        let bad_uuid = Uuid::new_v4();
        assert!(test_scope.dealloc(&uuid));
        assert_eq!(test_scope.white_set.len(), 0);
        assert_eq!(test_scope.grey_set.len(), 0);
        assert_eq!(test_scope.black_set.len(), 0);
        assert!(!test_scope.dealloc(&bad_uuid));
    }

    #[test]
    fn test_get_var_copy() {
        let mut test_scope = Scope::new(dummy_get_roots);
        let test_var = make_num(1.0);
        let uuid = test_scope.alloc(test_var);
        let bad_uuid = Uuid::new_v4();
        let var_copy = test_scope.get_var_copy(&uuid);
        assert!(var_copy.is_some());
        let var = var_copy.unwrap();
        assert_eq!(var.uuid, uuid);
        let bad_copy = test_scope.get_var_copy(&bad_uuid);
        assert!(bad_copy.is_none());
    }

    #[test]
    fn test_update_var() {
        let mut test_scope = Scope::new(dummy_get_roots);
        let test_var = make_num(1.0);
        let uuid = test_scope.alloc(test_var);
        let mut update = test_scope.get_var_copy(&uuid).unwrap();
        update = make_num(2.0);
        update.uuid = uuid;
        assert!(test_scope.update_var(update));
        let update = test_scope.get_var_copy(&uuid).unwrap();
        match update {
            JsVar{ t: JsType::JsNum(i), ..} => assert_eq!(i, 2.0),
            _ => ()
        }
        test_scope.dealloc(&uuid);
        assert!(!test_scope.update_var(update));
    }

    #[test]
    fn test_mark_roots() {
        let test_uuid = Uuid::new_v4();
        let mut test_var = make_num(1.0);
        let mut test_var2 = make_num(2.0);
        test_var.uuid = test_uuid;
        let simple_get_roots = move || { let mut set = HashSet::new();
                                    set.insert(test_uuid);
                                    set };
        let mut test_scope = Scope::new(simple_get_roots);
        let test_uuid = test_scope.alloc(test_var);
        let test_uuid2 = test_scope.alloc(test_var2);
        test_scope.mark_roots();
        assert_eq!(test_scope.black_set.len(), 1);
        assert!(test_scope.black_set.get(&test_uuid).is_some());
        assert!(test_scope.black_set.get(&test_uuid2).is_none());
    }

    #[test]
    fn test_mark_phase() {
        let test_uuid = Uuid::new_v4();
        let simple_get_roots = move || { let mut set = HashSet::new();
                                    set.insert(test_uuid);
                                    set };
        let mut test_scope = Scope::new(simple_get_roots);

        let mut test_var2 = make_num(2.0);
        let test_uuid2 = test_scope.alloc(test_var2);
        let test_var2_alloc = test_scope.find_id(&test_uuid2).unwrap().clone();

        let test_key = JsKey::new(JsKeyEnum::JsNum(1.0));
        let mut test_var = make_obj(vec![(test_key, test_var2_alloc)]);
        test_var.uuid = test_uuid;
        let test_uuid = test_scope.alloc(test_var);

        // Mark roots first...
        test_scope.mark_roots();
        assert!(test_scope.black_set.get(&test_uuid).is_some());
        assert_eq!(test_scope.black_set.len(), 1);
        assert_eq!(test_scope.grey_set.len(), 1);

        // ...then mark everything else
        test_scope.mark_phase();

        assert_eq!(test_scope.grey_set.len(), 0);
        assert!(test_scope.black_set.get(&test_uuid).is_some());
        assert!(test_scope.black_set.get(&test_uuid2).is_some());
    }
}
