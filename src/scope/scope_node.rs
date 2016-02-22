use std::cell::RefCell;
use std::rc::Rc;

use scope::Scope;
use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::binding::Binding;
use js_types::js_var::{JsPtrEnum, JsVar};

pub struct ScopeNode {
    scope: Scope,
    parent: Option<Weak<RefCell<ScopeNode>>>,
    children: Vec<Rc<RefCell<ScopeNode>>>,
}

enum ParentResult<T, E> {
    /// Scope id and binding are found.
    Match(T),
    /// Scope id found, but no binding existed. Work up the scope tree to find the binding.
    MatchNil,
    /// Scope id not found, so return an appropriate error.
    MatchError(E),
}

impl ScopeNode {
    pub fn new(id: i32, parent: Option<&Rc<RefCell<ScopeNode>>>, alloc_box: &Rc<RefCell<AllocBox>>) -> ScopeNode {
        ScopeNode {
            scope: Scope::new(id, &alloc_box),
            parent: if let Some(p) = parent { Some(Rc::downgrade(p)) } else { None },
            children: Vec::new(),
        }
    }

    pub fn find_scope_by_id(&mut self, id: i32) -> Option<&mut Scope> {
        if self.scope.id == id {
            Some(&mut self.scope)
        } else {
            for node in &mut self.children {
                if node.find_scope_by_id(id).is_some() { return Some(&mut node.scope); }
            }
            None
        }
    }

    /// Add a child Scope to the specified Scope by `new_id`
    pub fn add_child_to_id(&mut self, new_id: i32, parent_id: i32, alloc_box: &Rc<RefCell<AllocBox>>) -> Result<()> {
        if self.scope.id == parent_id {
            self.children.push(Rc::new(RefCell::new(ScopeNode::new(new_id, Some(Rc::downgrade(&self)), &alloc_box))));
            Ok(())
        } else {
            for s in &mut self.children {
                if s.borrow_mut().add_child_to_id(new_id, parent_id, alloc_box).is_ok() {
                    return Ok(());
                }
            }
            Err(GcError::Scope(parent_id))
        }
    }

    /// Try to update a variable in a Scope named by `id`
    pub fn update_var_in_id(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Result<()> {
        if let ParentResult::Match(_) = self.update_var_in_id_ok(id, var, ptr) {
            Ok(())
        } else {
            Err(GcError::Scope(id))
        }
    }

    fn update_var_in_id_ok(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> ParentResult<(), GcError> {
        if self.scope.id == id {
            // If the desired scope is the current one, try to update the variable.
            if self.scope.update_var(var, ptr).is_ok() {
                // If the update was successful, everything is copacetic.
                ParentResult::Match(())
            } else {
                // If not, something went wrong, so report that error.
                ParentResult::MatchError(GcError::Scope(id))
            }
        } else {
            // Otherwise, search the child scopes all the way down the tree.
            for s in &mut self.children {
                match s.borrow_mut().update_var_in_id_ok(id, var, ptr) {
                    // If we were able to update in this child, then we're copacetic again.
                    ParentResult::Match(t) => return ParentResult::Match(t),
                    // If we weren't, but the scope existed, try to put the variable into the
                    // current scope.
                    ParentResult::MatchNil =>
                        return if self.scope.update_var(var, ptr).is_ok() {
                            ParentResult::Match(())
                        } else {
                            ParentResult::MatchError(GcError::Scope(id))
                        },
                    // If we got an error, then keep searching the other child scopes.
                    _ => (),
                }
            }
            // In all other cases, the update failed in this scope.
            ParentResult::MatchNil
        }
    }

    /// Try to get a copy of a variable and an optional pointer into the heap from scope `id`
    pub fn get_var_copy_from_id(&self, id: i32, bnd: &Binding) -> Result<(Option<JsVar>, Option<JsPtrEnum>)>  {
        if let ParentResult::Match(x) = self.get_var_copy_from_id_ok(id, bnd) {
            Ok(x)
        } else {
            Err(GcError::Scope(id))
        }
    }

    fn get_var_copy_from_id_ok(&self, id: i32, bnd: &Binding) -> ParentResult<(Option<JsVar>, Option<JsPtrEnum>), GcError> {
        if self.scope.id == id {
            // If the desired scope is the current one, try to get a copy of variable.
            if let (Some(x), y) = self.scope.get_var_copy(bnd) {
                // If the copy was successful, everything is copacetic.
                ParentResult::Match((Some(x), y))
            } else {
                // If not, something went wrong, so report that error.
                ParentResult::MatchError(GcError::Scope(id))
            }
        } else {
            // TODO might have to look upwards in the tree as well? If scopes are functions then no
            for s in &self.children {
                // Otherwise, search the child scopes all the way down the tree.
                match s.borrow().get_var_copy_from_id_ok(id, bnd) {
                    // If we were able to get a copy in this child, then we're copacetic again.
                    ParentResult::Match(t) => return ParentResult::Match(t),
                    // If we weren't, but the scope existed, try to copy the variable from the
                    // current scope.
                    ParentResult::MatchNil =>
                        return if let (Some(x), y) = self.scope.get_var_copy(bnd) {
                            ParentResult::Match((Some(x), y))
                        } else {
                            ParentResult::MatchError(GcError::Scope(id))
                        },
                    // If we got an error, then keep searching the other child scopes.
                    _ => ()
                };
            }
            // In all other cases, the copy failed in this scope.
            ParentResult::MatchNil
        }
    }

    /// Push a variable onto the stack of Scope `id`
    pub fn push_var_to_id(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Result<()> {
        if self.scope.id == id {
            return self.scope.push_var(&var, ptr);
        }

        for ref mut s in &mut self.children {
            if s.borrow_mut().push_var_to_id(id, var, ptr).is_ok() {
                return Ok(())
            }
        }

        Err(GcError::Scope(id))
    }
}
