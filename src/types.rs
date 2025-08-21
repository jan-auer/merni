use std::time::Duration;

use crate::MetricMeta;

/// The Type of a Metric.
///
/// Counters, Gauges and Distributions are supported,
/// with more types to be added later.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// A counter metric.
    Counter,
    /// A gauge metric.
    Gauge,
    /// A distribution metric.
    Distribution,
    /// A timer metric.
    ///
    /// This is similar to [`MetricType::Distribution`], except it defaults to
    /// recording millisecond durations if no explicit [`MetricUnit`] was defined.
    Timer,
}

/// The Unit of a Metric.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MetricUnit {
    /// An unknown fallback unit.
    Unknown,
    /// The metric counts seconds.
    Seconds,
    /// The metric counts bytes.
    Bytes,
}

/// The value of a metric.
///
/// This is internally represented as a [`f64`].
#[derive(Debug, Clone, Copy)]
pub struct MetricValue {
    value: f64,
}

impl MetricValue {
    /// Creates a metric value.
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    /// Returns the metric value as a [`f64`].
    pub fn get(&self) -> f64 {
        self.value
    }
}

/// A trait that turns any value into a [`MetricValue`].
///
/// A metric value is represented as an [`f64`], and conversion has access to the
/// [`MetricMeta`] and in particular its [`MetricUnit`] to do an appropriate conversion.
pub trait IntoMetricValue {
    /// Converts the value into a metric value, guided by the given [`MetricMeta`].
    fn into_metric_value(self, meta: &MetricMeta) -> MetricValue;
}

macro_rules! into_metric_value {
    ($($ty:ident),+) => {
        $(
            impl IntoMetricValue for $ty {
                #[inline(always)]
                fn into_metric_value(self: $ty, _meta: &MetricMeta) -> MetricValue {
                    MetricValue::new(self as f64)
                }
            }
        )+
    };
}

#[rustfmt::skip]
into_metric_value!(
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64
);

impl IntoMetricValue for bool {
    fn into_metric_value(self, _meta: &MetricMeta) -> MetricValue {
        MetricValue::new(if self { 1. } else { 0. })
    }
}

impl IntoMetricValue for Duration {
    fn into_metric_value(self, meta: &MetricMeta) -> MetricValue {
        let secs = self.as_secs_f64();
        MetricValue::new(match meta.unit() {
            MetricUnit::Unknown if meta.ty() == MetricType::Timer => secs * 1_000.,
            _ => secs,
        })
    }
}
