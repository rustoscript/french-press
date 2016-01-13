#![feature(associated_consts)]
#![feature(drain)]

extern crate uuid;

mod alloc;
mod gc_error;
mod js_types;

use std::cell::RefCell;
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use uuid::Uuid;

use alloc::AllocBox;
use alloc::scope::Scope;
use gc_error::GcError;
use js_types::js_type::{JsPtrEnum, JsVar};


pub struct ScopeManager {
    curr_scope: Rc<Scope>,
    alloc_box: Rc<RefCell<AllocBox>>
}

impl ScopeManager {
    pub fn new<F>(alloc_box: Rc<RefCell<AllocBox>>, callback: F) -> ScopeManager
        where F: Fn() -> HashSet<Uuid> + 'static {
        ScopeManager {
            curr_scope: Rc::new(Scope::new(&alloc_box, callback)),
            alloc_box: alloc_box,
        }
    }

    pub fn push_scope<F>(&mut self, callback: F) where F: Fn() -> HashSet<Uuid> + 'static {
        self.curr_scope = Rc::new(Scope::as_child(&self.curr_scope, &self.alloc_box, callback));
    }

    pub fn pop_scope(&mut self) -> Result<(), GcError> {
        if let Some(parent) = self.curr_scope.parent.clone() {
            self.curr_scope = parent;
            Ok(())
        } else {
            Err(GcError::ScopeError)
        }
    }

    pub fn alloc(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<Uuid, GcError> {
        Rc::get_mut(&mut self.curr_scope).unwrap().push(var, ptr)
    }

    pub fn load(&self, uuid: &Uuid) -> Result<(JsVar, Option<JsPtrEnum>), GcError> {
        if let (Some(v), ptr) = self.curr_scope.get_var_copy(uuid) {
            Ok((v, ptr))
        } else { Err(GcError::LoadError(*uuid)) }
    }

    pub fn store(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> bool {
        // TODO error handling here
        Rc::get_mut(&mut self.curr_scope)
            .expect("Tried to mutate current scope but was unable!")
            .update_var(var, ptr)
    }
}

pub fn init_gc<F>(callback: F) -> ScopeManager
    where F: Fn() -> HashSet<Uuid> + 'static {
    let alloc_box = Rc::new(RefCell::new(AllocBox::new()));
    ScopeManager::new(alloc_box, callback)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_set::HashSet;
    use uuid::Uuid;

    fn dummy_callback() -> HashSet<Uuid> {
        HashSet::new()
    }
    // TODO
}
