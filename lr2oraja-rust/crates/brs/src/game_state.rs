// SharedGameState + GameStateProvider — bridges TimerManager to SkinStateProvider.
//
// SharedGameState holds a snapshot of the current game state that
// the SkinStateProvider reads from. A sync system updates it each frame.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bms_render::state_provider::SkinStateProvider;
use bms_skin::property_id::{BooleanId, FloatId, IntegerId, StringId, TIMER_MAX, TimerId};
use bms_skin::skin_object::SkinOffset;

use crate::timer_manager::{TIMER_OFF, TimerManager};

/// Snapshot of game state readable by the skin renderer.
#[derive(Debug, Clone, Default)]
pub struct SharedGameState {
    /// Active timers: timer_id -> elapsed milliseconds.
    /// Absent entries mean the timer is OFF.
    pub timers: HashMap<i32, i64>,
    #[allow(dead_code)]
    pub integers: HashMap<i32, i32>,
    #[allow(dead_code)]
    pub floats: HashMap<i32, f32>,
    #[allow(dead_code)]
    pub strings: HashMap<i32, String>,
    #[allow(dead_code)]
    pub booleans: HashMap<i32, bool>,
    #[allow(dead_code)]
    pub offsets: HashMap<i32, SkinOffset>,
    pub now_time_ms: i64,
}

/// SkinStateProvider implementation backed by SharedGameState.
#[allow(dead_code)]
pub struct GameStateProvider {
    state: Arc<RwLock<SharedGameState>>,
}

impl GameStateProvider {
    #[allow(dead_code)]
    pub fn new(state: Arc<RwLock<SharedGameState>>) -> Self {
        Self { state }
    }
}

impl SkinStateProvider for GameStateProvider {
    fn timer_value(&self, timer: TimerId) -> Option<i64> {
        let state = self.state.read().unwrap();
        state.timers.get(&timer.0).copied()
    }

    fn integer_value(&self, id: IntegerId) -> i32 {
        let state = self.state.read().unwrap();
        state.integers.get(&id.0).copied().unwrap_or(0)
    }

    fn float_value(&self, id: FloatId) -> f32 {
        let state = self.state.read().unwrap();
        state.floats.get(&id.0).copied().unwrap_or(0.0)
    }

    fn string_value(&self, id: StringId) -> Option<String> {
        let state = self.state.read().unwrap();
        state.strings.get(&id.0).cloned()
    }

    fn boolean_value(&self, id: BooleanId) -> bool {
        let state = self.state.read().unwrap();
        let raw = state.booleans.get(&id.abs_id()).copied().unwrap_or(false);
        if id.is_negated() { !raw } else { raw }
    }

    fn now_time_ms(&self) -> i64 {
        let state = self.state.read().unwrap();
        state.now_time_ms
    }

    fn offset_value(&self, id: i32) -> SkinOffset {
        let state = self.state.read().unwrap();
        state.offsets.get(&id).copied().unwrap_or_default()
    }
}

/// Synchronizes TimerManager state into SharedGameState.
///
/// Called once per frame to update the shared state snapshot
/// that the renderer reads from.
pub fn sync_timer_state(timer: &TimerManager, state: &Arc<RwLock<SharedGameState>>) {
    let mut shared = state.write().unwrap();
    shared.now_time_ms = timer.now_time();

    // Sync all standard timers
    shared.timers.clear();
    for id in 0..=TIMER_MAX {
        let val = timer.micro_timer(id);
        if val != TIMER_OFF {
            // Convert absolute microsecond time to elapsed milliseconds
            let elapsed_ms = (timer.now_micro_time() - val) / 1000;
            shared.timers.insert(id, elapsed_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Arc<RwLock<SharedGameState>> {
        Arc::new(RwLock::new(SharedGameState::default()))
    }

    #[test]
    fn timer_value_from_shared_state() {
        let state = make_state();
        state.write().unwrap().timers.insert(1, 500);

        let provider = GameStateProvider::new(state);
        assert_eq!(provider.timer_value(TimerId(1)), Some(500));
        assert_eq!(provider.timer_value(TimerId(999)), None);
    }

    #[test]
    fn integer_value_from_shared_state() {
        let state = make_state();
        state.write().unwrap().integers.insert(42, 123);

        let provider = GameStateProvider::new(state);
        assert_eq!(provider.integer_value(IntegerId(42)), 123);
        assert_eq!(provider.integer_value(IntegerId(99)), 0);
    }

    #[test]
    fn boolean_negation() {
        let state = make_state();
        state.write().unwrap().booleans.insert(5, true);

        let provider = GameStateProvider::new(state);
        assert!(provider.boolean_value(BooleanId(5)));
        assert!(!provider.boolean_value(BooleanId(-5)));
    }

    #[test]
    fn sync_timer_state_populates_shared() {
        let mut tm = TimerManager::new();
        tm.set_now_micro_time(10_000);
        tm.set_timer_on(1); // TIMER_STARTINPUT = now_micro_time = 10_000

        tm.set_now_micro_time(15_000);

        let state = make_state();
        sync_timer_state(&tm, &state);

        let shared = state.read().unwrap();
        assert_eq!(shared.now_time_ms, 15); // 15_000 / 1000
        // Timer 1 was set at 10_000, now is 15_000, elapsed = 5_000 us = 5 ms
        assert_eq!(shared.timers.get(&1), Some(&5));
        // Inactive timers should not be present
        assert!(!shared.timers.contains_key(&2));
    }

    #[test]
    fn offset_returns_default_when_missing() {
        let state = make_state();
        let provider = GameStateProvider::new(state);
        let offset = provider.offset_value(999);
        assert_eq!(offset.x, 0.0);
        assert_eq!(offset.y, 0.0);
    }
}
