use std::fmt::{Debug, Display};

use crate::sink::Sink;
use crate::tags::{record_tags, InputTags};
use crate::timer::Timer;
use crate::{IntoMetricValue, Metric, MetricKey, MetricMeta, MetricValue, TaggedMetricMeta};

/// A Dispatcher that can be used to emit metrics.
pub struct Dispatcher {
    pub(crate) timer: Timer,
    sink: Box<dyn Sink + Send + Sync + 'static>,
}

impl Debug for Dispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatcher")
            .field("timer", &self.timer)
            .field("sink", &format_args!("..."))
            .finish()
    }
}

impl Dispatcher {
    /// Creates a new [`Dispatcher`], dispatching metrics to the given [`Sink`].
    pub fn new<S>(sink: S) -> Self
    where
        S: Sink + Send + Sync + 'static,
    {
        Self {
            timer: Timer::new(),
            sink: Box::new(sink),
        }
    }

    /// Emit a metric value for the given metric.
    pub fn emit(&self, metric: &'static MetricMeta, value: impl IntoMetricValue) {
        let value = value.into_metric_value(metric);

        self.record(metric, value, &[]);
    }

    /// Emit a metric value along with tags for the given metric.
    pub fn emit_tagged<const N: usize>(
        &self,
        metric: &'static TaggedMetricMeta<N>,
        value: impl IntoMetricValue,
        tag_values: [&dyn Display; N],
    ) {
        let TaggedMetricMeta { meta } = metric;
        let value = value.into_metric_value(meta);

        self.record(meta, value, &tag_values);
    }

    fn record(&self, meta: &'static MetricMeta, value: MetricValue, tag_values: InputTags) {
        let key = MetricKey {
            meta,
            tag_values: record_tags(tag_values),
        };
        let timestamp = self.timer.timestamp();

        let metric = Metric {
            key,
            timestamp,
            value,
        };

        self.sink.emit(metric)
    }
}
