#![cfg(test)]
use std::cell::RefCell;
use std::rc::Rc;

use uuid::Uuid;

use alloc::AllocBox;
use jsrs_common::ast::{Exp, Stmt};
use js_types::js_fn::JsFnStruct;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use js_types::js_var::{JsKey, JsPtrEnum, JsPtrTag, JsType, JsVar};
use js_types::binding::Binding;

fn mangle_binding(b: &Binding) -> Binding {
    Binding(String::from("%---") +  &b.0 +  "---%" + &Uuid::new_v4().to_simple_string())
}

pub fn anon_binding() -> Binding {
    mangle_binding(&Binding::new(">anon_js_var<".to_string()))
}

pub fn make_str(s: &str) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsStr));
    (var, JsPtrEnum::JsStr(JsStrStruct::new(s)), anon_binding())
}

pub fn make_num(i: f64) -> JsVar {
    JsVar::new(JsType::JsNum(i))
}

pub fn make_obj(kvs: Vec<(JsKey, JsVar, Option<JsPtrEnum>)>, heap: Rc<RefCell<AllocBox>>) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsObj));
    (var, JsPtrEnum::JsObj(JsObjStruct::new(None, "test", kvs, &mut *heap.borrow_mut())), anon_binding())
}

pub fn make_fn(name: &Option<String>, params: &Vec<String>) -> (JsVar, JsPtrEnum, Binding) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsFn));
    (var, JsPtrEnum::JsFn(JsFnStruct::new(name, params, &Stmt::BareExp(Exp::Undefined))), anon_binding())
}

pub fn make_alloc_box() -> Rc<RefCell<AllocBox>> {
    Rc::new(RefCell::new(AllocBox::new()))
}

