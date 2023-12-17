use divan::{black_box, Bencher};
use merni::{metric, Recorder};

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

pub struct Sink;
impl merni::MetricSink for Sink {
    fn emit(&self, metric: &str) {
        black_box(metric);
    }
}

fn main() {
    let recorder = merni::StatsdRecorder::new("example.prefix", Sink);
    merni::init(recorder).map_err(|_| ()).unwrap();

    divan::main();
}

#[divan::bench]
fn a_simple_counter() {
    merni::counter!("counter.simple", 1);
}

#[divan::bench]
fn b_dynamic_counter() {
    merni::counter!(black_box(format_args!("counter.{}", "dynamic")), 1);
}

#[divan::bench]
fn c_simple_tags() {
    merni::counter!("counter.simple.tags", 1, "a" => "b");
}

#[divan::bench]
fn d_dynamic_tags() {
    let tag_key = black_box("tag");
    let tag_value = black_box("value");
    merni::counter!("counter.simple.tags.dynamic", 1, tag_key => tag_value);
}

#[divan::bench]
fn e_only_global_tags(bencher: Bencher) {
    let recorder = merni::StatsdRecorder::new("example.prefix", Sink)
        .with_tag("global_tag1", "tag_value")
        .with_tag_value("global_tag2");

    bencher.bench(|| {
        recorder.record_metric(metric!(Counter: "counter.global.tags", 1));
    });
}

#[divan::bench]
fn f_global_and_local_tags(bencher: Bencher) {
    let recorder = merni::StatsdRecorder::new("example.prefix", Sink)
        .with_tag("global_tag1", "tag_value")
        .with_tag_value("global_tag2");

    bencher.bench(|| {
        recorder.record_metric(metric!(
            Counter: "counter.global.tags", 1,
            "local_tag1" => "tag_value";
            "local_tag2"
        ));
    });
}
