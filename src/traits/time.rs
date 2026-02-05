/// Abstraction over time sources.
/// Implementations: SystemTimeProvider (production), MockTimeProvider (testing).
pub trait TimeProvider {
    /// Current time in microseconds from an arbitrary epoch.
    fn now_us(&self) -> i64;
}

/// System time provider using std::time::Instant.
pub struct SystemTimeProvider {
    start: std::time::Instant,
}

impl SystemTimeProvider {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Default for SystemTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeProvider for SystemTimeProvider {
    fn now_us(&self) -> i64 {
        self.start.elapsed().as_micros() as i64
    }
}

/// Mock time provider for deterministic testing.
pub struct MockTimeProvider {
    current_us: std::cell::Cell<i64>,
}

impl MockTimeProvider {
    pub fn new() -> Self {
        Self {
            current_us: std::cell::Cell::new(0),
        }
    }

    pub fn set_time(&self, us: i64) {
        self.current_us.set(us);
    }

    pub fn advance(&self, delta_us: i64) {
        self.current_us.set(self.current_us.get() + delta_us);
    }
}

impl Default for MockTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeProvider for MockTimeProvider {
    fn now_us(&self) -> i64 {
        self.current_us.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_time_provider_advance() {
        let tp = MockTimeProvider::new();
        assert_eq!(tp.now_us(), 0);
        tp.advance(1_000_000);
        assert_eq!(tp.now_us(), 1_000_000);
        tp.advance(500_000);
        assert_eq!(tp.now_us(), 1_500_000);
    }

    #[test]
    fn mock_time_provider_set() {
        let tp = MockTimeProvider::new();
        tp.set_time(5_000_000);
        assert_eq!(tp.now_us(), 5_000_000);
    }

    #[test]
    fn system_time_provider_monotonic() {
        let tp = SystemTimeProvider::new();
        let t1 = tp.now_us();
        let t2 = tp.now_us();
        assert!(t2 >= t1);
    }
}
