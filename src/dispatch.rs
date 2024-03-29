use std::fmt::Display;

use crate::tags::{record_tags, InputTags};
use crate::{IntoMetricValue, MetricKey, MetricMeta, TaggedMetric};

/// A Dispatcher that can be used to emit metrics.
pub struct Dispatcher {}

impl Dispatcher {
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
        RecordedMetric { key, value }
    }
}

/// A metric to be recorded, with its [`MetricKey`] and value.
pub struct RecordedMetric {
    key: MetricKey,
    value: f64,
}
