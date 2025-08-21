use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_utils::CachePadded;
use rustc_hash::FxHashMap as HashMap;
use thread_local::ThreadLocal;

use crate::tags::TagValues;
use crate::{Metric, MetricKey, MetricMeta, MetricType, Sink};

/// A Sink for aggregated metrics.
pub trait AggregationSink: Send + 'static {
    /// This fn is being called on a timer to emit aggregated metrics.
    fn emit(&self, metrics: Aggregations);
}

/// A wrapper around [`MetricKey`] optimized for [`HashMap`] operations
/// by using pointer equality for its [`MetricMeta`].
/// This will thus not aggregate otherwise identical metrics.
pub(crate) struct LocalKey(pub(crate) MetricKey<'static>);
impl LocalKey {
    fn into_metric(self) -> AggregatedMetric {
        AggregatedMetric {
            meta: *self.0.meta,
            tag_values: self.0.tag_values,
        }
    }
}

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

/// An aggregated Gauge.
pub struct AggregatedGauge {
    /// The minimum value within this aggregation.
    pub min: f64,
    /// The maximum value within this aggregation.
    pub max: f64,
    /// The total sum of values within this aggregation.
    pub sum: f64,
    /// The total number of values added to this aggregation.
    pub count: u64,

    /// The latest value added to this aggregation.
    pub last: f64,
}

impl Default for AggregatedGauge {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            count: 0,
            last: 0.0,
        }
    }
}

/// A precisely aggregated distribution, keeping a list of all the observed values.
#[derive(Default)]
pub struct PreciseAggregatedDistribution {
    /// All the aggregated values.
    pub values: Vec<f64>,
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

/// The thread-local "pre"-aggregations.
type ThreadLocalAggregations = Arc<ThreadLocal<CachePadded<Mutex<PreAggregations>>>>;

/// An aggregator that uses fast thread-local "pre"-aggregation.
pub struct ThreadLocalAggregator {
    /// The thread-local "pre"-aggregations.
    pub(crate) aggregations: ThreadLocalAggregations,

    pub(crate) thread: Option<(SyncSender<()>, JoinHandle<()>)>,
}

impl Drop for ThreadLocalAggregator {
    fn drop(&mut self) {
        if let Some((sender, thread)) = self.thread.take() {
            let _ = sender.try_send(());
            drop(sender);
            thread.join().unwrap();
        }
    }
}

impl ThreadLocalAggregator {
    /// Create a new thread-local aggregator.
    ///
    /// This will flush aggregated metrics to the given [`AggregationSink`] on a background thread,
    /// according to the `flush_interval`.
    pub fn new(flush_interval: Duration, sink: impl AggregationSink) -> Self {
        let aggregations = Default::default();

        let (send_signal, recv_signal) = std::sync::mpsc::sync_channel(0);

        let thread = std::thread::Builder::new()
            .name("merni-aggregator".into())
            .spawn({
                let aggregations = Arc::clone(&aggregations);
                move || Self::thread(aggregations, flush_interval, sink, recv_signal)
            })
            .unwrap();

        Self {
            aggregations,
            thread: Some((send_signal, thread)),
        }
    }

    fn thread(
        thread_locals: ThreadLocalAggregations,
        flush_interval: Duration,
        sink: impl AggregationSink,
        recv_signal: Receiver<()>,
    ) {
        loop {
            let should_shut_down = recv_signal.recv_timeout(flush_interval);

            let mut all_aggregations = Aggregations::default();
            for thread_local in thread_locals.iter() {
                let mut thread_local = thread_local.lock().unwrap();
                all_aggregations.merge_aggregations(&mut thread_local);
            }
            sink.emit(all_aggregations);

            if matches!(
                should_shut_down,
                Ok(()) | Err(RecvTimeoutError::Disconnected)
            ) {
                return;
            }
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
                gauge.last = value;

                gauge.min = gauge.min.min(value);
                gauge.max = gauge.max.max(value);
                gauge.sum += value;
                gauge.count += 1;
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

impl Sink for ThreadLocalAggregator {
    fn emit(&self, metric: Metric) {
        self.add_metric(metric)
    }
}

/// An aggregated metric key, along with its tag values.
#[derive(Hash, PartialEq, Eq)]
pub struct AggregatedMetric {
    meta: MetricMeta,
    tag_values: TagValues,
}

impl Deref for AggregatedMetric {
    type Target = MetricMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

impl AggregatedMetric {
    /// Iterates over the tag keys and values of this metric.
    pub fn tags(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        let values = self.tag_values.as_deref().unwrap_or_default();
        self.meta
            .tag_keys
            .iter()
            .copied()
            .zip(values.iter().map(|s| s.as_ref()))
    }
}

/// The final aggregated metrics.
#[derive(Default)]
pub struct Aggregations {
    /// All aggregated counter metrics.
    pub counters: HashMap<AggregatedMetric, f64>,
    /// All aggregated gauge metrics.
    pub gauges: HashMap<AggregatedMetric, AggregatedGauge>,
    /// All aggregated distribution-like metrics.
    pub distributions: HashMap<AggregatedMetric, PreciseAggregatedDistribution>,
}

impl Aggregations {
    /// Merges all the aggregates into `self`.
    pub(crate) fn merge_aggregations(&mut self, aggregations: &mut PreAggregations) {
        for (key, value) in aggregations.counters.drain() {
            let key = key.into_metric();
            *self.counters.entry(key).or_default() += value;
        }

        for (key, other) in aggregations.gauges.drain() {
            let key = key.into_metric();
            let gauge = self.gauges.entry(key).or_default();

            gauge.min = gauge.min.min(other.min);
            gauge.max = gauge.max.max(other.max);
            gauge.sum += other.sum;
            gauge.count += other.count;
            gauge.last = other.last;
        }

        for (key, other) in aggregations.distributions.drain() {
            let key = key.into_metric();
            self.distributions
                .entry(key)
                .or_default()
                .values
                .extend(other.values);
        }
    }
}
