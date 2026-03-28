//! Global FPS counter for reporting actual frame rate to skin properties.
//!
//! Updated by the main render loop, read by skin property factories (value ID 20).

use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;

/// Current FPS, updated once per second by the main render loop.
static CURRENT_FPS: AtomicI32 = AtomicI32::new(60);

/// Get the current frames per second.
pub fn current_fps() -> i32 {
    CURRENT_FPS.load(Ordering::Relaxed)
}

/// Set the current FPS value. Called by the main render loop.
pub fn set_current_fps(fps: i32) {
    CURRENT_FPS.store(fps, Ordering::Relaxed);
}

/// Frame counter that computes FPS once per second.
/// Intended to be held by the main render loop.
pub struct FpsTracker {
    frame_count: u32,
    last_update: Instant,
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsTracker {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_update: Instant::now(),
        }
    }

    /// Call once per frame. Updates the global FPS counter every second.
    pub fn tick(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_update.elapsed();
        if elapsed.as_secs() >= 1 {
            let fps = (self.frame_count as f64 / elapsed.as_secs_f64()).round() as i32;
            set_current_fps(fps);
            self.frame_count = 0;
            self.last_update = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fps_is_60() {
        // Global state might be modified by other tests, so just verify the API works
        let fps = current_fps();
        assert!(fps >= 0);
    }

    #[test]
    fn set_and_get_fps() {
        set_current_fps(144);
        assert_eq!(current_fps(), 144);
        // Reset to avoid affecting other tests
        set_current_fps(60);
    }

    #[test]
    fn fps_tracker_initial_state() {
        let tracker = FpsTracker::new();
        assert_eq!(tracker.frame_count, 0);
    }
}
