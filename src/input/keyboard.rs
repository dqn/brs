use crate::input::key_config::KeyboardConfig;
use crate::input::key_state::KeyState;
use macroquad::prelude::is_key_down;

/// Keyboard input handler using macroquad.
pub struct KeyboardInput;

impl KeyboardInput {
    /// Create a new keyboard input handler.
    pub fn new() -> Self {
        Self
    }

    /// Update key states based on current keyboard state.
    /// Returns indices of lanes that had state changes (lane_idx, pressed).
    pub fn update(
        &self,
        config: &KeyboardConfig,
        states: &mut [KeyState; 8],
        time_us: u64,
    ) -> Vec<(usize, bool)> {
        let mut changes = Vec::new();

        for (lane_idx, key) in config.lanes.iter().enumerate() {
            if let Some(keycode) = key.to_keycode() {
                let is_pressed = is_key_down(keycode);
                let was_pressed = states[lane_idx].pressed;

                if is_pressed && !was_pressed {
                    states[lane_idx].on_press(time_us);
                    changes.push((lane_idx, true));
                } else if !is_pressed && was_pressed {
                    states[lane_idx].on_release(time_us);
                    changes.push((lane_idx, false));
                }
            }
        }

        changes
    }

    /// Check if start key is pressed.
    pub fn is_start_pressed(&self, config: &KeyboardConfig) -> bool {
        config.start.to_keycode().map(is_key_down).unwrap_or(false)
    }

    /// Check if select key is pressed.
    pub fn is_select_pressed(&self, config: &KeyboardConfig) -> bool {
        config.select.to_keycode().map(is_key_down).unwrap_or(false)
    }
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}
