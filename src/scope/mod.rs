use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::mem;
use std::rc::Rc;

use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::js_var::{JsPtrEnum, JsType, JsVar};
use js_types::binding::Binding;
use js_types::allocator::Allocator;

pub mod scope_node;

// Tunable GC parameter. Probably should not be a constant, but good enough for now.
const GC_THRESHOLD: usize = 64;


/// A logical scope in the AST. Represents any scoped block of Javascript code.
/// roots: A set of all root references into the heap
/// parent: An optional parent scope, e.g. the caller of this function scope,
///         or the function that owns an `if` statement
/// heap: A shared reference to the heap allocator.
/// stack: The stack of the current scope, containing all variables allocated
///        by this scope.
pub struct Scope {
    stack: HashMap<Binding, JsVar>,
    heap: Rc<RefCell<AllocBox>>,
    roots: HashSet<Binding>,
    pub id: i32,
}

impl Scope {
    /// Create a new, parentless scope node.
    pub fn new(id: i32, heap: &Rc<RefCell<AllocBox>>) -> Scope {
        Scope {
            stack: HashMap::new(),
            heap: heap.clone(),
            roots: HashSet::new(),
            id: id,
        }
    }

    /// Sets the parent of a scope, and clones and unions its root bindings.
    /// This is not implemented as its own constructor due to ownership conflicts.
    /*pub fn set_parent(&mut self, parent: Scope) {
        self.roots = self.roots.union(&parent.roots).cloned().collect();
        self.parent = Some(box parent);
    }*/

    /// Push a new JsVar onto the stack, and maybe allocate a pointer in the heap.
    pub fn push_var(&mut self, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Result<()> {
        let res = match var.t {
            JsType::JsPtr =>
                if let Some(ptr) = ptr {
                    // Creating a new pointer creates a new root
                    self.roots.insert(var.binding.clone());
                    self.heap.borrow_mut().alloc(var.binding.clone(), ptr.clone())
                } else {
                    return Err(GcError::Ptr);
                },
            _ => if let Some(_) = ptr { Err(GcError::Ptr) } else { Ok(()) },
        };
        self.stack.insert(var.binding.clone(), var.clone());
        res
    }

    /// Return an optional copy of a variable and an optional pointer into the heap.
    pub fn get_var_copy(&self, bnd: &Binding) -> (Option<JsVar>, Option<JsPtrEnum>) {
        if let Some(var) = self.stack.get(bnd) {
            match var.t {
                JsType::JsPtr => {
                    if let Some(alloc) = self.heap.borrow().find_id(bnd) {
                        (Some(var.clone()), Some(alloc.borrow().clone()))
                    } else {
                        // This case should be impossible unless you have an
                        // invalid ptr, which should also be impossible.
                        (None, None)
                    }
                },
                _ => (Some(var.clone()), None),
            }
        } else { (None, None) }
    }

    /// Try to update a variable that's been allocated
    pub fn update_var(&mut self, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Result<()> {
        match var.t {
            JsType::JsPtr =>
                if let Some(ptr) = ptr {
                    // A new root was potentially created
                    self.roots.insert(var.binding.clone());
                    self.heap.borrow_mut().update_ptr(&var.binding, ptr.clone())
                } else {
                    Err(GcError::Ptr)
                },
            _ => {
                if let Some(_) = ptr { return Err(GcError::Ptr); }
                if let Entry::Occupied(mut view) = self.stack.entry(var.binding.clone()) {
                    // A root was potentially removed
                    self.roots.remove(&var.binding);
                    *view.get_mut() = var.clone();
                    return Ok(());
                }
                // Hack to skirt mutable borrow of self lasting longer than it should
                if let Entry::Vacant(_) = self.stack.entry(var.binding.clone()) {
                    // TODO Need to call `push_var` on the root scope?
                    // Push into this scope and just give to parent when done?
                    return self.push_var(&var, ptr);
                }
                unreachable!();
            },
        }
    }

    /// Called when a scope exits. Transfers the stack of this scope to its parent,
    /// and returns the parent scope, which may be `None`.
    pub fn transfer_stack(&mut self, gc_yield: bool) {
        if gc_yield {
            // The interpreter says we can GC now
            self.heap.borrow_mut().mark_roots(&self.roots);
            self.heap.borrow_mut().mark_ptrs();
            self.heap.borrow_mut().sweep_ptrs();
            // Pop all of the roots we just deleted
            for bnd in &self.roots {
                if let None = self.heap.borrow().find_id(bnd) {
                    self.stack.remove(bnd);
                }
            }
        }
        // TODO move the code below somewhere else
        /*if let Some(ref mut parent) = self.parent {
            for (_, var) in self.stack.drain() {
                if let JsType::JsPtr = var.t {
                        // Mangle each binding before giving it to the parent
                        // scope. This avoids binding collisions, and helps
                        // identify to a human observer which bindings are
                        // not from the current scope.
                        let mut mangled_var = var.clone();
                        mangled_var.binding = Binding::mangle(var.binding);
                        parent.own_var(mangled_var);
                }
            }
            parent.roots = parent.roots.union(&self.roots).cloned().collect();
        }*/
        //mem::replace(&mut self.parent, None)
    }

    /// Take ownership of a variable (usually from another scope).
    fn own_var(&mut self, var: JsVar) {
        self.stack.insert(var.binding.clone(), var);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use gc_error::GcError;
    use js_types::js_var::{JsPtrEnum, JsKey, JsKeyEnum, JsType};
    use js_types::binding::Binding;
    use js_types::js_str::JsStrStruct;
    use test_utils;

    #[test]
    fn test_new_scope() {
        let heap = test_utils::make_alloc_box();
        let test_scope = Scope::new(&heap);
        assert!(test_scope.parent.is_none());
    }

    #[test]
    fn test_as_child_scope() {
        let heap = test_utils::make_alloc_box();
        let parent_scope = Scope::new(&heap);
        let test_scope = Scope::as_child(parent_scope, &heap);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_set_parent() {
        let heap = test_utils::make_alloc_box();
        let parent_scope = Scope::new(&heap);
        let mut test_scope = Scope::new(&heap);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(parent_scope);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_push_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (var, ptr, _) = test_utils::make_str("test");
        assert!(test_scope.push_var(var, Some(ptr)).is_ok());
        assert_eq!(test_scope.heap.borrow().len(), 1);
        let var = test_utils::make_num(1.);
        assert!(test_scope.push_var(var, None).is_ok());
        assert_eq!(test_scope.heap.borrow().len(), 1);
    }

    #[test]
    fn test_push_var_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (var, ptr, _) = test_utils::make_str("test");
        let res = test_scope.push_var(var, None);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Ptr)));
        assert!(test_scope.heap.borrow().is_empty());
        let var = test_utils::make_num(1.);
        let res = test_scope.push_var(var, Some(ptr));
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Ptr)));
        assert!(test_scope.heap.borrow().is_empty());
    }

    #[test]
    fn test_get_var_copy() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        test_scope.push_var(x, Some(x_ptr)).unwrap();

        let (var_copy, ptr_copy) = test_scope.get_var_copy(&x_bnd);
        assert!(var_copy.is_some());
        assert!(ptr_copy.is_some());
    }

    #[test]
    fn test_get_var_copy_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (bad_copy, ptr_copy) = test_scope.get_var_copy(&Binding::anon());
        assert!(bad_copy.is_none());
        assert!(ptr_copy.is_none());
    }

    #[test]
    fn test_get_var_copy_from_parent_scope() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        parent_scope.push_var(x, Some(x_ptr)).unwrap();

        let child_scope = Scope::as_child(parent_scope, &heap);

        let (var_copy, ptr_copy) = child_scope.get_var_copy(&x_bnd);
        assert!(var_copy.is_some());
        assert!(ptr_copy.is_some());
    }

    #[test]
    fn test_update_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        assert!(test_scope.push_var(x, Some(x_ptr)).is_ok());
        let (update, _) = test_scope.get_var_copy(&x_bnd);
        let update_ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new("test")));
        assert!(test_scope.update_var(update.unwrap(), update_ptr).is_ok());

        let (update, update_ptr) = test_scope.get_var_copy(&x_bnd);
        match update_ptr.unwrap() {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => unreachable!(),
        }
        assert_eq!(update.unwrap().binding, x_bnd);
    }

    #[test]
    fn test_update_var_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        assert!(test_scope.push_var(x, Some(x_ptr)).is_ok());
        let (mut update, update_ptr) = test_scope.get_var_copy(&x_bnd);
        let res = test_scope.update_var(update.clone().unwrap(), None);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Ptr)));

        let mut update = update.unwrap();
        update.t = JsType::JsNum(1.);
        let res = test_scope.update_var(update, update_ptr);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Ptr)));
    }

    #[test]
    fn test_transfer_stack() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(&heap);
        {
            let mut test_scope = Scope::as_child(parent_scope, &heap);
            test_scope.push_var(test_utils::make_num(0.), None).unwrap();
            test_scope.push_var(test_utils::make_num(1.), None).unwrap();
            test_scope.push_var(test_utils::make_num(2.), None).unwrap();
            let kvs = vec![(JsKey::new(JsKeyEnum::JsBool(true)),
                            test_utils::make_num(1.), None)];
            let (var, ptr, _) = test_utils::make_obj(kvs, heap.clone());
            test_scope.push_var(var, Some(ptr)).unwrap();
            parent_scope = *test_scope.transfer_stack(false).unwrap();
        }
        assert_eq!(parent_scope.stack.len(), 1);
    }

    #[test]
    fn test_transfer_stack_with_yield() {
        let heap = test_utils::make_alloc_box();
        // Make a scope
        let mut parent_scope = Scope::new(&heap);
        {
            // Push a child scope
            let mut test_scope = Scope::as_child(parent_scope, &heap);
            // Allocate some non-root variables (numbers)
            test_scope.push_var(test_utils::make_num(0.), None).unwrap();
            test_scope.push_var(test_utils::make_num(1.), None).unwrap();
            test_scope.push_var(test_utils::make_num(2.), None).unwrap();

            // Make a string to put into an object
            // (so it's heap-allocated and we can lose its ref from the object)
            let (var, ptr, _) = test_utils::make_str("test");

            // Create an obj of { true: 1.0, false: heap("test") }
            let kvs = vec![(JsKey::new(JsKeyEnum::JsBool(true)),
                            test_utils::make_num(1.), None),
                           (JsKey::new(JsKeyEnum::JsBool(false)),
                            var, Some(ptr))];
            let key_bnd = kvs[1].0.binding.clone();
            let (var, ptr, bnd) = test_utils::make_obj(kvs, heap.clone());

            // Push the obj into the current scope
            test_scope.push_var(var, Some(ptr)).unwrap();

            // Replace the string in the object with something else so it's no longer live
            let copy = test_scope.get_var_copy(&bnd);
            let (var_cp, mut ptr_cp) = (copy.0.unwrap(), copy.1.unwrap());
            let key = JsKey { binding: key_bnd, k: JsKeyEnum::JsBool(false) };
            match ptr_cp {
                JsPtrEnum::JsObj(ref mut obj) => { obj.dict.insert(key, test_utils::make_num(-1.)); }
                _ => unreachable!()
            }
            test_scope.update_var(var_cp, Some(ptr_cp)).unwrap();

            // Kill the current scope & give its refs to the parent,
            // allowing the GC to kick in beforehand.
            parent_scope = *test_scope.transfer_stack(true).unwrap();
        }
        // The object we created above should still exist
        assert_eq!(parent_scope.stack.len(), 1);
        assert_eq!(heap.borrow().len(), 1);
    }

}
