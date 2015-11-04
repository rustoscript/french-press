use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::Rc;
use std::cmp;
use std::mem;

use js_types::js_type::{JsVar, JsType, JsPtrEnum};
use uuid::Uuid;

// Initial Arena size in bytes
const INITIAL_SIZE: usize = 1024;
// Minimum Arena capacity is at least 1 byte
const MIN_CAP: usize = 1;

pub type Alloc<T> = Rc<RefCell<T>>;

pub struct Scope {
    parent: Option<Rc<Scope>>,
    children: Vec<Box<Scope>>,
    black_set: HashMap<Uuid, Alloc<JsVar>>,
    grey_set: HashMap<Uuid, Alloc<JsVar>>,
    white_set: HashMap<Uuid, Alloc<JsVar>>,
    get_roots: Box<Fn() -> HashSet<Uuid>>,
}

impl Scope {
    pub fn new<F>(get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: None,
            children: Vec::new(),
            black_set: HashMap::new(),
            grey_set: HashMap::new(),
            white_set: HashMap::new(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn as_child<F>(parent: Rc<Scope>, get_roots: F) -> Scope
        where F: Fn() -> HashSet<Uuid> + 'static {
        Scope {
            parent: Some(parent),
            children: Vec::new(),
            black_set: HashMap::new(),
            grey_set: HashMap::new(),
            white_set: HashMap::new(),
            get_roots: Box::new(get_roots),
        }
    }

    pub fn set_parent(&mut self, parent: Rc<Scope>) {
        self.parent = Some(parent);
    }

    pub fn add_child(&mut self, child: Scope) {
        self.children.push(Box::new(child));
    }

    pub fn alloc(&mut self, var: JsVar) -> Uuid {
        let uuid = var.uuid;
        self.white_set.insert(uuid, Rc::new(RefCell::new(var)));
        uuid
    }

    pub fn dealloc(&mut self, uuid: &Uuid) -> bool {
        if let Some(_) = self.white_set.remove(uuid) { true } else { false }
    }

    pub fn get_var_copy(&self, uuid: &Uuid) -> Option<JsVar> {
        self.find_id(uuid).map(|var|
                               (*var.clone()).clone().into_inner())
    }

    pub fn update_var(&mut self, var: JsVar) -> bool {
        if let Entry::Occupied(mut view) = self.find_id_mut(&var.uuid) {
            let inner = view.get_mut();
            *inner.borrow_mut() = var;
            true
        } else { false }
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
        for mark in marks.iter() {
            if let Some(var) = self.white_set.remove(mark) {
                let uuid = var.borrow().uuid;
                // Get all child references
                let child_ids = Scope::get_var_children(&var);
                // Mark current ref as black
                self.black_set.insert(uuid, var);
                // Mark child references as grey
                self.grey_children(child_ids);
            }
        }
    }

    pub fn mark_phase(&mut self) {
        // Mark any grey object as black, and mark all white objs it refs as grey
        let mut new_grey_set = HashMap::new();
        for (uuid, var) in self.grey_set.drain() {
            let child_ids = Scope::get_var_children(&var);
            self.black_set.insert(uuid, var);
            for child_id in child_ids {
                if let Some(var) = self.white_set.remove(&child_id) {
                    new_grey_set.insert(child_id, var);
                }
            }
        }
        self.grey_set = new_grey_set;
    }

    pub fn sweep_phase(&mut self) {
        self.white_set = HashMap::new();
    }

    pub fn find_id(&self, uuid: &Uuid) -> Option<&Alloc<JsVar>> {
        self.white_set.get(uuid).or_else(||
            self.grey_set.get(uuid).or_else(||
                self.black_set.get(uuid)))
    }

    fn find_id_mut(&mut self, uuid: &Uuid) -> Entry<Uuid, Alloc<JsVar>> {
        if let e @ Entry::Occupied(_) = self.white_set.entry(*uuid) {
            e
        } else if let e @ Entry::Occupied(_) = self.grey_set.entry(*uuid) {
            e
        } else { self.black_set.entry(*uuid) }
    }

    fn grey_children(&mut self, child_ids: HashSet<Uuid>) {
        for child_id in child_ids.iter() {
            if let Some(var) = self.white_set.remove(child_id) {
                self.grey_set.insert(*child_id, var);
            }
        }
    }

    fn get_var_children(var: &Alloc<JsVar>) -> HashSet<Uuid> {
        if let JsType::JsPtr(ref ptr) = (*var.borrow()).t {
            match ptr {
                &JsPtrEnum::JsObj(ref obj) => obj.get_children(),
                _ => HashSet::new(),
            }
        } else { HashSet::new() }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_set::HashSet;
    use std::rc::Rc;
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
        let parent_scope = Scope::new(dummy_get_roots);
        let mut test_scope = Scope::as_child(Rc::new(parent_scope), dummy_get_roots);

        assert!(test_scope.parent.is_some());
        assert!(test_scope.black_set.is_empty());
        assert!(test_scope.grey_set.is_empty());
        assert!(test_scope.white_set.is_empty());
        assert_eq!(test_scope.children.len(), 0);
    }

    #[test]
    fn test_set_parent() {
        let parent_scope = Scope::new(dummy_get_roots);
        let mut test_scope = Scope::new(dummy_get_roots);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(Rc::new(parent_scope));
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
