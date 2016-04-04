#!/bin/sh

benches=( init_only push_scope push_pop_no_gc push_pop_gc small_stack_alloc_no_gc small_heap_alloc_no_gc small_heap_alloc_no_gc_2 large_alloc_no_gc_flat_obj small_local_store shallow_load deep_load small_alloc_gc small_flat_heap_alloc_gc large_flat_alloc_gc leak_many )

num=000

workdir=/home/djmally/src/penn/masters/thesis/rustoscript/french_press/

(cd $workdir/benches/mem && cargo build)

for bench in "${benches[@]}"
do
    mkdir -p $workdir/benches/results/space/$bench
    (cd $workdir/benches/mem &&
     valgrind --tool=massif --time-unit=B --massif-out-file=$workdir/benches/results/space/$bench/$num cargo run -- $bench)
    ms_print ./space/$bench/$num > ./space/$bench/pp_${num}
done
