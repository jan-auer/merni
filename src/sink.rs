use std::sync::Arc;

use crate::Metric;

pub trait Sink {
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
