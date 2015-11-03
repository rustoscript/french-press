use std::string::String;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Clone)]
pub struct JsVar {
    pub binding: Option<String>,
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

#[derive(Clone)]
pub enum JsPtrEnum {
    JsSym(String),
    JsStr(JsStrStruct),
    JsObj(JsObjStruct),
}

#[derive(Clone)]
pub enum JsType {
    JsUndef,
    JsNull,
    JsNum(f64),
    JsPtr(JsPtrEnum),
}

#[derive(Clone)]
pub enum JsKeyEnum {
    JsNum(f64),
    JsStr(JsStrStruct),
    JsSym(String),
}

#[derive(Clone)]
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
