extern crate num;
use std::string::String;
use std::hash::Hash;
use std::collections::hash_map::HashMap;

pub trait JsType {}

pub struct JsUndef;
impl JsType for JsUndef {}
pub struct JsNull;
impl JsType for JsNull {}
pub struct JsBool(bool);
impl JsType for JsBool {}
pub struct JsNum(f64);
impl JsType for JsNum {}
pub struct JsSym(String);
impl JsType for JsSym {}

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
    proto: Option<Box<JsObj>>,
    dict: HashMap<Box<JsType>, Box<JsType>>,
}

impl JsType for JsObj {}

impl JsObj {
    pub fn new() -> JsObj {
        JsObj {
            proto: None,
            dict: HashMap::new(),
        }
    }
}

pub struct JsStr {
    text: String,
}

impl JsType for JsStr {}

impl JsStr {
    //const MAX_STR_LEN: u64 = num::pow(2u64, 53) - 1;

    fn new(s: &str) -> JsStr {
        //assert!((s.len() as u64) < JsStr::MAX_STR_LEN);
        JsStr { text: s.to_string(), }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_hash_map_macro() {
        let h = hash_map!("a" => 1, "b" => 2);
        assert_eq!(h["a"], 1);
        assert_eq!(h["b"], 2);
    }
}
