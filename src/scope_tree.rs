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
        self.update_var_in_id_ok(id, var, ptr).unwrap_or(Err(GcError::ScopeError))
    }

    fn update_var_in_id_ok(&mut self, id: i32, var: &JsVar, ptr: Option<&JsPtrEnum>) -> Option<Result<()>> {
        if self.scope.id == id {
            return Some(self.scope.update_var(var, ptr));
        }


        for ref mut s in &mut self.children {
            if let Some(r) = s.update_var_in_id_ok(id, var, ptr) {
                return Some(r);
            }
        }

        None
    }

    pub fn get_var_copy_from_id(&self, id: i32, bnd: &Binding) -> Result<(Option<JsVar>, Option<JsPtrEnum>)>  {
        if self.scope.id == id {
            return Ok(self.scope.get_var_copy(bnd));
        }

        for ref s in &self.children {
            if let Ok(t) = s.get_var_copy_from_id(id, bnd) {
                return Ok(t)
            }
        }

        Err(GcError::ScopeError)
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

    // pub fn get_id_of_parent(&self, id: i32) -> Option<i32> {
    //     self.get_id_of_parent_given_last(id, None)
    // }
    //
    // fn get_id_of_parent_given_last(&self, id: i32, parent: Option<i32>) -> Option<i32> {
    //     if self.scope.id == id {
    //         return parent
    //     }
    //     for ref s in &self.children {
    //         if let Some(i) = s.get_id_of_parent_given_last(id, Some(self.scope.id)) {
    //             return Some(i);
    //         }
    //     }
    //
    //     None
    // }
}
