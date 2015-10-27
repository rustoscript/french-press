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

impl Hash for JsVar {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.uuid.hash(state);
    }

    fn hash_slice<H>(data: &[Self], state: &mut H) where H: Hasher {
        for ref d in data {
            d.uuid.hash(state);
        }
    }
}

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

// `array`
pub type JsArr = Vec<JsType>;
