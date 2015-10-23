use std::string::String;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Marking {
    White,
    Grey,
    Black,
}

#[derive(Clone)]
pub struct JsT {
    pub binding: Option<String>,
    pub gc_flag: Marking,
    pub uuid: Uuid,
    pub t: JsType,
}

impl JsT {
    pub fn new(t: JsType) -> JsT {
        JsT {
            binding: None,
            gc_flag: Marking::Grey,
            uuid: Uuid::new_v4(),
            t: t,
        }
    }

    pub fn bind(binding: &str, t: JsType) -> JsT {
        JsT {
            binding: Some(String::from(binding)),
            gc_flag: Marking::Grey,
            uuid: Uuid::new_v4(),
            t: t,
        }
    }
}

impl PartialEq for JsT {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsT {}

impl Hash for JsT {
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
pub enum JsType {
    JsUndef,
    JsNull,
    JsNum(f64),
    JsSym(String),
    JsStr(JsStrStruct),
    JsObj(JsObjStruct),
}

// `array`
pub type JsArr = Vec<JsType>;
