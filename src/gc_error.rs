use std::error::Error;
use std::fmt;
use std::result;

use js_types::binding::Binding;

#[derive(Debug)]
pub enum GcError {
    AllocError(Binding),
    LoadError(Binding),
    PtrError,
    ScopeError,
    StoreError,
}

pub type Result<T> = result::Result<T, GcError>;

impl fmt::Display for GcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GcError::AllocError(ref bnd) => write!(f, "Binding {} was already allocated, allocation failed!", bnd),
            GcError::LoadError(ref bnd) => write!(f, "Lookup of binding {} failed!", bnd),
            GcError::PtrError => write!(f, "Attempted allocation of invalid heap pointer"),
            GcError::ScopeError => write!(f, "Parent scope did not exist"),
            GcError::StoreError => write!(f, "Invalid store!"), // TODO update this error
        }
    }
}

impl Error for GcError {
    fn description(&self) -> &str {
        match *self {
            GcError::AllocError(_) => "bad alloc",
            GcError::LoadError(_) => "load of invalid ID",
            GcError::PtrError => "bad ptr",
            GcError::ScopeError => "no parent scope",
            GcError::StoreError => "store of invalid ID",
        }
    }
}
