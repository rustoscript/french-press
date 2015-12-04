#![feature(associated_consts)]
#![feature(drain)]

extern crate uuid;

mod js_types;
mod alloc;

use std::cell::RefCell;
use std::collections::hash_set::HashSet;
use std::mem;
use std::rc::Rc;

use uuid::Uuid;

use alloc::AllocBox;
use alloc::scope::Scope;
use js_types::js_type::{JsPtrEnum, JsVar};

pub struct ScopeManager {
    curr_scope: Scope,
    alloc_box: Rc<RefCell<AllocBox>>
}

impl ScopeManager {
    pub fn new<F>(alloc_box: Rc<RefCell<AllocBox>>, callback: F) -> ScopeManager
        where F: Fn() -> HashSet<Uuid> + 'static {
        ScopeManager {
            curr_scope: Scope::new(&alloc_box, callback),
            alloc_box: alloc_box,
        }
    }

    pub fn push_scope<F>(&mut self, callback: F) where F: Fn() -> HashSet<Uuid> + 'static {
        let parent = mem::replace(&mut self.curr_scope, Scope::new(&self.alloc_box, callback));
        self.curr_scope.set_parent(parent);
    }

    pub fn pop_scope(&mut self) {
        let parent = mem::replace(&mut self.curr_scope.parent, None);
        mem::replace(&mut self.curr_scope,
                     *parent.expect("Tried to pop to parent scope, but parent did not exist!"));
    }

    pub fn alloc(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Uuid {
        self.curr_scope.push(var, ptr)
    }

    pub fn load(&self, uuid: &Uuid) -> Result<(JsVar, Option<JsPtrEnum>), String> {
        if let (Some(v), ptr) = self.curr_scope.get_var_copy(uuid) {
            Ok((v, ptr))
        } else { Err(format!("Lookup of uuid {} failed!", uuid)) }
    }

    pub fn store(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> bool {
        self.curr_scope.update_var(var, ptr)
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
