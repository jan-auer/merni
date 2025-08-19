use crate::testing::TestDispatcher;

use super::*;

#[test]
fn test_local_dispatcher() {
    let dispatcher = TestDispatcher::new();

    let called = with_dispatcher(|_dispatcher| true);
    assert!(called);

    let metrics = dispatcher.finish();
    assert!(metrics.is_empty());

    let called: bool = with_dispatcher(|_dispatcher| unreachable!());
    assert!(!called);
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
        static TAGGED_METRIC: TaggedMetricMeta<2> =
            MetricMeta::new(MetricType::Gauge, MetricUnit::Unknown, "manual.gauge")
                .with_tags(&["tag1", "tag2"]);

        dispatcher.emit_tagged(&TAGGED_METRIC, 2, [&123, &"tag value 2"]);
    });

    let metrics = dispatcher.finish();
    assert_eq!(metrics.len(), 2);

    assert_eq!(metrics[0].ty(), MetricType::Counter);
    assert_eq!(metrics[0].key(), "manual.counter");
    assert_eq!(metrics[0].value().get(), 1.);

    assert_eq!(metrics[1].ty(), MetricType::Gauge);
    assert_eq!(metrics[1].key(), "manual.gauge");
    assert_eq!(metrics[1].value().get(), 2.);
    assert_eq!(
        metrics[1].tags().collect::<Vec<_>>(),
        &[("tag1", "123"), ("tag2", "tag value 2")]
    );
}

#[cfg(feature = "aggregator")]
#[test]
fn test_aggregation() {
    use std::sync::Arc;

    let aggregations = Default::default();
    let sink = ThreadLocalAggregator {
        aggregations: Arc::clone(&aggregations),
        thread: None,
    };
    let dispatcher = Dispatcher::new(sink);

    let guard = set_local_dispatcher(dispatcher);

    gauge!("some.gauge": 1, "tag_key" => "tag_value");
    gauge!("some.gauge": 2, "tag_key" => "tag_value");
    gauge!("some.gauge": 3, "tag_key" => "tag_value");
    gauge!("some.gauge": 4, "tag_key" => "tag_value");

    drop(guard);

    let mut total_aggregation = Aggregations::default();
    for aggregation in aggregations.iter() {
        let mut aggregation = aggregation.lock().unwrap();
        assert_eq!(aggregation.gauges.len(), 4); // implementation detail of `LocalKey`
        total_aggregation.merge_aggregations(&mut aggregation);
    }

    assert_eq!(total_aggregation.gauges.len(), 1);
    let gauge = total_aggregation.gauges.into_values().next().unwrap();

    assert_eq!(gauge.count, 4);
    assert_eq!(gauge.min, 1.);
    assert_eq!(gauge.max, 4.);
    assert_eq!(gauge.sum, 10.);
}
