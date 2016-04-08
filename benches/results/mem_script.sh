#!/bin/bash

benches=( init_only push_scope push_pop_no_gc push_pop_gc small_stack_alloc_no_gc small_stack_alloc_gc small_str_alloc_no_gc small_str_alloc_gc small_str_alloc_no_gc_2 small_str_alloc_gc_2 small_obj_alloc_no_gc small_obj_alloc_gc small_obj_alloc_no_gc_2 small_obj_alloc_gc_2 large_obj_alloc_no_gc large_obj_alloc_gc huge_obj_alloc_no_gc huge_obj_alloc_gc shallow_load deca_load centi_load kilo_load small_local_store large_local_store leak_many_no_gc leak_many_gc )

num=000

workdir=/home/djmally/src/penn/masters/thesis/rustoscript/french_press/

(cd $workdir/benches/mem && cargo build)

for bench in "${benches[@]}"
do
    mkdir -p $workdir/benches/results/space/$bench
    (cd $workdir/benches/mem &&
        echo "Running $bench" &&
        RUST_LOG=mem,french-press::* cargo run $bench &> ../results/space/$bench/$num)
done
