#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

// #[cfg(feature = "cadence1")]
// mod cadence1;
mod dispatch;
mod globals;
mod macros;
mod metric;
// mod statsd;
mod aggregator;
mod sink;
mod tags;
mod timer;
mod types;

pub use dispatch::Dispatcher;
pub use globals::{
    set_global_dispatcher, set_local_dispatcher, with_dispatcher, LocalDispatcherGuard,
};
pub use metric::{Location, Metric, MetricKey, MetricMeta, TaggedMetricMeta};
pub use types::{IntoMetricValue, MetricType, MetricUnit, MetricValue};

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;
