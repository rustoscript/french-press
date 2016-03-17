#![feature(associated_consts)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(question_mark)]
//#![feature(plugin)]

//#![plugin(clippy)]

extern crate jsrs_common;
extern crate js_types;

#[macro_use] extern crate matches;

pub mod alloc;
mod gc_error;
mod scope;
mod test_utils;

use std::cell::RefCell;
use std::rc::Rc;

use jsrs_common::ast::Exp;
use js_types::js_var::{JsPtrEnum, JsVar};
use js_types::binding::Binding;

use alloc::AllocBox;
use gc_error::{GcError, Result};
use scope::{LookupError, Scope, ScopeTag};

pub struct ScopeManager {
    scopes: Vec<Scope>,
    closures: Vec<Scope>,
    alloc_box: Rc<RefCell<AllocBox>>,
}

impl ScopeManager {
    fn new(alloc_box: Rc<RefCell<AllocBox>>) -> ScopeManager {
        ScopeManager {
            scopes: vec![Scope::new(ScopeTag::Call, &alloc_box)],
            closures: Vec::new(),
            alloc_box: alloc_box,
        }
    }

    #[inline]
    fn curr_scope(&self) -> &Scope {
        self.scopes.last().expect("Tried to access current scope, but none existed")
    }

    #[inline]
    fn curr_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("Tried to access current scope, but none existed")
    }

    pub fn push_scope(&mut self, exp: &Exp) {
        let tag = match *exp {
            Exp::Call(..) => ScopeTag::Call,
            _ => ScopeTag::Block,
        };
        self.scopes.push(Scope::new(tag, &self.alloc_box));
    }

    pub fn pop_scope(&mut self, returning_closure: bool, gc_yield: bool) -> Result<()> {
        if let Some(mut scope) = self.scopes.pop() {
            // Potentially trigger the garbage collector
            if gc_yield {
                scope.trigger_gc();
            }
            // Clean up the dying scope's stack and take ownership of its heap-allocated data for
            // later collection
            if !self.scopes.is_empty() {
                if returning_closure {
                    let mut closure_scope = Scope::new(ScopeTag::Call, &self.alloc_box);
                    scope.transfer_stack(&mut closure_scope, returning_closure)?;
                    self.closures.push(closure_scope);
                } else {
                    scope.transfer_stack(self.curr_scope_mut(), returning_closure)?;
                }
                Ok(())
            } else {
                Err(GcError::Scope)
            }
        } else {
            Err(GcError::Scope)
        }
    }

    pub fn alloc(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<Binding> {
        let binding = var.binding.clone();
        self.curr_scope_mut().push_var(var, ptr)?;
        Ok(binding)
    }

    /// Try to load the variable behind a binding
    pub fn load(&self, bnd: &Binding) -> Result<(JsVar, Option<JsPtrEnum>)> {
        let lookup = || {
            for scope in self.scopes.iter().rev() {
                match scope.get_var_copy(bnd) {
                    Ok(v) => { return Ok(v); },
                    Err(LookupError::FnBoundary) => {
                        return Err(GcError::Load(bnd.clone()));
                    },
                    Err(LookupError::CheckParent) => {},
                    Err(LookupError::Unreachable) => unreachable!(),
                }
            }
            Err(GcError::Load(bnd.clone()))
        };
        match lookup() {
            Ok(v) => Ok(v),
            Err(GcError::Load(bnd)) =>
                self.scopes[0].get_var_copy(&bnd)
                .map_err(|_| GcError::Load(bnd.clone())),
            _ => unreachable!(),
        }
    }

    pub fn store(&mut self, var: JsVar, ptr: Option<JsPtrEnum>) -> Result<()> {
        let update = self.curr_scope_mut().update_var(var, ptr);
        if let Err(GcError::Store(var, ptr)) = update {
            self.alloc(var, ptr).map(|_| ())
        } else {
            update
        }
    }
}

pub fn init_gc() -> ScopeManager {
    let alloc_box = Rc::new(RefCell::new(AllocBox::new()));
    ScopeManager::new(alloc_box)
}


#[cfg(test)]
mod tests {
    use super::*;

    use jsrs_common::ast::Exp;
    use js_types::js_var::JsType;
    use js_types::binding::Binding;

    use gc_error::GcError;
    use test_utils;

    #[test]
    fn test_pop_scope() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        mgr.push_scope(&Exp::Undefined);
        assert_eq!(mgr.scopes.len(), 2);
        mgr.pop_scope(false, false).unwrap();
        assert_eq!(mgr.scopes.len(), 1);
    }

    #[test]
    fn test_pop_scope_fail() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        let res = mgr.pop_scope(false, false);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Scope)));
    }

    #[test]
    fn test_alloc() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        mgr.alloc(test_utils::make_num(1.), None).unwrap();
        mgr.push_scope(&Exp::Undefined);
        mgr.alloc(test_utils::make_num(2.), None).unwrap();
        assert!(mgr.alloc_box.borrow().is_empty());
    }

    #[test]
    fn test_load() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box);
        let x = test_utils::make_num(1.);
        let x_bnd = mgr.alloc(x, None).unwrap();
        let load = mgr.load(&x_bnd);
        assert!(load.is_ok());
        let load = load.unwrap();
        match load.0.t {
            JsType::JsNum(n) => assert!(f64::abs(n - 1.) < 0.0001),
            _ => unreachable!(),
        }
        assert!(load.1.is_none());
    }

    #[test]
    fn test_load_fail() {
        let alloc_box = test_utils::make_alloc_box();
        let mgr = ScopeManager::new(alloc_box);
        let bnd = Binding::new("".to_owned());
        let res = mgr.load(&bnd);
        assert!(res.is_err());
        assert!(matches!(res, Err(GcError::Load(_))));
        if let Err(GcError::Load(res_bnd)) = res {
            assert_eq!(bnd, res_bnd);
        }
    }

    #[test]
    fn test_store() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box,);
        mgr.push_scope(&Exp::Undefined);
        let x = test_utils::make_num(1.);
        let x_bnd = mgr.alloc(x, None).unwrap();

        let (mut var, _) = mgr.load(&x_bnd).unwrap();
        var.t = JsType::JsNum(2.);

        assert!(mgr.store(var, None).is_ok());
    }

    #[test]
    fn test_store_failed_store() {
        let alloc_box = test_utils::make_alloc_box();
        let mut mgr = ScopeManager::new(alloc_box,);
        let x = test_utils::make_num(1.);
        let x_bnd = x.binding.clone();
        assert!(mgr.store(x, None).is_ok());

        let load = mgr.load(&x_bnd);
        assert!(load.is_ok());
        let (var, ptr) = load.unwrap();

        assert!(matches!(var.t, JsType::JsNum(1.)));
        assert!(ptr.is_none());
    }

}
