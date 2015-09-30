use std::collections::hash_map::HashMap;
use js_types::js_type::{JsType, JsThing};

pub struct JsObj {
    proto: Option<Box<JsObj>>,
    dict: HashMap<JsType, JsType>,
}

impl JsThing for JsObj {}

impl JsObj {
    pub fn new() -> JsObj {
        JsObj {
            proto: None,
            dict: HashMap::new(),
        }
    }
}
