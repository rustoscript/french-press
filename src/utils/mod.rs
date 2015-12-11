use std::cell::RefCell;
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use uuid::Uuid;

use alloc::AllocBox;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use js_types::js_type::{JsKey, JsPtrEnum, JsType, JsVar};

pub fn dummy_callback() -> HashSet<Uuid> {
    HashSet::new()
}

pub fn make_str(s: &str) -> JsPtrEnum {
    JsPtrEnum::JsStr(JsStrStruct::new(s))
}

pub fn make_num(i: f64) -> JsVar {
    JsVar::new(JsType::JsNum(i))
}

pub fn make_obj(kvs: Vec<(JsKey, JsVar)>) -> (JsVar, JsPtrEnum) {
    (JsVar::new(JsType::JsPtr),
     JsPtrEnum::JsObj(JsObjStruct::new(None, "test", kvs)))
}

pub fn make_alloc_box() -> Rc<RefCell<AllocBox>> {
    Rc::new(RefCell::new(AllocBox::new()))
}

