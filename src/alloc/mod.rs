use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use uuid::Uuid;

use gc_error::GcError;
use js_types::js_type::JsPtrEnum;

pub mod scope;

pub type Alloc<T> = Rc<RefCell<T>>;

pub struct AllocBox {
    black_set: HashMap<Uuid, Alloc<JsPtrEnum>>,
    grey_set: HashMap<Uuid, Alloc<JsPtrEnum>>,
    white_set: HashMap<Uuid, Alloc<JsPtrEnum>>,
}

impl AllocBox {
    pub fn new() -> AllocBox {
        AllocBox {
            black_set: HashMap::new(),
            grey_set: HashMap::new(),
            white_set: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.black_set.len() + self.grey_set.len() + self.white_set.len()
    }

    pub fn alloc(&mut self, uuid: Uuid, ptr: JsPtrEnum) -> Result<Uuid, GcError> {
        if let None = self.white_set.insert(uuid, Rc::new(RefCell::new(ptr))) {
            Ok(uuid)
        } else {
            // If a UUID already exists and we try to allocate it, this should
            // be an unrecoverable error. In practice, this shouldn't happen
            // unless the UUID generator creates a collision.
            Err(GcError::AllocError(uuid))
        }
    }

    pub fn mark_roots(&mut self, marks: HashSet<Uuid>) {
        for mark in marks {
            if let Some(ptr) = self.white_set.remove(&mark) {
                // Get all child references
                let child_ids = AllocBox::get_ptr_children(&ptr);
                // Mark current ref as black
                self.black_set.insert(mark, ptr);
                // Mark child references as grey
                self.grey_children(child_ids);
            } else if let Some(ptr) = self.grey_set.remove(&mark) {
                // Get all child references
                let child_ids = AllocBox::get_ptr_children(&ptr);
                // Mark current ref as black
                self.black_set.insert(mark, ptr);
                // Mark child references as grey
                self.grey_children(child_ids);
            }
        }
    }

    pub fn mark_ptrs(&mut self) {
        // Mark any grey object as black, and mark all white objs it refs as grey
        let mut new_grey_set = HashMap::new();
        for (uuid, var) in self.grey_set.drain() {
            let child_ids = AllocBox::get_ptr_children(&var);
            self.black_set.insert(uuid, var);
            for child_id in child_ids {
                if let Some(var) = self.white_set.remove(&child_id) {
                    new_grey_set.insert(child_id, var);
                }
            }
        }
        self.grey_set = new_grey_set;
    }

    pub fn sweep_ptrs(&mut self) {
        // Delete all white pointers and reset the GC state.
        self.white_set = self.black_set.clone();
        self.grey_set = HashMap::new();
        self.black_set = HashMap::new();
    }

    pub fn find_id(&self, uuid: &Uuid) -> Option<&Alloc<JsPtrEnum>> {
        self.white_set.get(uuid).or(
            self.grey_set.get(uuid).or(
                self.black_set.get(uuid)))
    }

    pub fn update_ptr(&mut self, uuid: &Uuid, ptr: JsPtrEnum) -> Result<Uuid, GcError> {
        if let Entry::Occupied(mut view) = self.find_id_mut(&uuid) {
            let inner = view.get_mut();
            *inner.borrow_mut() = ptr;
            Ok(uuid.clone())
        } else {
            Err(GcError::StoreError)
        }
    }

    fn grey_children(&mut self, child_ids: HashSet<Uuid>) {
        for child_id in child_ids.iter() {
            if let Some(var) = self.white_set.remove(child_id) {
                self.grey_set.insert(*child_id, var);
            }
        }
    }

    fn get_ptr_children(ptr: &Alloc<JsPtrEnum>) -> HashSet<Uuid> {
        if let JsPtrEnum::JsObj(ref obj) = *ptr.borrow() {
            obj.get_children()
        } else { HashSet::new() }
    }

    fn find_id_mut(&mut self, uuid: &Uuid) -> Entry<Uuid, Alloc<JsPtrEnum>> {
        if let e @ Entry::Occupied(_) = self.white_set.entry(*uuid) {
            e
        } else if let e @ Entry::Occupied(_) = self.grey_set.entry(*uuid) {
            e
        } else { self.black_set.entry(*uuid) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_set::HashSet;

    use uuid::Uuid;

    use utils;

    #[test]
    fn test_len() {
        let mut ab = AllocBox::new();
        assert_eq!(ab.len(), 0);
        ab.alloc(Uuid::new_v4(), utils::make_str("")).unwrap();
        assert_eq!(ab.len(), 1);
    }

    #[test]
    fn test_alloc() {
        let mut ab = AllocBox::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let _id1 = ab.alloc(id1.clone(), utils::make_str("")).unwrap();
        let _id2 = ab.alloc(id2.clone(), utils::make_str("")).unwrap();
        let _id3 = ab.alloc(id3.clone(), utils::make_str("")).unwrap();

        assert_eq!(id1, _id1);
        assert_eq!(id2, _id2);
        assert_eq!(id3, _id3);
    }

    #[test]
    fn test_mark_roots() {
        let mut ab = AllocBox::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id1 = ab.alloc(id1, utils::make_str("")).unwrap();
        let id2 = ab.alloc(id2, utils::make_str("")).unwrap();

        let mut marks = HashSet::new();
        marks.insert(id1); marks.insert(id2);
        ab.mark_roots(marks);
    }
}
