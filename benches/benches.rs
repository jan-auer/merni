use std::time::Duration;

use merni::{counter, distribution, gauge};
use merni::{AggregationSink, ThreadLocalAggregator};

pub struct NoopSink;
impl AggregationSink for NoopSink {
    fn emit(&mut self, _metrics: merni::Aggregations) {}
}
pub fn noop_aggregator() -> ThreadLocalAggregator {
    ThreadLocalAggregator::new(Duration::from_millis(20), NoopSink)
}

pub fn emit_simple() {
    counter!("some.counter": 1);
    counter!("some.tagged.counter": 2, "tag_key" => "tag_value");
    gauge!("some.gauge": 3);
    gauge!("some.tagged.gauge": 4, "tag_key" => "tag_value");

    counter!("some.counter": 1);
    counter!("some.tagged.counter": 2, "tag_key" => "tag_value");
    gauge!("some.gauge": 3);
    gauge!("some.tagged.gauge": 4, "tag_key" => "tag_value");
}

pub fn emit_distribution() {
    distribution!("some.distribution": 1);
    distribution!("some.tagged.distribution": 2, "tag_key" => "tag_value");

    distribution!("some.distribution": 1);
    distribution!("some.tagged.distribution": 2, "tag_key" => "tag_value");
}
