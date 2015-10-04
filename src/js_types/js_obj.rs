use std::collections::btree_map::BTreeMap;
use js_types::js_type::{JsType, JsT};

pub struct JsObj {
    proto: Option<Box<JsObj>>,
    dict: BTreeMap<JsType, JsType>,
}

impl JsT for JsObj {}

impl JsObj {
    pub fn new() -> JsObj {
        JsObj {
            proto: None,
            dict: BTreeMap::new(),
        }
    }
}
