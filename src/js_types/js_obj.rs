use std::collections::hash_map::HashMap;
use std::vec::Vec;
use js_types::js_type::{JsType,JsT};

pub struct JsObjStruct {
    pub proto: JsProto,
    pub dict: HashMap<JsT, JsT>,
}

impl JsObjStruct {
    pub fn new(proto: JsProto, kv_pairs: Vec<(JsT, JsT)>) -> JsObjStruct {
        let mut obj_map = HashMap::new();
        kv_pairs.into_iter().map(|(k,v)| obj_map.insert(k, v));
        JsObjStruct {
            proto: None,
            dict: obj_map,
        }
    }
}

pub type JsProto = Option<Box<JsObjStruct>>;

#[cfg(test)]
mod test {
    use js_types::js_obj::JsObjStruct;
    use js_types::js_type::{JsType,JsT};
    use js_types::js_str::JsStrStruct;

    #[test]
    fn test_js_obj() {
        let mut vec: Vec<(JsT, JsT)> = Vec::new();
        for i in 0..10 {
            let k = JsT::new(JsType::JsNum(i as f64));
            let v = JsT::new(JsType::JsStr(JsStrStruct::new(
                                            &format!("test{}", i))));
            vec.push((k,v));
        }
        let o = JsObjStruct::new(None, vec);
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
    }

}
