#![feature(associated_consts)]
#![feature(plugin)]

#![plugin(clippy)]

extern crate uuid;
extern crate jsrs_common;
extern crate js_types;

#[cfg(test)]
#[macro_use] extern crate matches;

pub mod alloc;
mod gc_error;
mod scope_tree;
mod test_utils;

use std::cell::RefCell;
use std::rc::Rc;

use alloc::AllocBox;
use gc_error::{GcError, Result};
use js_types::js_var::{JsPtrEnum, JsVar};
use js_types::binding::Binding;
use scope_tree::ScopeTree;

pub struct ScopeManager {
    scopes: ScopeTree,
    current_stack: Vec<i32>,
    next: i32,
    alloc_box: Rc<RefCell<AllocBox>>,
}

impl ScopeManager {
    fn new(alloc_box: Rc<RefCell<AllocBox>>) -> ScopeManager {
        ScopeManager {
            scopes: ScopeTree::new(0, &alloc_box),
            current_stack: vec![0],
            next: 1,
            alloc_box: alloc_box,
        }
    }

    fn get_current_scope_id(&self) -> Result<i32> {
        if let Some(i) = self.current_stack.last() {
            return Ok(*i);
        }

        Err(GcError::ScopeError)
    }

    pub fn add_scope(&mut self) -> Result<i32> {
        let id = try!(self.get_current_scope_id());
        if self.scopes.add_child_to_id(self.next, id, &self.alloc_box) {
            self.current_stack.push(self.next);
            self.next += 1;
            return self.get_current_scope_id()
        }

        Err(GcError::ScopeError)
    }

    pub fn push_scope(&mut self, id: i32) {
        self.current_stack.push(id);
    }

    pub fn pop_scope(&mut self, gc_yield: bool) {
        self.current_stack.pop();
    }

    pub fn alloc(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        let id = try!(self.get_current_scope_id());

        self.scopes.push_on_id(id, &var, ptr.as_ref())
    }

    pub fn load(&self, bnd: &Binding) -> Result<(JsVar, Option<JsPtrEnum>)> {
        let id = try!(self.get_current_scope_id());
        if let (Some(v), ptr) = try!(self.scopes.get_var_copy_from_id(id, bnd)) {
            Ok((v, ptr))
        } else { Err(GcError::LoadError(bnd.clone())) }
    }

    pub fn store(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        let id = try!(self.get_current_scope_id());
        self.scopes.update_var_in_id(id, &var, ptr.as_ref())
    }
}

pub fn init_gc() -> ScopeManager {
    let alloc_box = Rc::new(RefCell::new(AllocBox::new()));
    ScopeManager::new(alloc_box)
}


#[cfg(test)]
mod tests {
    use super::*;

    use gc_error::GcError;
    use js_types::js_var::JsType;
    use js_types::binding::Binding;
    use test_utils;

    #[test]
    fn test_pop_scope() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        mgr.push_scope();
        assert!(mgr.curr_scope.parent.is_some());
        mgr.pop_scope(false).unwrap();
        assert!(mgr.curr_scope.parent.is_none());
    }

    #[test]
    fn test_pop_scope_fail() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        let res = mgr.pop_scope(false);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::ScopeError)));
    }

    #[test]
    fn test_alloc() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        mgr.alloc(test_utils::make_num(1.), None).unwrap();
        mgr.push_scope();
        mgr.alloc(test_utils::make_num(2.), None).unwrap();
        assert!(mgr.alloc_box.borrow().is_empty());
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
            _ => unreachable!(),
        }
        assert!(load.1.is_none());
    }

    #[test]
    fn test_load_fail() {
        let alloc_box = test_utils::make_alloc_box();
        let mgr = ScopeManager::new(alloc_box);
        let bnd = Binding::anon();
        let res = mgr.load(&bnd);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::LoadError(bnd))));
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
