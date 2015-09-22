extern crate num;

mod js_obj;
mod js_type;

use std::string::String;
use std::hash::Hash;
use std::collections::hash_map::HashMap;

pub struct JsUndef;
pub struct JsNull;
pub struct JsBool(bool);
pub struct JsNum(f64);
pub struct JsSym(String);

macro_rules! hash_map {
    ( $( $k:expr => $v:expr),* ) => {{
            let mut temp_hash_map = HashMap::new();
            $(
                temp_hash_map.insert($k, $v);
            )*
            temp_hash_map
    }}
}


pub struct JsStr {
    text: String,
}

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
