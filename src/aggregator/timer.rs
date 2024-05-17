use std::time::{Duration, SystemTime};

use quanta::{Clock, Instant};

#[derive(Debug)]
pub struct Timer {
    pub(crate) clock: Clock,
    pub(crate) start_instant: Instant,
    pub(crate) start_timestamp: Duration,
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

    pub(crate) fn timestamp(&self) -> Timestamp {
        let now = self.clock.recent();
        let elapsed = now.duration_since(self.start_instant);
        Timestamp {
            timestamp: self.start_timestamp + elapsed,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// The timestamp a metric was emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    timestamp: Duration,
}

impl Timestamp {
    /// The lowest possible timestamp.
    pub(crate) const ZERO: Timestamp = Self {
        timestamp: Duration::ZERO,
    };

    /// Returns the [`Duration`] since the unix epoch.
    ///
    /// This can further be used with [`SystemTime`] or any other datetime crate
    /// that supports unix epoch offsets.
    pub fn duration_since_unix_epoch(&self) -> Duration {
        self.timestamp
    }
}
