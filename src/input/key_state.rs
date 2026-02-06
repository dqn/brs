/// Tracks the press/release state and timing for each key.
#[derive(Debug, Clone)]
pub struct KeyState {
    /// Number of tracked keys.
    key_count: usize,
    /// Whether each key is currently held.
    pressed: Vec<bool>,
    /// Timestamp (microseconds) of the last state change for each key.
    last_change_us: Vec<i64>,
}

impl KeyState {
    /// Create a new key state tracker.
    pub fn new(key_count: usize) -> Self {
        Self {
            key_count,
            pressed: vec![false; key_count],
            last_change_us: vec![0; key_count],
        }
    }

    /// Update a key's state.
    pub fn set(&mut self, key: usize, pressed: bool, time_us: i64) {
        if key < self.key_count {
            self.pressed[key] = pressed;
            self.last_change_us[key] = time_us;
        }
    }

    /// Check if a key is currently pressed.
    pub fn is_pressed(&self, key: usize) -> bool {
        self.pressed.get(key).copied().unwrap_or(false)
    }

    /// Get the timestamp of the last state change for a key.
    pub fn last_change(&self, key: usize) -> i64 {
        self.last_change_us.get(key).copied().unwrap_or(0)
    }

    /// Number of tracked keys.
    pub fn key_count(&self) -> usize {
        self.key_count
    }

    /// Reset all keys to released state.
    pub fn reset(&mut self) {
        self.pressed.fill(false);
        self.last_change_us.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_all_released() {
        let state = KeyState::new(8);
        for i in 0..8 {
            assert!(!state.is_pressed(i));
            assert_eq!(state.last_change(i), 0);
        }
    }

    #[test]
    fn set_and_check() {
        let mut state = KeyState::new(8);
        state.set(3, true, 1000);
        assert!(state.is_pressed(3));
        assert_eq!(state.last_change(3), 1000);
        assert!(!state.is_pressed(0));

        state.set(3, false, 2000);
        assert!(!state.is_pressed(3));
        assert_eq!(state.last_change(3), 2000);
    }

    #[test]
    fn out_of_bounds_safe() {
        let mut state = KeyState::new(4);
        state.set(10, true, 100); // no panic
        assert!(!state.is_pressed(10));
        assert_eq!(state.last_change(10), 0);
    }

    #[test]
    fn reset() {
        let mut state = KeyState::new(4);
        state.set(0, true, 100);
        state.set(2, true, 200);
        state.reset();
        for i in 0..4 {
            assert!(!state.is_pressed(i));
            assert_eq!(state.last_change(i), 0);
        }
    }

    #[test]
    fn key_count() {
        let state = KeyState::new(16);
        assert_eq!(state.key_count(), 16);
    }
}
