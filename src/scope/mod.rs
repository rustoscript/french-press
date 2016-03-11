use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::mem;
use std::rc::Rc;

use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::js_var::{JsPtrEnum, JsPtrTag, JsType, JsVar};
use js_types::binding::{Binding, UniqueBinding};
use js_types::allocator::Allocator;

/// A logical scope in the AST. Represents any scoped block of Javascript code.
/// roots: A set of all root references into the heap
/// parent: An optional parent scope, e.g. the caller of this function scope,
///         or the function that owns an `if` statement
/// heap: A shared reference to the heap allocator.
/// stack: The stack of the current scope, containing all variables allocated
///        by this scope.
pub struct Scope {
    roots: HashSet<UniqueBinding>,
    pub parent: Option<Box<Scope>>,
    heap: Rc<RefCell<AllocBox>>,
    locals: HashMap<Binding, UniqueBinding>,
    stack: HashMap<UniqueBinding, JsVar>,
    tag: ScopeTag,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScopeTag {
    Call,
    Block,
}

impl Scope {
    /// Create a new, parentless scope node.
    pub fn new(tag: ScopeTag, heap: &Rc<RefCell<AllocBox>>) -> Scope {
        Scope {
            roots: HashSet::new(),
            parent: None,
            heap: heap.clone(),
            locals: HashMap::new(),
            stack: HashMap::new(),
            tag: tag,
        }
    }

    /// Sets the parent of a scope, and clones and unions its root bindings.
    pub fn set_parent(&mut self, parent: Scope) {
        self.roots = self.roots.union(&parent.roots).cloned().collect();
        self.parent = Some(box parent);
    }

    /// Push a new JsVar onto the stack, and maybe allocate a pointer in the heap.
    pub fn push_var(&mut self, local: Binding, mut var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        // Mangle the local binding to create a globally-unique name for the variable,
        // so that it can be safely allocated anywhere without name collisions.
        var.binding = UniqueBinding::mangle(&local);
        // Maybe insert the variable's pointer data into the heap
        let res = match var.t {
            JsType::JsPtr(_) =>
                if let Some(ptr) = ptr {
                    // Creating a new pointer creates a new root
                    self.roots.insert(var.binding.clone());
                    self.heap.borrow_mut().alloc(var.binding.clone(), ptr)
                } else {
                    return Err(GcError::PtrAlloc);
                },
            _ => if let Some(_) = ptr { Err(GcError::PtrAlloc) } else { Ok(()) },
        };
        // Create a mapping from the local binding to the unique binding
        self.locals.insert(local, var.binding.clone());
        // Push the unique binding onto the stack
        self.stack.insert(var.binding.clone(), var);
        res
    }

    fn rebind_var(&mut self, local: Binding, unique: UniqueBinding, var: JsVar) {
        self.locals.insert(local, unique.clone());
        self.stack.insert(unique, var);
    }

    /// Return an optional copy of a variable and an optional pointer into the heap.
    pub fn get_var_copy(&self, local: &Binding) -> Option<(JsVar, Option<JsPtrEnum>)> {
        if let Some(unique) = self.locals.get(local) {
            if let Some(var) = self.stack.get(unique) {
                match var.t {
                    JsType::JsPtr(_) => {
                        if let Some(alloc) = self.heap.borrow().find_id(unique) {
                            Some((var.clone(), Some(alloc.borrow().clone())))
                        } else {
                            // This case should be impossible unless you have an
                            // invalid ptr, which should also be impossible.
                            None
                        }
                    },
                    _ => Some((var.clone(), None)),
                }
            } else { None }
        } else if self.tag == ScopeTag::Call {
            None
        } else {
            // FIXME? This is slow.
            // A nonexistent binding in the current scope might require searching
            // the scope tree upwards for the binding. However, if the current
            // scope is a function call, it does not have access to anything from
            // its parent scope, so it should not do this lookup. If the overall
            // lookup fails, the ScopeManager will check the global scope.
            if let Some(ref parent) = self.parent {
                parent.get_var_copy(local)
            } else { None }
        }
    }

    /// Try to update a variable that's been allocated.
    pub fn update_var(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        match var.t {
            JsType::JsPtr(tag) =>
                if let Some(ptr) = ptr {
                    // A new root was potentially created
                    // If the pointer and its underlying type are not equal, return an error.
                    if !tag.eq_ptr_type(&ptr) { return Err(GcError::PtrAlloc); }
                    self.roots.insert(var.binding.clone());
                    self.heap.borrow_mut().update_ptr(&var.binding, ptr)
                } else {
                    Err(GcError::PtrAlloc)
                },
            _ => {
                if let Some(_) = ptr { return Err(GcError::PtrAlloc); }
                if let Entry::Occupied(mut view) = self.stack.entry(var.binding.clone()) {
                    // A root was potentially removed
                    self.roots.remove(&var.binding);
                    *view.get_mut() = var;
                    return Ok(());
                } else {
                    Err(GcError::Store(var, ptr))
                }
            },
        }
    }

    /// Called when a scope exits. Transfers the stack of this scope to its parent,
    /// and returns the parent scope, which may be `None`.
    pub fn transfer_stack(&mut self, closures: &mut Vec<Scope>, gc_yield: bool) -> Result<Option<Box<Scope>>> {
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
        if let Some(ref mut parent) = self.parent {
            let returning_closure = self.stack.iter()
                                              .any(|(_, v)|
                                                   matches!(v.t, JsType::JsPtr(JsPtrTag::JsFn)));
            // If we're returning a closure, conservatively assume the closure takes ownership of
            // every binding defined in this scope, so it must all live into the parent scope.
            if returning_closure {
                let mut closure_scope = Scope::new(ScopeTag::Call, &self.heap);
                for (local, unique) in self.locals.drain() {
                    let var = match self.stack.remove(&unique) {
                        Some(var) => var,
                        None => return Err(GcError::Scope),
                    };
                    closure_scope.rebind_var(local, unique, var);
                };
                closures.push(closure_scope);
            } else {
                for (local, unique) in self.locals.drain() {
                    let var = match self.stack.remove(&unique) {
                        Some(var) => var,
                        None => return Err(GcError::Scope),
                    };
                    // Rebind all heap-allocated variables into the parent scope, so they may be
                    // GC'd at a later time.
                    if let JsType::JsPtr(_) = var.t {
                        parent.rebind_var(local, unique, var);
                    }
                }
            }
            parent.roots = parent.roots.union(&self.roots).cloned().collect();
        }
        Ok(mem::replace(&mut self.parent, None))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::hash_map::HashMap;
    use std::rc::Rc;

    use alloc::AllocBox;
    use gc_error::GcError;
    use js_types::js_var::{JsVar, JsPtrEnum, JsKey, JsType};
    use js_types::binding::Binding;
    use js_types::js_str::JsStrStruct;
    use test_utils;

    fn new_scope_as_child(parent: Scope, tag: ScopeTag, heap: &Rc<RefCell<AllocBox>>) -> Scope {
        Scope {
            roots: parent.roots.clone(),
            parent: Some(box parent),
            heap: heap.clone(),
            locals: HashMap::new(),
            stack: HashMap::new(),
            tag: tag,
        }
    }

    #[test]
    fn test_new_scope() {
        let heap = test_utils::make_alloc_box();
        let test_scope = Scope::new(ScopeTag::Block, &heap);
        assert!(test_scope.parent.is_none());
    }

    #[test]
    fn test_as_child_scope() {
        let heap = test_utils::make_alloc_box();
        let parent_scope = Scope::new(ScopeTag::Block, &heap);
        let test_scope = new_scope_as_child(parent_scope, ScopeTag::Block, &heap);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_set_parent() {
        let heap = test_utils::make_alloc_box();
        let parent_scope = Scope::new(ScopeTag::Block, &heap);
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        assert!(test_scope.parent.is_none());
        test_scope.set_parent(parent_scope);
        assert!(test_scope.parent.is_some());
    }

    #[test]
    fn test_push_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (var, ptr, bnd) = test_utils::make_str("test");
        assert!(test_scope.push_var(bnd, var, Some(ptr)).is_ok());
        assert_eq!(test_scope.heap.borrow().len(), 1);
        let var = test_utils::make_num(1.);
        assert!(test_scope.push_var(Binding::new("test".to_string()), var, None).is_ok());
        assert_eq!(test_scope.heap.borrow().len(), 1);
    }

    #[test]
    fn test_push_var_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (var, ptr, bnd) = test_utils::make_str("test");
        let res = test_scope.push_var(bnd, var, None);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::PtrAlloc)));
        assert!(test_scope.heap.borrow().is_empty());
        let var = test_utils::make_num(1.);
        let res = test_scope.push_var(Binding::new("test".to_string()), var, Some(ptr));
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::PtrAlloc)));
        assert!(test_scope.heap.borrow().is_empty());
    }

    #[test]
    fn test_get_var_copy() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        test_scope.push_var(x_bnd.clone(), x, Some(x_ptr)).unwrap();

        let copy = test_scope.get_var_copy(&x_bnd);
        assert!(copy.is_some());
        let (var_copy, ptr_copy) = copy.unwrap();
        assert!(matches!(var_copy, JsVar { t: JsType::JsPtr(_), .. }));
        assert!(ptr_copy.is_some());
    }

    #[test]
    fn test_get_var_copy_fail() {
        let heap = test_utils::make_alloc_box();
        let test_scope = Scope::new(ScopeTag::Block, &heap);
        let copy = test_scope.get_var_copy(&Binding::new("".to_string()));
        assert!(copy.is_none());
    }

    #[test]
    fn test_get_var_copy_from_parent_scope_across_fn_boundary() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        parent_scope.push_var(x_bnd.clone(), x, Some(x_ptr)).unwrap();
        let child_scope = new_scope_as_child(parent_scope, ScopeTag::Call, &heap);

        let copy = child_scope.get_var_copy(&x_bnd);
        assert!(copy.is_none());
    }

    #[test]
    fn test_get_var_copy_from_parent_scope_no_fn_call() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        parent_scope.push_var(x_bnd.clone(), x, Some(x_ptr)).unwrap();

        let child_scope = new_scope_as_child(parent_scope, ScopeTag::Block, &heap);

        let copy = child_scope.get_var_copy(&x_bnd);
        assert!(copy.is_some());
        let (var_copy, ptr_copy) = copy.unwrap();
        assert!(matches!(var_copy, JsVar { t: JsType::JsPtr(_), .. }));
        assert!(ptr_copy.is_some());
    }

    #[test]
    fn test_update_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        assert!(test_scope.push_var(x_bnd.clone(), x, Some(x_ptr)).is_ok());
        let (update, _) = test_scope.get_var_copy(&x_bnd).unwrap();
        let update_ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new("test")));
        assert!(test_scope.update_var(update, update_ptr).is_ok());

        let (update, update_ptr) = test_scope.get_var_copy(&x_bnd).unwrap();
        match update_ptr.unwrap() {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => unreachable!(),
        }
        assert_eq!(update.binding, *test_scope.locals.get(&x_bnd).unwrap());
    }

    #[test]
    fn test_update_var_fail() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(ScopeTag::Block, &heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        assert!(test_scope.push_var(x_bnd.clone(), x, Some(x_ptr)).is_ok());
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
    fn test_transfer_stack_no_gc() {
        let heap = test_utils::make_alloc_box();
        let mut closures = Vec::new();
        let mut parent_scope = Scope::new(ScopeTag::Block, &heap);
        {
            let mut test_scope = new_scope_as_child(parent_scope, ScopeTag::Block, &heap);
            test_scope.push_var(Binding::new("zero".to_string()), test_utils::make_num(0.), None).unwrap();
            test_scope.push_var(Binding::new("one".to_string()), test_utils::make_num(1.), None).unwrap();
            test_scope.push_var(Binding::new("two".to_string()), test_utils::make_num(2.), None).unwrap();
            let kvs = vec![(JsKey::JsSym("true".to_string()),
                            test_utils::make_num(1.), None)];
            let (var, ptr, bnd) = test_utils::make_obj(kvs, heap.clone());
            test_scope.push_var(bnd, var, Some(ptr)).unwrap();
            parent_scope = *test_scope.transfer_stack(&mut closures, false).unwrap().unwrap();
        }
        assert_eq!(parent_scope.stack.len(), 1);
        assert_eq!(closures.len(), 0);
    }

    #[test]
    fn test_transfer_stack_with_yield() {
        let heap = test_utils::make_alloc_box();
        // Make some scopes
        let mut closures = Vec::new();
        let mut parent_scope = Scope::new(ScopeTag::Block, &heap);
        {
            // Push a child scope
            let mut test_scope = new_scope_as_child(parent_scope, ScopeTag::Block, &heap);
            // Allocate some non-root variables (numbers)
            test_scope.push_var(Binding::new("zero".to_string()), test_utils::make_num(0.), None).unwrap();
            test_scope.push_var(Binding::new("one".to_string()), test_utils::make_num(1.), None).unwrap();
            test_scope.push_var(Binding::new("two".to_string()), test_utils::make_num(2.), None).unwrap();

            // Make a string to put into an object
            // (so it's heap-allocated and we can lose its ref from the object)
            let (var, ptr, _) = test_utils::make_str("test");

            // Create an obj of { true: 1.0, false: heap("test") }
            let kvs = vec![(JsKey::JsSym("true".to_string()),
                            test_utils::make_num(1.), None),
                           (JsKey::JsSym("false".to_string()),
                            var, Some(ptr))];
            let (var, ptr, bnd) = test_utils::make_obj(kvs, heap.clone());

            // Push the obj into the current scope
            test_scope.push_var(bnd.clone(), var, Some(ptr)).unwrap();
            // The heap should now have 2 things in it: an object and a string
            assert_eq!(heap.borrow().len(), 2);

            // Replace the string in the object with something else so it's no longer live
            let copy = test_scope.get_var_copy(&bnd);
            let (var_cp, mut ptr_cp) = copy.unwrap();
            let key = JsKey::JsSym("false".to_string());
            match *&mut ptr_cp {
                Some(JsPtrEnum::JsObj(ref mut obj)) => {
                    obj.dict.insert(key, test_utils::make_num(-1.));
                },
                _ => unreachable!()
            }
            test_scope.update_var(var_cp, ptr_cp).unwrap();
            // The heap should still have 2 things in it: an object and a string
            assert_eq!(heap.borrow().len(), 2);

            // Kill the current scope & give its refs to the parent,
            // allowing the GC to kick in beforehand.
            parent_scope = *test_scope.transfer_stack(&mut closures, true).unwrap().unwrap();
        }
        // The object we created above should still exist
        assert_eq!(parent_scope.stack.len(), 1);
        // But the string it had allocated shouldn't, since we leaked it into the void
        assert_eq!(heap.borrow().len(), 1);
    }

    #[test]
    fn test_transfer_stack_return_closure() {
        let heap = test_utils::make_alloc_box();
        let mut closures = Vec::new();
        let mut parent_scope = Scope::new(ScopeTag::Block, &heap);
        let fn_bnd = {
            let mut test_scope = new_scope_as_child(parent_scope, ScopeTag::Block, &heap);
            let (var, test_fn, fn_bnd) = test_utils::make_fn(&Some("test".to_owned()), &Vec::new());
            test_scope.push_var(Binding::new("".to_string()), test_utils::make_num(1.), None).unwrap();
            test_scope.push_var(fn_bnd.clone(), var, Some(test_fn)).unwrap();
            let (var, ptr, bnd) = test_utils::make_str("test");
            test_scope.push_var(bnd, var, Some(ptr)).unwrap();
            let fn_bnd = test_scope.get_var_copy(&fn_bnd).unwrap().0.binding;
            parent_scope = *test_scope.transfer_stack(&mut closures, false).unwrap().unwrap();
            fn_bnd
        };
        assert_eq!(parent_scope.stack.len(), 0);
        assert_eq!(closures.len(), 1);
        assert_eq!(closures[0].stack.len(), 3);
        assert_eq!(heap.borrow().len(), 2);
        assert!(heap.borrow().find_id(&fn_bnd).is_some());
        for bnd in parent_scope.stack.keys() {
            assert!(heap.borrow().find_id(bnd).is_some());
        }
    }

}
