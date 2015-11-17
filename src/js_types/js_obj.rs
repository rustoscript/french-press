use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::string::String;
use std::vec::Vec;
use uuid::Uuid;
use js_types::js_type::{JsType, JsVar, JsKey};
use alloc::Alloc;

#[derive(Clone)]
pub struct JsObjStruct {
    pub proto: JsProto,
    pub name: String,
    pub dict: HashMap<JsKey, JsVar>,
}

impl JsObjStruct {
    pub fn new(proto: JsProto, name: &str, kv_pairs: Vec<(JsKey, JsVar)>) -> JsObjStruct {
        JsObjStruct {
            proto: None,
            name: String::from(name),
            dict: kv_pairs.into_iter().collect(),
        }
    }

    pub fn add_key(&mut self, k: JsKey, v: JsVar) {
        self.dict.insert(k, v);
    }

    pub fn get_children(&self) -> HashSet<Uuid> {
        self.dict.values().map(|v| v.uuid).collect()
    }
}

pub type JsProto = Option<Box<JsObjStruct>>;

// TODO nice JS object creation macro
//macro_rules! js_obj {
//    ( $kt:ty : $ke:expr => $vt:ty : $ve:expr ),* {
//        {
//
//        }
//    };
//}


#[cfg(test)]
mod tests {
    use super::*;
    use js_types::js_type::{JsType,JsVar};
    use js_types::js_str::{JsStrStruct};

    #[test]
    fn test_js_obj() {
        /*let mut vec: Vec<(JsVar, JsVar)> = Vec::new();
        for i in 0..10 {
            let k = JsVar::new(JsType::JsNum(i as f64));
            let v = JsVar::new(JsType::JsStr(JsStrStruct::new(
                                            &format!("test{}", i))));
            vec.push((k,v));
        }
        let o = JsObjStruct::new(None, "test", vec);
        for (k, v) in o.dict {
            match k.t {
                JsType::JsNum(ki) => { assert!(ki >= 0.0f64);
                                       assert!(ki < 10.0f64) },
                _ => panic!("Expected a JsNum!"),
            };
            match v.t {
                JsType::JsStr(vs) => assert!(vs.text.starts_with("test")),
                _ => panic!("Expected a JsStr!"),
            };
        }
        assert_eq!(&o.name, "test");*/
    }

}
