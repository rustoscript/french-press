#![feature(associated_consts)]

extern crate uuid;
extern crate jsrs_common;

pub mod alloc;
mod gc_error;
pub mod js_types;
mod test_utils;

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use alloc::AllocBox;
use alloc::scope::Scope;
use gc_error::{GcError, Result};
use js_types::js_type::{JsPtrEnum, JsVar, Binding};

pub struct ScopeManager {
    curr_scope: Scope,
    alloc_box: Rc<RefCell<AllocBox>>,
}

impl ScopeManager {
    fn new(alloc_box: Rc<RefCell<AllocBox>>) -> ScopeManager {
        ScopeManager {
            curr_scope: Scope::new(&alloc_box),
            alloc_box: alloc_box,
        }
    }

    pub fn push_scope(&mut self) {
        let parent = mem::replace(&mut self.curr_scope, Scope::new(&self.alloc_box));
        self.curr_scope.set_parent(parent);
    }

    pub fn pop_scope(&mut self) -> Result<()> {
        let parent = self.curr_scope.transfer_stack();
        if let Some(parent) = parent {
            mem::replace(&mut self.curr_scope, *parent);
            Ok(())
        } else {
            Err(GcError::ScopeError)
        }
    }

    pub fn alloc(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        self.curr_scope.push(var, ptr)
    }

    pub fn load(&self, bnd: &Binding) -> Result<(JsVar, Option<JsPtrEnum>)> {
        if let (Some(v), ptr) = self.curr_scope.get_var_copy(bnd) {
            Ok((v, ptr))
        } else { Err(GcError::LoadError(bnd.clone())) }
    }

    pub fn store(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        self.curr_scope.update_var(var, ptr)
    }
}

pub fn init_gc() -> ScopeManager {
    let alloc_box = Rc::new(RefCell::new(AllocBox::new()));
    ScopeManager::new(alloc_box)
}


#[cfg(test)]
mod tests {
    use super::*;

    use test_utils;
    use js_types::js_type::{Binding, JsType};

    #[test]
    fn test_pop_scope() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        mgr.push_scope();
        assert!(mgr.curr_scope.parent.is_some());
        mgr.pop_scope().unwrap();
        assert!(mgr.curr_scope.parent.is_none());
    }

    #[test]
    fn test_alloc() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        mgr.alloc(test_utils::make_num(1.), None).unwrap();
        mgr.push_scope();
        mgr.alloc(test_utils::make_num(2.), None).unwrap();
        assert_eq!(mgr.alloc_box.borrow().len(), 0);
    }

    #[test]
    fn test_load() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        let x = test_utils::make_num(1.);
        let x_bnd = x.binding.clone();
        mgr.alloc(x, None).unwrap();
        let load = mgr.load(&x_bnd);
        assert!(load.is_ok());
        let load = load.unwrap();
        match load.0.t {
            JsType::JsNum(n) => assert_eq!(n, 1.),
            _ => panic!("load result was not equal to value allocated!"),
        }
        assert!(load.1.is_none());
        assert!(mgr.load(&Binding::anon()).is_err());
    }

    #[test]
    fn test_store() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box,);
        mgr.push_scope();
        let x = test_utils::make_num(1.);
        let x_bnd = x.binding.clone();
        mgr.alloc(x, None).unwrap();

        let mut test_num = test_utils::make_num(2.);
        test_num.binding = x_bnd;
        assert!(mgr.store(test_num, None).is_ok());
    }

}
