use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::string::String;
use std::vec::Vec;

use uuid::Uuid;

use js_types::js_type::{JsVar, JsKey};

#[derive(Clone, Debug)]
pub struct JsObjStruct {
    pub proto: JsProto,
    pub name: String,
    pub dict: HashMap<JsKey, JsVar>,
}

impl JsObjStruct {
    pub fn new(proto: JsProto, name: &str, kv_pairs: Vec<(JsKey, JsVar)>) -> JsObjStruct {
        JsObjStruct {
            proto: None,
            name: String::from(name),
            dict: kv_pairs.into_iter().collect(),
        }
    }

    pub fn add_key(&mut self, k: JsKey, v: JsVar) {
        self.dict.insert(k, v);
    }

    pub fn get_children(&self) -> HashSet<Uuid> {
        self.dict.values().map(|v| v.uuid).collect()
    }
}

pub type JsProto = Option<Box<JsObjStruct>>;

// TODO nice JS object creation macro
//macro_rules! js_obj {
//    ( $kt:ty : $ke:expr => $vt:ty : $ve:expr ),* {
//        {
//
//        }
//    };
//}


#[cfg(test)]
mod tests {
    // TODO tests for objs
}
