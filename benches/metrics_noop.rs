use std::sync::Arc;

use divan::black_box;
use metrics::{Counter, CounterFn, Gauge, Histogram, Key, KeyName, SharedString, Unit};

fn main() {
    struct NoopRecorder;
    static NOOP_RECORDER: NoopRecorder = NoopRecorder;

    struct NoopCounter(Key);
    impl CounterFn for NoopCounter {
        fn increment(&self, _value: u64) {}
        fn absolute(&self, _value: u64) {}
    }

    impl metrics::Recorder for NoopRecorder {
        fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        }
        fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
            todo!()
        }
        fn describe_histogram(
            &self,
            _key: KeyName,
            _unit: Option<Unit>,
            _description: SharedString,
        ) {
            todo!()
        }
        fn register_counter(&self, key: &Key) -> Counter {
            Counter::from(Arc::new(NoopCounter(key.clone())))
        }
        fn register_gauge(&self, _key: &Key) -> Gauge {
            Gauge::noop()
        }
        fn register_histogram(&self, _key: &Key) -> Histogram {
            Histogram::noop()
        }
    }

    metrics::set_recorder(&NOOP_RECORDER).unwrap();

    divan::main();
}

#[divan::bench]
fn a_simple_counter() {
    metrics::counter!("counter.simple", 1);
}

#[divan::bench]
fn b_dynamic_counter() {
    metrics::counter!(black_box(format!("counter.{}", "dynamic")), 1);
}

#[divan::bench]
fn c_simple_tags() {
    metrics::counter!("counter.simple.tags", 1, "a" => "b");
}

#[divan::bench]
fn d_dynamic_tags() {
    let tag_key = black_box("tag");
    let tag_value = black_box("value");
    metrics::counter!("counter.simple.tags.dynamic", 1, tag_key => tag_value);
}
