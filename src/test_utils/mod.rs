#![cfg(test)]
use std::cell::RefCell;
use std::rc::Rc;

use alloc::AllocBox;
use jsrs_common::ast::{Exp, Stmt};
use js_types::js_fn::JsFnStruct;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use js_types::js_var::{JsKey, JsPtrEnum, JsPtrTag, JsType, JsVar};
use js_types::binding::Binding;

pub fn make_str(s: &str) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsStr));
    let bnd = var.binding.clone();
    (var, JsPtrEnum::JsStr(JsStrStruct::new(s)), bnd)
}

pub fn make_num(i: f64) -> JsVar {
    JsVar::new(JsType::JsNum(i))
}

pub fn make_obj(kvs: Vec<(JsKey, JsVar, Option<JsPtrEnum>)>, heap: Rc<RefCell<AllocBox>>) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsObj));
    let bnd = var.binding.clone();
    (var, JsPtrEnum::JsObj(JsObjStruct::new(None, "test", kvs, &mut *heap.borrow_mut())), bnd)
}

pub fn make_fn(name: &Option<String>, params: &Vec<String>) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsFn));
    let bnd = var.binding.clone();
    (var, JsPtrEnum::JsFn(JsFnStruct::new(name, params, &Stmt::BareExp(Exp::Undefined))), bnd)
}

pub fn make_alloc_box() -> Rc<RefCell<AllocBox>> {
    Rc::new(RefCell::new(AllocBox::new()))
}

