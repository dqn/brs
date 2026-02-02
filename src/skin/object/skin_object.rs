use crate::skin::{MainState, SkinSourceManager};

/// Trait for renderable skin objects.
pub trait SkinObject {
    /// Prepare the object for rendering.
    fn prepare(&mut self, sources: &SkinSourceManager);

    /// Draw the object.
    fn draw(&self, state: &MainState, sources: &SkinSourceManager, now_time_us: i64);

    /// Check if the object should be visible based on options.
    fn is_visible(&self, state: &MainState) -> bool;
}

/// Check visibility based on option conditions.
pub fn check_option_visibility(options: &[i32], state: &MainState) -> bool {
    for &op in options {
        if op > 0 {
            // Positive: option must be true
            if !state.option(op) {
                return false;
            }
        } else if op < 0 {
            // Negative: option must be false
            if state.option(-op) {
                return false;
            }
        }
    }
    true
}

/// Check if a timer is active.
pub fn is_timer_active(timer_id: i32, state: &MainState) -> bool {
    if timer_id == 0 {
        return true; // No timer = always active
    }
    state.timer(timer_id) != crate::skin::skin_property::TIMER_OFF_VALUE
}

/// Get elapsed time since timer started (in microseconds).
pub fn get_timer_elapsed(timer_id: i32, state: &MainState, now_time_us: i64) -> i64 {
    if timer_id == 0 {
        return now_time_us; // Use absolute time
    }
    let timer_start = state.timer(timer_id);
    if timer_start == crate::skin::skin_property::TIMER_OFF_VALUE {
        return -1; // Timer not active
    }
    now_time_us - timer_start
}
