#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

mod dispatch;
mod globals;
#[doc(hidden)]
pub mod macros;
mod metric;
mod sink;
mod tags;
mod types;

pub use dispatch::Dispatcher;
pub use globals::{
    set_global_dispatcher, set_local_dispatcher, with_dispatcher, LocalDispatcherGuard,
};
pub use metric::{Metric, MetricKey, MetricMeta, TaggedMetricMeta};
pub use types::{IntoMetricValue, MetricType, MetricUnit, MetricValue};

#[cfg(feature = "aggregator")]
mod aggregator;
#[cfg(feature = "aggregator")]
pub use aggregator::{
    AggregatedGauge, AggregatedMetric, AggregationSink, Aggregations,
    PreciseAggregatedDistribution, ThreadLocalAggregator,
};

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;
