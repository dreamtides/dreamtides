use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use tv_lib::traits::Clock;

pub struct MockClock {
    current_time: Mutex<SystemTime>,
}

impl MockClock {
    pub fn new(time: SystemTime) -> Self {
        Self { current_time: Mutex::new(time) }
    }

    pub fn at_unix_epoch() -> Self {
        Self::new(SystemTime::UNIX_EPOCH)
    }

    pub fn advance(&self, duration: Duration) {
        let mut time = self.current_time.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}"));
        *time = time
            .checked_add(duration)
            .unwrap_or_else(|| panic!("Time overflow when advancing by {duration:?}"));
    }

    pub fn set_time(&self, time: SystemTime) {
        *self.current_time.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}")) = time;
    }
}

impl Clock for MockClock {
    fn now(&self) -> SystemTime {
        *self.current_time.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}"))
    }
}
