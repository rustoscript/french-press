use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use uuid::Uuid;

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

    pub fn alloc(&mut self, uuid: Uuid, ptr: JsPtrEnum) -> Uuid {
        self.white_set.insert(uuid, Rc::new(RefCell::new(ptr)));
        uuid
    }

    pub fn dealloc(&mut self, uuid: &Uuid) -> bool {
        if let Some(_) = self.white_set.remove(uuid) { true } else { false }
    }

    pub fn mark_roots(&mut self, marks: HashSet<Uuid>) {
        for mark in marks {
            // FIXME? Could a root be grey already?
            if let Some(ptr) = self.white_set.remove(&mark) {
                // Get all child references
                let child_ids = AllocBox::get_var_children(&ptr);
                // Mark current ref as black
                self.black_set.insert(mark, ptr);
                // Mark child references as grey
                self.grey_children(child_ids);
            }
        }
    }

    pub fn mark_vars(&mut self) {
        // Mark any grey object as black, and mark all white objs it refs as grey
        let mut new_grey_set = HashMap::new();
        for (uuid, var) in self.grey_set.drain() {
            let child_ids = AllocBox::get_var_children(&var);
            self.black_set.insert(uuid, var);
            for child_id in child_ids {
                if let Some(var) = self.white_set.remove(&child_id) {
                    new_grey_set.insert(child_id, var);
                }
            }
        }
        self.grey_set = new_grey_set;
    }

    pub fn sweep_vars(&mut self) {
        self.white_set = HashMap::new();
    }

    pub fn find_id(&self, uuid: &Uuid) -> Option<&Alloc<JsPtrEnum>> {
        self.white_set.get(uuid).or_else(||
            self.grey_set.get(uuid).or_else(||
                self.black_set.get(uuid)))
    }

    pub fn update_var(&mut self, uuid: &Uuid, ptr: JsPtrEnum) -> bool {
        if let Entry::Occupied(mut view) = self.find_id_mut(&uuid) {
            let inner = view.get_mut();
            *inner.borrow_mut() = ptr;
            true
        } else { false }
    }

    fn grey_children(&mut self, child_ids: HashSet<Uuid>) {
        for child_id in child_ids.iter() {
            if let Some(var) = self.white_set.remove(child_id) {
                self.grey_set.insert(*child_id, var);
            }
        }
    }

    fn get_var_children(ptr: &Alloc<JsPtrEnum>) -> HashSet<Uuid> {
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
