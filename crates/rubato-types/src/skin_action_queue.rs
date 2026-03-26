use crate::main_state_type::MainStateType;
use crate::timer_id::TimerId;

/// Collects write-back actions from skin rendering (mouse clicks, Lua scripts).
///
/// During skin rendering, the skin system is given a read-only `PropertySnapshot`.
/// Any mutations (config changes, audio playback, state transitions, etc.) are
/// queued here instead of mutating game state directly. The owning screen state
/// drains the queue after the skin render pass and applies the mutations.
#[derive(Clone, Debug, Default)]
pub struct SkinActionQueue {
    /// Float property writes (slider position, volume, etc.).
    pub float_writes: Vec<(i32, f32)>,
    /// Timer set requests (timer ID, microsecond value).
    pub timer_sets: Vec<(TimerId, i64)>,
    /// State transition requests.
    pub state_changes: Vec<MainStateType>,
    /// Custom event executions (event_id, arg1, arg2).
    pub custom_events: Vec<(i32, i32, i32)>,
    /// Audio play requests (path, volume, is_loop).
    pub audio_plays: Vec<(String, f32, bool)>,
    /// Audio stop requests (path).
    pub audio_stops: Vec<String>,
    /// Audio config changed notification flag.
    pub audio_config_changed: bool,
    /// Option change sound request flag.
    pub option_change_sound: bool,
    /// Bar update request flag (music select).
    pub update_bar_after_change: bool,
    /// Song selection mode requests (event_id).
    pub select_song_mode_requests: Vec<i32>,
}

impl SkinActionQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if no actions have been queued.
    pub fn is_empty(&self) -> bool {
        self.float_writes.is_empty()
            && self.timer_sets.is_empty()
            && self.state_changes.is_empty()
            && self.custom_events.is_empty()
            && self.audio_plays.is_empty()
            && self.audio_stops.is_empty()
            && !self.audio_config_changed
            && !self.option_change_sound
            && !self.update_bar_after_change
            && self.select_song_mode_requests.is_empty()
    }

    /// Clears all queued actions.
    pub fn clear(&mut self) {
        self.float_writes.clear();
        self.timer_sets.clear();
        self.state_changes.clear();
        self.custom_events.clear();
        self.audio_plays.clear();
        self.audio_stops.clear();
        self.audio_config_changed = false;
        self.option_change_sound = false;
        self.update_bar_after_change = false;
        self.select_song_mode_requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let queue = SkinActionQueue::new();
        assert!(queue.is_empty());
    }

    #[test]
    fn queue_not_empty_after_push() {
        let mut queue = SkinActionQueue::new();
        queue.float_writes.push((1, 0.5));
        assert!(!queue.is_empty());
    }

    #[test]
    fn clear_resets_all_fields() {
        let mut queue = SkinActionQueue::new();
        queue.float_writes.push((1, 0.5));
        queue.timer_sets.push((TimerId::new(10), 1000));
        queue.state_changes.push(MainStateType::Play);
        queue.custom_events.push((100, 0, 0));
        queue.audio_plays.push(("bgm.wav".into(), 1.0, false));
        queue.audio_stops.push("bgm.wav".into());
        queue.audio_config_changed = true;
        queue.option_change_sound = true;
        queue.update_bar_after_change = true;
        queue.select_song_mode_requests.push(42);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn boolean_flags_make_non_empty() {
        let mut queue = SkinActionQueue::new();
        queue.audio_config_changed = true;
        assert!(!queue.is_empty());

        queue.audio_config_changed = false;
        queue.option_change_sound = true;
        assert!(!queue.is_empty());

        queue.option_change_sound = false;
        queue.update_bar_after_change = true;
        assert!(!queue.is_empty());
    }
}
