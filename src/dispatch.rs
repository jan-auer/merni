use std::fmt::Display;

use quanta::Instant;

use crate::tags::{record_tags, InputTags};
use crate::timer::Timer;
use crate::{IntoMetricValue, MetricKey, MetricMeta, TaggedMetric};

/// A Dispatcher that can be used to emit metrics.
#[derive(Debug)]
pub struct Dispatcher {
    timer: Timer,
}

impl Dispatcher {
    #[cfg(test)]
    pub fn with_timer(timer: Timer) -> Self {
        Self { timer }
    }

    /// Emit a metric value for the given metric.
    pub fn emit(&self, metric: &'static MetricMeta, value: impl IntoMetricValue) {
        let value = value.into_metric_value(metric);

        self.record(metric, value, &[]);
    }

    /// Emit a metric value along with tags for the given metric.
    pub fn emit_tagged<const N: usize>(
        &self,
        metric: &'static TaggedMetric<N>,
        value: impl IntoMetricValue,
        tag_values: [&dyn Display; N],
    ) {
        let TaggedMetric { meta } = metric;
        let value = value.into_metric_value(meta);

        self.record(meta, value, &tag_values);
    }

    fn record(
        &self,
        meta: &'static MetricMeta,
        value: f64,
        tag_values: InputTags,
    ) -> RecordedMetric {
        let key = MetricKey {
            meta,
            tag_values: record_tags(tag_values),
        };
        let instant = self.timer.now();

        RecordedMetric {
            key,
            instant,
            value,
        }
    }
}

/// A metric to be recorded, with its [`MetricKey`], value, and the current timestamp.
pub(crate) struct RecordedMetric {
    key: MetricKey,
    instant: Instant,
    value: f64,
}
