use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use crossbeam_utils::CachePadded;
use thread_local::ThreadLocal;

use crate::sink::Sink;
use crate::tags::TagValues;
use crate::timer::Timestamp;
use crate::{Metric, MetricKey, MetricMeta, MetricType};

/// A wrapper around [`MetricKey`] optimized for [`HashMap`] operations
/// by using pointer equality for its [`MetricMeta`].
/// This will thus not aggregate otherwise identical metrics.
pub struct LocalKey(pub(crate) MetricKey);

impl LocalKey {
    fn key(&self) -> (*const MetricMeta, &TagValues) {
        (self.0.meta as *const _, &self.0.tag_values)
    }
}

impl Hash for LocalKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl PartialEq for LocalKey {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
impl Eq for LocalKey {}

pub struct AggregatedGauge {
    /// The minimum value within this aggregation.
    pub(crate) min: f64,
    /// The maximum value within this aggregation.
    pub(crate) max: f64,
    /// The total sum of values within this aggregation.
    pub(crate) sum: f64,
    /// The total number of values added to this aggregation.
    pub(crate) count: u64,

    /// The latest value added to this aggregation.
    pub(crate) last: f64,
    /// The timestamp of the latest value in this aggregation.
    last_timestamp: Timestamp,
}

impl Default for AggregatedGauge {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            count: 0,
            last: 0.0,
            last_timestamp: Timestamp::ZERO,
        }
    }
}

#[derive(Default)]
pub struct PreciseAggregatedDistribution {
    /// All the aggregated values.
    values: Vec<f64>,
}

/// The thread-local "pre"-aggregations.
///
/// They use the optimized [`LocalKey`], and might thus under-aggregate the same key.
#[derive(Default)]
pub(crate) struct PreAggregations {
    /// All aggregated counter metrics.
    counters: HashMap<LocalKey, f64>,
    /// All aggregated gauge metrics.
    pub(crate) gauges: HashMap<LocalKey, AggregatedGauge>,
    /// All aggregated distribution-like metrics.
    distributions: HashMap<LocalKey, PreciseAggregatedDistribution>,
}

/// An aggregator that uses fast thread-local "pre"-aggregation.
#[derive(Clone)]
pub struct ThreadLocalAggregator {
    /// The thread-local "pre"-aggregations.
    pub(crate) aggregations: Arc<ThreadLocal<CachePadded<Mutex<PreAggregations>>>>,
}

impl ThreadLocalAggregator {
    /// Create a new thread-local aggregator.
    pub fn new() -> Self {
        Self {
            aggregations: Default::default(),
        }
    }

    /// Adds the [`Metric`] to this thread-local aggregator.
    pub fn add_metric(&self, metric: Metric) {
        let mut aggregations = self.aggregations.get_or_default().lock().unwrap();
        let ty = metric.ty();
        let key = LocalKey(metric.key);
        let value = metric.value.get();

        match ty {
            MetricType::Counter => {
                *aggregations.counters.entry(key).or_default() += value;
            }
            MetricType::Gauge => {
                let gauge = aggregations.gauges.entry(key).or_default();
                gauge.min = gauge.min.min(value);
                gauge.max = gauge.max.max(value);
                gauge.sum += value;
                gauge.count += 1;
                if metric.timestamp >= gauge.last_timestamp {
                    gauge.last_timestamp = metric.timestamp;
                    gauge.last = value;
                }
            }
            MetricType::Distribution | MetricType::Timer => {
                aggregations
                    .distributions
                    .entry(key)
                    .or_default()
                    .values
                    .push(value);
            }
        }
    }
}

impl Default for ThreadLocalAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Sink for ThreadLocalAggregator {
    fn emit(&self, metric: Metric) {
        self.add_metric(metric)
    }
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) struct AggregationKey((MetricMeta, TagValues));

/// The final aggregated metrics.
#[derive(Default)]
pub(crate) struct Aggregations {
    /// All aggregated counter metrics.
    counters: HashMap<AggregationKey, f64>,
    /// All aggregated gauge metrics.
    pub(crate) gauges: HashMap<AggregationKey, AggregatedGauge>,
    /// All aggregated distribution-like metrics.
    distributions: HashMap<AggregationKey, PreciseAggregatedDistribution>,
}

impl Aggregations {
    /// Merges all the aggregates into `self`.
    pub(crate) fn merge_aggregations(&mut self, aggregations: &mut PreAggregations) {
        for (key, value) in aggregations.counters.drain() {
            let key = AggregationKey(key.0.without_location());
            *self.counters.entry(key).or_default() += value;
        }

        for (key, other) in aggregations.gauges.drain() {
            let key = AggregationKey(key.0.without_location());
            let gauge = self.gauges.entry(key).or_default();

            gauge.min = gauge.min.min(other.min);
            gauge.max = gauge.max.max(other.max);
            gauge.sum += other.sum;
            gauge.count += other.count;
            if other.last_timestamp >= gauge.last_timestamp {
                gauge.last_timestamp = other.last_timestamp;
                gauge.last = other.last;
            }
        }

        for (key, other) in aggregations.distributions.drain() {
            let key = AggregationKey(key.0.without_location());
            self.distributions
                .entry(key)
                .or_default()
                .values
                .extend(other.values);
        }
    }
}
