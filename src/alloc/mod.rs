use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use heapsize::HeapSizeOf;
use jsrs_common::gc_error::{GcError, Result};
use jsrs_common::types::js_var::JsPtrEnum;
use jsrs_common::types::allocator::Allocator;
use jsrs_common::types::binding::UniqueBinding;

#[derive(Clone, Debug)]
pub struct Alloc<T> {
    pub inner: Rc<RefCell<T>>,
}

impl<T: HeapSizeOf> HeapSizeOf for Alloc<T> {
    fn heap_size_of_children(&self) -> usize {
        (*self.inner).heap_size_of_children()
    }
}

impl<T: HeapSizeOf> Alloc<T> {
    fn new(t: T) -> Alloc<T> {
        Alloc {
            inner: Rc::new(RefCell::new(t)),
        }
    }
}

#[derive(Debug, HeapSizeOf)]
pub struct AllocBox {
    black_set: HashMap<UniqueBinding, Alloc<JsPtrEnum>>,
    grey_set: HashMap<UniqueBinding, Alloc<JsPtrEnum>>,
    white_set: HashMap<UniqueBinding, Alloc<JsPtrEnum>>,
}

impl Allocator for AllocBox {
    type Error = GcError;

    fn alloc(&mut self, binding: UniqueBinding, ptr: JsPtrEnum) -> Result<()> {
        if self.grey_set.insert(binding.clone(), Alloc::new(ptr)).is_none() {
            info!(target: "mem", "AllocBox::alloc: heap size: {}", self.heap_size_of_children());
            Ok(())
        } else {
            // If a binding already exists and we try to allocate it, this should
            // be an unrecoverable error, as we may be clobbering data that someone
            // else has reference to.
            Err(GcError::Alloc(binding))
        }
    }

    fn condemn(&mut self, unique: UniqueBinding) -> Result<()> {
        if let Some(ptr) = self.remove_binding(&unique) {
            self.white_set.insert(unique, ptr);
            info!(target: "mem", "AllocBox::condemn: heap size: {}", self.heap_size_of_children());
            Ok(())
        } else {
            Err(GcError::HeapUpdate)
        }
    }
}

impl AllocBox {
    pub fn new() -> AllocBox {
        let ab = AllocBox {
            black_set: HashMap::new(),
            grey_set: HashMap::new(),
            white_set: HashMap::new(),
        };
        println!("AllocBox::new: total size: {}", ab.heap_size_of_children());
        ab
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.black_set.len() + self.grey_set.len() + self.white_set.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        self.white_set = HashMap::new();
        // TODO is it a good assumption to reset everything to grey?
        self.grey_set = self.black_set.drain().collect();
        self.black_set = HashMap::new();
        info!(target: "mem", "AllocBox::sweep_ptrs: heap size: {}", self.heap_size_of_children());
    }

    pub fn find_id(&self, bnd: &UniqueBinding) -> Option<&Alloc<JsPtrEnum>> {
        self.white_set.get(bnd).or(
            self.grey_set.get(bnd).or(
                self.black_set.get(bnd)))
    }

    pub fn is_allocated(&self, bnd: &UniqueBinding) -> bool {
        self.white_set.contains_key(bnd) ||
        self.grey_set.contains_key(bnd)  ||
        self.black_set.contains_key(bnd)
    }

    pub fn update_ptr(&mut self, binding: &UniqueBinding, ptr: JsPtrEnum) -> Result<()> {
        // Updating a pointer means it is definitely reachable
        if let Some(alloc) = self.remove_binding(binding) {
            *alloc.inner.borrow_mut() = ptr;
            self.grey_set.insert(binding.clone(), alloc);
            info!(target: "mem", "AllocBox::update_ptr: heap size: {}", self.heap_size_of_children());
            Ok(())
        } else {
            Err(GcError::HeapUpdate)
        }
    }

    fn remove_binding(&mut self, binding: &UniqueBinding) -> Option<Alloc<JsPtrEnum>> {
        self.white_set.remove(binding).or(
            self.grey_set.remove(binding).or(
                self.black_set.remove(binding)))
    }

    fn get_ptr_children(ptr: &Alloc<JsPtrEnum>) -> HashSet<UniqueBinding> {
        if let JsPtrEnum::JsObj(ref obj) = *ptr.inner.borrow() {
            obj.get_children()
        } else { HashSet::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jsrs_common::gc_error::GcError;
    use jsrs_common::types::allocator::Allocator;
    use jsrs_common::types::binding::UniqueBinding;
    use jsrs_common::types::js_var::JsPtrEnum;
    use jsrs_common::types::js_str::JsStrStruct;
    use test_utils;

    #[test]
    fn test_len() {
        let mut ab = AllocBox::new();
        assert!(ab.is_empty());
        assert!(ab.alloc(UniqueBinding::anon(), test_utils::make_str("").1).is_ok());
        assert_eq!(ab.len(), 1);
    }

    #[test]
    fn test_alloc() {
        let mut ab = AllocBox::new();
        let (x, x_ptr) = test_utils::make_str("x");
        let (y, y_ptr) = test_utils::make_str("y");
        assert!(ab.alloc(x.unique, x_ptr.clone()).is_ok());
        assert!(ab.alloc(y.unique, y_ptr).is_ok());
    }

    #[test]
    fn test_alloc_fail() {
        let mut ab = AllocBox::new();
        let (x, x_ptr) = test_utils::make_str("x");
        assert!(ab.alloc(x.unique.clone(), x_ptr.clone()).is_ok());
        let res = ab.alloc(x.unique.clone(), x_ptr);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Alloc(_))));
        if let Err(GcError::Alloc(bnd)) = res {
            assert_eq!(x.unique, bnd);
        }
    }

    #[test]
    fn test_update_ptr() {
        let mut ab = AllocBox::new();
        let (x, x_ptr) = test_utils::make_str("x");
        assert!(ab.alloc(x.unique.clone(), x_ptr.clone()).is_ok());
        let (_, new_ptr) = test_utils::make_str("y");
        assert!(ab.update_ptr(&x.unique, new_ptr).is_ok());
        let opt_ptr = ab.find_id(&x.unique);
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
        let (_, ptr) = test_utils::make_str("");
        let res = ab.update_ptr(&UniqueBinding::anon(), ptr);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::HeapUpdate)));
    }
}
