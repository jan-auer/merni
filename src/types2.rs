// use std::panic::Location;

use std::fmt::Display;
use std::ops::Deref;

use crate::{MetricType, MetricUnit};

#[doc(hidden)]
pub struct Location<'a> {
    file: &'a str,
    line: u32,
    module_path: &'a str,
}

impl<'a> Location<'a> {
    pub const fn new(file: &'a str, line: u32, module_path: &'a str) -> Self {
        Self {
            file,
            line,
            module_path,
        }
    }
}

pub struct MetricMeta {
    ty: MetricType,
    unit: MetricUnit,
    key: &'static str,
    location: Option<&'static Location<'static>>,
    tag_keys: &'static [&'static str],
}

pub struct TaggedMetric<const N: usize> {
    meta: MetricMeta,
}

impl MetricMeta {
    pub const fn new(ty: MetricType, unit: MetricUnit, key: &'static str) -> Self {
        Self {
            ty,
            unit,
            key,
            location: None,
            tag_keys: &[],
        }
    }

    pub const fn with_location(mut self, location: &'static Location<'static>) -> Self {
        self.location = Some(location);
        self
    }

    pub const fn with_tags<const N: usize>(
        mut self,
        tag_keys: &'static [&'static str; N],
    ) -> TaggedMetric<N> {
        self.tag_keys = tag_keys;
        TaggedMetric { meta: self }
    }

    fn record(&'static self, value: f64, tag_values: InputTags) -> RecordedMetric {
        let key = MetricKey {
            meta: self,
            tag_values: record_tags(tag_values),
        };
        RecordedMetric { key, value }
    }
}

impl<const N: usize> TaggedMetric<N> {
    pub fn emit(
        &'static self,
        value: impl IntoMetricValue,
        tag_values: [&dyn Display; N],
    ) -> RecordedMetric {
        let value = value.into_metric_value(self.unit);
        self.record(value, &tag_values)
    }
}

impl<const N: usize> Deref for TaggedMetric<N> {
    type Target = MetricMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

pub trait IntoMetricValue {
    fn into_metric_value(self, unit: MetricUnit) -> f64;
}

pub struct RecordedMetric {
    key: MetricKey,
    value: f64,
}

type InputTags<'a> = &'a [&'a dyn Display];
type SmolStr = Box<str>;
type TagValues = Box<[SmolStr]>;

pub struct MetricKey {
    meta: &'static MetricMeta,
    tag_values: TagValues,
}

fn record_tags(tags: InputTags) -> TagValues {
    tags.iter().map(|d| d.to_string().into()).collect()
}
