use crate::Metric;

pub trait Sink {
    fn emit(&self, metric: Metric);
}
