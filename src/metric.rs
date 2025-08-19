use std::ops::Deref;

use crate::tags::TagValues;
use crate::{MetricType, MetricUnit, MetricValue};

/// The metadata of a particular metric.
///
/// This includes its type, unit, the metrics name (or key), and possibly
/// the code location at which it is emitted.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MetricMeta {
    ty: MetricType,
    unit: MetricUnit,
    key: &'static str,
    pub(crate) tag_keys: &'static [&'static str],
}

impl MetricMeta {
    /// Creates a new [`MetricMeta`] with the given type, unit and key.
    pub const fn new(ty: MetricType, unit: MetricUnit, key: &'static str) -> Self {
        Self {
            ty,
            unit,
            key,
            tag_keys: &[],
        }
    }

    /// Adds the expected metric tags, turning this into a [`TaggedMetric`].
    pub const fn with_tags<const N: usize>(
        mut self,
        tag_keys: &'static [&'static str; N],
    ) -> TaggedMetricMeta<N> {
        self.tag_keys = tag_keys;
        TaggedMetricMeta { meta: self }
    }

    /// The metrics type.
    pub fn ty(&self) -> MetricType {
        self.ty
    }

    /// The metrics unit.
    pub fn unit(&self) -> MetricUnit {
        self.unit
    }

    /// The key, or name of the metric.
    pub fn key(&self) -> &'static str {
        self.key
    }
}

/// Metric metadata parameterized with the number of expected tags.
#[derive(Debug)]
pub struct TaggedMetricMeta<const N: usize> {
    pub(crate) meta: MetricMeta,
}

/// The metric key, which represents a unique metric and its tags that is being emitted.
#[derive(Debug)]
pub struct MetricKey<'meta> {
    pub(crate) meta: &'meta MetricMeta,
    pub(crate) tag_values: TagValues,
}

impl<'meta> Deref for MetricKey<'meta> {
    type Target = MetricMeta;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl<'meta> MetricKey<'meta> {
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

/// A metric that is being emitted.
///
/// This consists of its [`MetricKey`] along with tag values, and the [`MetricValue`].
#[derive(Debug)]
pub struct Metric {
    pub(crate) key: MetricKey<'static>,
    pub(crate) value: MetricValue,
}

impl Deref for Metric {
    type Target = MetricKey<'static>;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl Metric {
    /// Returns the captured [`MetricValue`].
    pub fn value(&self) -> MetricValue {
        self.value
    }
}
