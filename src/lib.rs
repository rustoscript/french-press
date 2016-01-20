#![feature(associated_consts)]

extern crate uuid;
extern crate jsrs_common;

pub mod alloc;
mod ast;
mod gc_error;
pub mod js_types;
mod utils;

use std::cell::RefCell;
use std::collections::hash_set::HashSet;
use std::mem;
use std::rc::Rc;

use uuid::Uuid;

use alloc::AllocBox;
use alloc::scope::Scope;
use gc_error::GcError;
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

    pub fn pop_scope(&mut self) -> Result<(), GcError> {
        let parent = self.curr_scope.transfer_stack();
        if let Some(parent) = parent {
            mem::replace(&mut self.curr_scope, *parent);
            Ok(())
        } else {
            Err(GcError::ScopeError)
        }
    }

    pub fn alloc(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<Uuid, GcError> {
        self.curr_scope.push(var, ptr)
    }

    pub fn load(&self, uuid: &Uuid) -> Result<(JsVar, Option<JsPtrEnum>), GcError> {
        if let (Some(v), ptr) = self.curr_scope.get_var_copy(uuid) {
            Ok((v, ptr))
        } else { Err(GcError::LoadError(*uuid)) }
    }

    pub fn store(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<Uuid, GcError> {
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

    use uuid;

    use utils;
    use js_types::js_type::JsType;

    #[test]
    fn test_pop_scope() {
        let alloc_box = utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box, utils::dummy_callback);
        mgr.push_scope(utils::dummy_callback);
        assert!(mgr.curr_scope.parent.is_some());
        mgr.pop_scope().unwrap();
        assert!(mgr.curr_scope.parent.is_none());
    }

    #[test]
    fn test_alloc() {
        let alloc_box = utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box, utils::dummy_callback);
        mgr.alloc(utils::make_num(1.), None).unwrap();
        mgr.push_scope(utils::dummy_callback);
        mgr.alloc(utils::make_num(2.), None).unwrap();
        assert_eq!(mgr.alloc_box.borrow().len(), 0);
    }

    #[test]
    fn test_load() {
        let alloc_box = utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box, utils::dummy_callback);
        let id1 = mgr.alloc(utils::make_num(1.), None).unwrap();
        let load = mgr.load(&id1);
        assert!(load.is_ok());
        let load = load.unwrap();
        match load.0.t {
            JsType::JsNum(n) => assert_eq!(n, 1.),
            _ => panic!("load result was not equal to value allocated!"),
        }
        assert!(load.1.is_none());
        assert!(mgr.load(&uuid::Uuid::nil()).is_err());
    }

    #[test]
    fn test_store() {
        let alloc_box = utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box, utils::dummy_callback);
        mgr.push_scope(utils::dummy_callback);
        mgr.alloc(utils::make_num(1.), None).unwrap();
        let test_id = mgr.alloc(utils::make_num(2.), None).unwrap();
        mgr.alloc(utils::make_num(3.), None).unwrap();

        let mut test_num = utils::make_num(4.);
        test_num.uuid = test_id;
        assert!(mgr.store(test_num, None).is_ok());
    }

}
