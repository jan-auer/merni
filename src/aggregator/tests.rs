use super::*;
use crate::*;

#[test]
fn test_aggregation() {
    let (timer, mock) = Timer::mock();
    let aggregations = Default::default();
    let sink = ThreadLocalAggregator {
        aggregations: Arc::clone(&aggregations),
        timer,
        thread: None,
    };
    let dispatcher = Dispatcher::new(sink);

    let guard = set_local_dispatcher(dispatcher);

    gauge!("some.gauge": 1, "tag_key" => "tag_value");
    gauge!("some.gauge": 2, "tag_key" => "tag_value");
    mock.increment(Duration::from_millis(100));
    gauge!("some.gauge": 3, "tag_key" => "tag_value");
    mock.decrement(Duration::from_millis(100));
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
    assert_eq!(gauge.sum, 10.);
    assert_eq!(gauge.last, 3.);
}
