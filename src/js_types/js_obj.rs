use std::collections::hash_map::HashMap;
use std::vec::Vec;
use js_types::js_type::{JsType, JsT};

pub type JsProto = Option<Box<JsObj>>;

macro_rules! hash_map {
    ( $( $k:expr => $v:expr),* ) => {{
            let mut temp_hash_map = HashMap::new();
            $(
                temp_hash_map.insert($k, $v);
            )*
            temp_hash_map
    }}
}

pub struct JsObj {
    proto: JsProto,
    dict: HashMap<JsType, JsType>,
}

impl JsT for JsObj {}

impl JsObj {
    pub fn new(proto: JsProto, kv_pairs: Vec<(JsType, JsType)>) -> JsObj {
        let mut obj_map = HashMap::new();
        kv_pairs.into_iter().map(|(k,v)| obj_map.insert(k, v));
        JsObj {
            proto: None,
            dict: obj_map,
        }
    }
}
