use std::time::{Duration, SystemTime};

use quanta::{Clock, Instant};

pub struct Timer {
    clock: Clock,
    start_instant: Instant,
    start_timestamp: Duration,
}

impl Timer {
    pub fn new() -> Self {
        let start_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let clock = Clock::new();
        let start_instant = clock.now();

        Self {
            clock,
            start_instant,
            start_timestamp,
        }
    }

    pub(crate) fn now(&self) -> Instant {
        self.clock.recent()
    }
}
