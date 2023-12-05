use divan::black_box;

fn main() {
    pub struct Sink;
    impl merni::MetricSink for Sink {
        fn emit(&self, _metric: &str) {}
    }

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
