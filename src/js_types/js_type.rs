use std::hash::{Hash, Hasher};
use std::string::String;

use uuid::Uuid;

use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;

pub type Binding = Option<String>;

#[derive(Clone, Debug)]
pub struct JsVar {
    pub binding: Binding,
    pub uuid: Uuid,
    pub t: JsType,
}

impl JsVar {
    pub fn new(t: JsType) -> JsVar {
        JsVar {
            binding: None,
            uuid: Uuid::new_v4(),
            t: t,
        }
    }

    pub fn bind(binding: &str, t: JsType) -> JsVar {
        JsVar {
            binding: Some(String::from(binding)),
            uuid: Uuid::new_v4(),
            t: t,
        }
    }
}

impl PartialEq for JsVar {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
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
    pub uuid: Uuid,
    pub k: JsKeyEnum,
}

impl JsKey {
    pub fn new(k: JsKeyEnum) -> JsKey {
        JsKey {
            uuid: Uuid::new_v4(),
            k: k,
        }
    }
}

impl PartialEq for JsKey {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsKey {}

impl Hash for JsKey {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.uuid.hash(state);
    }

    fn hash_slice<H>(data: &[Self], state: &mut H) where H: Hasher {
        for ref d in data {
            d.uuid.hash(state);
        }
    }
}


// `array`
pub type JsArr = Vec<JsType>;
