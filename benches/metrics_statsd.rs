use cadence29::{MetricSink, StatsdClient};
use divan::black_box;
use metrics_exporter_statsd::StatsdRecorder;

fn main() {
    struct NoopCadenceSink;
    impl MetricSink for NoopCadenceSink {
        fn emit(&self, _metric: &str) -> std::io::Result<usize> {
            Ok(0)
        }
    }

    let metrics = StatsdClient::from_sink("example.prefix", NoopCadenceSink);
    let recorder = StatsdRecorder::new(metrics, "distribution".into());

    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

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
