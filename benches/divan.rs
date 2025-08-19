use std::sync::Arc;
use std::time::Duration;

use divan::Bencher;
use merni::{counter, distribution, gauge, AggregationSink};
use merni::{set_global_dispatcher, set_local_dispatcher, Dispatcher, ThreadLocalAggregator};

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

struct NoopSink;
impl AggregationSink for NoopSink {
    fn emit(&self, _metrics: merni::Aggregations) {}
}
fn noop_aggregator() -> ThreadLocalAggregator {
    ThreadLocalAggregator::new(Duration::from_millis(10), NoopSink)
}

fn main() {
    set_global_dispatcher(Dispatcher::new(noop_aggregator())).unwrap();

    divan::main();
}

fn emit_simple() {
    counter!("some.counter": 1);
    counter!("some.tagged.counter": 2, "tag_key" => "tag_value");
    gauge!("some.gauge": 3);
    gauge!("some.tagged.gauge": 4, "tag_key" => "tag_value");
}

fn emit_distribution() {
    distribution!("some.distribution": 1);
    distribution!("some.tagged.distribution": 2, "tag_key" => "tag_value");
}

#[divan::bench(items_count = 4u8)]
fn simple_global() {
    emit_simple();
}

#[divan::bench]
fn simple(bencher: Bencher) {
    let sink = Arc::new(noop_aggregator());
    bencher
        .counter(divan::counter::ItemsCount::new(4u8))
        .with_inputs(|| Dispatcher::new(Arc::clone(&sink)))
        .bench_values(|dispatcher| {
            let guard = set_local_dispatcher(dispatcher);
            emit_simple();
            guard.take()
        });
}

#[divan::bench]
fn distribution(bencher: Bencher) {
    bencher
        .counter(divan::counter::ItemsCount::new(2u8))
        .with_inputs(|| Dispatcher::new(noop_aggregator()))
        .bench_values(|dispatcher| {
            let guard = set_local_dispatcher(dispatcher);
            emit_distribution();
            guard.take()
        });
}
