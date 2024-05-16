use std::hash::{Hash, Hasher};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_utils::CachePadded;
use quanta::Clock;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use thread_local::ThreadLocal;

use crate::sink::Sink;
use crate::tags::TagValues;
use crate::timer::Timestamp;
use crate::{Location, Metric, MetricKey, MetricMeta, MetricType};

/// A wrapper around [`MetricKey`] optimized for [`HashMap`] operations
/// by using pointer equality for its [`MetricMeta`].
/// This will thus not aggregate otherwise identical metrics.
pub struct LocalKey(pub(crate) MetricKey<'static>);

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
    pub fn new() -> Self {
        let aggregations = Default::default();

        let (send_signal, recv_signal) = std::sync::mpsc::sync_channel(0);

        let thread = std::thread::Builder::new()
            .name("merni-aggregator".into())
            .spawn({
                let aggregations = Arc::clone(&aggregations);
                move || Self::thread(aggregations, recv_signal)
            })
            .unwrap();

        Self {
            aggregations,
            thread: Some((send_signal, thread)),
        }
    }

    fn thread(thread_locals: ThreadLocalAggregations, recv_signal: Receiver<()>) {
        const TICK_TIMER: Duration = Duration::from_millis(125);
        let clock = Clock::new();

        let mut all_aggregations = Aggregations::default();

        loop {
            let should_shut_down = recv_signal.recv_timeout(TICK_TIMER);
            quanta::set_recent(clock.now());

            for thread_local in thread_locals.iter() {
                let mut thread_local = thread_local.lock().unwrap();
                all_aggregations.merge_aggregations(&mut thread_local);
            }

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
    /// All code locations of the emitted metrics.
    code_locations: HashMap<MetricMeta, HashSet<&'static Location<'static>>>,
    /// All aggregated counter metrics.
    counters: HashMap<AggregationKey, f64>,
    /// All aggregated gauge metrics.
    pub(crate) gauges: HashMap<AggregationKey, AggregatedGauge>,
    /// All aggregated distribution-like metrics.
    distributions: HashMap<AggregationKey, PreciseAggregatedDistribution>,
}

impl Aggregations {
    /// Removes the [`Location`] from this metric,
    /// in order to aggregate metrics ignoring the source code location.
    ///
    /// The removed [`Location`] is rather saved in a per-metric Set,
    /// "per metric" in this case meaning the metric without any unique tags.
    fn split_location(&mut self, LocalKey(key): LocalKey) -> AggregationKey {
        let Some(location) = key.location else {
            return AggregationKey((*key.meta, key.tag_values));
        };

        // remove the location from the aggregation key
        let mut meta = *key.meta;
        meta.location = None;

        // and then remove all the tag keys from the locations key
        let mut locations_meta = meta;
        locations_meta.tag_keys = &[];

        self.code_locations
            .entry(locations_meta)
            .or_default()
            .insert(location);

        AggregationKey((meta, key.tag_values))
    }

    /// Merges all the aggregates into `self`.
    pub(crate) fn merge_aggregations(&mut self, aggregations: &mut PreAggregations) {
        for (key, value) in aggregations.counters.drain() {
            let key = self.split_location(key);
            *self.counters.entry(key).or_default() += value;
        }

        for (key, other) in aggregations.gauges.drain() {
            let key = self.split_location(key);
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
            let key = self.split_location(key);
            self.distributions
                .entry(key)
                .or_default()
                .values
                .extend(other.values);
        }
    }
}
