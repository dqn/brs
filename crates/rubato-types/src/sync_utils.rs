//! Shared synchronization utilities.

use std::sync::{Mutex, MutexGuard};

/// Acquire a mutex lock, recovering from poison if a thread panicked while holding it.
///
/// In a game/media application, a poisoned mutex almost always indicates a programming
/// bug in another thread rather than corrupt data. Recovering gracefully (instead of
/// panicking the current thread) avoids cascading failures and keeps the application
/// responsive for debugging.
pub fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_succeeds_on_healthy_mutex() {
        let mutex = Mutex::new(42);
        let guard = lock_or_recover(&mutex);
        assert_eq!(*guard, 42);
    }

    #[test]
    fn lock_recovers_from_poisoned_mutex() {
        let mutex = Mutex::new(42);
        // Poison the mutex by panicking while holding the lock.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().expect("lock");
            panic!("intentional poison");
        }));
        assert!(mutex.is_poisoned());

        let guard = lock_or_recover(&mutex);
        assert_eq!(*guard, 42);
    }
}
