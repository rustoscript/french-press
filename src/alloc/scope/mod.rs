use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::hash_set::HashSet;
use std::mem;
use std::rc::Rc;

use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::js_type::{JsPtrEnum, JsType, JsVar, Binding};

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
    roots: HashSet<Binding>,
    pub parent: Option<Box<Scope>>,
    heap: Rc<RefCell<AllocBox>>,
    stack: HashMap<Binding, JsVar>,
}

impl Scope {
    /// Create a new, parentless scope node.
    pub fn new(heap: &Rc<RefCell<AllocBox>>) -> Scope {
        Scope {
            roots: HashSet::new(),
            parent: None,
            heap: heap.clone(),
            stack: HashMap::new(),
        }
    }

    /// Create a scope as a child of another scope. Clones all of bindings of
    /// root references from the parent.
    pub fn as_child(parent: Scope, heap: &Rc<RefCell<AllocBox>>) -> Scope {
        Scope {
            roots: parent.roots.clone(),
            parent: Some(Box::new(parent)),
            heap: heap.clone(),
            stack: HashMap::new(),
        }
    }

    /// Sets the parent of a scope, and clones and unions its root bindings.
    pub fn set_parent(&mut self, parent: Scope) {
        self.parent = Some(Box::new(parent));
    }

    /// Push a new JsVar onto the stack, and maybe allocate a pointer in the heap.
    pub fn push(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        let res = match &var.t {
            &JsType::JsPtr =>
                if let Some(ptr) = ptr {
                    // Creating a new pointer creates a new root
                    self.roots.insert(var.binding.clone());
                    self.heap.borrow_mut().alloc(var.binding.clone(), ptr)
                } else {
                    return Err(GcError::PtrError);
                },
            _ => Ok(()),
        };
        self.stack.insert(var.binding.clone(), var);
        res
    }

    /// Take ownership of a variable (usually from another scope).
    pub fn own(&mut self, var: JsVar) {
        self.stack.insert(var.binding.clone(), var);
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
        } else if let Some(ref parent) = self.parent {
            // FIXME? This is slow.
            parent.get_var_copy(bnd)
        } else { (None, None) }
    }

    /// Try to update a variable that's been allocated.
    pub fn update_var(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        match var.t {
            JsType::JsPtr =>
                if let Some(ptr) = ptr {
                    // A new root was potentially created
                    self.roots.insert(var.binding.clone());
                    self.heap.borrow_mut().update_ptr(&var.binding, ptr)
                } else {
                    Err(GcError::PtrError)
                },
            _ => {
                if let Entry::Occupied(mut view) = self.stack.entry(var.binding.clone()) {
                    // A root was potentially removed
                    self.roots.remove(&var.binding);
                    *view.get_mut() = var;
                    return Ok(());
                }
                // Hack to skirt mutable borrow of self lasting longer than it should
                if let Entry::Vacant(_) = self.stack.entry(var.binding.clone()) {
                    // TODO Need to call `push` on the root scope?
                    // Push into this scope and just give to parent when done?
                    return self.push(var, ptr);
                }
                unreachable!();
            },
        }
    }

    /// Called when a scope exits. Transfers the stack of this scope to its parent,
    /// and returns the parent scope, which may be `None`.
    pub fn transfer_stack(&mut self) -> Option<Box<Scope>> {
        if self.heap.borrow().len() > GC_THRESHOLD {
            self.heap.borrow_mut().mark_roots(&self.roots);
            self.heap.borrow_mut().mark_ptrs();
            self.heap.borrow_mut().sweep_ptrs();
        }
        if let Some(ref mut parent) = self.parent {
            for (_, var) in self.stack.drain() {
                match var.t {
                    JsType::JsPtr => {
                        // Mangle each binding before giving it to the parent
                        // scope. This avoids binding collisions, and helps
                        // identify to a human observer which bindings are
                        // not from the current scope.
                        let mut mangled_var = var.clone();
                        mangled_var.binding = Binding::mangle(var.binding);
                        parent.own(mangled_var);
                    }
                    _ => (),
                }
            }
            parent.roots = parent.roots.union(&self.roots).cloned().collect();
        }
        mem::replace(&mut self.parent, None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use js_types::js_type::{Binding, JsPtrEnum, JsKey, JsKeyEnum};
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
    fn test_get_var_copy() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        test_scope.push(x, Some(x_ptr)).unwrap();
        let bad_bnd = Binding::anon();

        let (var_copy, ptr_copy) = test_scope.get_var_copy(&x_bnd);
        assert!(var_copy.is_some());
        assert!(ptr_copy.is_some());

        let (bad_copy, ptr_copy) = test_scope.get_var_copy(&bad_bnd);
        assert!(bad_copy.is_none());
        assert!(ptr_copy.is_none());
    }

    #[test]
    fn test_get_var_copy_from_parent_scope() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        parent_scope.push(x, Some(x_ptr)).unwrap();

        let mut child_scope= Scope::as_child(parent_scope, &heap);

        let (var_copy, ptr_copy) = child_scope.get_var_copy(&x_bnd);
        assert!(var_copy.is_some());
        assert!(ptr_copy.is_some());
    }

    #[test]
    fn test_update_var() {
        let heap = test_utils::make_alloc_box();
        let mut test_scope = Scope::new(&heap);
        let (x, x_ptr, x_bnd) = test_utils::make_str("x");
        assert!(test_scope.push(x, Some(x_ptr)).is_ok());
        let (update, _) = test_scope.get_var_copy(&x_bnd);
        let update_ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new("test")));
        assert!(test_scope.update_var(update.unwrap(), update_ptr).is_ok());

        let (update, update_ptr) = test_scope.get_var_copy(&x_bnd);
        match update_ptr.unwrap() {
            JsPtrEnum::JsStr(JsStrStruct{text: ref s}) => assert_eq!(s, "test"),
            _ => panic!("Updated var was not equal to expected!")
        }
        assert_eq!(update.unwrap().binding, x_bnd);
    }

    #[test]
    fn test_transfer_stack() {
        let heap = test_utils::make_alloc_box();
        let mut parent_scope = Scope::new(&heap);
        {
            let mut test_scope = Scope::as_child(parent_scope, &heap);
            test_scope.push(test_utils::make_num(0.), None).unwrap();
            test_scope.push(test_utils::make_num(1.), None).unwrap();
            test_scope.push(test_utils::make_num(2.), None).unwrap();
            let kvs = vec![(JsKey::new(JsKeyEnum::JsBool(true)),
                            test_utils::make_num(1.))];
            let (var, ptr) = test_utils::make_obj(kvs);
            test_scope.push(var, Some(ptr)).unwrap();
            parent_scope = *test_scope.transfer_stack().unwrap();
        }
        assert_eq!(parent_scope.stack.len(), 1);
    }
}
