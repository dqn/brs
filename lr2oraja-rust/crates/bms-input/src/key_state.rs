/// Key state management for BMS player input.
///
/// Ported from Java `BMSPlayerInputProcessor` key state arrays.
/// Tracks on/off state, change timestamps, and analog input state for up to 256 keys.
pub struct KeyStateManager {
    keystate: [bool; Self::SIZE],
    time: [i64; Self::SIZE],
    is_analog: [bool; Self::SIZE],
    last_analog_value: [f32; Self::SIZE],
    current_analog_value: [f32; Self::SIZE],
    analog_last_reset_time: [i64; Self::SIZE],
}

impl KeyStateManager {
    pub const SIZE: usize = 256;
    pub const TIME_NOT_SET: i64 = i64::MIN;

    pub fn new() -> Self {
        Self {
            keystate: [false; Self::SIZE],
            time: [Self::TIME_NOT_SET; Self::SIZE],
            is_analog: [false; Self::SIZE],
            last_analog_value: [0.0; Self::SIZE],
            current_analog_value: [0.0; Self::SIZE],
            analog_last_reset_time: [0; Self::SIZE],
        }
    }

    /// Set key on/off state and its change timestamp.
    pub fn set_key_state(&mut self, id: usize, pressed: bool, time: i64) {
        if id < Self::SIZE {
            self.keystate[id] = pressed;
            self.time[id] = time;
        }
    }

    /// Get whether a key is currently pressed. Returns `false` for out-of-range IDs.
    pub fn get_key_state(&self, id: usize) -> bool {
        if id < Self::SIZE {
            self.keystate[id]
        } else {
            false
        }
    }

    /// Get the timestamp of the last state change for a key.
    /// Returns `TIME_NOT_SET` for out-of-range IDs.
    pub fn get_key_changed_time(&self, id: usize) -> i64 {
        if id < Self::SIZE {
            self.time[id]
        } else {
            Self::TIME_NOT_SET
        }
    }

    /// Reset a key's change timestamp to `TIME_NOT_SET`.
    /// Returns `true` if the timestamp was previously set (i.e., not `TIME_NOT_SET`).
    pub fn reset_key_changed_time(&mut self, id: usize) -> bool {
        if id < Self::SIZE {
            let was_set = self.time[id] != Self::TIME_NOT_SET;
            self.time[id] = Self::TIME_NOT_SET;
            was_set
        } else {
            false
        }
    }

    /// Reset all key states to unpressed and all timestamps to `TIME_NOT_SET`.
    pub fn reset_all_key_state(&mut self) {
        self.keystate.fill(false);
        self.time.fill(Self::TIME_NOT_SET);
    }

    /// Reset all key change timestamps to `TIME_NOT_SET` without changing key states.
    pub fn reset_all_key_changed_time(&mut self) {
        self.time.fill(Self::TIME_NOT_SET);
    }

    /// Set analog state for a key.
    pub fn set_analog_state(&mut self, id: usize, is_analog: bool, value: f32) {
        if id < Self::SIZE {
            self.is_analog[id] = is_analog;
            self.current_analog_value[id] = value;
        }
    }

    /// Snapshot the current analog value as the baseline and record the reset time.
    pub fn reset_analog_input(&mut self, id: usize, now_ms: i64) {
        if id < Self::SIZE {
            self.last_analog_value[id] = self.current_analog_value[id];
            self.analog_last_reset_time[id] = now_ms;
        }
    }

    /// Get the elapsed time (ms) since the last analog reset.
    pub fn get_time_since_last_analog_reset(&self, id: usize, now_ms: i64) -> i64 {
        if id < Self::SIZE {
            now_ms - self.analog_last_reset_time[id]
        } else {
            i64::MAX
        }
    }

    /// Compute the raw analog diff as an integer.
    ///
    /// This is a simplified version; the full wrapping diff logic lives in `analog_scratch`.
    pub fn get_analog_diff(&self, id: usize) -> i32 {
        if id < Self::SIZE {
            (self.current_analog_value[id] - self.last_analog_value[id]) as i32
        } else {
            0
        }
    }

    /// Check whether a key is configured as analog input.
    pub fn is_analog_input(&self, id: usize) -> bool {
        if id < Self::SIZE {
            self.is_analog[id]
        } else {
            false
        }
    }

    /// Get the analog diff (clamped to >= 0) if within the tolerance window, then reset.
    ///
    /// Mirrors Java `getAnalogDiffAndReset(int i, int msTolerance)`.
    pub fn get_analog_diff_and_reset(&mut self, id: usize, ms_tolerance: i32, now_ms: i64) -> i32 {
        let mut d_ticks = 0;
        if self.get_time_since_last_analog_reset(id, now_ms) <= ms_tolerance as i64 {
            d_ticks = self.get_analog_diff(id).max(0);
        }
        self.reset_analog_input(id, now_ms);
        d_ticks
    }
}

impl Default for KeyStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_all_defaults() {
        let ks = KeyStateManager::new();
        for i in 0..KeyStateManager::SIZE {
            assert!(!ks.get_key_state(i));
            assert_eq!(ks.get_key_changed_time(i), KeyStateManager::TIME_NOT_SET);
            assert!(!ks.is_analog_input(i));
        }
    }

    #[test]
    fn set_get_key_state() {
        let mut ks = KeyStateManager::new();
        ks.set_key_state(10, true, 5000);
        assert!(ks.get_key_state(10));
        assert_eq!(ks.get_key_changed_time(10), 5000);

        ks.set_key_state(10, false, 6000);
        assert!(!ks.get_key_state(10));
        assert_eq!(ks.get_key_changed_time(10), 6000);
    }

    #[test]
    fn out_of_bounds_returns_defaults() {
        let mut ks = KeyStateManager::new();
        // get operations on out-of-range
        assert!(!ks.get_key_state(300));
        assert_eq!(ks.get_key_changed_time(300), KeyStateManager::TIME_NOT_SET);
        assert!(!ks.is_analog_input(500));
        assert_eq!(ks.get_analog_diff(999), 0);

        // set operations on out-of-range should not panic
        ks.set_key_state(256, true, 100);
        ks.set_analog_state(1000, true, 1.0);
        ks.reset_analog_input(1000, 0);
    }

    #[test]
    fn reset_key_changed_time_returns_whether_was_set() {
        let mut ks = KeyStateManager::new();
        // not set yet
        assert!(!ks.reset_key_changed_time(5));

        ks.set_key_state(5, true, 1000);
        assert!(ks.reset_key_changed_time(5));
        assert_eq!(ks.get_key_changed_time(5), KeyStateManager::TIME_NOT_SET);

        // already reset
        assert!(!ks.reset_key_changed_time(5));

        // out of range
        assert!(!ks.reset_key_changed_time(300));
    }

    #[test]
    fn reset_all_key_state_clears_everything() {
        let mut ks = KeyStateManager::new();
        ks.set_key_state(0, true, 100);
        ks.set_key_state(100, true, 200);
        ks.set_key_state(255, true, 300);

        ks.reset_all_key_state();

        for i in 0..KeyStateManager::SIZE {
            assert!(!ks.get_key_state(i));
            assert_eq!(ks.get_key_changed_time(i), KeyStateManager::TIME_NOT_SET);
        }
    }

    #[test]
    fn reset_all_key_changed_time_preserves_key_state() {
        let mut ks = KeyStateManager::new();
        ks.set_key_state(42, true, 1000);

        ks.reset_all_key_changed_time();

        // key remains pressed
        assert!(ks.get_key_state(42));
        // but time is reset
        assert_eq!(ks.get_key_changed_time(42), KeyStateManager::TIME_NOT_SET);
    }

    #[test]
    fn analog_state_set_get() {
        let mut ks = KeyStateManager::new();
        assert!(!ks.is_analog_input(7));

        ks.set_analog_state(7, true, 0.5);
        assert!(ks.is_analog_input(7));
    }

    #[test]
    fn analog_diff_basic() {
        let mut ks = KeyStateManager::new();
        ks.set_analog_state(3, true, 0.0);
        ks.reset_analog_input(3, 100);

        // Move analog forward
        ks.set_analog_state(3, true, 5.0);
        assert_eq!(ks.get_analog_diff(3), 5);

        // After reset, diff should be 0
        ks.reset_analog_input(3, 200);
        assert_eq!(ks.get_analog_diff(3), 0);
    }

    #[test]
    fn analog_diff_and_reset_within_tolerance() {
        let mut ks = KeyStateManager::new();
        ks.set_analog_state(0, true, 0.0);
        ks.reset_analog_input(0, 100); // last reset at 100ms

        ks.set_analog_state(0, true, 3.0);
        // now=150, tolerance=100 => elapsed=50 <= 100 => returns diff
        let result = ks.get_analog_diff_and_reset(0, 100, 150);
        assert_eq!(result, 3);
    }

    #[test]
    fn analog_diff_and_reset_outside_tolerance() {
        let mut ks = KeyStateManager::new();
        ks.set_analog_state(0, true, 0.0);
        ks.reset_analog_input(0, 100); // last reset at 100ms

        ks.set_analog_state(0, true, 3.0);
        // now=300, tolerance=100 => elapsed=200 > 100 => returns 0
        let result = ks.get_analog_diff_and_reset(0, 100, 300);
        assert_eq!(result, 0);
    }

    #[test]
    fn analog_diff_and_reset_clamps_negative() {
        let mut ks = KeyStateManager::new();
        ks.set_analog_state(1, true, 5.0);
        ks.reset_analog_input(1, 0);

        // Move backward
        ks.set_analog_state(1, true, 2.0);
        // Within tolerance, but diff is negative => clamped to 0
        let result = ks.get_analog_diff_and_reset(1, 1000, 10);
        assert_eq!(result, 0);
    }

    #[test]
    fn time_since_last_analog_reset() {
        let mut ks = KeyStateManager::new();
        ks.reset_analog_input(5, 1000);
        assert_eq!(ks.get_time_since_last_analog_reset(5, 1500), 500);
        assert_eq!(ks.get_time_since_last_analog_reset(5, 1000), 0);
    }

    #[test]
    fn time_since_last_analog_reset_out_of_bounds() {
        let ks = KeyStateManager::new();
        assert_eq!(ks.get_time_since_last_analog_reset(300, 1000), i64::MAX);
    }

    #[test]
    fn multiple_keys_independent() {
        let mut ks = KeyStateManager::new();
        ks.set_key_state(0, true, 100);
        ks.set_key_state(1, false, 200);
        ks.set_key_state(255, true, 300);

        assert!(ks.get_key_state(0));
        assert!(!ks.get_key_state(1));
        assert!(ks.get_key_state(255));

        assert_eq!(ks.get_key_changed_time(0), 100);
        assert_eq!(ks.get_key_changed_time(1), 200);
        assert_eq!(ks.get_key_changed_time(255), 300);
    }

    #[test]
    fn default_trait() {
        let ks = KeyStateManager::default();
        assert!(!ks.get_key_state(0));
        assert_eq!(ks.get_key_changed_time(0), KeyStateManager::TIME_NOT_SET);
    }
}
