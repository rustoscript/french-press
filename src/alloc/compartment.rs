use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::cmp;
use std::mem;

use alloc::ref_manip::UuidMap;
use js_types::js_type::{JsVar, JsType};
use uuid::Uuid;

// Initial Arena size in bytes
const INITIAL_SIZE: usize = 1024;
// Minimum Arena capacity is at least 1 byte
const MIN_CAP: usize = 1;

pub struct Scope {
    pub source: String,
    parent: Option<Box<Scope>>,
    children: Vec<Box<Scope>>,
    black_set: HashMap<Uuid, RefCell<JsT>>,
    grey_set: HashMap<Uuid, RefCell<JsT>>,
    white_set: HashMap<Uuid, RefCell<JsT>>,
}

impl Scope {
    pub fn new(source: &str) -> Scope {
        Scope {
            source: String::from(source),
            parent: None,
            children: Vec::new(),
            black_set: HashMap::new(),
            grey_set: HashMap::new(),
            white_set: HashMap::new(),
        }
    }

    pub fn as_child(source: &str, parent: Box<Scope>) -> Scope {
        Scope {
            source: String::from(source),
            parent: Some(parent),
            children: Vec::new(),
            black_set: HashMap::new(),
            grey_set: HashMap::new(),
            white_set: HashMap::new(),
        }
    }

    pub fn set_parent(&mut self, parent: Box<Scope>) {
        self.parent = Some(parent);
    }

    pub fn add_child(&mut self, child: Box<Scope>) {
        self.children.push(child);
    }

    pub fn alloc(&mut self, jst: JsVar) -> Uuid {
        let uuid = jst.uuid;
        self.white_set.insert(uuid, RefCell::new(jst));
        uuid
    }

    pub fn dealloc(&mut self, uuid: &Uuid) -> bool {
        if let Some(_) = self.white_set.remove(uuid) { true } else { false }
    }

    pub fn get_jst_copy(&self, uuid: &Uuid) -> Option<JsVar> {
        if let Some(jst) = self.black_set.get(uuid) {
            Some(jst.clone().into_inner())
        } else if let Some(jst) = self.grey_set.get(uuid) {
            Some(jst.clone().into_inner())
        } else if let Some(jst) = self.white_set.get(uuid) {
            Some(jst.clone().into_inner())
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
    pub fn compute_roots(&self) -> HashSet<Uuid> {
        unimplemented!();
    }

    /// Roots always get marked as Black, since they're always reachable from
    /// the current scope.
    pub fn mark_roots(&mut self, marks: HashSet<Uuid>) {
        for mark in marks.iter() {
            if let Some(jst) = self.white_set.remove(mark) {
                let uuid = jst.borrow().uuid;
                self.black_set.insert(uuid, jst);
                // TODO mark child references as grey
            }
        }
    }

    pub fn mark_phase(&mut self) {
        // TODO mark object as black, mark all white objs it refs as grey
        for (_, v) in self.grey_set.drain() {
            match (*v.borrow()).t {
                JsType::JsPtr(ref ptr) => unimplemented!(),
                _ => ()
            }
        }
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
