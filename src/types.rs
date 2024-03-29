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
    // TODO?:
    // Histogram,
    // Meter,
    // Set,
}

/// The Unit of a Metric.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricUnit {
    /// An unknown fallback unit.
    Unknown,
}

/// A trait that turns any value into a metric value.
///
/// A metric value is represented as a [`f64`], and conversion has access to the
/// [`MetricMeta`] and in particular its [`MetricUnit`] to do an appropriate conversion.
pub trait IntoMetricValue {
    /// Converts [`self`] into a metric value, guided by the given [`MetricMeta`].
    fn into_metric_value(self, meta: &MetricMeta) -> f64;
}

macro_rules! into_metric_value {
    ($($ty:ident),+) => {
        $(
            impl IntoMetricValue for $ty {
                #[inline(always)]
                fn into_metric_value(self: $ty, _meta: &MetricMeta) -> f64 {
                    self as f64
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
    fn into_metric_value(self, _meta: &MetricMeta) -> f64 {
        if self {
            1.
        } else {
            0.
        }
    }
}

impl IntoMetricValue for Duration {
    fn into_metric_value(self, meta: &MetricMeta) -> f64 {
        let secs = self.as_secs_f64();
        match meta.unit() {
            MetricUnit::Unknown => {
                if meta.ty() == MetricType::Timer {
                    secs * 1_000.
                } else {
                    secs
                }
            }
        }
    }
}
