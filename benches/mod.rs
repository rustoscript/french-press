#![feature(test)]
#![feature(box_patterns, box_syntax)]
extern crate test;
extern crate french_press;
extern crate jsrs_common;

use std::cell::RefCell;
use std::rc::Rc;

use test::Bencher;
use french_press::*;
use french_press::alloc::AllocBox;
use jsrs_common::ast::{Exp, Stmt};
use jsrs_common::backend::Backend;
use jsrs_common::types::js_fn::JsFnStruct;
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsKey, JsPtrEnum, JsPtrTag, JsType, JsVar};

fn make_num(i: f64) -> JsVar {
    JsVar::new(JsType::JsNum(i))
}

fn make_str(s: &str) -> (JsVar, JsPtrEnum) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsStr));
    (var, JsPtrEnum::JsStr(JsStrStruct::new(s)))
}

fn make_obj(kvs: Vec<(JsKey, JsVar, Option<JsPtrEnum>)>, heap: Rc<RefCell<AllocBox>>) -> (JsVar, JsPtrEnum) {
    let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsObj));
    (var, JsPtrEnum::JsObj(JsObjStruct::new(None, "test", kvs, &mut *heap.borrow_mut())))
}

//#[bench]
fn small_no_gc(b: &mut Bencher) {
    b.iter(|| {
        let mut mgr = init_gc();
        mgr.alloc(make_num(0.), None).unwrap();
        mgr.alloc(make_num(1.), None).unwrap();
        mgr.alloc(make_num(2.), None).unwrap();
        mgr.pop_scope(None, false).unwrap();
    });
}

//#[bench]
fn deep_lookup(b: &mut Bencher) {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..1000 {
        mgr.push_scope(&Exp::Undefined);
    }
    b.iter(|| {
        mgr.load(&bnd).unwrap();
    });
}

#[bench]
fn leak_many(b: &mut Bencher) {
    let mut mgr = init_gc();
    for _ in 0..100 {
        mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));

        let (var, ptr) = make_str("test");
        let key = JsKey::JsSym("true".to_string());
        let kvs = vec![(key.clone(), var, Some(ptr))];

        let (var, ptr) = make_obj(kvs, mgr.alloc_box.clone());
        let bnd = var.binding.clone();
        mgr.alloc(var, Some(ptr)).unwrap();

        let copy = mgr.load(&bnd);
        let (mut var, mut ptr) = copy.unwrap();

        for _ in 0..1000 {
            let (leak_var, leak_ptr) = make_str("test");
            match *&mut ptr {
                Some(JsPtrEnum::JsObj(ref mut obj)) => {
                    obj.add_key(key.clone(), leak_var, Some(leak_ptr), &mut *(mgr.alloc_box.borrow_mut()));
                },
                _ => unreachable!()
            }
        }
        mgr.store(var, ptr).unwrap();
    }
    let f = || { mgr.pop_scope(None, true).unwrap(); };
    b.iter(f);
}
