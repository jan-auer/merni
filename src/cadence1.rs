use crate::MetricSink;

impl<T> MetricSink for T
where
    T: cadence::MetricSink,
{
    fn emit(&self, metric: &str) {
        let _ = self.emit(metric);
    }
}
