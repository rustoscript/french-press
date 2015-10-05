use std::collections::hash_map::HashMap;
use std::vec::Vec;
use js_types::js_type::{JsTrait, JsT};


pub struct JsObj {
    proto: JsProto,
    dict: HashMap<JsT, JsT>,
}

impl JsTrait for JsObj {}

impl JsObj {
    pub fn new(proto: JsProto, kv_pairs: Vec<(JsT, JsT)>) -> JsObj {
        let mut obj_map = HashMap::new();
        kv_pairs.into_iter().map(|(k,v)| obj_map.insert(k, v));
        JsObj {
            proto: None,
            dict: obj_map,
        }
    }
}
