use std::string::String;
use std::vec::Vec;

// `string`
#[derive(Clone)]
pub struct JsStrStruct {
    pub text: String,
}

impl JsStrStruct {
    const MAX_STR_LEN: u64 = 9007199254740991; // 2^53 - 1

    pub fn new(s: &str) -> JsStrStruct {
        assert!((s.len() as u64) < JsStrStruct::MAX_STR_LEN);
        JsStrStruct { text: s.to_string(), }
    }
}

