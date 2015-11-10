#![feature(associated_consts)]
#![feature(drain)]

extern crate uuid;
extern crate typed_arena;
mod js_types;
mod alloc;

use std::collections::hash_set::HashSet;
use std::ptr::null_mut;
use std::rc::Rc;
use uuid::Uuid;

use alloc::scope::Scope;
use js_types::js_type::JsVar;

pub struct ScopeManager {
    root_scope: Rc<Scope>,
    curr_scope: *mut Rc<Scope>,
}

impl ScopeManager {
    pub fn new<F>(callback: F) -> ScopeManager where F: Fn() -> HashSet<Uuid> + 'static {
        let scope = Rc::new(Scope::new(callback));
        let mut mgr =
            ScopeManager {
                root_scope: scope,
                curr_scope: null_mut(),
            };
        mgr.curr_scope = &mut (mgr.root_scope) as *mut Rc<Scope>;
        mgr
    }

    pub fn push_scope<F>(&mut self, callback: F) where F: Fn() -> HashSet<Uuid> + 'static {
        unsafe {
            let weak_clone = Rc::downgrade(&*self.curr_scope.clone());
            self.curr_scope =
                Rc::get_mut(&mut *self.curr_scope)
                    .unwrap()
                    .add_child(Scope::as_child(weak_clone, callback)) as *mut Rc<Scope>;
        }
    }

    pub fn pop_scope(&mut self) {
        let parent = unsafe { (*self.curr_scope).parent.clone() };
        if let Some(parent) = parent {
            if let Some(mut scope) = parent.upgrade() {
                // Pop old scope
                let num_children = scope.children.len();
                Rc::get_mut(&mut scope).unwrap().children.remove(num_children - 1);
                // Set curr_scope to old scope's parent
                self.curr_scope = &mut scope as *mut Rc<Scope>;
            }
        } else {
            panic!("Tried to pop to parent scope, but parent did not exist!");
        }
    }

    pub fn alloc(&mut self, var: JsVar) -> Uuid {
        unsafe { Rc::get_mut(&mut *self.curr_scope).unwrap().alloc(var) }
    }

    pub fn load(&self, uuid: Uuid) -> Option<JsVar> {
        unsafe { (*self.curr_scope).get_var_copy(&uuid) }
    }

    pub fn store(&mut self, var: JsVar) -> bool {
        unsafe { Rc::get_mut(&mut *self.curr_scope).unwrap().update_var(var) }
    }
}

pub fn init_gc<F>(callback: F) -> ScopeManager
    where F: Fn() -> HashSet<Uuid> + 'static {
    ScopeManager::new(callback)
}

