#![feature(box_patterns, box_syntax)]

extern crate env_logger;
extern crate french_press;
extern crate heapsize;
extern crate jsrs_common;
#[macro_use]
extern crate log;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use french_press::*;
use french_press::alloc::AllocBox;
use french_press::scope::{Scope, ScopeTag};
use jsrs_common::ast::Exp;
use jsrs_common::backend::Backend;
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsKey, JsPtrEnum, JsPtrTag, JsType, JsVar};

static UNDEF: Exp = Exp::Undefined;

fn main() {
    data_structures();
    return;
    env_logger::init().unwrap();
    info!("Beginning memory profile...");
    // TODO docopt
    let mut args = env::args();
    if args.len() != 2 {
        panic!("Usage: ./mem benchmark_name");
    }
    let bench = args.nth(1).unwrap();
    match &*bench {
        "init_only"               => init_only(),
        "push_scope"              => push_scope(),
        "push_pop_no_gc"          => push_pop_no_gc(),
        "push_pop_gc"             => push_pop_gc(),
        "small_stack_alloc_no_gc" => small_stack_alloc_no_gc(),
        "small_stack_alloc_gc"    => small_stack_alloc_gc(),
        "small_str_alloc_no_gc"   => small_str_alloc_no_gc(),
        "small_str_alloc_gc"      => small_str_alloc_gc(),
        "small_str_alloc_no_gc_2" => small_str_alloc_no_gc_2(),
        "small_str_alloc_gc_2"    => small_str_alloc_gc_2(),
        "small_obj_alloc_no_gc"   => small_obj_alloc_no_gc(),
        "small_obj_alloc_gc"      => small_obj_alloc_gc(),
        "small_obj_alloc_no_gc_2" => small_obj_alloc_no_gc_2(),
        "small_obj_alloc_gc_2"    => small_obj_alloc_gc_2(),
        "large_obj_alloc_no_gc"   => large_obj_alloc_no_gc(),
        "large_obj_alloc_gc"      => large_obj_alloc_gc(),
        "huge_obj_alloc_no_gc"    => huge_obj_alloc_no_gc(),
        "huge_obj_alloc_gc"       => huge_obj_alloc_gc(),
        "shallow_load"            => shallow_load(),
        "deca_load"               => deca_load(),
        "centi_load"              => centi_load(),
        "kilo_load"               => kilo_load(),
        "small_local_store"       => small_local_store(),
        "large_local_store"       => large_local_store(),
        "leak_many_no_gc"         => leak_many_no_gc(),
        "leak_many_gc"            => leak_many_gc(),
        _                         => panic!("Invalid benchmark"),
    }
}

// vv Data Structure Sizes vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
fn data_structures() {
    let ab = Rc::new(RefCell::new(AllocBox::new()));
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    mgr.alloc(make_num(0.), None).unwrap();
    mgr.alloc(make_num(0.), None).unwrap();
    mgr.alloc(make_num(0.), None).unwrap();
    mgr.alloc(make_num(0.), None).unwrap();
    mgr.alloc(make_num(0.), None).unwrap();
}
// ^^ Data Structure Sizes ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

// vv GC-Independent Tests vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
fn init_only() {
    init_gc();
}

fn push_scope() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
}
// ^^ GC-Independent Tests ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Push & Pop Only vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
fn push_pop_no_gc() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    mgr.pop_scope(None, false).unwrap();
}

fn push_pop_gc() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    mgr.pop_scope(None, true).unwrap();
}
// ^^ Push & Pop Only ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Stack Allocation vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
fn small_stack_alloc_no_gc() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    mgr.alloc(make_num(0.), None).unwrap();
    mgr.pop_scope(None, false).unwrap();
}

fn small_stack_alloc_gc() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    mgr.alloc(make_num(0.), None).unwrap();
    mgr.pop_scope(None, true).unwrap();
}
// ^^ Stack Allocation ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Small Heap Allocation vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
fn small_str_alloc_no_gc() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_str("");
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, false).unwrap();
}

fn small_str_alloc_gc() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_str("");
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}


fn small_str_alloc_no_gc_2() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, false).unwrap();
}


fn small_str_alloc_gc_2() {
    let mut mgr = init_gc();
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}


fn small_obj_alloc_no_gc() {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None)];
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, false).unwrap();
}


fn small_obj_alloc_gc() {
    let mut mgr = init_gc();
    let kvs = vec![(JsKey::JsSym("0".to_string()), make_num(0.), None)];
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}


fn small_obj_alloc_no_gc_2() {
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
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, false).unwrap();
}


fn small_obj_alloc_gc_2() {
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
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}
// ^^ Small Heap Allocation ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Large Flat Heap Allocation vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv

fn large_obj_alloc_no_gc() {
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
    mgr.push_scope(&UNDEF);
    for _ in 0..100 {
        let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
        mgr.alloc(var, Some(ptr)).unwrap();
    }
    mgr.pop_scope(None, false).unwrap();
}


fn large_obj_alloc_gc() {
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
    mgr.push_scope(&UNDEF);
    for _ in 0..100 {
        let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
        mgr.alloc(var, Some(ptr)).unwrap();
    }
    mgr.pop_scope(None, true).unwrap();
}
// ^^ Large Flat Heap Allocation ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Huge Flat Heap Allocation vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv


fn huge_obj_alloc_no_gc() {
    let mut mgr = init_gc();
    let kvs: Vec<_> = (0..100_000).map(|i| (JsKey::JsSym(i.to_string()), make_num(i as f64), None)).collect();
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, false).unwrap();
}


fn huge_obj_alloc_gc() {
    let mut mgr = init_gc();
    let kvs: Vec<_> = (0..100_000).map(|i| (JsKey::JsSym(i.to_string()), make_num(i as f64), None)).collect();
    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    mgr.alloc(var, Some(ptr)).unwrap();
    mgr.pop_scope(None, true).unwrap();
}
// ^^ Huge Flat Heap Allocation ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Variable Load Tests vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv

fn shallow_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    mgr.load(&bnd).unwrap();
}


fn deca_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..10 {
        mgr.push_scope(&UNDEF);
    }
    mgr.load(&bnd).unwrap();
}


fn centi_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..100 {
        mgr.push_scope(&UNDEF);
    }
    mgr.load(&bnd).unwrap();
}


fn kilo_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..1_000 {
        mgr.push_scope(&UNDEF);
    }
    mgr.load(&bnd).unwrap();
}


fn mega_load() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    let bnd = var.binding.clone();
    mgr.alloc(var, None).unwrap();
    for _ in 0..1_000_000 {
        mgr.push_scope(&UNDEF);
    }
    mgr.load(&bnd).unwrap();
}
// ^^ Variable Load Tests ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Variable Store Tests vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv

fn small_local_store() {
    let mut mgr = init_gc();
    mgr.push_scope(&Exp::Call(box Exp::Undefined, vec![]));
    let var = make_num(0.);
    mgr.alloc(var.clone(), None).unwrap();
    mgr.store(var.clone(), None).unwrap();
}


fn large_local_store() {
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
    let (var, ptr) = make_obj(kvs, mgr.alloc_box.clone());
    mgr.alloc(var.clone(), Some(ptr.clone())).unwrap();
    mgr.store(var.clone(), Some(ptr.clone())).unwrap();
}
// ^^ Variable Store Tests ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Leak Tests vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv

fn leak_many_no_gc() {
    let mut mgr = init_gc();

    let (var, ptr) = make_str("test");
    let key = JsKey::JsSym("true".to_string());
    let kvs = vec![(key.clone(), var, Some(ptr))];

    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    let bnd = var.binding.clone();
    mgr.alloc(var.clone(), Some(ptr.clone())).unwrap();

    let copy = mgr.load(&bnd);
    let (var, mut ptr) = copy.unwrap();

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
    mgr.pop_scope(None, false).unwrap();
}

fn leak_many_gc() {
    let mut mgr = init_gc();

    let (var, ptr) = make_str("test");
    let key = JsKey::JsSym("true".to_string());
    let kvs = vec![(key.clone(), var, Some(ptr))];

    mgr.push_scope(&UNDEF);
    let (var, ptr) = make_obj(kvs.clone(), mgr.alloc_box.clone());
    let bnd = var.binding.clone();
    mgr.alloc(var.clone(), Some(ptr.clone())).unwrap();

    let copy = mgr.load(&bnd);
    let (var, mut ptr) = copy.unwrap();

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
// ^^ Leak Tests ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// vv Setup Functions vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
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
// ^^ Setup Functions ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

