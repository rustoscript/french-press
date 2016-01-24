use std::fmt;
use std::hash::{Hash, Hasher};
use std::string::String;

use uuid::Uuid;

use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Binding(String);

#[derive(Clone, Debug)]
pub struct JsVar {
    pub binding: Binding,
    pub t: JsType,
}

impl Binding {
    pub fn new(s: &str) -> Binding {
        Binding(s.to_string())
    }

    pub fn mangle(s: &str) -> Binding {
        Binding(String::from("%___") +  s +  "___" + &Uuid::new_v4().to_simple_string())
    }

    pub fn anon() -> Binding {
        Binding::mangle(">anon_js_var<")
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl JsVar {
    pub fn new(t: JsType) -> JsVar {
        JsVar {
            binding: Binding::anon(),
            t: t,
        }
    }

    pub fn bind(binding: &str, t: JsType) -> JsVar {
        JsVar {
            binding: Binding::new(binding),
            t: t,
        }
    }
}

impl PartialEq for JsVar {
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsVar {}

#[derive(Clone, Debug)]
pub enum JsPtrEnum {
    JsSym(String),
    JsStr(JsStrStruct),
    JsObj(JsObjStruct),
}

#[derive(Clone, Debug)]
pub enum JsType {
    JsUndef,
    JsNum(f64),
    JsBool(bool),
    JsPtr,
    JsNull, // null is not a ptr since it doesn't actually require allocation
}

impl PartialEq for JsType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&JsType::JsUndef, &JsType::JsUndef) => true,
            (&JsType::JsNum(x), &JsType::JsNum(y)) => x == y,
            (&JsType::JsBool(b1), &JsType::JsBool(b2)) => b1 == b2,
            (&JsType::JsNull, &JsType::JsNull) => true,
            (_, _) => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsType {}

#[derive(Clone, Debug)]
pub enum JsKeyEnum {
    JsNum(f64),
    JsBool(bool),
    JsStr(JsStrStruct),
    JsSym(String),
}

#[derive(Clone, Debug)]
pub struct JsKey {
    pub binding: Binding,
    pub k: JsKeyEnum,
}

impl JsKey {
    pub fn new(k: JsKeyEnum) -> JsKey {
        JsKey {
            binding: Binding::anon(),
            k: k,
        }
    }
}

impl PartialEq for JsKey {
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsKey {}

impl Hash for JsKey {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.binding.hash(state);
    }

    fn hash_slice<H>(data: &[Self], state: &mut H) where H: Hasher {
        for ref d in data {
            d.binding.hash(state);
        }
    }
}


// `array`
pub type JsArr = Vec<JsType>;

