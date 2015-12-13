#![feature(associated_consts)]

extern crate uuid;
extern crate jsrs_common;

mod alloc;
mod ast;
mod js_types;
mod utils;

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
        self.curr_scope = Scope::as_child(&mut self.curr_scope, &self.alloc_box, callback);
    }

    pub fn pop_scope(&mut self) {
        if !(self.curr_scope).parent.is_null() {
            unsafe {
                //self.curr_scope.transfer_stack();
                let ref mut parent = *(self.curr_scope.parent);
                mem::swap(&mut self.curr_scope, parent);
            }
        } else {
            panic!("Tried to pop to parent scope, but parent did not exist!");
        }
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use uuid::Uuid;

    use alloc::AllocBox;
    use js_types::js_type::{JsType, JsVar};
    use utils;

    #[test]
    fn test_alloc() {
        let alloc_box = utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box, utils::dummy_callback);
        mgr.alloc(utils::make_num(1.), None);
        mgr.push_scope(utils::dummy_callback);
        mgr.alloc(utils::make_num(2.), None);
        assert_eq!(mgr.alloc_box.borrow().len(), 0);
    }

    #[test]
    fn test_store() {
        let alloc_box = utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box, utils::dummy_callback);
        mgr.push_scope(utils::dummy_callback);
        mgr.alloc(utils::make_num(1.), None);
        let test_id = mgr.alloc(utils::make_num(2.), None);
        mgr.alloc(utils::make_num(3.), None);

        let mut test_num = utils::make_num(4.);
        test_num.uuid = test_id;
        assert!(mgr.store(test_num, None));
    }
}
