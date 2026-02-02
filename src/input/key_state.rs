/// State of a single key/button with microsecond-precision timestamps.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyState {
    /// Whether the key is currently held down.
    pub pressed: bool,
    /// Whether the key was just pressed this frame.
    pub just_pressed: bool,
    /// Whether the key was just released this frame.
    pub just_released: bool,
    /// Timestamp when the key was pressed (microseconds).
    pub press_time_us: u64,
    /// Timestamp when the key was released (microseconds).
    pub release_time_us: u64,
}

impl KeyState {
    /// Create a new key state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset frame-specific state (just_pressed/just_released).
    /// Called at the start of each frame.
    pub fn reset_frame_state(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
    }

    /// Update state when key is pressed.
    pub fn on_press(&mut self, time_us: u64) {
        if !self.pressed {
            self.pressed = true;
            self.just_pressed = true;
            self.press_time_us = time_us;
        }
    }

    /// Update state when key is released.
    pub fn on_release(&mut self, time_us: u64) {
        if self.pressed {
            self.pressed = false;
            self.just_released = true;
            self.release_time_us = time_us;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_default() {
        let state = KeyState::new();
        assert!(!state.pressed);
        assert!(!state.just_pressed);
        assert!(!state.just_released);
        assert_eq!(state.press_time_us, 0);
        assert_eq!(state.release_time_us, 0);
    }

    #[test]
    fn test_key_state_press() {
        let mut state = KeyState::new();
        state.on_press(1000);

        assert!(state.pressed);
        assert!(state.just_pressed);
        assert!(!state.just_released);
        assert_eq!(state.press_time_us, 1000);
    }

    #[test]
    fn test_key_state_release() {
        let mut state = KeyState::new();
        state.on_press(1000);
        state.reset_frame_state();
        state.on_release(2000);

        assert!(!state.pressed);
        assert!(!state.just_pressed);
        assert!(state.just_released);
        assert_eq!(state.release_time_us, 2000);
    }

    #[test]
    fn test_double_press_ignored() {
        let mut state = KeyState::new();
        state.on_press(1000);
        state.on_press(2000);

        assert_eq!(state.press_time_us, 1000);
    }

    #[test]
    fn test_double_release_ignored() {
        let mut state = KeyState::new();
        state.on_press(1000);
        state.on_release(2000);
        state.on_release(3000);

        assert_eq!(state.release_time_us, 2000);
    }

    #[test]
    fn test_reset_frame_state() {
        let mut state = KeyState::new();
        state.on_press(1000);
        assert!(state.just_pressed);

        state.reset_frame_state();
        assert!(!state.just_pressed);
        assert!(state.pressed);
    }
}
