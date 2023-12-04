use std::cell::{Cell, RefCell};
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
    static LOCAL_RECORDER: Cell<Option<&'static (dyn Recorder + Send + Sync + 'static)>> = Cell::new(GLOBAL_RECORDER.get().map(|b| &**b));
}

pub fn init<R: Recorder + Send + Sync + 'static>(recorder: R) -> Result<(), R> {
    let mut result = Err(recorder);
    {
        let result = &mut result;
        let _ = GLOBAL_RECORDER.get_or_init(|| {
            let recorder = std::mem::replace(result, Ok(())).unwrap_err();
            Box::new(recorder)
        });
        if result.is_ok() {
            let global = GLOBAL_RECORDER.get().map(|b| &**b);
            LOCAL_RECORDER.set(global);
        }
    }
    result
}

pub fn record_metric(metric: Metric<'_>) {
    if let Some(recorder) = LOCAL_RECORDER.get() {
        STRING_BUFFER.with_borrow_mut(|s| {
            s.clear();
            metric.write_base_metric(s);
            metric.write_tags(s);
            recorder.emit(s);
        });
    }
}
