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

pub use dispatch::*;
pub use globals::*;
pub use metric::*;
pub use sink::*;
pub use types::*;

#[cfg(feature = "aggregator")]
mod aggregator;
#[cfg(feature = "aggregator")]
pub use aggregator::*;

#[cfg(any(test, feature = "testing"))]
/// This contains some utilities used for testing
pub mod testing;

#[cfg(test)]
mod tests;
