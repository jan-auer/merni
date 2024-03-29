use std::fmt::Display;

use crate::tags::{record_tags, InputTags};
use crate::{IntoMetricValue, MetricKey, MetricMeta, TaggedMetric};

pub struct Dispatcher {}

impl Dispatcher {
    pub fn emit(&self, meta: &'static MetricMeta, value: impl IntoMetricValue) {
        let value = value.into_metric_value(meta);

        self.record(meta, value, &[]);
    }

    pub fn emit_tagged<const N: usize>(
        &self,
        meta: &'static TaggedMetric<N>,
        value: impl IntoMetricValue,
        tag_values: [&dyn Display; N],
    ) {
        let TaggedMetric { meta } = meta;
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

pub struct RecordedMetric {
    key: MetricKey,
    value: f64,
}
