//! Process-global monotonic clock.
//!
//! Java's `System.nanoTime()` provides a monotonic clock with a fixed (but
//! arbitrary) origin that is shared by every thread in the JVM process.
//! Both `TimerManager` and the various input processors in beatoraja relied
//! on this single shared timeline.
//!
//! In Rust, `std::time::Instant` is the monotonic equivalent, but each
//! `Instant::now()` call is only meaningful relative to other `Instant`
//! values.  To replicate the Java pattern we create a process-wide epoch
//! `Instant` at first access and express all times as microseconds elapsed
//! since that epoch.

use std::sync::OnceLock;
use std::time::Instant;

static PROCESS_EPOCH: OnceLock<Instant> = OnceLock::new();

/// Returns the process-wide epoch instant.
pub fn process_epoch() -> &'static Instant {
    PROCESS_EPOCH.get_or_init(Instant::now)
}

/// Returns monotonic microseconds elapsed since the process-wide epoch.
///
/// This is the Rust equivalent of `System.nanoTime() / 1000` in Java.
pub fn monotonic_micros() -> i64 {
    process_epoch().elapsed().as_micros() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonic_micros_increases() {
        let a = monotonic_micros();
        // Busy-wait briefly to ensure measurable difference
        let start = Instant::now();
        while start.elapsed().as_micros() < 100 {}
        let b = monotonic_micros();
        assert!(b > a, "b={b} should be greater than a={a}");
    }
}
