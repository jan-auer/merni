use std::sync::Arc;

use divan::Bencher;
use merni::{Dispatcher, set_global_dispatcher, set_local_dispatcher};

mod benches;
use benches::*;

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    set_global_dispatcher(Dispatcher::new(noop_aggregator())).unwrap();

    divan::main();
}

#[divan::bench]
fn simple_global() {
    emit_simple();
}

#[divan::bench]
fn distribution_global() {
    emit_distribution();
}

#[divan::bench]
fn simple(bencher: Bencher) {
    let sink = Arc::new(noop_aggregator());
    bencher
        .with_inputs(|| Dispatcher::new(Arc::clone(&sink)))
        .bench_values(|dispatcher| {
            let guard = set_local_dispatcher(dispatcher);
            emit_simple();
            guard.take()
        });
}

#[divan::bench]
fn distribution(bencher: Bencher) {
    let sink = Arc::new(noop_aggregator());
    bencher
        .with_inputs(|| Dispatcher::new(Arc::clone(&sink)))
        .bench_values(|dispatcher| {
            let guard = set_local_dispatcher(dispatcher);
            emit_distribution();
            guard.take()
        });
}
