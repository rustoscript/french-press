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

#[bench]
fn init_only(b: &mut Bencher) {
    b.iter(|| {
        init_gc();
    });
}

#[bench]
fn push_scope(b: &mut Bencher) {
    let mut mgr = init_gc();
    let exp = Exp::Undefined;
    b.iter(|| {
        mgr.push_scope(&exp);
    });
}

#[bench]
fn push_pop_no_gc(b: &mut Bencher) {
    let mut mgr = init_gc();
    let exp = Exp::Undefined;
    b.iter(|| {
        mgr.push_scope(&exp);
        mgr.pop_scope(None, false);
    });
}

#[bench]
fn push_pop_gc(b: &mut Bencher) {
    let mut mgr = init_gc();
    let exp = Exp::Undefined;
    b.iter(|| {
        mgr.push_scope(&exp);
        mgr.pop_scope(None, true);
    });
}

#[bench]
fn small_stack_alloc_no_gc(b: &mut Bencher) {
    let mut mgr = init_gc();
    b.iter(|| {
        mgr.alloc(make_num(0.), None).unwrap();
    });
}

#[bench]
fn small_heap_alloc_no_gc(b: &mut Bencher) {
    let mut mgr = init_gc();
    b.iter(|| {
        let (var, ptr) = make_str("");
        mgr.alloc(var, Some(ptr)).unwrap();
    });
}
#[bench]
fn small_heap_alloc_no_gc_2(b: &mut Bencher) {
    let mut mgr = init_gc();
    b.iter(|| {
        let (var, ptr) = make_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        mgr.alloc(var, Some(ptr)).unwrap();
    });
}

#[bench]
fn large_alloc_no_gc_flat_obj(b: &mut Bencher) {
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
    b.iter(|| {
        for _ in 0..100 {
            let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
            mgr.alloc(var, Some(ptr)).unwrap();
        }
    });
}

#[bench]
fn small_local_store(b: &mut Bencher) {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let (bnd, unique) = (var.binding.clone(), var.unique.clone());
    mgr.alloc(var.clone(), None).unwrap();
    b.iter(|| {
        mgr.store(var.clone(), None).unwrap();
    });
}

#[bench]
fn shallow_load(b: &mut Bencher) {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    b.iter(|| {
        mgr.load(&bnd).unwrap();
    });
}

#[bench]
fn deep_load(b: &mut Bencher) {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..100 {
        mgr.push_scope(&Exp::Undefined);
    }
    b.iter(|| {
        mgr.load(&bnd).unwrap();
    });
}

#[bench]
fn small_alloc_gc(b: &mut Bencher) {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None)];
    let exp = &Exp::Call(box Exp::Undefined, vec![]);
    b.iter(|| {
        mgr.push_scope(&exp);
        let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
        mgr.alloc(var, Some(ptr)).unwrap();
        mgr.pop_scope(None, true).unwrap();
    });
}

#[bench]
fn small_flat_heap_alloc_gc(b: &mut Bencher) {
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
    b.iter(|| {
        mgr.push_scope(&exp);
        let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
        mgr.alloc(var, Some(ptr)).unwrap();
        mgr.pop_scope(None, true).unwrap();
    });
}

#[bench]
fn large_flat_alloc_gc(b: &mut Bencher) {
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
    b.iter(|| {
        mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
        for _ in 0..100 {
            let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
            mgr.alloc(var, Some(ptr)).unwrap();
        }
        mgr.pop_scope(None, true).unwrap();
    });
}

#[bench]
fn leak_many(b: &mut Bencher) {
    let mut mgr = init_gc();

    let (var, ptr) = make_str("test");
    let key = JsKey::JsSym("true".to_string());
    let kvs = vec![(key.clone(), var, Some(ptr))];

    b.iter(|| {
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
    });
}
