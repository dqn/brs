// Bevy KeyCode -> LibGDX-compatible keycode mapping for bms-input.
//
// Maps Bevy keyboard input into the integer keycodes expected by bms-input.

use std::collections::HashSet;

use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bms_input::keyboard::KeyboardBackend;

/// Keyboard backend that snapshots Bevy's keyboard state each frame.
pub struct BevyKeyboardBackend {
    pressed_keys: HashSet<i32>,
}

impl BevyKeyboardBackend {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    /// Snapshot the current Bevy keyboard state into integer keycodes.
    pub fn snapshot(&mut self, input: &ButtonInput<KeyCode>) {
        self.pressed_keys.clear();
        for key in input.get_pressed() {
            if let Some(code) = bevy_to_keycode(*key) {
                self.pressed_keys.insert(code);
            }
        }
    }
}

impl Default for BevyKeyboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardBackend for BevyKeyboardBackend {
    fn is_key_pressed(&self, keycode: i32) -> bool {
        self.pressed_keys.contains(&keycode)
    }
}

/// Map Bevy KeyCode to LibGDX-compatible integer keycode.
///
/// Based on LibGDX Input.Keys constants used by beatoraja.
fn bevy_to_keycode(key: KeyCode) -> Option<i32> {
    Some(match key {
        // Letters A-Z (LibGDX: 29-54)
        KeyCode::KeyA => 29,
        KeyCode::KeyB => 30,
        KeyCode::KeyC => 31,
        KeyCode::KeyD => 32,
        KeyCode::KeyE => 33,
        KeyCode::KeyF => 34,
        KeyCode::KeyG => 35,
        KeyCode::KeyH => 36,
        KeyCode::KeyI => 37,
        KeyCode::KeyJ => 38,
        KeyCode::KeyK => 39,
        KeyCode::KeyL => 40,
        KeyCode::KeyM => 41,
        KeyCode::KeyN => 42,
        KeyCode::KeyO => 43,
        KeyCode::KeyP => 44,
        KeyCode::KeyQ => 45,
        KeyCode::KeyR => 46,
        KeyCode::KeyS => 47,
        KeyCode::KeyT => 48,
        KeyCode::KeyU => 49,
        KeyCode::KeyV => 50,
        KeyCode::KeyW => 51,
        KeyCode::KeyX => 52,
        KeyCode::KeyY => 53,
        KeyCode::KeyZ => 54,

        // Digits 0-9 (LibGDX: 7-16)
        KeyCode::Digit0 => 7,
        KeyCode::Digit1 => 8,
        KeyCode::Digit2 => 9,
        KeyCode::Digit3 => 10,
        KeyCode::Digit4 => 11,
        KeyCode::Digit5 => 12,
        KeyCode::Digit6 => 13,
        KeyCode::Digit7 => 14,
        KeyCode::Digit8 => 15,
        KeyCode::Digit9 => 16,

        // F-keys (LibGDX: 244-255)
        KeyCode::F1 => 244,
        KeyCode::F2 => 245,
        KeyCode::F3 => 246,
        KeyCode::F4 => 247,
        KeyCode::F5 => 248,
        KeyCode::F6 => 249,
        KeyCode::F7 => 250,
        KeyCode::F8 => 251,
        KeyCode::F9 => 252,
        KeyCode::F10 => 253,
        KeyCode::F11 => 254,
        KeyCode::F12 => 255,

        // Common keys
        KeyCode::Space => 62,
        KeyCode::Enter => 66,
        KeyCode::Escape => 111,
        KeyCode::Backspace => 67,
        KeyCode::Tab => 61,
        KeyCode::ShiftLeft => 59,
        KeyCode::ShiftRight => 60,
        KeyCode::ControlLeft => 129,
        KeyCode::ControlRight => 130,
        KeyCode::AltLeft => 57,
        KeyCode::AltRight => 58,

        // Arrow keys
        KeyCode::ArrowUp => 19,
        KeyCode::ArrowDown => 20,
        KeyCode::ArrowLeft => 21,
        KeyCode::ArrowRight => 22,

        // Punctuation / symbols
        KeyCode::Comma => 55,
        KeyCode::Period => 56,
        KeyCode::Slash => 76,
        KeyCode::Backslash => 73,
        KeyCode::Semicolon => 74,
        KeyCode::Quote => 75,
        KeyCode::BracketLeft => 71,
        KeyCode::BracketRight => 72,
        KeyCode::Minus => 69,
        KeyCode::Equal => 70,
        KeyCode::Backquote => 68,

        // Numpad (LibGDX: 144-153)
        KeyCode::Numpad0 => 144,
        KeyCode::Numpad1 => 145,
        KeyCode::Numpad2 => 146,
        KeyCode::Numpad3 => 147,
        KeyCode::Numpad4 => 148,
        KeyCode::Numpad5 => 149,
        KeyCode::Numpad6 => 150,
        KeyCode::Numpad7 => 151,
        KeyCode::Numpad8 => 152,
        KeyCode::Numpad9 => 153,

        // Other
        KeyCode::Insert => 124,
        KeyCode::Delete => 112,
        KeyCode::Home => 122,
        KeyCode::End => 123,
        KeyCode::PageUp => 92,
        KeyCode::PageDown => 93,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_mapping_letters() {
        assert_eq!(bevy_to_keycode(KeyCode::KeyA), Some(29));
        assert_eq!(bevy_to_keycode(KeyCode::KeyZ), Some(54));
    }

    #[test]
    fn key_mapping_digits() {
        assert_eq!(bevy_to_keycode(KeyCode::Digit0), Some(7));
        assert_eq!(bevy_to_keycode(KeyCode::Digit9), Some(16));
    }

    #[test]
    fn key_mapping_f_keys() {
        assert_eq!(bevy_to_keycode(KeyCode::F1), Some(244));
        assert_eq!(bevy_to_keycode(KeyCode::F12), Some(255));
    }

    #[test]
    fn key_mapping_special() {
        assert_eq!(bevy_to_keycode(KeyCode::Space), Some(62));
        assert_eq!(bevy_to_keycode(KeyCode::Enter), Some(66));
        assert_eq!(bevy_to_keycode(KeyCode::Escape), Some(111));
    }

    #[test]
    fn backend_is_key_pressed() {
        let mut backend = BevyKeyboardBackend::new();
        assert!(!backend.is_key_pressed(29));
        backend.pressed_keys.insert(29);
        assert!(backend.is_key_pressed(29));
    }
}
