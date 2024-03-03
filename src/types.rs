use core::fmt::{self, Display};
use std::time::Duration;

use crate::MetricMeta;

/// The Type of a Metric.
///
/// Counters, Gauges and Distributions are supported,
/// with more types to be added later.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// A counter metric, using the StatsD `c` type.
    Counter,
    /// A gauge metric, using the StatsD `g` type.
    Gauge,
    /// A distribution metric, using the StatsD `d` type.
    Distribution,
    /// A timer metric, similar to `Distribution`, but using the StatsD `ms` type.
    Timer,
    /// A histogram metric, similar to `Distribution`, but using the StatsD `h` type.
    Histogram,
    // TODO?:
    // Meter,
    // Set,
}

impl MetricType {
    /// Returns the StatsD metrics type.
    pub fn as_str(&self) -> &str {
        match self {
            MetricType::Counter => "c",
            MetricType::Gauge => "g",
            MetricType::Distribution => "d",
            MetricType::Timer => "ms",
            MetricType::Histogram => "h",
        }
    }
}

impl Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The Unit of a Metric.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricUnit {
    /// An unknown fallback unit.
    Unknown,
}

pub trait IntoMetricValue {
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
    fn into_metric_value(self, meta: &MetricMeta) -> f64 {
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
