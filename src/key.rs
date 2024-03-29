use std::ops::Deref;

use crate::tags::TagValues;
use crate::MetricMeta;

/// The metric key, which represents a unique metric and its tags that is being emitted.
pub struct MetricKey {
    pub(crate) meta: &'static MetricMeta,
    pub(crate) tag_values: TagValues,
}

impl Deref for MetricKey {
    type Target = MetricMeta;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl MetricKey {
    /// Iterates over the tag keys and values of this metric.
    pub fn tags(&self) -> impl Iterator<Item = (&str, &str)> {
        let values = self.tag_values.as_deref().unwrap_or_default();
        self.meta
            .tag_keys
            .iter()
            .copied()
            .zip(values.iter().map(|s| s.as_ref()))
    }
}
