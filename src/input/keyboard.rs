use winit::keyboard::KeyCode;

use crate::state::config::key_config::KeyConfig;
use crate::traits::input::KeyEvent;

/// Handles keyboard input and maps physical keys to game lane keys
/// based on the key configuration.
pub struct KeyboardHandler {
    /// Key config mapping physical keys to lanes.
    key_config: KeyConfig,
    /// Physical key states (true = pressed).
    physical_state: Vec<bool>,
}

impl KeyboardHandler {
    /// Create a new keyboard handler with the given key configuration.
    pub fn new(key_config: KeyConfig) -> Self {
        // Track up to 256 physical key codes
        Self {
            key_config,
            physical_state: vec![false; 256],
        }
    }

    /// Process a winit keyboard event and return any resulting game key events.
    pub fn handle_key_event(
        &mut self,
        keycode: KeyCode,
        pressed: bool,
        time_us: i64,
    ) -> Vec<KeyEvent> {
        let code = keycode_to_u32(keycode);
        let idx = code as usize;

        // Filter duplicate press/release events
        if idx < self.physical_state.len() {
            if self.physical_state[idx] == pressed {
                return Vec::new();
            }
            self.physical_state[idx] = pressed;
        }

        let mut events = Vec::new();
        for (lane, binding) in self.key_config.bindings.iter().enumerate() {
            if binding.primary == Some(code) || binding.secondary == Some(code) {
                events.push(KeyEvent {
                    key: lane,
                    pressed,
                    time_us,
                });
            }
        }
        events
    }

    /// Reset all physical key states.
    pub fn reset(&mut self) {
        self.physical_state.fill(false);
    }

    /// Update the key configuration.
    pub fn set_key_config(&mut self, key_config: KeyConfig) {
        self.key_config = key_config;
    }
}

/// Convert a winit KeyCode to a u32 key code.
/// Uses a simple numeric mapping for common keys.
fn keycode_to_u32(keycode: KeyCode) -> u32 {
    match keycode {
        KeyCode::Digit0 => 0,
        KeyCode::Digit1 => 1,
        KeyCode::Digit2 => 2,
        KeyCode::Digit3 => 3,
        KeyCode::Digit4 => 4,
        KeyCode::Digit5 => 5,
        KeyCode::Digit6 => 6,
        KeyCode::Digit7 => 7,
        KeyCode::Digit8 => 8,
        KeyCode::Digit9 => 9,
        KeyCode::KeyA => 65,
        KeyCode::KeyB => 66,
        KeyCode::KeyC => 67,
        KeyCode::KeyD => 68,
        KeyCode::KeyE => 69,
        KeyCode::KeyF => 70,
        KeyCode::KeyG => 71,
        KeyCode::KeyH => 72,
        KeyCode::KeyI => 73,
        KeyCode::KeyJ => 74,
        KeyCode::KeyK => 75,
        KeyCode::KeyL => 76,
        KeyCode::KeyM => 77,
        KeyCode::KeyN => 78,
        KeyCode::KeyO => 79,
        KeyCode::KeyP => 80,
        KeyCode::KeyQ => 81,
        KeyCode::KeyR => 82,
        KeyCode::KeyS => 83,
        KeyCode::KeyT => 84,
        KeyCode::KeyU => 85,
        KeyCode::KeyV => 86,
        KeyCode::KeyW => 87,
        KeyCode::KeyX => 88,
        KeyCode::KeyY => 89,
        KeyCode::KeyZ => 90,
        KeyCode::ShiftLeft => 16,
        KeyCode::ShiftRight => 17,
        KeyCode::ControlLeft => 18,
        KeyCode::ControlRight => 19,
        KeyCode::AltLeft => 20,
        KeyCode::AltRight => 21,
        KeyCode::Space => 32,
        KeyCode::Enter => 13,
        KeyCode::Escape => 27,
        KeyCode::Backspace => 8,
        KeyCode::Tab => 9,
        KeyCode::ArrowUp => 38,
        KeyCode::ArrowDown => 40,
        KeyCode::ArrowLeft => 37,
        KeyCode::ArrowRight => 39,
        KeyCode::F1 => 112,
        KeyCode::F2 => 113,
        KeyCode::F3 => 114,
        KeyCode::F4 => 115,
        KeyCode::F5 => 116,
        KeyCode::F6 => 117,
        KeyCode::F7 => 118,
        KeyCode::F8 => 119,
        KeyCode::F9 => 120,
        KeyCode::F10 => 121,
        KeyCode::F11 => 122,
        KeyCode::F12 => 123,
        _ => 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mapping() {
        let config = KeyConfig::default_7k();
        let mut handler = KeyboardHandler::new(config);

        // ShiftLeft (16) -> Scratch (lane 0)
        let events = handler.handle_key_event(KeyCode::ShiftLeft, true, 1000);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, 0);
        assert!(events[0].pressed);

        // Z (90) -> KEY1 (lane 1)
        let events = handler.handle_key_event(KeyCode::KeyZ, true, 2000);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, 1);
    }

    #[test]
    fn duplicate_press_filtered() {
        let config = KeyConfig::default_7k();
        let mut handler = KeyboardHandler::new(config);

        let events = handler.handle_key_event(KeyCode::KeyZ, true, 1000);
        assert_eq!(events.len(), 1);

        // Same key press again - should be filtered
        let events = handler.handle_key_event(KeyCode::KeyZ, true, 1100);
        assert!(events.is_empty());
    }

    #[test]
    fn press_and_release() {
        let config = KeyConfig::default_7k();
        let mut handler = KeyboardHandler::new(config);

        let events = handler.handle_key_event(KeyCode::KeyZ, true, 1000);
        assert_eq!(events.len(), 1);
        assert!(events[0].pressed);

        let events = handler.handle_key_event(KeyCode::KeyZ, false, 2000);
        assert_eq!(events.len(), 1);
        assert!(!events[0].pressed);
    }

    #[test]
    fn unmapped_key_produces_no_event() {
        let config = KeyConfig::default_7k();
        let mut handler = KeyboardHandler::new(config);

        let events = handler.handle_key_event(KeyCode::F12, true, 1000);
        assert!(events.is_empty());
    }

    #[test]
    fn reset_clears_state() {
        let config = KeyConfig::default_7k();
        let mut handler = KeyboardHandler::new(config);

        handler.handle_key_event(KeyCode::KeyZ, true, 1000);
        handler.reset();

        // After reset, pressing same key should produce event again
        let events = handler.handle_key_event(KeyCode::KeyZ, true, 2000);
        assert_eq!(events.len(), 1);
    }
}
