use std::sync::Arc;
use std::time::Duration;

use quanta::{Clock, Mock};

use super::timer::Timer;

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
