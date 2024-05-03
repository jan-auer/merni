use iai_callgrind::{library_benchmark, library_benchmark_group, main};

mod benches;

library_benchmark_group!(
    name = bench_group;
    benchmarks = _1_vec_string, _2_boxed_string, _3_boxed_boxed, _4_thread_local, _5_smallvec, _6_smolstr, _7_smallvec_smolstr, _8_smolbuf,
);

main!(library_benchmark_groups = bench_group);

#[library_benchmark]
fn _1_vec_string() {
    benches::vec_string()
}

#[library_benchmark]
fn _2_boxed_string() {
    benches::boxed_string()
}

#[library_benchmark]
fn _3_boxed_boxed() {
    benches::boxed_boxed()
}

#[library_benchmark]
fn _4_thread_local() {
    benches::thread_local()
}

#[library_benchmark]
fn _5_smallvec() {
    benches::smallvec()
}

#[library_benchmark]
fn _6_smolstr() {
    benches::smolstr()
}

#[library_benchmark]
fn _7_smallvec_smolstr() {
    benches::smallvec_smolstr()
}

#[library_benchmark]
fn _8_smolbuf() {
    benches::smolbuf()
}
