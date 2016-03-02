use std::error::Error;
use std::fmt;
use std::result;

use js_types::binding::Binding;
use js_types::js_var::{JsPtrEnum, JsVar};

#[derive(Debug)]
pub enum GcError {
    Alloc(Binding),
    HeapUpdate,
    Load(Binding),
    PtrAlloc,
    Scope,
    Store(JsVar, Option<JsPtrEnum>),
}

pub type Result<T> = result::Result<T, GcError>;

impl fmt::Display for GcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GcError::Alloc(ref bnd) => write!(f, "Binding {} was already allocated, allocation failed", bnd),
            GcError::HeapUpdate => write!(f, "Attempted update of invalid heap pointer"),
            GcError::Load(ref bnd) => write!(f, "Lookup of binding {} failed", bnd),
            GcError::PtrAlloc => write!(f, "Attempted allocation of bad pointer"),
            GcError::Scope => write!(f, "Parent scope did not exist"),
            GcError::Store(ref v, ref p) => write!(f, "Invalid store of var {:?}, ptr {:?}", v, p),
        }
    }
}

impl Error for GcError {
    fn description(&self) -> &str {
        match *self {
            GcError::Alloc(_) => "bad alloc",
            GcError::HeapUpdate => "bad ptr update",
            GcError::Load(_)  => "load of invalid ID",
            GcError::PtrAlloc => "bad ptr allocation",
            GcError::Scope    => "no parent scope",
            GcError::Store(_,_) => "store of invalid ID",
        }
    }
}
