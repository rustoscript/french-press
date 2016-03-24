#![feature(test)]
extern crate test;
extern crate french_press;
extern crate test_utils;

use test::Bencher;
use french_press::*;

#[bench]
fn small_no_gc(b: &mut Bencher) {
    let mut mgr = init_gc();
    mgr.alloc(test_utils::make_num(0.), None).unwrap();
    mgr.alloc(test_utils::make_num(1.), None).unwrap();
    mgr.alloc(test_utils::make_num(2.), None).unwrap();
}
