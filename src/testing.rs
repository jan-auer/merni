use std::sync::{Arc, Mutex};
use std::time::Duration;

use quanta::{Clock, Mock};

use crate::sink::Sink;
use crate::timer::Timer;
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

impl Dispatcher {
    pub fn with_timer(mut self, timer: Timer) -> Self {
        self.timer = timer;
        self
    }
}

impl Timer {
    pub fn mock() -> (Self, Arc<Mock>) {
        let (clock, mock) = Clock::mock();
        let start_instant = clock.now();

        let timer = Self {
            clock,
            start_instant,
            start_timestamp: Duration::ZERO,
        };
        (timer, mock)
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
    mock: Arc<Mock>,
}

impl TestDispatcher {
    /// Starts a new test [`Dispatcher`].
    pub fn new() -> Self {
        let metrics: TestMetrics = Default::default();
        let (timer, mock) = Timer::mock();
        let sink = TestSink {
            metrics: metrics.clone(),
        };
        let dispatcher = Dispatcher::new(sink).with_timer(timer);

        let guard = set_local_dispatcher(dispatcher);

        Self {
            dispatcher: guard,
            metrics,
            mock,
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

    /// Advances the mocked timestamp by the given [`Duration`]
    pub fn advance_time(&self, duration: Duration) {
        self.mock.increment(duration)
    }
}
