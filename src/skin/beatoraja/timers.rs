//! Timer system for beatoraja skin animations
//!
//! Manages animation timing based on game events.

use std::collections::HashMap;

use super::conditions::timers;

/// Timer state for a single timer
#[derive(Debug, Clone, Copy, Default)]
pub struct TimerState {
    /// When the timer started (seconds from scene start)
    /// None if timer hasn't been triggered
    start_time: Option<f64>,
    /// Whether the timer is currently active
    active: bool,
}

impl TimerState {
    /// Create a new inactive timer
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the timer at the given time
    pub fn start(&mut self, time: f64) {
        self.start_time = Some(time);
        self.active = true;
    }

    /// Stop/reset the timer
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Get elapsed time since timer start
    /// Returns None if timer hasn't started
    pub fn elapsed(&self, current_time: f64) -> Option<f64> {
        self.start_time.map(|start| current_time - start)
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self, current_time: f64) -> Option<i32> {
        self.elapsed(current_time).map(|e| (e * 1000.0) as i32)
    }

    /// Check if timer is active
    pub fn is_active(&self) -> bool {
        self.active && self.start_time.is_some()
    }

    /// Check if timer has started (even if not active)
    pub fn has_started(&self) -> bool {
        self.start_time.is_some()
    }
}

/// Timer manager for skin animation
#[derive(Debug, Clone)]
pub struct TimerManager {
    /// Standard timers by ID
    timers: HashMap<i32, TimerState>,
    /// Custom timers defined by skin
    custom_timers: HashMap<String, TimerState>,
    /// Scene start time (for reference)
    scene_start: f64,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerManager {
    /// Create a new timer manager
    pub fn new() -> Self {
        let mut manager = Self {
            timers: HashMap::new(),
            custom_timers: HashMap::new(),
            scene_start: 0.0,
        };

        // Initialize scene start timer
        manager
            .timers
            .insert(timers::SCENE_START, TimerState::new());

        manager
    }

    /// Initialize the manager with scene start time
    pub fn init(&mut self, scene_start: f64) {
        self.scene_start = scene_start;

        // Start the scene start timer
        if let Some(timer) = self.timers.get_mut(&timers::SCENE_START) {
            timer.start(scene_start);
        }
    }

    /// Start a timer by ID
    pub fn start_timer(&mut self, id: i32, time: f64) {
        self.timers.entry(id).or_default().start(time);
    }

    /// Stop a timer by ID
    pub fn stop_timer(&mut self, id: i32) {
        if let Some(timer) = self.timers.get_mut(&id) {
            timer.stop();
        }
    }

    /// Get timer elapsed time in milliseconds
    pub fn get_elapsed_ms(&self, id: i32, current_time: f64) -> Option<i32> {
        // Special case: timer -1 means always visible
        if id == timers::DISABLED {
            return Some(0);
        }

        self.timers
            .get(&id)
            .and_then(|t| t.elapsed_ms(current_time))
    }

    /// Check if a timer is active
    pub fn is_active(&self, id: i32) -> bool {
        // Timer -1 is always "active"
        if id == timers::DISABLED {
            return true;
        }

        self.timers.get(&id).is_some_and(|t| t.is_active())
    }

    /// Check if a timer has ever been triggered
    pub fn has_started(&self, id: i32) -> bool {
        if id == timers::DISABLED {
            return true;
        }

        self.timers.get(&id).is_some_and(|t| t.has_started())
    }

    /// Start a custom timer by name
    pub fn start_custom_timer(&mut self, name: &str, time: f64) {
        self.custom_timers
            .entry(name.to_string())
            .or_default()
            .start(time);
    }

    /// Get custom timer elapsed time
    pub fn get_custom_elapsed_ms(&self, name: &str, current_time: f64) -> Option<i32> {
        self.custom_timers
            .get(name)
            .and_then(|t| t.elapsed_ms(current_time))
    }

    // ==========================================================================
    // Game event triggers
    // ==========================================================================

    /// Trigger skin loaded event
    pub fn on_skin_loaded(&mut self, time: f64) {
        self.start_timer(timers::SKIN_LOADED, time);
    }

    /// Trigger play start event
    pub fn on_play_start(&mut self, time: f64) {
        self.start_timer(timers::PLAY_START, time);
    }

    /// Trigger music start event
    pub fn on_music_start(&mut self, time: f64) {
        self.start_timer(timers::MUSIC_START, time);
    }

    /// Trigger play end event
    pub fn on_play_end(&mut self, time: f64) {
        self.start_timer(timers::PLAY_END, time);
    }

    /// Trigger failed event
    pub fn on_failed(&mut self, time: f64) {
        self.start_timer(timers::FAILED, time);
    }

    /// Trigger result loaded event
    pub fn on_result_loaded(&mut self, time: f64) {
        self.start_timer(timers::RESULT_LOADED, time);
    }

    /// Trigger judge event for 1P
    pub fn on_judge_1p(&mut self, judge_type: JudgeTimerType, time: f64) {
        let timer_id = match judge_type {
            JudgeTimerType::PGreat => timers::JUDGE_1P_PGREAT,
            JudgeTimerType::Great => timers::JUDGE_1P_GREAT,
            JudgeTimerType::Good => timers::JUDGE_1P_GOOD,
            JudgeTimerType::Bad => timers::JUDGE_1P_BAD,
            JudgeTimerType::Poor => timers::JUDGE_1P_POOR,
            JudgeTimerType::Miss => timers::JUDGE_1P_MISS,
        };
        self.start_timer(timer_id, time);
    }

    /// Trigger key press event for 1P
    pub fn on_key_press_1p(&mut self, lane: usize, time: f64) {
        if lane < 8 {
            self.start_timer(timers::KEY_1P_1_ON + lane as i32, time);
        }
    }

    /// Trigger key release event for 1P
    pub fn on_key_release_1p(&mut self, lane: usize, time: f64) {
        if lane < 8 {
            self.start_timer(timers::KEY_1P_1_OFF + lane as i32, time);
        }
    }

    /// Trigger combo milestone event
    pub fn on_combo_milestone(&mut self, combo: u32, time: f64) {
        if (100..500).contains(&combo) {
            self.start_timer(timers::COMBO_100, time);
        } else if (500..1000).contains(&combo) {
            self.start_timer(timers::COMBO_500, time);
        } else if combo >= 1000 {
            self.start_timer(timers::COMBO_1000, time);
        }
    }
}

/// Judge type for timer selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeTimerType {
    PGreat,
    Great,
    Good,
    Bad,
    Poor,
    Miss,
}

/// Calculate animation frame based on timer and cycle
pub fn calculate_frame(
    elapsed_ms: i32,
    cycle: i32,
    total_frames: i32,
    loop_type: i32,
) -> Option<i32> {
    if total_frames <= 0 {
        return Some(0);
    }

    if cycle <= 0 {
        // No animation, always first frame
        return Some(0);
    }

    let frame_duration = cycle / total_frames;
    if frame_duration <= 0 {
        return Some(0);
    }

    let frame = elapsed_ms / frame_duration;

    match loop_type {
        0 => {
            // No loop - stay at last frame after cycle
            if frame >= total_frames {
                Some(total_frames - 1)
            } else {
                Some(frame)
            }
        }
        1 => {
            // Loop
            Some(frame % total_frames)
        }
        _ => Some(frame % total_frames),
    }
}

/// Interpolate between two destination keyframes
pub fn interpolate_destination(
    elapsed_ms: i32,
    time1: i32,
    time2: i32,
    value1: i32,
    value2: i32,
    acc: i32,
) -> i32 {
    if time2 <= time1 {
        return value1;
    }

    let progress = (elapsed_ms - time1) as f32 / (time2 - time1) as f32;
    let progress = progress.clamp(0.0, 1.0);

    // Apply acceleration curve
    let adjusted = apply_acceleration(progress, acc);

    let diff = value2 - value1;
    value1 + (diff as f32 * adjusted) as i32
}

/// Apply acceleration curve to progress
fn apply_acceleration(progress: f32, acc: i32) -> f32 {
    match acc {
        0 => progress,                                  // Linear
        1 => progress * progress,                       // Ease in (quadratic)
        2 => 1.0 - (1.0 - progress) * (1.0 - progress), // Ease out
        3 => {
            // Ease in-out
            if progress < 0.5 {
                2.0 * progress * progress
            } else {
                1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
            }
        }
        4 => progress * progress * progress, // Ease in (cubic)
        5 => 1.0 - (1.0 - progress).powi(3), // Ease out (cubic)
        _ => progress,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_state() {
        let mut timer = TimerState::new();
        assert!(!timer.is_active());
        assert!(!timer.has_started());

        timer.start(1.0);
        assert!(timer.is_active());
        assert!(timer.has_started());

        assert_eq!(timer.elapsed(1.5), Some(0.5));
        assert_eq!(timer.elapsed_ms(1.5), Some(500));

        timer.stop();
        assert!(!timer.is_active());
        assert!(timer.has_started());
    }

    #[test]
    fn test_timer_manager_scene_start() {
        let mut manager = TimerManager::new();
        manager.init(0.0);

        assert!(manager.is_active(timers::SCENE_START));
        assert_eq!(manager.get_elapsed_ms(timers::SCENE_START, 1.0), Some(1000));
    }

    #[test]
    fn test_timer_manager_disabled() {
        let manager = TimerManager::new();

        // Timer -1 should always return Some(0)
        assert!(manager.is_active(timers::DISABLED));
        assert_eq!(manager.get_elapsed_ms(timers::DISABLED, 999.0), Some(0));
    }

    #[test]
    fn test_calculate_frame_no_loop() {
        // 4 frames, 1000ms cycle
        assert_eq!(calculate_frame(0, 1000, 4, 0), Some(0));
        assert_eq!(calculate_frame(250, 1000, 4, 0), Some(1));
        assert_eq!(calculate_frame(500, 1000, 4, 0), Some(2));
        assert_eq!(calculate_frame(750, 1000, 4, 0), Some(3));
        // After cycle, stay at last frame
        assert_eq!(calculate_frame(1000, 1000, 4, 0), Some(3));
        assert_eq!(calculate_frame(2000, 1000, 4, 0), Some(3));
    }

    #[test]
    fn test_calculate_frame_loop() {
        // 4 frames, 1000ms cycle, looping
        assert_eq!(calculate_frame(0, 1000, 4, 1), Some(0));
        assert_eq!(calculate_frame(1000, 1000, 4, 1), Some(0));
        assert_eq!(calculate_frame(1250, 1000, 4, 1), Some(1));
    }

    #[test]
    fn test_interpolate_linear() {
        assert_eq!(interpolate_destination(0, 0, 1000, 0, 100, 0), 0);
        assert_eq!(interpolate_destination(500, 0, 1000, 0, 100, 0), 50);
        assert_eq!(interpolate_destination(1000, 0, 1000, 0, 100, 0), 100);
    }

    #[test]
    fn test_interpolate_clamped() {
        // Before start
        assert_eq!(interpolate_destination(-100, 0, 1000, 0, 100, 0), 0);
        // After end
        assert_eq!(interpolate_destination(2000, 0, 1000, 0, 100, 0), 100);
    }

    #[test]
    fn test_acceleration_curves() {
        // Linear
        assert!((apply_acceleration(0.5, 0) - 0.5).abs() < 0.01);

        // Ease in (should be less than linear at 0.5)
        assert!(apply_acceleration(0.5, 1) < 0.5);

        // Ease out (should be more than linear at 0.5)
        assert!(apply_acceleration(0.5, 2) > 0.5);
    }
}
