// use std::sync::{Arc, OnceLock};

// use cadence::Counted as _;

use crate::testing::TestDispatcher;

use super::*;

#[test]
fn test_local_dispatcher() {
    let dispatcher = TestDispatcher::new();

    let called = with_dispatcher(|_dispatcher| true);
    assert!(called);

    let metrics = dispatcher.finish();
    assert!(metrics.is_empty());

    with_dispatcher(|_dispatcher| unreachable!());
}

#[test]
fn test_emit_macro() {
    let dispatcher = TestDispatcher::new();

    counter!("some.counter": 1);

    let foo = 2;
    distribution!("some.distribution": 2, "foo" => foo, "bar" => "bar");

    gauge!("some.gauge": 3, "a" => 1 + 2 + 3, "b" => foo * 2);

    let metrics = dispatcher.finish();
    assert_eq!(metrics.len(), 3);

    assert_eq!(metrics[0].ty(), MetricType::Counter);
    assert_eq!(metrics[0].key(), "some.counter");
    assert_eq!(metrics[0].value().get(), 1.);

    assert_eq!(metrics[1].ty(), MetricType::Distribution);
    assert_eq!(metrics[1].key(), "some.distribution");
    assert_eq!(metrics[1].value().get(), 2.);
    assert_eq!(
        metrics[1].tags().collect::<Vec<_>>(),
        &[("foo", "2"), ("bar", "bar")]
    );

    assert_eq!(metrics[2].ty(), MetricType::Gauge);
    assert_eq!(metrics[2].key(), "some.gauge");
    assert_eq!(metrics[2].value().get(), 3.);
    assert_eq!(
        metrics[2].tags().collect::<Vec<_>>(),
        &[("a", "6"), ("b", "4")]
    );
}

#[test]
fn test_manual_emit() {
    let dispatcher = TestDispatcher::new();

    with_dispatcher(|dispatcher| {
        static METRIC: MetricMeta =
            MetricMeta::new(MetricType::Counter, MetricUnit::Unknown, "manual.counter");

        dispatcher.emit(&METRIC, 1);
    });

    with_dispatcher(|dispatcher| {
        static LOCATION: Location<'static> = Location::new("this_file.rs", 123, "merni::tests");
        static TAGGED_METRIC: TaggedMetricMeta<2> =
            MetricMeta::new(MetricType::Gauge, MetricUnit::Unknown, "manual.gauge")
                .with_location(&LOCATION)
                .with_tags(&["tag1", "tag2"]);

        dispatcher.emit_tagged(&TAGGED_METRIC, 2, [&123, &"tag value 2"]);
    });

    let metrics = dispatcher.finish();
    assert_eq!(metrics.len(), 2);

    assert_eq!(metrics[0].ty(), MetricType::Counter);
    assert_eq!(metrics[0].key(), "manual.counter");
    assert_eq!(metrics[0].file(), None);
    assert_eq!(metrics[0].value().get(), 1.);

    assert_eq!(metrics[1].ty(), MetricType::Gauge);
    assert_eq!(metrics[1].key(), "manual.gauge");
    assert_eq!(metrics[1].file(), Some("this_file.rs"));
    assert_eq!(metrics[1].value().get(), 2.);
    assert_eq!(
        metrics[1].tags().collect::<Vec<_>>(),
        &[("tag1", "123"), ("tag2", "tag value 2")]
    );
}

// #[test]
// fn compare_with_cadence() {
//     type OnceString = Arc<OnceLock<String>>;

//     struct NoopCadenceSink(OnceString);
//     impl cadence::MetricSink for NoopCadenceSink {
//         fn emit(&self, metric: &str) -> std::io::Result<usize> {
//             self.0.get_or_init(|| metric.into());
//             Ok(0)
//         }
//     }

//     let cadence_output = OnceString::default();
//     let cadence_client =
//         cadence::StatsdClient::builder("some.prefix", NoopCadenceSink(cadence_output.clone()))
//             .with_tag_value("tag_only_a")
//             .with_tag_value("tag_only_a")
//             .with_tag_value("tag_only_b")
//             .with_tag_value("tag_only_c")
//             .with_tag("tag_a", "value_a")
//             .with_tag("tag_a", "value_a")
//             .with_tag("tag_b", "value_b")
//             .with_tag("tag_c", "value_c")
//             .build();

//     cadence_client
//         .count_with_tags("some.metric", 1)
//         .with_tag("tag_a", "override_a")
//         .with_tag("tag_d", "tag_d")
//         .with_tag_value("tag_only_b")
//         .with_tag_value("tag_only_d")
//         .send();

//     struct NoopMerniSink(OnceString);
//     impl MetricSink for NoopMerniSink {
//         fn emit(&self, metric: &str) {
//             self.0.get_or_init(|| metric.into());
//         }
//     }

//     let merni_output = OnceString::default();
//     let merni_client = StatsdRecorder::new("some.prefix", NoopMerniSink(merni_output.clone()))
//         .with_tag_value("tag_only_a")
//         .with_tag_value("tag_only_a")
//         .with_tag_value("tag_only_b")
//         .with_tag_value("tag_only_c")
//         .with_tag("tag_a", "value_a")
//         .with_tag("tag_a", "value_a")
//         .with_tag("tag_b", "value_b")
//         .with_tag("tag_c", "value_c");

//     merni_client.record_metric(metric!(
//         Counter: "some.metric", 1,
//         "tag_a" => "override_a", "tag_d" => "tag_d";
//         "tag_only_b", "tag_only_d"
//     ));

//     assert_eq!(cadence_output, merni_output);
// }
