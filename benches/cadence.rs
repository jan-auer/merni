use cadence::{MetricSink, StatsdClient};
use divan::black_box;

fn main() {
    struct NoopCadenceSink;
    impl MetricSink for NoopCadenceSink {
        fn emit(&self, _metric: &str) -> std::io::Result<usize> {
            Ok(0)
        }
    }

    let metrics = StatsdClient::from_sink("example.prefix", NoopCadenceSink);
    cadence_macros::set_global_default(metrics);

    divan::main();
}

#[divan::bench]
fn a_simple_counter() {
    cadence_macros::statsd_count!("counter.simple", 1);
}

#[divan::bench]
fn b_dynamic_counter() {
    let key = black_box(format!("counter.{}", "dynamic"));
    cadence_macros::statsd_count!(&key, 1);
}

#[divan::bench]
fn c_simple_tags() {
    cadence_macros::statsd_count!("counter.simple.tags.simple", 1, "tag" => "value");
}

#[divan::bench]
fn d_dynamic_tags() {
    let tag_key = black_box("tag");
    let tag_value = black_box("value");
    cadence_macros::statsd_count!("counter.simple.tags.dynamic", 1, tag_key => tag_value);
}
