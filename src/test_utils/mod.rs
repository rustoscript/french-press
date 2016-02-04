#![cfg(test)]
use std::cell::RefCell;
use std::rc::Rc;

use alloc::AllocBox;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use js_types::js_var::{JsKey, JsPtrEnum, JsType, JsVar};
use js_types::binding::Binding;

pub fn make_str(s: &str) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr);
    let bnd = var.binding.clone();
    (var, JsPtrEnum::JsStr(JsStrStruct::new(s)), bnd)
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

