use std::sync::Arc;

use crate::Metric;

/// A Sink for metrics emmission.
pub trait Sink {
    /// This fn is being called when a metric is emitted.
    fn emit(&self, metric: Metric);
}

impl<S: Sink> Sink for Arc<S> {
    fn emit(&self, metric: Metric) {
        (**self).emit(metric)
    }
}
impl<S: Sink> Sink for &S {
    #[inline]
    fn emit(&self, metric: Metric) {
        (**self).emit(metric)
    }
}
