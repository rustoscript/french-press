#![feature(associated_consts)]
#![feature(drain)]

extern crate uuid;
extern crate typed_arena;
mod js_types;
mod alloc;

use std::collections::hash_set::HashSet;
use uuid::Uuid;

use alloc::compartment::Scope;
use js_types::js_type::JsVar;

// TODO Maybe make a scope manager?
pub fn init<F>(callback: F) -> Scope
    where F: Fn() -> HashSet<Uuid> + 'static {
    Scope::new(callback)
}

pub fn alloc(scope: &mut Scope, var: JsVar) -> Uuid {
    scope.alloc(var)
}

pub fn load(scope: &Scope, uuid: Uuid) -> Option<JsVar> {
    scope.get_jst_copy(&uuid)
}

pub fn store(scope: &mut Scope, var: JsVar) -> bool {
    scope.update_var(var)
}

#[test]
fn it_works() {
}
