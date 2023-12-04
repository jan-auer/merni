use std::cell::RefCell;
use std::sync::OnceLock;

use crate::Metric;

pub trait Recorder {
    fn emit(&self, metric: &str);
}

impl<T: Recorder + ?Sized> Recorder for Box<T> {
    fn emit(&self, metric: &str) {
        (**self).emit(metric)
    }
}

static GLOBAL_RECORDER: OnceLock<Box<dyn Recorder + Send + Sync + 'static>> = OnceLock::new();

thread_local! {
    static STRING_BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
}

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
    if let Some(recorder) = GLOBAL_RECORDER.get() {
        STRING_BUFFER.with_borrow_mut(|s| {
            s.clear();
            s.reserve(256);
            metric.write_base_metric(s);
            metric.write_tags(s);
            recorder.emit(s);
        });
    }
}
