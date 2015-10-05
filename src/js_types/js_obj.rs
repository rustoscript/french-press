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

pub type JsProto = Option<Box<JsObj>>;

#[cfg(test)]
mod test {
    use js_types::js_obj::JsObj;
    use js_types::js_type::JsT;
    use js_types::js_primitive::{JsNum, JsStr};
    #[test]
    fn test_js_obj() {
        let mut vec: Vec<(JsT, JsT)> = Vec::new();
        for i in 0..10 {
            let k = JsT::new(Box::new(JsNum(i as f64)));
            let v = JsT::new(Box::new(JsStr::new(&format!("test{}", i))));
            vec.push((k,v));
        }
        let o = JsObj::new(None, vec);
        for (k, v) in o.dict {
            assert!(k >= 0); // Problem: is this type erased at runtime?
            assert!(k < 10);
            assert!(v.starts_with("test"));
        }
    }

}
