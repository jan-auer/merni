// use std::sync::{Arc, OnceLock};

// use cadence::Counted as _;

use self::timer::Timer;

use super::*;

#[test]
fn test_declare_macro() {
    let metric = declare_metric!(Counter => "some.counter");
    dbg!(metric);

    let tagged_metric = declare_metric!(Gauge => "some.gauge": "tag1", "tag1");
    dbg!(tagged_metric);
}

#[test]
fn test_dispatcher() {
    let (timer, _mock) = Timer::mock();
    let dispatcher = Dispatcher::with_timer(timer);
    dbg!(&dispatcher);

    let guard = set_local_dispatcher(dispatcher);
    let called = with_dispatcher(|dispatcher| {
        dbg!(dispatcher);
        true
    });
    assert!(called);

    let dispatcher = guard.take();
    dbg!(&dispatcher);

    with_dispatcher(|_dispatcher| unreachable!());
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
