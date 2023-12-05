use std::sync::OnceLock;

use crate::Metric;

pub trait Recorder {
    fn record_metric(&self, metric: Metric<'_>);
}

impl<T: Recorder + ?Sized> Recorder for Box<T> {
    fn record_metric(&self, metric: Metric<'_>) {
        (**self).record_metric(metric)
    }
}

static GLOBAL_RECORDER: OnceLock<Box<dyn Recorder + Send + Sync + 'static>> = OnceLock::new();

pub fn init<R: Recorder + Send + Sync + 'static>(recorder: R) -> Result<(), R> {
    let mut result = Err(recorder);
    {
        let result = &mut result;
        let _ = GLOBAL_RECORDER.get_or_init(|| {
            let recorder = std::mem::replace(result, Ok(())).unwrap_err();
            Box::new(recorder)
        });
    }
    result
}

pub fn record_metric(metric: Metric<'_>) {
    with_recorder(|r| r.record_metric(metric))
}

pub fn with_recorder<F, R>(f: F) -> R
where
    F: FnOnce(&dyn Recorder) -> R,
    R: Default,
{
    if let Some(recorder) = GLOBAL_RECORDER.get() {
        f(recorder)
    } else {
        Default::default()
    }
}
