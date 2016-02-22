use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use gc_error::{GcError, Result};
use js_types::js_var::JsPtrEnum;
use js_types::allocator::Allocator;
use js_types::binding::Binding;

pub type Alloc<T> = Rc<RefCell<T>>;

pub struct AllocBox {
    black_set: HashMap<Binding, Alloc<JsPtrEnum>>,
    grey_set: HashMap<Binding, Alloc<JsPtrEnum>>,
    white_set: HashMap<Binding, Alloc<JsPtrEnum>>,
}

impl Allocator for AllocBox {
    type Error = GcError;

    fn alloc(&mut self, binding: Binding, ptr: JsPtrEnum) -> Result<()> {
        if let None = self.white_set.insert(binding.clone(), Rc::new(RefCell::new(ptr))) {
            Ok(())
        } else {
            // If a binding already exists and we try to allocate it, this should
            // be an unrecoverable error.
            Err(GcError::AllocError(binding))
        }
    }
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn mark_roots(&mut self, marks: &HashSet<Binding>) {
        for mark in marks {
            if let Some(ptr) = self.white_set.remove(mark) {
                // Get all child references
                let child_ids = AllocBox::get_ptr_children(&ptr);
                // Mark current ref as black
                self.black_set.insert(mark.clone(), ptr);
                // Mark child references as grey
                self.grey_children(child_ids);
            } else if let Some(ptr) = self.grey_set.remove(&mark) {
                // Get all child references
                let child_ids = AllocBox::get_ptr_children(&ptr);
                // Mark current ref as black
                self.black_set.insert(mark.clone(), ptr);
                // Mark child references as grey
                self.grey_children(child_ids);
            }
        }
    }

    pub fn mark_ptrs(&mut self) {
        // Mark any grey object as black, and mark all white objs it refs as grey
        let mut new_grey_set = HashMap::new();
        for (bnd, var) in self.grey_set.drain() {
            let child_ids = AllocBox::get_ptr_children(&var);
            self.black_set.insert(bnd, var);
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
        self.white_set = self.black_set.drain().collect();
        self.grey_set.clear();
        self.black_set.clear();
    }

    pub fn find_id(&self, bnd: &Binding) -> Option<&Alloc<JsPtrEnum>> {
        self.white_set.get(bnd).or(
            self.grey_set.get(bnd).or(
                self.black_set.get(bnd)))
    }

    pub fn update_ptr(&mut self, binding: &Binding, ptr: JsPtrEnum) -> Result<()> {
        if let Entry::Occupied(mut view) = self.find_id_mut(binding) {
            let inner = view.get_mut();
            *inner.borrow_mut() = ptr;
            Ok(())
        } else {
            Err(GcError::StoreError)
        }
    }

    fn grey_children(&mut self, child_ids: HashSet<Binding>) {
        for child_id in child_ids {
            if let Some(var) = self.white_set.remove(&child_id) {
                self.grey_set.insert(child_id, var);
            }
        }
    }

    fn get_ptr_children(ptr: &Alloc<JsPtrEnum>) -> HashSet<Binding> {
        if let JsPtrEnum::JsObj(ref obj) = *ptr.borrow() {
            obj.get_children()
        } else { HashSet::new() }
    }

    fn find_id_mut(&mut self, bnd: &Binding) -> Entry<Binding, Alloc<JsPtrEnum>> {
        if let e @ Entry::Occupied(_) = self.white_set.entry(bnd.clone()) {
            e
        } else if let e @ Entry::Occupied(_) = self.grey_set.entry(bnd.clone()) {
            e
        } else { self.black_set.entry(bnd.clone()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_set::HashSet;

    use gc_error::GcError;
    use js_types::allocator::Allocator;
    use js_types::binding::Binding;
    use js_types::js_var::JsPtrEnum;
    use js_types::js_str::JsStrStruct;
    use test_utils;

    #[test]
    fn test_len() {
        let mut ab = AllocBox::new();
        assert!(ab.is_empty());
        assert!(ab.alloc(Binding::anon(), test_utils::make_str("").1).is_ok());
        assert_eq!(ab.len(), 1);
    }

    #[test]
    fn test_alloc() {
        let mut ab = AllocBox::new();
        let (_, x_ptr, x_bnd) = test_utils::make_str("x");
        let (_, y_ptr, y_bnd) = test_utils::make_str("y");
        let x_bnd_2 = x_bnd.clone();
        assert!(ab.alloc(x_bnd, x_ptr.clone()).is_ok());
        assert!(ab.alloc(y_bnd, y_ptr).is_ok());
    }

    #[test]
    fn test_alloc_fail() {
        let mut ab = AllocBox::new();
        let (_, x_ptr, x_bnd) = test_utils::make_str("x");
        let x_bnd_2 = x_bnd.clone();
        assert!(ab.alloc(x_bnd, x_ptr.clone()).is_ok());
        let res = ab.alloc(x_bnd_2, x_ptr);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::AllocError(x_bnd_2))));
    }

    #[test]
    fn test_update_ptr() {
        let mut ab = AllocBox::new();
        let (_, x_ptr, x_bnd) = test_utils::make_str("x");
        let x_bnd_2 = x_bnd.clone();
        assert!(ab.alloc(x_bnd, x_ptr.clone()).is_ok());
        let (_, new_ptr, _) = test_utils::make_str("y");
        assert!(ab.update_ptr(&x_bnd_2, new_ptr).is_ok());
        let opt_ptr = ab.find_id(&x_bnd_2);
        assert!(opt_ptr.is_some());
        // Hack to get around some borrowck failures I don't fully understand
        if let Some(ptr) = opt_ptr {
            match ptr.borrow().clone() {
                JsPtrEnum::JsStr(JsStrStruct { ref text }) => assert_eq!(text.clone(), "y".to_string()),
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }
    }

    #[test]
    fn test_update_ptr_fail() {
        let mut ab = AllocBox::new();
        let (_, ptr, _) = test_utils::make_str("");
        let res = ab.update_ptr(&Binding::anon(), ptr);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::StoreError)));
    }

    #[test]
    fn test_mark_roots() {
        let mut ab = AllocBox::new();
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        let (y, y_ptr, y_bnd) = test_utils::make_str("y");

        ab.alloc(x.binding, x_ptr).unwrap();
        ab.alloc(y.binding, y_ptr).unwrap();

        let mut marks = HashSet::new();
        marks.insert(x_bnd.clone()); marks.insert(y_bnd.clone());
        ab.mark_roots(&marks);
        assert!(ab.black_set.contains_key(&x_bnd));
        assert!(ab.black_set.contains_key(&y_bnd));
    }
}
