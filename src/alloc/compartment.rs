use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::cmp;
use std::mem;

use alloc::ref_manip::UuidMap;
use js_types::js_type::{JsT, Marking};
use uuid::Uuid;

// Initial Arena size in bytes
const INITIAL_SIZE: usize = 1024;
// Minimum Arena capacity is at least 1 byte
const MIN_CAP: usize = 1;

pub struct Scope {
    pub source: String,
    parent: Option<Box<Scope>>,
    children: Vec<Box<Scope>>,
    arena: HashMap<Uuid, RefCell<JsT>>,
}

impl Scope {
    pub fn new(source: &str) -> Scope {
        Scope {
            source: String::from(source),
            parent: None,
            children: Vec::new(),
            arena: HashMap::new(),
        }
    }

    pub fn alloc(&mut self, jst: JsT) -> Uuid {
        let uuid = jst.uuid;
        self.arena.insert(uuid, RefCell::new(jst));
        uuid
    }

    pub fn dealloc(&mut self, uuid: &Uuid) -> bool {
        if let Some(_) = self.arena.remove(uuid) { true } else { false }
    }

    pub fn get_jst_copy(&self, uuid: &Uuid) -> Option<JsT> {
        if let Some(jst) = self.arena.get(uuid) {
            Some(jst.clone().into_inner())
        } else { None }
    }

    pub fn mark_uuid(&mut self, uuid: &Uuid, marking: Marking) -> bool {
        if let Some(jst_refcell) = self.arena.get_mut(uuid) {
            jst_refcell.borrow_mut().gc_flag = marking;
            true
        } else { false }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use js_types::js_type::{JsT, JsType, Marking};

    #[test]
    fn test_new_scope() {
        let test_jst0 = JsT::new(JsType::JsNum(0.0));
        let test_jst1 = JsT::new(JsType::JsNum(1.0));

        let mut test_scope = Scope::new("test");
        let uuid0 = test_scope.alloc(test_jst0);
        test_scope.mark_uuid(&uuid0, Marking::Black);
        let uuid1 = test_scope.alloc(test_jst1);
        test_scope.mark_uuid(&uuid1, Marking::White);
        assert_eq!(test_scope.get_jst_copy(&uuid0).unwrap().gc_flag, Marking::Black);
        assert_eq!(test_scope.get_jst_copy(&uuid1).unwrap().gc_flag, Marking::White);
    }
}
