use std::ops::Deref;

use crate::tags::TagValues;
use crate::timer::Timestamp;
use crate::{MetricType, MetricUnit, MetricValue};

/// A source code location that can be added to a metric.
#[derive(Debug)]
pub struct Location<'a> {
    file: &'a str,
    line: u32,
    module_path: &'a str,
}

impl<'a> Location<'a> {
    /// Creates a new source code location, as file, line and module path.
    pub const fn new(file: &'a str, line: u32, module_path: &'a str) -> Self {
        Self {
            file,
            line,
            module_path,
        }
    }
}

/// The metadata of a particular metric.
///
/// This includes its type, unit, the metrics name (or key), and possibly
/// the code location at which it is emitted.
#[derive(Debug)]
pub struct MetricMeta {
    ty: MetricType,
    unit: MetricUnit,
    key: &'static str,
    // TODO:
    // target: &'static str,
    location: Option<&'static Location<'static>>,
    pub(crate) tag_keys: &'static [&'static str],
}

impl MetricMeta {
    /// Creates a new [`MetricMeta`] with the given type, unit and key.
    pub const fn new(ty: MetricType, unit: MetricUnit, key: &'static str) -> Self {
        Self {
            ty,
            unit,
            key,
            location: None,
            tag_keys: &[],
        }
    }

    /// Adds a source code [`Location`] to this metric.
    pub const fn with_location(mut self, location: &'static Location<'static>) -> Self {
        self.location = Some(location);
        self
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

    /// The source code file the metric is defined in, if available.
    pub fn file(&self) -> Option<&'static str> {
        self.location.as_ref().map(|l| l.file)
    }

    /// The source code line the metric is defined on, if available.
    pub fn line(&self) -> Option<u32> {
        self.location.as_ref().map(|l| l.line)
    }

    /// The source code module path the metric is defined in, if available.
    pub fn module_path(&self) -> Option<&'static str> {
        self.location.as_ref().map(|l| l.module_path)
    }
}

/// Metric metadata parameterized with the number of expected tags.
#[derive(Debug)]
pub struct TaggedMetricMeta<const N: usize> {
    pub(crate) meta: MetricMeta,
}

/// The metric key, which represents a unique metric and its tags that is being emitted.
#[derive(Debug)]
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

/// A metric that is being emitted.
///
/// This consists of its [`MetricKey`], [`MetricValue`], and the current [`Timestamp`].
#[derive(Debug)]
pub struct Metric {
    pub(crate) key: MetricKey,
    pub(crate) timestamp: Timestamp,
    pub(crate) value: MetricValue,
}

impl Deref for Metric {
    type Target = MetricKey;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl Metric {
    /// Splits the recorded metric into its key, timestamp and value.
    pub fn into_parts(self) -> (MetricKey, Timestamp, MetricValue) {
        let Self {
            key,
            timestamp,
            value,
        } = self;
        (key, timestamp, value)
    }

    /// Returns the captured [`MetricValue`].
    pub fn value(&self) -> MetricValue {
        self.value
    }

    /// Returns the timestamp at which the metric was captured.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}
