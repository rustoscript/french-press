use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::rc::Rc;
use std::result;

use alloc::AllocBox;
use jsrs_common::gc_error::{GcError, Result};
use jsrs_common::types::js_var::{JsPtrEnum, JsType, JsVar};
use jsrs_common::types::binding::{Binding, UniqueBinding};
use jsrs_common::types::allocator::Allocator;

/// A logical scope in the AST. Represents any scoped block of Javascript code.
/// parent: An optional parent scope, e.g. the caller of this function scope,
///         or the function that owns an `if` statement
/// heap: A shared reference to the heap allocator.
/// stack: The stack of the current scope, containing all variables allocated
///        by this scope.
pub struct Scope {
    heap: Rc<RefCell<AllocBox>>,
    locals: HashMap<Binding, UniqueBinding>,
    stack: HashMap<UniqueBinding, JsVar>,
    maybe_globals: HashSet<Binding>,
    tag: ScopeTag,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScopeTag {
    Call,
    Block,
}

#[derive(Copy, Clone, Debug)]
pub enum LookupError {
    Unreachable,
    FnBoundary,
    CheckParent,
}

impl Scope {
    /// Create a new, parentless scope node.
    pub fn new(tag: ScopeTag, heap: &Rc<RefCell<AllocBox>>) -> Scope {
        Scope {
            heap: heap.clone(),
            locals: HashMap::new(),
            stack: HashMap::new(),
            maybe_globals: HashSet::new(),
            tag: tag,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Push a new JsVar onto the stack, and maybe allocate a pointer in the heap.
    pub fn push_var(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        if self.locals.contains_key(&var.binding) {
            // If the variable we're trying to create was already allocated,
            // just update its value and mark it as non-global.
            self.maybe_globals.remove(&var.binding);
            return self.update_var(var, ptr);
        }
        // Maybe insert the variable's pointer data into the heap
        let res = match var.t {
            JsType::JsPtr(_) =>
                if let Some(ptr) = ptr {
                    // Creating a new pointer creates a new root
                    self.heap.borrow_mut().alloc(var.unique.clone(), ptr)
                } else {
                    return Err(GcError::PtrAlloc);
                },
            _ => if let Some(_) = ptr { Err(GcError::PtrAlloc) } else { Ok(()) },
        };
        self.bind_var(var);
        res
    }

    /// Push an already-allocated JsVar onto the stack.
    pub fn bind_var(&mut self, var: JsVar) {
        // Create a mapping from the local binding to the unique binding
        self.locals.insert(var.binding.clone(), var.unique.clone());
        // Push the unique binding onto the stack
        self.stack.insert(var.unique.clone(), var);
    }

    fn rebind_var(&mut self, local: Binding, unique: UniqueBinding, var: JsVar) {
        self.locals.insert(local, unique.clone());
        self.stack.insert(unique, var);
    }

    pub fn mark_global(&mut self, binding: &Binding) {
        self.maybe_globals.insert(binding.clone());
    }

    /// Return an optional copy of a variable and an optional pointer into the heap.
    pub fn get_var_copy(&self, local: &Binding) -> result::Result<(JsVar, Option<JsPtrEnum>), LookupError> {
        if let Some(unique) = self.locals.get(local) {
            if let Some(var) = self.stack.get(unique) {
                match var.t {
                    JsType::JsPtr(_) => {
                        if let Some(alloc) = self.heap.borrow().find_id(unique) {
                            Ok((var.clone(), Some(alloc.borrow().clone())))
                        } else {
                            // This case should be impossible unless you have an
                            // invalid ptr, which should also be impossible.
                            Err(LookupError::Unreachable)
                        }
                    },
                    _ => Ok((var.clone(), None)),
                }
            } else { Err(LookupError::Unreachable) }
        } else if self.tag == ScopeTag::Call {
            // A nonexistent binding in the current scope might require searching
            // the scope tree upwards for the binding. However, if the current
            // scope is a function call, it does not have access to anything from
            // its parent scope, so it should not do this lookup. If the overall
            // lookup fails, the ScopeManager will check the global scope.
            Err(LookupError::FnBoundary)
        } else {
            Err(LookupError::CheckParent)
        }
    }

    /// Try to update a variable that's been allocated.
    pub fn update_var(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        if !self.locals.contains_key(&var.binding) {
            // Variable was not allocated.
            return Err(GcError::Store(var, ptr));
        }
        match var.t {
            JsType::JsPtr(ref tag) =>
                if let Some(ref ptr) = ptr {
                    // If the pointer and its underlying type are not equal, return an error.
                    if !tag.eq_ptr_type(&ptr) { return Err(GcError::PtrAlloc); }
                    // TODO FIXME? cloning ptr is potentially expensive
                    // A new root was potentially created
                    self.heap.borrow_mut().update_ptr(&var.unique, ptr.clone())?;
                } else {
                    return Err(GcError::PtrAlloc);
                },
            _ => {
                if let Some(_) = ptr {
                    return Err(GcError::PtrAlloc);
                }
                // A root was potentially removed.
                // Blindly accept this Result, since we have no information
                // about the type we're overwriting, and if we fail to condemn
                // a stack-allocated variable that's completely fine, since the
                // heap doesn't store those anyway.
                self.heap.borrow_mut().condemn(var.unique.clone()).ok();
            },
        }
        // Update the variable on the stack
        if let Entry::Occupied(mut view) = self.stack.entry(var.unique.clone()) {
            *view.get_mut() = var;
            Ok(())
        } else {
            Err(GcError::Store(var, ptr))
        }
    }

    pub fn trigger_gc(&mut self) {
        // The interpreter says we can GC now
        self.heap.borrow_mut().mark_ptrs();
        self.heap.borrow_mut().sweep_ptrs();
        // Pop any variables we just deleted
        // TODO rewrite this to not have to clone keys, if possible
        let locals: Vec<_> = self.locals.keys().cloned().collect();
        for bnd in &locals {
            if let Some(unique) = self.locals.remove(bnd) {
                if self.heap.borrow().find_id(&unique).is_none() {
                    self.stack.remove(&unique);
                }
            }
        }
    }

    /// Called when a scope exits. Transfers the stack of this scope to its parent,
    /// and returns the parent scope, which may be `None`.
    pub fn transfer_stack(&mut self, parent: &mut Scope, returning_closure: bool) -> Result<HashSet<JsVar>> {
        let mut globals = HashSet::new();
        for (local, unique) in self.locals.drain() {
            let var = match self.stack.remove(&unique) {
                Some(var) => var,
                None => return Err(GcError::Scope),
            };
            if self.maybe_globals.contains(&local) {
                // Don't give global variables to the parent scope. Return them
                // so they may be properly stored.
                globals.insert(var);
            } else if returning_closure {
                // If we're returning a closure, conservatively assume the
                // closure takes ownership of every binding defined in this
                // scope, so it must all live into the parent scope.
                parent.rebind_var(local, unique, var);
            } else {
                // If not returning a closure, rebind all heap-allocated
                // variables into the parent scope, so they may be GC'd at a
                // later time.
                if let JsType::JsPtr(_) = var.t {
                    parent.rebind_var(local, unique, var);
                }
            }
        }
        Ok(globals)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use jsrs_common::gc_error::GcError;
    use jsrs_common::types::js_var::{JsVar, JsPtrEnum, JsKey, JsType};
    use jsrs_common::types::binding::Binding;
    use jsrs_common::types::js_str::JsStrStruct;
    use test_utils;

    #[test]
    fn test_push_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (var, ptr) = test_utils::make_str("test");
        assert!(test_scope.push_var(var, Some(ptr)).is_ok());
        assert_eq!(test_scope.heap.borrow().len(), 1);
        let var = test_utils::make_num(1.);
        assert!(test_scope.push_var(var, None).is_ok());
        assert_eq!(test_scope.heap.borrow().len(), 1);
    }

    #[test]
    fn test_push_var_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (var, ptr) = test_utils::make_str("test");
        let res = test_scope.push_var(var, None);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::PtrAlloc)));
        assert!(test_scope.heap.borrow().is_empty());
        let var = test_utils::make_num(1.);
        let res = test_scope.push_var(var, Some(ptr));
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::PtrAlloc)));
        assert!(test_scope.heap.borrow().is_empty());
    }

    #[test]
    fn test_get_var_copy() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr) = test_utils::make_str("x");
        let x_bnd = x.binding.clone();
        test_scope.push_var(x, Some(x_ptr)).unwrap();

        let copy = test_scope.get_var_copy(&x_bnd);
        assert!(copy.is_ok());
        let (var_copy, ptr_copy) = copy.unwrap();
        assert!(matches!(var_copy, JsVar { t: JsType::JsPtr(_), .. }));
        assert!(ptr_copy.is_some());
    }

    #[test]
    fn test_get_var_copy_fail() {
        let heap = test_utils::make_alloc_box();
        let test_scope = Scope::new(ScopeTag::Block, &heap);
        let copy = test_scope.get_var_copy(&Binding::new("".to_string()));
        assert!(copy.is_err());
    }

    #[test]
    fn test_update_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr) = test_utils::make_str("x");
        let x_bnd = x.binding.clone();
        assert!(test_scope.push_var(x, Some(x_ptr)).is_ok());
        let (update, _) = test_scope.get_var_copy(&x_bnd).unwrap();
        let update_ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new("test")));
        assert!(test_scope.update_var(update, update_ptr).is_ok());

        let (update, update_ptr) = test_scope.get_var_copy(&x_bnd).unwrap();
        match update_ptr.unwrap() {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => unreachable!(),
        }
        assert_eq!(update.unique, *test_scope.locals.get(&x_bnd).unwrap());
    }

    #[test]
    fn test_update_var_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr) = test_utils::make_str("x");
        let x_bnd = x.binding.clone();
        assert!(test_scope.push_var(x, Some(x_ptr)).is_ok());
        let (mut update, update_ptr) = test_scope.get_var_copy(&x_bnd).unwrap();
        let res = test_scope.update_var(update.clone(), None);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::PtrAlloc)));

        update.t = JsType::JsNum(1.);
        let res = test_scope.update_var(update, update_ptr);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::PtrAlloc)));
    }

    #[test]
    fn test_transfer_stack_no_closure() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(ScopeTag::Block, &heap);
        {
            let mut test_scope = Scope::new(ScopeTag::Block, &heap);
            test_scope.push_var(test_utils::make_num(0.), None).unwrap();
            test_scope.push_var(test_utils::make_num(1.), None).unwrap();
            test_scope.push_var(test_utils::make_num(2.), None).unwrap();
            let kvs = vec![(JsKey::JsSym("true".to_string()),
                            test_utils::make_num(1.), None)];
            let (var, ptr) = test_utils::make_obj(kvs, heap.clone());
            test_scope.push_var(var, Some(ptr)).unwrap();
            test_scope.transfer_stack(&mut parent_scope, false).unwrap();
        }
        assert_eq!(parent_scope.stack.len(), 1);
    }

    #[test]
    fn test_transfer_stack_return_closure() {
        let heap = test_utils::make_alloc_box();
        let mut closure_scope = Scope::new(ScopeTag::Block, &heap);
        let fn_unique = {
            // Create a child scope
            let mut test_scope = Scope::new(ScopeTag::Block, &heap);

            // Create a function object
            let (var, test_fn) = test_utils::make_fn(&Some("test".to_owned()), &Vec::new());
            let fn_unique = var.unique.clone();

            // Alocate the function
            test_scope.push_var(var, Some(test_fn)).unwrap();

            // Create and allocate a number
            test_scope.push_var(test_utils::make_num(1.), None).unwrap();

            // Create and allocate a string
            let (var, ptr) = test_utils::make_str("test");
            test_scope.push_var(var, Some(ptr)).unwrap();

            // Kill the current scope, signalling that we're returning a closure
            test_scope.transfer_stack(&mut closure_scope, true).unwrap();
            fn_unique
        };
        // The closure scope should contain the entire environment of the old scope
        assert_eq!(closure_scope.stack.len(), 3);
        // The heap should contain a string and a function
        assert_eq!(heap.borrow().len(), 2);
        // The function should still be allocated
        assert!(heap.borrow().find_id(&fn_unique).is_some());
    }
}
