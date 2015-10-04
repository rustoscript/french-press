mod js_obj;
mod js_type;

use std::string::String;
use std::vec::Vec;

// `undefined`
pub struct JsUndef;
// `null`
pub struct JsNull;
// `bool`
pub struct JsBool(bool);
// `number`
pub struct JsNum(f64);
// `symbol`
pub struct JsSym(String);
// `array`
pub type JsArr<JsT> = Vec<Box<JsT>>;

// `string`
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

