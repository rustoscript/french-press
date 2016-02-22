use std::error::Error;
use std::fmt;
use std::result;

use js_types::binding::Binding;

#[derive(Debug)]
pub enum GcError {
    Alloc(Binding),
    Load(Binding),
    Ptr,
    Scope,
    Store,
}

pub type Result<T> = result::Result<T, GcError>;

impl fmt::Display for GcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GcError::Alloc(ref bnd) => write!(f, "Binding {} was already allocated, allocation failed!", bnd),
            GcError::Load(ref bnd) => write!(f, "Lookup of binding {} failed!", bnd),
            GcError::Ptr => write!(f, "Attempted allocation of invalid heap pointer"),
            GcError::Scope => write!(f, "Parent scope did not exist"),
            GcError::Store => write!(f, "Invalid store!"), // TODO update this error
        }
    }
}

impl Error for GcError {
    fn description(&self) -> &str {
        match *self {
            GcError::Alloc(_) => "bad alloc",
            GcError::Load(_)  => "load of invalid ID",
            GcError::Ptr      => "bad ptr",
            GcError::Scope    => "no parent scope",
            GcError::Store    => "store of invalid ID",
        }
    }
}
