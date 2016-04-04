#![feature(box_patterns, box_syntax)]

extern crate french_press;
extern crate jsrs_common;
extern crate rustc_serialize;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use french_press::*;
use french_press::alloc::AllocBox;
use jsrs_common::ast::Exp;
use jsrs_common::backend::Backend;
use jsrs_common::types::js_fn::JsFnStruct;
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsKey, JsPtrEnum, JsPtrTag, JsType, JsVar};

fn main() {
    // TODO docopt
    let mut args = env::args();
    if args.len() != 2 {
        panic!("Usage: ./mem benchmark_name");
    }
    let bench = args.nth(1).unwrap();
    match &*bench {
        "init_only" => init_only(),
        "push_scope" => push_scope(),
        "push_pop_no_gc" => push_pop_no_gc(),
        "push_pop_gc" => push_pop_gc(),
        "small_stack_alloc_no_gc" => small_stack_alloc_no_gc(),
        "small_heap_alloc_no_gc" => small_heap_alloc_no_gc(),
        "small_heap_alloc_no_gc_2" => small_heap_alloc_no_gc_2(),
        "large_alloc_no_gc_flat_obj" => large_alloc_no_gc_flat_obj(),
        "small_local_store" => small_local_store(),
        "shallow_load" => shallow_load(),
        "deep_load" => deep_load(),
        "small_alloc_gc" => small_alloc_gc(),
        "small_flat_heap_alloc_gc" => small_flat_heap_alloc_gc(),
        "large_flat_alloc_gc" => large_flat_alloc_gc(),
        "leak_many" => leak_many(),
        _ => panic!("Invalid benchmark"),
    }
}

fn init_only() {
    init_gc();
}

fn push_scope() {
    let mut mgr = init_gc();
    let exp = Exp::Undefined;
    mgr.push_scope(&exp);
}


fn push_pop_no_gc() {
    let mut mgr = init_gc();
    let exp = Exp::Undefined;
    mgr.push_scope(&exp);
    mgr.pop_scope(None, false);
}


fn push_pop_gc() {
    let mut mgr = init_gc();
    let exp = Exp::Undefined;
    mgr.push_scope(&exp);
    mgr.pop_scope(None, true);
}


fn small_stack_alloc_no_gc() {
    let mut mgr = init_gc();
    mgr.alloc(make_num(0.), None).unwrap();
}


fn small_heap_alloc_no_gc() {
    let mut mgr = init_gc();
    let (var, ptr) = make_str("");
    mgr.alloc(var, Some(ptr)).unwrap();
}

fn small_heap_alloc_no_gc_2() {
    let mut mgr = init_gc();
    let (var, ptr) = make_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    mgr.alloc(var, Some(ptr)).unwrap();
}


fn large_alloc_no_gc_flat_obj() {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None),
                   (JsKey::JsSym("1".to_string()), make_num(1.), None),
                   (JsKey::JsSym("2".to_string()), make_num(2.), None),
                   (JsKey::JsSym("3".to_string()), make_num(3.), None),
                   (JsKey::JsSym("4".to_string()), make_num(4.), None),
                   (JsKey::JsSym("5".to_string()), make_num(5.), None),
                   (JsKey::JsSym("6".to_string()), make_num(6.), None),
                   (JsKey::JsSym("7".to_string()), make_num(7.), None),
                   (JsKey::JsSym("8".to_string()), make_num(8.), None),
                   (JsKey::JsSym("9".to_string()), make_num(9.), None),
                   (JsKey::JsSym("10".to_string()), make_num(10.), None)];
    for _ in 0..100 {
        let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
        mgr.alloc(var, Some(ptr)).unwrap();
    }
}


fn small_local_store() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let (bnd, unique) = (var.binding.clone(), var.unique.clone());
    mgr.alloc(var.clone(), None).unwrap();
    mgr.store(var.clone(), None).unwrap();
}


fn shallow_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    mgr.load(&bnd).unwrap();
}


fn deep_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..100 {
        mgr.push_scope(&Exp::Undefined);
    }
    mgr.load(&bnd).unwrap();
}


fn small_alloc_gc() {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None)];
    let exp = &Exp::Call(box Exp::Undefined, vec![]);
    mgr.push_scope(&exp);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}


fn small_flat_heap_alloc_gc() {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None),
                   (JsKey::JsSym("1".to_string()), make_num(1.), None),
                   (JsKey::JsSym("2".to_string()), make_num(2.), None),
                   (JsKey::JsSym("3".to_string()), make_num(3.), None),
                   (JsKey::JsSym("4".to_string()), make_num(4.), None),
                   (JsKey::JsSym("5".to_string()), make_num(5.), None),
                   (JsKey::JsSym("6".to_string()), make_num(6.), None),
                   (JsKey::JsSym("7".to_string()), make_num(7.), None),
                   (JsKey::JsSym("8".to_string()), make_num(8.), None),
                   (JsKey::JsSym("9".to_string()), make_num(9.), None),
                   (JsKey::JsSym("10".to_string()), make_num(10.), None)];
    let exp = Exp::Call(box Exp::Undefined, vec![]);
    mgr.push_scope(&exp);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}


fn large_flat_alloc_gc() {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None),
                   (JsKey::JsSym("1".to_string()), make_num(1.), None),
                   (JsKey::JsSym("2".to_string()), make_num(2.), None),
                   (JsKey::JsSym("3".to_string()), make_num(3.), None),
                   (JsKey::JsSym("4".to_string()), make_num(4.), None),
                   (JsKey::JsSym("5".to_string()), make_num(5.), None),
                   (JsKey::JsSym("6".to_string()), make_num(6.), None),
                   (JsKey::JsSym("7".to_string()), make_num(7.), None),
                   (JsKey::JsSym("8".to_string()), make_num(8.), None),
                   (JsKey::JsSym("9".to_string()), make_num(9.), None),
                   (JsKey::JsSym("10".to_string()), make_num(10.), None)];
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    for _ in 0..100 {
        let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
        mgr.alloc(var, Some(ptr)).unwrap();
    }
    mgr.pop_scope(None, true).unwrap();
}


fn leak_many() {
    let mut mgr = init_gc();

    let (var, ptr) = make_str("test");
    let key = JsKey::JsSym("true".to_string());
    let kvs = vec![(key.clone(), var, Some(ptr))];

    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    let bnd = var.binding.clone();
    mgr.alloc(var.clone(), Some(ptr.clone())).unwrap();

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
    mgr.pop_scope(None, true).unwrap();
}

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
