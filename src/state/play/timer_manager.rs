// Timer IDs matching beatoraja SkinProperty timer constants.
// Timer values are stored as Option<i64> (microseconds from play start).
// None means the timer has not been started.

// Play state timers
pub const TIMER_PLAY: usize = 0;
pub const TIMER_READY: usize = 1;
pub const TIMER_FAILED: usize = 2;
pub const TIMER_FADEOUT: usize = 3;
pub const TIMER_ENDOFNOTE_1P: usize = 4;
pub const TIMER_ENDOFNOTE_2P: usize = 5;

// Judge timers per lane (1P)
pub const TIMER_JUDGE_1P_BASE: usize = 10;
// lane 0..7 -> TIMER_JUDGE_1P_BASE + lane
pub const TIMER_JUDGE_1P_COUNT: usize = 8;

// Judge timers per lane (2P)
pub const TIMER_JUDGE_2P_BASE: usize = 20;
pub const TIMER_JUDGE_2P_COUNT: usize = 8;

// Bomb effect timers per lane (1P)
pub const TIMER_BOMB_1P_BASE: usize = 30;
pub const TIMER_BOMB_1P_COUNT: usize = 8;

// Bomb effect timers per lane (2P)
pub const TIMER_BOMB_2P_BASE: usize = 40;
pub const TIMER_BOMB_2P_COUNT: usize = 8;

// Key-on timers per lane (1P)
pub const TIMER_KEYON_1P_BASE: usize = 50;
pub const TIMER_KEYON_1P_COUNT: usize = 8;

// Key-on timers per lane (2P)
pub const TIMER_KEYON_2P_BASE: usize = 60;
pub const TIMER_KEYON_2P_COUNT: usize = 8;

// Key-off timers per lane (1P)
pub const TIMER_KEYOFF_1P_BASE: usize = 70;
pub const TIMER_KEYOFF_1P_COUNT: usize = 8;

// Key-off timers per lane (2P)
pub const TIMER_KEYOFF_2P_BASE: usize = 80;
pub const TIMER_KEYOFF_2P_COUNT: usize = 8;

// Total number of timers
const TIMER_COUNT: usize = 90;

/// Manages named timers for the play state.
/// Each timer stores an optional start time in microseconds.
#[derive(Debug, Clone)]
pub struct TimerManager {
    timers: Vec<Option<i64>>,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            timers: vec![None; TIMER_COUNT],
        }
    }

    /// Set a timer to the given time (microseconds).
    pub fn set(&mut self, id: usize, time_us: i64) {
        if id < self.timers.len() {
            self.timers[id] = Some(time_us);
        }
    }

    /// Get the timer start time, or None if not started.
    pub fn get(&self, id: usize) -> Option<i64> {
        self.timers.get(id).copied().flatten()
    }

    /// Reset a timer to None.
    pub fn reset(&mut self, id: usize) {
        if id < self.timers.len() {
            self.timers[id] = None;
        }
    }

    /// Check if a timer is active (has been set).
    pub fn is_active(&self, id: usize) -> bool {
        self.get(id).is_some()
    }

    /// Get elapsed time since a timer was set.
    /// Returns None if the timer is not active.
    pub fn elapsed(&self, id: usize, current_time_us: i64) -> Option<i64> {
        self.get(id).map(|start| current_time_us - start)
    }

    /// Set the judge timer for a lane.
    pub fn set_judge(&mut self, lane: usize, player: usize, time_us: i64) {
        let base = if player == 0 {
            TIMER_JUDGE_1P_BASE
        } else {
            TIMER_JUDGE_2P_BASE
        };
        self.set(base + lane, time_us);
    }

    /// Set the bomb timer for a lane.
    pub fn set_bomb(&mut self, lane: usize, player: usize, time_us: i64) {
        let base = if player == 0 {
            TIMER_BOMB_1P_BASE
        } else {
            TIMER_BOMB_2P_BASE
        };
        self.set(base + lane, time_us);
    }

    /// Set the key-on timer for a lane.
    pub fn set_keyon(&mut self, lane: usize, player: usize, time_us: i64) {
        let base = if player == 0 {
            TIMER_KEYON_1P_BASE
        } else {
            TIMER_KEYON_2P_BASE
        };
        self.set(base + lane, time_us);
        // Reset the corresponding key-off timer
        let off_base = if player == 0 {
            TIMER_KEYOFF_1P_BASE
        } else {
            TIMER_KEYOFF_2P_BASE
        };
        self.reset(off_base + lane);
    }

    /// Set the key-off timer for a lane.
    pub fn set_keyoff(&mut self, lane: usize, player: usize, time_us: i64) {
        let base = if player == 0 {
            TIMER_KEYOFF_1P_BASE
        } else {
            TIMER_KEYOFF_2P_BASE
        };
        self.set(base + lane, time_us);
        // Reset the corresponding key-on timer
        let on_base = if player == 0 {
            TIMER_KEYON_1P_BASE
        } else {
            TIMER_KEYON_2P_BASE
        };
        self.reset(on_base + lane);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_set_get_reset() {
        let mut tm = TimerManager::new();
        assert_eq!(tm.get(TIMER_PLAY), None);
        assert!(!tm.is_active(TIMER_PLAY));

        tm.set(TIMER_PLAY, 1_000_000);
        assert_eq!(tm.get(TIMER_PLAY), Some(1_000_000));
        assert!(tm.is_active(TIMER_PLAY));

        tm.reset(TIMER_PLAY);
        assert_eq!(tm.get(TIMER_PLAY), None);
    }

    #[test]
    fn timer_elapsed() {
        let mut tm = TimerManager::new();
        tm.set(TIMER_READY, 500_000);
        assert_eq!(tm.elapsed(TIMER_READY, 1_500_000), Some(1_000_000));
        assert_eq!(tm.elapsed(TIMER_PLAY, 1_500_000), None);
    }

    #[test]
    fn timer_judge_per_lane() {
        let mut tm = TimerManager::new();
        tm.set_judge(3, 0, 2_000_000);
        assert_eq!(tm.get(TIMER_JUDGE_1P_BASE + 3), Some(2_000_000));

        tm.set_judge(5, 1, 3_000_000);
        assert_eq!(tm.get(TIMER_JUDGE_2P_BASE + 5), Some(3_000_000));
    }

    #[test]
    fn timer_bomb_per_lane() {
        let mut tm = TimerManager::new();
        tm.set_bomb(0, 0, 1_000_000);
        assert_eq!(tm.get(TIMER_BOMB_1P_BASE), Some(1_000_000));
    }

    #[test]
    fn timer_keyon_resets_keyoff() {
        let mut tm = TimerManager::new();
        tm.set_keyoff(2, 0, 1_000_000);
        assert!(tm.is_active(TIMER_KEYOFF_1P_BASE + 2));

        tm.set_keyon(2, 0, 2_000_000);
        assert!(tm.is_active(TIMER_KEYON_1P_BASE + 2));
        assert!(!tm.is_active(TIMER_KEYOFF_1P_BASE + 2));
    }

    #[test]
    fn timer_keyoff_resets_keyon() {
        let mut tm = TimerManager::new();
        tm.set_keyon(1, 0, 1_000_000);
        assert!(tm.is_active(TIMER_KEYON_1P_BASE + 1));

        tm.set_keyoff(1, 0, 2_000_000);
        assert!(tm.is_active(TIMER_KEYOFF_1P_BASE + 1));
        assert!(!tm.is_active(TIMER_KEYON_1P_BASE + 1));
    }

    #[test]
    fn timer_out_of_bounds_no_panic() {
        let mut tm = TimerManager::new();
        tm.set(9999, 100);
        assert_eq!(tm.get(9999), None);
        tm.reset(9999);
    }

    #[test]
    fn timer_default() {
        let tm = TimerManager::default();
        assert_eq!(tm.get(TIMER_PLAY), None);
    }
}
