use std::error::Error;
use std::fmt;

use uuid::Uuid;

use js_types::js_type::Binding;

#[derive(Debug)]
pub enum GcError {
    AllocError(Uuid),
    LoadError(Binding),
    PtrError,
    ScopeError,
    StoreError,
}

impl fmt::Display for GcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GcError::AllocError(id) => write!(f, "UUID {} was already allocated, allocation failed!", id),
            GcError::LoadError(ref bnd) => {
                match bnd {
                    &Some(ref s) => write!(f, "Lookup of binding {} failed!", s),
                    &None => write!(f, "Lookup of binding None failed! (How was that possible?)"),
                }
            }
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
