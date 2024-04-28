mod benches;

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench]
fn _1_vec_string() {
    benches::vec_string()
}

#[divan::bench]
fn _2_boxed_string() {
    benches::boxed_string()
}

#[divan::bench]
fn _3_boxed_boxed() {
    benches::boxed_boxed()
}

#[divan::bench]
fn _4_thread_local() {
    benches::thread_local()
}

#[divan::bench]
fn _5_smallvec() {
    benches::smallvec()
}

#[divan::bench]
fn _6_smolstr() {
    benches::smolstr()
}

#[divan::bench]
fn _7_smallvec_smolstr() {
    benches::smallvec_smolstr()
}
