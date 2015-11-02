use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::cmp;
use std::mem;

use js_types::js_type::{JsVar, JsType, JsPtrEnum};
use uuid::Uuid;

// Initial Arena size in bytes
const INITIAL_SIZE: usize = 1024;
// Minimum Arena capacity is at least 1 byte
const MIN_CAP: usize = 1;

pub struct Scope {
    parent: Option<Box<Scope>>,
    children: Vec<Box<Scope>>,
    black_set: HashMap<Uuid, RefCell<JsVar>>,
    grey_set: HashMap<Uuid, RefCell<JsVar>>,
    white_set: HashMap<Uuid, RefCell<JsVar>>,
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

    pub fn as_child<F>(parent: Box<Scope>, get_roots: F) -> Scope
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

    pub fn set_parent(&mut self, parent: Box<Scope>) {
        self.parent = Some(parent);
    }

    pub fn add_child(&mut self, child: Box<Scope>) {
        self.children.push(child);
    }

    pub fn alloc(&mut self, var: JsVar) -> Uuid {
        let uuid = var.uuid;
        self.white_set.insert(uuid, RefCell::new(var));
        uuid
    }

    pub fn dealloc(&mut self, uuid: &Uuid) -> bool {
        if let Some(_) = self.white_set.remove(uuid) { true } else { false }
    }

    pub fn get_var_copy(&self, uuid: &Uuid) -> Option<JsVar> {
        if let Some(var) = self.black_set.get(uuid) {
            Some(var.clone().into_inner())
        } else if let Some(var) = self.grey_set.get(uuid) {
            Some(var.clone().into_inner())
        } else if let Some(var) = self.white_set.get(uuid) {
            Some(var.clone().into_inner())
        } else { None }
    }

    pub fn update_var(&mut self, var: JsVar) -> bool {
        unimplemented!()
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

    //pub fn compute_roots(&self) -> HashSet<Uuid> {
    //    self.get_roots();
    //}

    /// Roots always get marked as Black, since they're always reachable from
    /// the current scope. NB that this assumes all root references are actually
    /// valid reference types, i.e. they're not numbers, etc.
    pub fn mark_roots(&mut self, marks: HashSet<Uuid>) {
        for mark in marks.iter() {
            if let Some(var) = self.white_set.remove(mark) {
                let uuid = var.borrow().uuid;
                // Get all child references
                let child_ids = self.get_var_children(&var);
                self.black_set.insert(uuid, var);
                // Mark child references as grey
                self.grey_children(child_ids);
            }
        }
    }

    pub fn mark_phase(&mut self) {
        // Mark any grey object as black, and mark all white objs it refs as grey
        while let Some(&uuid) = self.grey_set.keys().take(1).next() {
            if let Some(var) = self.grey_set.remove(&uuid) {
                let child_ids = self.get_var_children(&var);
                self.black_set.insert(uuid, var);
                for child_id in child_ids {
                    if let Some(var) = self.white_set.remove(&child_id) {
                        self.grey_set.insert(child_id, var);
                    }
                }
            }
        }
    }

    fn grey_children(&mut self, child_ids: HashSet<Uuid>) {
        for child_id in child_ids {
            if let Some(var) = self.white_set.remove(&child_id) {
                self.grey_set.insert(child_id, var);
            }
        }
    }

    fn get_var_children(&self, var: &RefCell<JsVar>) -> HashSet<Uuid> {
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
    use js_types::js_type::{JsVar, JsType, Marking};

    #[test]
    fn test_new_scope() {
        let test_jst0 = JsVar::new(JsType::JsNum(0.0));
        let test_jst1 = JsVar::new(JsType::JsNum(1.0));

        let mut test_scope = Scope::new("test");
        let uuid0 = test_scope.alloc(test_jst0);
        test_scope.mark_uuid(&uuid0, Marking::Black);
        let uuid1 = test_scope.alloc(test_jst1);
        test_scope.mark_uuid(&uuid1, Marking::White);
        assert_eq!(test_scope.get_jst_copy(&uuid0).unwrap().gc_flag, Marking::Black);
        assert_eq!(test_scope.get_jst_copy(&uuid1).unwrap().gc_flag, Marking::White);
    }
}
