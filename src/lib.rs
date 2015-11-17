#![feature(associated_consts)]
#![feature(drain)]
#![feature(rc_would_unwrap)]

extern crate uuid;
extern crate typed_arena;
mod js_types;
mod alloc;

use std::collections::hash_set::HashSet;
use std::ptr::null_mut;
use std::rc::{Rc, Weak};
use uuid::Uuid;

use alloc::scope::Scope;
use js_types::js_type::JsVar;

pub struct ScopeManager {
    root_scope: Rc<Scope>,
    //curr_scope: Weak<Scope>,
    pub size: usize,
}

/*impl ScopeManager {
    pub fn new<F>(callback: F) -> ScopeManager
        where F: Fn() -> HashSet<Uuid> + 'static {
        let mut mgr =
            ScopeManager {
                root_scope: Rc::new(Scope::new(callback)),
                curr_scope: Rc::downgrade(&Rc::new(Scope::new(callback))),
                size: 1,
            };
        mgr.curr_scope = Rc::downgrade(&mgr.root_scope.clone());
        mgr
    }

    pub fn push_scope<F>(&mut self, callback: F) where F: Fn() -> HashSet<Uuid> + 'static {
        let weak_clone = self.curr_scope.clone();
        self.curr_scope =
            // Is this possible?
            if let Some(curr_scope) = self.curr_scope.upgrade() {
                Rc::downgrade(Rc::get_mut(&mut curr_scope)
                                .unwrap()
                                .add_child(Scope::as_child(weak_clone, callback)))
            } else {
                panic!("Unable to upgrade curr_scope to Rc! Underlying data was destroyed!");
            };
        self.size += 1;
    }

    pub fn pop_scope(&mut self) {
        let parent = self.curr_scope.parent.clone();
        if let Some(parent) = parent {
            if let Some(mut scope) = parent.upgrade() {
                // Pop old scope
                let num_children = scope.children.len();
                Rc::get_mut(&mut scope).unwrap().children.remove(num_children - 1);
                // Set curr_scope to old scope's parent
                self.curr_scope = &mut scope;
                self.size -= 1;
            }
        } else {
            panic!("Tried to pop to parent scope, but parent did not exist!");
        }
    }

    pub fn alloc(&mut self, var: JsVar) -> Uuid {
        Rc::get_mut(self.curr_scope).unwrap().alloc(var)
    }

    pub fn load(&self, uuid: Uuid) -> Option<JsVar> {
        self.curr_scope.get_var_copy(&uuid)
    }

    pub fn store(&mut self, var: JsVar) -> bool {
        Rc::get_mut(self.curr_scope).unwrap().update_var(var)
    }
}

pub fn init_gc<F>(callback: F) -> ScopeManager
    where F: Fn() -> HashSet<Uuid> + 'static {
    ScopeManager::new(callback)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_set::HashSet;
    use std::ptr::null_mut;
    use std::rc::Rc;
    use uuid::Uuid;

    fn dummy_callback() -> HashSet<Uuid> {
        HashSet::new()
    }

    #[test]
    fn test_init_gc() {
        let mgr = init_gc(dummy_callback);
        //assert_eq!(mgr.size, 1);
        //assert!(mgr.curr_scope != null_mut());
        //assert!(mgr.root_scope.parent.is_none());
        unsafe { assert!(Rc::would_unwrap(&*mgr.curr_scope)); }
    }

    #[test]
    fn test_push_scope() {
        let mut mgr = init_gc(dummy_callback);
        mgr.push_scope(dummy_callback);
    }
}*/
