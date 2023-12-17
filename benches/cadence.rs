use cadence::{Counted, MetricSink, StatsdClient};
use divan::{black_box, Bencher};

// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

struct NoopCadenceSink;
impl MetricSink for NoopCadenceSink {
    fn emit(&self, metric: &str) -> std::io::Result<usize> {
        black_box(metric);
        Ok(0)
    }
}

fn main() {
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

#[divan::bench]
fn e_only_global_tags(bencher: Bencher) {
    let metrics = StatsdClient::builder("example.prefix", NoopCadenceSink)
        .with_tag("global_tag1", "tag_value")
        .with_tag_value("global_tag2")
        .build();

    bencher.bench(|| metrics.count_with_tags("counter.global.tags", 1).send());
}

#[divan::bench]
fn f_global_and_local_tags(bencher: Bencher) {
    let metrics = StatsdClient::builder("example.prefix", NoopCadenceSink)
        .with_tag("global_tag1", "tag_value")
        .with_tag_value("global_tag2")
        .build();

    bencher.bench(|| {
        metrics
            .count_with_tags("counter.global.tags", 1)
            .with_tag("local_tag1", "tag_value")
            .with_tag_value("local_tag2")
            .send()
    });
}
