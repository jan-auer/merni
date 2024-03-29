#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

// #[cfg(feature = "cadence1")]
// mod cadence1;
mod dispatch;
// mod globals;
mod key;
// mod macros;
mod metric;
// mod statsd;
mod tags;
mod timer;
mod types;

// #[cfg(feature = "cadence1")]
// pub use cadence1::*;
pub use dispatch::*;
// pub use globals::*;
pub use key::*;
// pub use macros::*;
pub use metric::*;
// pub use statsd::*;
pub use types::*;

// #[cfg(test)]
// mod tests;
