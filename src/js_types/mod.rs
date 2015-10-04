mod js_obj;
mod js_type;

use std::string::String;

pub struct JsUndef;
pub struct JsNull;
pub struct JsBool(bool);
pub struct JsNum(f64);
pub struct JsSym(String);

pub struct JsStr {
    text: String,
}

impl JsStr {
    const MAX_STR_LEN: u64 = 9007199254740991; // 2^53 - 1

    fn new(s: &str) -> JsStr {
        assert!((s.len() as u64) < JsStr::MAX_STR_LEN);
        JsStr { text: s.to_string(), }
    }
}

