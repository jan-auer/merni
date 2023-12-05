mod globals;
mod macros;
mod statsd;
mod types;

pub use globals::*;
pub use macros::*;
pub use statsd::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use cadence::Counted as _;

    use super::*;

    #[test]
    fn compare_with_cadence() {
        struct NoopCadenceSink;
        impl cadence::MetricSink for NoopCadenceSink {
            fn emit(&self, metric: &str) -> std::io::Result<usize> {
                dbg!(metric);
                Ok(0)
            }
        }

        let cadence_client = cadence::StatsdClient::builder("some.prefix", NoopCadenceSink)
            .with_tag_value("tag_only_a")
            .with_tag_value("tag_only_a")
            .with_tag_value("tag_only_b")
            .with_tag_value("tag_only_c")
            .with_tag("tag_a", "value_a")
            .with_tag("tag_a", "value_a")
            .with_tag("tag_b", "value_b")
            .with_tag("tag_c", "value_c")
            .build();

        cadence_client
            .count_with_tags("some.metric", 1)
            .with_tag("tag_a", "override_a")
            .with_tag("tag_d", "tag_d")
            .with_tag_value("tag_only_b")
            .with_tag_value("tag_only_d")
            .send();

        struct NoopMerniSink;
        impl MetricSink for NoopMerniSink {
            fn emit(&self, metric: &str) {
                dbg!(metric);
            }
        }
        let merni_client = StatsdRecorder::new("some.prefix", NoopMerniSink)
            .with_tag_value("tag_only_a")
            .with_tag_value("tag_only_a")
            .with_tag_value("tag_only_b")
            .with_tag_value("tag_only_c")
            .with_tag("tag_a", "value_a")
            .with_tag("tag_a", "value_a")
            .with_tag("tag_b", "value_b")
            .with_tag("tag_c", "value_c");

        merni_client.record_metric(
            metric!(Counter: "some.metric", 1, "tag_a" => "override_a", "tag_d" => "tag_d"; "tag_only_b", "tag_only_d"));
    }
}
