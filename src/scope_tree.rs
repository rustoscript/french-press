use std::cell::RefCell;
use std::rc::Rc;

use alloc::scope::Scope;
use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::binding::Binding;
use js_types::js_var::{JsPtrEnum, JsVar};

pub struct ScopeTree {
    scope: Scope,
    children: Vec<Box<ScopeTree>>,
}

enum ParentResult<T> {
    NoMatch,
    MatchError,
    Match(T),
}

impl ScopeTree {
    pub fn new(id: i32, alloc_box: &Rc<RefCell<AllocBox>>) -> ScopeTree {
        ScopeTree { scope: Scope::new(id, &alloc_box), children: Vec::new() }
    }

    pub fn add_child_to_id(&mut self, new_id: i32, parent: i32, alloc_box: &Rc<RefCell<AllocBox>>) -> bool {
        if self.scope.id == parent {
            self.children.push(Box::new(ScopeTree::new(new_id, &alloc_box)));
            return true;
        }

        for ref mut s in &mut self.children {
            if s.add_child_to_id(new_id, parent, alloc_box) {
                return true;
            }
        }

        false
    }

    pub fn update_var_in_id(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Result<()> {
        if let ParentResult::Match(_) = self.update_var_in_id_ok(id, var, ptr) {
            Ok(())
        } else {
            Err(GcError::ScopeError)
        }
    }

    fn update_var_in_id_ok(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> ParentResult<()> {
        if self.scope.id == id {
            return if self.scope.update_var(var, ptr).is_ok() {
                ParentResult::Match(())
            } else {
                ParentResult::MatchError
            }
        }

        for ref mut s in &mut self.children {
            match s.update_var_in_id_ok(id, var ,ptr) {
                ParentResult::Match(t) => return ParentResult::Match(t),
                ParentResult::MatchError => return if self.scope.update_var(var, ptr).is_ok() {
                    ParentResult::Match(())
                } else {
                    ParentResult::MatchError
                },
                _ => ()
            };
        }

        ParentResult::NoMatch
    }

    pub fn get_var_copy_from_id(&self, id: i32, bnd: &Binding) -> Result<(Option<JsVar>, Option<JsPtrEnum>)>  {
        if let ParentResult::Match(x) = self.get_var_copy_from_id_ok(id, bnd) {
            Ok(x)
        } else {
            Err(GcError::ScopeError)
        }
    }

    fn get_var_copy_from_id_ok(&self, id: i32, bnd: &Binding) -> ParentResult<(Option<JsVar>, Option<JsPtrEnum>)> {
        if self.scope.id == id {
            return if let (Some(x), y) = self.scope.get_var_copy(bnd) {
                ParentResult::Match((Some(x), y))
            } else {
                ParentResult::MatchError
            }
        }

        for ref s in &self.children {
            match s.get_var_copy_from_id_ok(id, bnd) {
                ParentResult::Match(t) => return ParentResult::Match(t),
                ParentResult::MatchError => return if let (Some(x), y) = self.scope.get_var_copy(bnd) {
                    ParentResult::Match((Some(x), y))
                } else {
                    ParentResult::MatchError
                },
                _ => ()
            };
        }

        ParentResult::NoMatch
    }

    pub fn push_on_id(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Result<()> {
        if self.scope.id == id {
            return self.scope.push(&var, ptr);
        }

        for ref mut s in &mut self.children {
            if s.push_on_id(id, var, ptr).is_ok() {
                return Ok(())
            }
        }

        Err(GcError::ScopeError)
    }
}
