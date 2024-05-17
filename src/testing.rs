use std::sync::{Arc, Mutex};

use crate::sink::Sink;
use crate::{set_local_dispatcher, Dispatcher, LocalDispatcherGuard, Metric};

type TestMetrics = Arc<Mutex<Vec<Metric>>>;

pub struct TestSink {
    metrics: TestMetrics,
}

impl Sink for TestSink {
    fn emit(&self, metric: Metric) {
        self.metrics.lock().unwrap().push(metric)
    }
}

/// A guard for a currently configured test [`Dispatcher`].
///
/// The test dispatcher will record all the emitted metrics.
/// Calling [`finish()`](TestDispatcher::finish) unregisters the dispatcher,
/// and returns all previously captured [`Metric`]s.
///
/// It is possible to manipulate the timestamps of emitted [`Metric`]s by calling
/// [`advance_time`](TestDispatcher::advance_time)
pub struct TestDispatcher {
    dispatcher: LocalDispatcherGuard,
    metrics: TestMetrics,
}

impl TestDispatcher {
    /// Starts a new test [`Dispatcher`].
    pub fn new() -> Self {
        let metrics: TestMetrics = Default::default();
        let sink = TestSink {
            metrics: metrics.clone(),
        };
        let dispatcher = Dispatcher::new(sink);

        let guard = set_local_dispatcher(dispatcher);

        Self {
            dispatcher: guard,
            metrics,
        }
    }
    /// Consumes this guard, returning all the captured [`Metric`]s.
    pub fn finish(self) -> Vec<Metric> {
        drop(self.dispatcher);
        Arc::into_inner(self.metrics)
            .expect("dispatcher should be dropped")
            .into_inner()
            .unwrap()
    }
}
