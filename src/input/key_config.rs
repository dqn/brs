use anyhow::Result;
use macroquad::prelude::KeyCode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::model::note::LANE_COUNT;
const KEY_CONFIG_FILE: &str = "keyconfig.json";

/// Serializable key code representation using string names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct SerializableKeyCode(pub String);

impl SerializableKeyCode {
    pub fn from_keycode(key: KeyCode) -> Self {
        Self(keycode_to_string(key))
    }

    pub fn to_keycode(&self) -> Option<KeyCode> {
        string_to_keycode(&self.0)
    }
}

impl From<KeyCode> for SerializableKeyCode {
    fn from(key: KeyCode) -> Self {
        Self::from_keycode(key)
    }
}

/// Keyboard key bindings for all supported lanes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyboardConfig {
    /// Lane keys indexed by Lane::index().
    pub lanes: Vec<SerializableKeyCode>,
    /// Start/pause key.
    pub start: SerializableKeyCode,
    /// Select/back key.
    pub select: SerializableKeyCode,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            lanes: vec![
                SerializableKeyCode::from_keycode(KeyCode::LeftShift), // Scratch
                SerializableKeyCode::from_keycode(KeyCode::Z),         // Key 1
                SerializableKeyCode::from_keycode(KeyCode::S),         // Key 2
                SerializableKeyCode::from_keycode(KeyCode::X),         // Key 3
                SerializableKeyCode::from_keycode(KeyCode::D),         // Key 4
                SerializableKeyCode::from_keycode(KeyCode::C),         // Key 5
                SerializableKeyCode::from_keycode(KeyCode::F),         // Key 6
                SerializableKeyCode::from_keycode(KeyCode::V),         // Key 7
                SerializableKeyCode::from_keycode(KeyCode::RightShift), // Scratch2
                SerializableKeyCode::from_keycode(KeyCode::Slash),     // Key 8
                SerializableKeyCode::from_keycode(KeyCode::Period),    // Key 9
                SerializableKeyCode::from_keycode(KeyCode::Comma),     // Key 10
                SerializableKeyCode::from_keycode(KeyCode::L),         // Key 11
                SerializableKeyCode::from_keycode(KeyCode::K),         // Key 12
                SerializableKeyCode::from_keycode(KeyCode::M),         // Key 13
                SerializableKeyCode::from_keycode(KeyCode::J),         // Key 14
            ],
            start: SerializableKeyCode::from_keycode(KeyCode::Space),
            select: SerializableKeyCode::from_keycode(KeyCode::Escape),
        }
    }
}

/// Gamepad button bindings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GamepadConfig {
    /// Lane buttons indexed by Lane::index() (gilrs button names).
    pub lanes: Vec<Option<String>>,
    /// Axis name for analog scratch (e.g., "LeftStickX").
    pub scratch_axis: Option<String>,
    /// Threshold for axis-to-button conversion.
    pub axis_threshold: f32,
    /// Start button name.
    pub start: String,
    /// Select button name.
    pub select: String,
}

impl Default for GamepadConfig {
    fn default() -> Self {
        Self {
            lanes: vec![
                None,                              // Scratch (use axis)
                Some("West".to_string()),          // Key 1 - X/Square
                Some("LeftTrigger".to_string()),   // Key 2 - LB
                Some("South".to_string()),         // Key 3 - A/Cross
                Some("North".to_string()),         // Key 4 - Y/Triangle
                Some("East".to_string()),          // Key 5 - B/Circle
                Some("RightTrigger".to_string()),  // Key 6 - RB
                Some("RightTrigger2".to_string()), // Key 7 - RT
                None,                              // Scratch2 (axis not mapped)
                None,                              // Key 8
                None,                              // Key 9
                None,                              // Key 10
                None,                              // Key 11
                None,                              // Key 12
                None,                              // Key 13
                None,                              // Key 14
            ],
            scratch_axis: Some("LeftStickX".to_string()),
            axis_threshold: 0.5,
            start: "Start".to_string(),
            select: "Select".to_string(),
        }
    }
}

/// Complete key configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyConfig {
    pub keyboard: KeyboardConfig,
    pub gamepad: Option<GamepadConfig>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            keyboard: KeyboardConfig::default(),
            gamepad: Some(GamepadConfig::default()),
        }
    }
}

impl KeyConfig {
    /// Load configuration from the default file.
    pub fn load() -> Result<Self> {
        Self::load_from(KEY_CONFIG_FILE)
    }

    /// Load configuration from a specific path.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let mut config: Self = serde_json::from_str(&content)?;
        config.normalize();
        Ok(config)
    }

    /// Save configuration to the default file.
    pub fn save(&self) -> Result<()> {
        self.save_to(KEY_CONFIG_FILE)
    }

    /// Save configuration to a specific path.
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn normalize(&mut self) {
        let defaults = Self::default();

        if self.keyboard.lanes.len() < LANE_COUNT {
            let start = self.keyboard.lanes.len();
            self.keyboard
                .lanes
                .extend_from_slice(&defaults.keyboard.lanes[start..LANE_COUNT]);
        } else if self.keyboard.lanes.len() > LANE_COUNT {
            self.keyboard.lanes.truncate(LANE_COUNT);
        }

        if let Some(ref mut gamepad) = self.gamepad {
            if gamepad.lanes.len() < LANE_COUNT {
                let start = gamepad.lanes.len();
                gamepad.lanes.extend_from_slice(
                    &defaults.gamepad.as_ref().unwrap().lanes[start..LANE_COUNT],
                );
            } else if gamepad.lanes.len() > LANE_COUNT {
                gamepad.lanes.truncate(LANE_COUNT);
            }
        }
    }
}

/// Convert KeyCode to string representation.
fn keycode_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Space => "Space",
        KeyCode::Apostrophe => "Apostrophe",
        KeyCode::Comma => "Comma",
        KeyCode::Minus => "Minus",
        KeyCode::Period => "Period",
        KeyCode::Slash => "Slash",
        KeyCode::Key0 => "Key0",
        KeyCode::Key1 => "Key1",
        KeyCode::Key2 => "Key2",
        KeyCode::Key3 => "Key3",
        KeyCode::Key4 => "Key4",
        KeyCode::Key5 => "Key5",
        KeyCode::Key6 => "Key6",
        KeyCode::Key7 => "Key7",
        KeyCode::Key8 => "Key8",
        KeyCode::Key9 => "Key9",
        KeyCode::Semicolon => "Semicolon",
        KeyCode::Equal => "Equal",
        KeyCode::A => "A",
        KeyCode::B => "B",
        KeyCode::C => "C",
        KeyCode::D => "D",
        KeyCode::E => "E",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::H => "H",
        KeyCode::I => "I",
        KeyCode::J => "J",
        KeyCode::K => "K",
        KeyCode::L => "L",
        KeyCode::M => "M",
        KeyCode::N => "N",
        KeyCode::O => "O",
        KeyCode::P => "P",
        KeyCode::Q => "Q",
        KeyCode::R => "R",
        KeyCode::S => "S",
        KeyCode::T => "T",
        KeyCode::U => "U",
        KeyCode::V => "V",
        KeyCode::W => "W",
        KeyCode::X => "X",
        KeyCode::Y => "Y",
        KeyCode::Z => "Z",
        KeyCode::LeftBracket => "LeftBracket",
        KeyCode::Backslash => "Backslash",
        KeyCode::RightBracket => "RightBracket",
        KeyCode::GraveAccent => "GraveAccent",
        KeyCode::World1 => "World1",
        KeyCode::World2 => "World2",
        KeyCode::Escape => "Escape",
        KeyCode::Enter => "Enter",
        KeyCode::Tab => "Tab",
        KeyCode::Backspace => "Backspace",
        KeyCode::Insert => "Insert",
        KeyCode::Delete => "Delete",
        KeyCode::Right => "Right",
        KeyCode::Left => "Left",
        KeyCode::Down => "Down",
        KeyCode::Up => "Up",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::CapsLock => "CapsLock",
        KeyCode::ScrollLock => "ScrollLock",
        KeyCode::NumLock => "NumLock",
        KeyCode::PrintScreen => "PrintScreen",
        KeyCode::Pause => "Pause",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::F13 => "F13",
        KeyCode::F14 => "F14",
        KeyCode::F15 => "F15",
        KeyCode::F16 => "F16",
        KeyCode::F17 => "F17",
        KeyCode::F18 => "F18",
        KeyCode::F19 => "F19",
        KeyCode::F20 => "F20",
        KeyCode::F21 => "F21",
        KeyCode::F22 => "F22",
        KeyCode::F23 => "F23",
        KeyCode::F24 => "F24",
        KeyCode::F25 => "F25",
        KeyCode::Kp0 => "Kp0",
        KeyCode::Kp1 => "Kp1",
        KeyCode::Kp2 => "Kp2",
        KeyCode::Kp3 => "Kp3",
        KeyCode::Kp4 => "Kp4",
        KeyCode::Kp5 => "Kp5",
        KeyCode::Kp6 => "Kp6",
        KeyCode::Kp7 => "Kp7",
        KeyCode::Kp8 => "Kp8",
        KeyCode::Kp9 => "Kp9",
        KeyCode::KpDecimal => "KpDecimal",
        KeyCode::KpDivide => "KpDivide",
        KeyCode::KpMultiply => "KpMultiply",
        KeyCode::KpSubtract => "KpSubtract",
        KeyCode::KpAdd => "KpAdd",
        KeyCode::KpEnter => "KpEnter",
        KeyCode::KpEqual => "KpEqual",
        KeyCode::LeftShift => "LeftShift",
        KeyCode::LeftControl => "LeftControl",
        KeyCode::LeftAlt => "LeftAlt",
        KeyCode::LeftSuper => "LeftSuper",
        KeyCode::RightShift => "RightShift",
        KeyCode::RightControl => "RightControl",
        KeyCode::RightAlt => "RightAlt",
        KeyCode::RightSuper => "RightSuper",
        KeyCode::Menu => "Menu",
        KeyCode::Back => "Back",
        KeyCode::Unknown => "Unknown",
    }
    .to_string()
}

/// Convert string to KeyCode.
fn string_to_keycode(s: &str) -> Option<KeyCode> {
    match s {
        "Space" => Some(KeyCode::Space),
        "Apostrophe" => Some(KeyCode::Apostrophe),
        "Comma" => Some(KeyCode::Comma),
        "Minus" => Some(KeyCode::Minus),
        "Period" => Some(KeyCode::Period),
        "Slash" => Some(KeyCode::Slash),
        "Key0" => Some(KeyCode::Key0),
        "Key1" => Some(KeyCode::Key1),
        "Key2" => Some(KeyCode::Key2),
        "Key3" => Some(KeyCode::Key3),
        "Key4" => Some(KeyCode::Key4),
        "Key5" => Some(KeyCode::Key5),
        "Key6" => Some(KeyCode::Key6),
        "Key7" => Some(KeyCode::Key7),
        "Key8" => Some(KeyCode::Key8),
        "Key9" => Some(KeyCode::Key9),
        "Semicolon" => Some(KeyCode::Semicolon),
        "Equal" => Some(KeyCode::Equal),
        "A" => Some(KeyCode::A),
        "B" => Some(KeyCode::B),
        "C" => Some(KeyCode::C),
        "D" => Some(KeyCode::D),
        "E" => Some(KeyCode::E),
        "F" => Some(KeyCode::F),
        "G" => Some(KeyCode::G),
        "H" => Some(KeyCode::H),
        "I" => Some(KeyCode::I),
        "J" => Some(KeyCode::J),
        "K" => Some(KeyCode::K),
        "L" => Some(KeyCode::L),
        "M" => Some(KeyCode::M),
        "N" => Some(KeyCode::N),
        "O" => Some(KeyCode::O),
        "P" => Some(KeyCode::P),
        "Q" => Some(KeyCode::Q),
        "R" => Some(KeyCode::R),
        "S" => Some(KeyCode::S),
        "T" => Some(KeyCode::T),
        "U" => Some(KeyCode::U),
        "V" => Some(KeyCode::V),
        "W" => Some(KeyCode::W),
        "X" => Some(KeyCode::X),
        "Y" => Some(KeyCode::Y),
        "Z" => Some(KeyCode::Z),
        "LeftBracket" => Some(KeyCode::LeftBracket),
        "Backslash" => Some(KeyCode::Backslash),
        "RightBracket" => Some(KeyCode::RightBracket),
        "GraveAccent" => Some(KeyCode::GraveAccent),
        "World1" => Some(KeyCode::World1),
        "World2" => Some(KeyCode::World2),
        "Escape" => Some(KeyCode::Escape),
        "Enter" => Some(KeyCode::Enter),
        "Tab" => Some(KeyCode::Tab),
        "Backspace" => Some(KeyCode::Backspace),
        "Insert" => Some(KeyCode::Insert),
        "Delete" => Some(KeyCode::Delete),
        "Right" => Some(KeyCode::Right),
        "Left" => Some(KeyCode::Left),
        "Down" => Some(KeyCode::Down),
        "Up" => Some(KeyCode::Up),
        "PageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "CapsLock" => Some(KeyCode::CapsLock),
        "ScrollLock" => Some(KeyCode::ScrollLock),
        "NumLock" => Some(KeyCode::NumLock),
        "PrintScreen" => Some(KeyCode::PrintScreen),
        "Pause" => Some(KeyCode::Pause),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        "F13" => Some(KeyCode::F13),
        "F14" => Some(KeyCode::F14),
        "F15" => Some(KeyCode::F15),
        "F16" => Some(KeyCode::F16),
        "F17" => Some(KeyCode::F17),
        "F18" => Some(KeyCode::F18),
        "F19" => Some(KeyCode::F19),
        "F20" => Some(KeyCode::F20),
        "F21" => Some(KeyCode::F21),
        "F22" => Some(KeyCode::F22),
        "F23" => Some(KeyCode::F23),
        "F24" => Some(KeyCode::F24),
        "F25" => Some(KeyCode::F25),
        "Kp0" => Some(KeyCode::Kp0),
        "Kp1" => Some(KeyCode::Kp1),
        "Kp2" => Some(KeyCode::Kp2),
        "Kp3" => Some(KeyCode::Kp3),
        "Kp4" => Some(KeyCode::Kp4),
        "Kp5" => Some(KeyCode::Kp5),
        "Kp6" => Some(KeyCode::Kp6),
        "Kp7" => Some(KeyCode::Kp7),
        "Kp8" => Some(KeyCode::Kp8),
        "Kp9" => Some(KeyCode::Kp9),
        "KpDecimal" => Some(KeyCode::KpDecimal),
        "KpDivide" => Some(KeyCode::KpDivide),
        "KpMultiply" => Some(KeyCode::KpMultiply),
        "KpSubtract" => Some(KeyCode::KpSubtract),
        "KpAdd" => Some(KeyCode::KpAdd),
        "KpEnter" => Some(KeyCode::KpEnter),
        "KpEqual" => Some(KeyCode::KpEqual),
        "LeftShift" => Some(KeyCode::LeftShift),
        "LeftControl" => Some(KeyCode::LeftControl),
        "LeftAlt" => Some(KeyCode::LeftAlt),
        "LeftSuper" => Some(KeyCode::LeftSuper),
        "RightShift" => Some(KeyCode::RightShift),
        "RightControl" => Some(KeyCode::RightControl),
        "RightAlt" => Some(KeyCode::RightAlt),
        "RightSuper" => Some(KeyCode::RightSuper),
        "Menu" => Some(KeyCode::Menu),
        "Back" => Some(KeyCode::Back),
        "Unknown" => Some(KeyCode::Unknown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keyboard_config() {
        let config = KeyboardConfig::default();
        assert_eq!(config.lanes.len(), LANE_COUNT);
        assert_eq!(config.lanes[0].0, "LeftShift");
        assert_eq!(config.lanes[1].0, "Z");
    }

    #[test]
    fn test_keycode_roundtrip() {
        let key = KeyCode::Z;
        let serializable = SerializableKeyCode::from_keycode(key);
        assert_eq!(serializable.to_keycode(), Some(key));
    }

    #[test]
    fn test_key_config_serialization() {
        let config = KeyConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: KeyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_key_config_json_readable() {
        let config = KeyConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"LeftShift\""));
        assert!(json.contains("\"Z\""));
    }
}
