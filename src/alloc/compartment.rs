use std::cell::RefCell;
use std::cmp;
use std::mem;

use alloc::ref_manip::UuidMap;
use js_types::js_type::JsT;
use uuid::Uuid;

// Initial Arena size in bytes
const INITIAL_SIZE: usize = 1024;
// Minimum Arena capacity is at least 1 byte
const MIN_CAP: usize = 1;

struct ChunkList<T> {
    curr: Vec<RefCell<T>>,
    rest: Vec<Vec<RefCell<T>>>,
}

impl<T> ChunkList<T> {
    fn grow(&mut self) {
        let new_cap = self.curr.capacity()
                               .checked_mul(2)
                               .expect("ChunkList: In method `grow`, `checked_mul` returned None. Aborting!");
        let chunk = mem::replace(&mut self.curr, Vec::with_capacity(new_cap));
        self.rest.push(chunk);
    }
}

struct GranularArena<T> {
    chunks: RefCell<ChunkList<T>>,
}

impl<T> GranularArena<T> {
    fn new() -> GranularArena<T> {
        let sz = cmp::max(1, mem::size_of::<T>());
        GranularArena::with_capacity(INITIAL_SIZE / sz)
    }

    fn with_capacity(cap: usize) -> GranularArena<T> {
        let cap = cmp::max(MIN_CAP, cap);
        GranularArena {
            chunks: RefCell::new(ChunkList {
                curr: Vec::with_capacity(cap),
                rest: Vec::new(),
            }),
        }
    }

    fn alloc(&self, val: T) -> &RefCell<T> {
        let mut chunks = self.chunks.borrow_mut();
        let next_item_idx = chunks.curr.len();
        chunks.curr.push(RefCell::new(val));

        let new_item_ref = {
            let new_item_ref = &chunks.curr[next_item_idx];

            unsafe { mem::transmute::<&RefCell<T>, &RefCell<T>>(new_item_ref) }
        };

        if chunks.curr.len() == chunks.curr.capacity() {
            chunks.grow();
        }

        new_item_ref
    }

    // TODO Figure out what granular deallocation will mean
    // TODO Is there a better way to allocate? Should I group items that are
    // temporally-local together? They sort of will be anyway, but is it worth
    // it to force such a thing?
}

pub struct Scope<'r> {
    pub source: String,
    parent: Option<Box<Scope<'r>>>,
    children: Vec<Box<Scope<'r>>>,
    arena: GranularArena<JsT>,
    uuid_map: UuidMap<'r>,
}

impl<'r> Scope<'r> {
    pub fn new(source: &str) -> Scope {
        Scope {
            source: String::from(source),
            parent: None,
            children: Vec::new(),
            arena: GranularArena::new(),
            uuid_map: UuidMap::new(),
        }
    }

    pub fn alloc_inside(&mut self, jst: JsT) -> Uuid {
        let uuid = jst.uuid;
        let jst_ref: &RefCell<JsT> = self.arena.alloc(jst);
        self.uuid_map.insert_by_refcell(jst_ref);
        uuid
    }

    pub fn get_val_copy(&self, uuid: Uuid) -> Option<JsT> {
        self.uuid_map.get_by_uuid(uuid)
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
        let uuid0 = test_scope.alloc_inside(test_jst0);
        test_scope.uuid_map.mark_uuid(uuid0, Marking::Black);
    }
}
