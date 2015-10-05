use std::string::String;
use std::vec::Vec;
use js_types::js_type::{JsTrait, JsT};

// `undefined`
pub struct JsUndef;
impl JsTrait for JsUndef {}

// `null`
pub struct JsNull;
impl JsTrait for JsNull {}

// `bool`
pub struct JsBool(pub bool);
impl JsTrait for JsBool {}

// `number`
pub struct JsNum(pub f64);
impl JsTrait for JsNum {}

// `symbol`
pub struct JsSym(pub String);
impl JsTrait for JsSym {}

// `array`
pub type JsArr<T> = Vec<T>;
impl<T> JsTrait for JsArr<T> {}

// `string`
pub struct JsStr {
    text: String,
}

impl JsStr {
    const MAX_STR_LEN: u64 = 9007199254740991; // 2^53 - 1

    pub fn new(s: &str) -> JsStr {
        assert!((s.len() as u64) < JsStr::MAX_STR_LEN);
        JsStr { text: s.to_string(), }
    }
}

impl JsTrait for JsStr {}

