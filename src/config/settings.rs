use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use macroquad::prelude::KeyCode;
use serde::{Deserialize, Serialize};

use crate::game::{GaugeType, JudgeSystemType, RandomOption};
use crate::ir::IrServerType;

/// Key bindings for 7-key + scratch (BMS mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub scratch: String,
    pub key1: String,
    pub key2: String,
    pub key3: String,
    pub key4: String,
    pub key5: String,
    pub key6: String,
    pub key7: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            scratch: "LeftShift".to_string(),
            key1: "Z".to_string(),
            key2: "S".to_string(),
            key3: "X".to_string(),
            key4: "D".to_string(),
            key5: "C".to_string(),
            key6: "F".to_string(),
            key7: "V".to_string(),
        }
    }
}

impl KeyBindings {
    /// Convert to array of KeyCodes for InputHandler
    pub fn to_keycodes(&self) -> [KeyCode; 8] {
        [
            string_to_keycode(&self.scratch).unwrap_or(KeyCode::LeftShift),
            string_to_keycode(&self.key1).unwrap_or(KeyCode::Z),
            string_to_keycode(&self.key2).unwrap_or(KeyCode::S),
            string_to_keycode(&self.key3).unwrap_or(KeyCode::X),
            string_to_keycode(&self.key4).unwrap_or(KeyCode::D),
            string_to_keycode(&self.key5).unwrap_or(KeyCode::C),
            string_to_keycode(&self.key6).unwrap_or(KeyCode::F),
            string_to_keycode(&self.key7).unwrap_or(KeyCode::V),
        ]
    }

    /// Set binding for a specific lane
    pub fn set(&mut self, lane: usize, key: KeyCode) {
        let key_str = keycode_to_string(key);
        match lane {
            0 => self.scratch = key_str,
            1 => self.key1 = key_str,
            2 => self.key2 = key_str,
            3 => self.key3 = key_str,
            4 => self.key4 = key_str,
            5 => self.key5 = key_str,
            6 => self.key6 = key_str,
            7 => self.key7 = key_str,
            _ => {}
        }
    }
}

/// Key bindings for 9-key (PMS mode)
/// Default layout: A S D F Space J K L ;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings9Key {
    pub key1: String,
    pub key2: String,
    pub key3: String,
    pub key4: String,
    pub key5: String,
    pub key6: String,
    pub key7: String,
    pub key8: String,
    pub key9: String,
}

impl Default for KeyBindings9Key {
    fn default() -> Self {
        Self {
            key1: "A".to_string(),
            key2: "S".to_string(),
            key3: "D".to_string(),
            key4: "F".to_string(),
            key5: "Space".to_string(),
            key6: "J".to_string(),
            key7: "K".to_string(),
            key8: "L".to_string(),
            key9: "Semicolon".to_string(),
        }
    }
}

impl KeyBindings9Key {
    /// Convert to array of KeyCodes for InputHandler
    #[allow(dead_code)]
    pub fn to_keycodes(&self) -> [KeyCode; 9] {
        [
            string_to_keycode(&self.key1).unwrap_or(KeyCode::A),
            string_to_keycode(&self.key2).unwrap_or(KeyCode::S),
            string_to_keycode(&self.key3).unwrap_or(KeyCode::D),
            string_to_keycode(&self.key4).unwrap_or(KeyCode::F),
            string_to_keycode(&self.key5).unwrap_or(KeyCode::Space),
            string_to_keycode(&self.key6).unwrap_or(KeyCode::J),
            string_to_keycode(&self.key7).unwrap_or(KeyCode::K),
            string_to_keycode(&self.key8).unwrap_or(KeyCode::L),
            string_to_keycode(&self.key9).unwrap_or(KeyCode::Semicolon),
        ]
    }

    /// Set binding for a specific lane (0-8)
    #[allow(dead_code)]
    pub fn set(&mut self, lane: usize, key: KeyCode) {
        let key_str = keycode_to_string(key);
        match lane {
            0 => self.key1 = key_str,
            1 => self.key2 = key_str,
            2 => self.key3 = key_str,
            3 => self.key4 = key_str,
            4 => self.key5 = key_str,
            5 => self.key6 = key_str,
            6 => self.key7 = key_str,
            7 => self.key8 = key_str,
            8 => self.key9 = key_str,
            _ => {}
        }
    }
}

/// Key bindings for DP 14-key mode (P1: scratch + 7 keys, P2: 7 keys + scratch)
/// Default layout:
/// P1: LeftShift(S) Z S X D C F V (1-7)
/// P2: M K , L . ; / RightShift(S)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingsDp {
    // P1 side (lanes 0-7)
    pub p1_scratch: String,
    pub p1_key1: String,
    pub p1_key2: String,
    pub p1_key3: String,
    pub p1_key4: String,
    pub p1_key5: String,
    pub p1_key6: String,
    pub p1_key7: String,
    // P2 side (lanes 8-15)
    pub p2_key1: String,
    pub p2_key2: String,
    pub p2_key3: String,
    pub p2_key4: String,
    pub p2_key5: String,
    pub p2_key6: String,
    pub p2_key7: String,
    pub p2_scratch: String,
}

impl Default for KeyBindingsDp {
    fn default() -> Self {
        Self {
            // P1 side (left hand)
            p1_scratch: "LeftShift".to_string(),
            p1_key1: "Z".to_string(),
            p1_key2: "S".to_string(),
            p1_key3: "X".to_string(),
            p1_key4: "D".to_string(),
            p1_key5: "C".to_string(),
            p1_key6: "F".to_string(),
            p1_key7: "V".to_string(),
            // P2 side (right hand)
            p2_key1: "M".to_string(),
            p2_key2: "K".to_string(),
            p2_key3: "Comma".to_string(),
            p2_key4: "L".to_string(),
            p2_key5: "Period".to_string(),
            p2_key6: "Semicolon".to_string(),
            p2_key7: "Slash".to_string(),
            p2_scratch: "RightShift".to_string(),
        }
    }
}

impl KeyBindingsDp {
    /// Convert to array of KeyCodes for InputHandler (16 keys)
    #[allow(dead_code)]
    pub fn to_keycodes(&self) -> [KeyCode; 16] {
        [
            // P1 side
            string_to_keycode(&self.p1_scratch).unwrap_or(KeyCode::LeftShift),
            string_to_keycode(&self.p1_key1).unwrap_or(KeyCode::Z),
            string_to_keycode(&self.p1_key2).unwrap_or(KeyCode::S),
            string_to_keycode(&self.p1_key3).unwrap_or(KeyCode::X),
            string_to_keycode(&self.p1_key4).unwrap_or(KeyCode::D),
            string_to_keycode(&self.p1_key5).unwrap_or(KeyCode::C),
            string_to_keycode(&self.p1_key6).unwrap_or(KeyCode::F),
            string_to_keycode(&self.p1_key7).unwrap_or(KeyCode::V),
            // P2 side
            string_to_keycode(&self.p2_key1).unwrap_or(KeyCode::M),
            string_to_keycode(&self.p2_key2).unwrap_or(KeyCode::K),
            string_to_keycode(&self.p2_key3).unwrap_or(KeyCode::Comma),
            string_to_keycode(&self.p2_key4).unwrap_or(KeyCode::L),
            string_to_keycode(&self.p2_key5).unwrap_or(KeyCode::Period),
            string_to_keycode(&self.p2_key6).unwrap_or(KeyCode::Semicolon),
            string_to_keycode(&self.p2_key7).unwrap_or(KeyCode::Slash),
            string_to_keycode(&self.p2_scratch).unwrap_or(KeyCode::RightShift),
        ]
    }

    /// Set binding for a specific lane (0-15)
    #[allow(dead_code)]
    pub fn set(&mut self, lane: usize, key: KeyCode) {
        let key_str = keycode_to_string(key);
        match lane {
            0 => self.p1_scratch = key_str,
            1 => self.p1_key1 = key_str,
            2 => self.p1_key2 = key_str,
            3 => self.p1_key3 = key_str,
            4 => self.p1_key4 = key_str,
            5 => self.p1_key5 = key_str,
            6 => self.p1_key6 = key_str,
            7 => self.p1_key7 = key_str,
            8 => self.p2_key1 = key_str,
            9 => self.p2_key2 = key_str,
            10 => self.p2_key3 = key_str,
            11 => self.p2_key4 = key_str,
            12 => self.p2_key5 = key_str,
            13 => self.p2_key6 = key_str,
            14 => self.p2_key7 = key_str,
            15 => self.p2_scratch = key_str,
            _ => {}
        }
    }
}

/// Controller bindings for IIDX-style controllers
///
/// Format:
/// - "Button:South" - Button input (A/Cross button)
/// - "Axis:LeftStickX:+" - Axis input (positive direction)
/// - "Axis:LeftStickX:-" - Axis input (negative direction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerBindings {
    pub scratch: String,
    pub key1: String,
    pub key2: String,
    pub key3: String,
    pub key4: String,
    pub key5: String,
    pub key6: String,
    pub key7: String,
    /// Axis threshold for scratch detection (0.0-1.0)
    pub axis_threshold: f32,
}

impl Default for ControllerBindings {
    fn default() -> Self {
        Self {
            // Default: axis for scratch (for IIDX controllers with turntable)
            scratch: "Axis:LeftStickX:+".to_string(),
            key1: "Button:South".to_string(),
            key2: "Button:East".to_string(),
            key3: "Button:North".to_string(),
            key4: "Button:West".to_string(),
            key5: "Button:LeftTrigger".to_string(),
            key6: "Button:RightTrigger".to_string(),
            key7: "Button:LeftTrigger2".to_string(),
            axis_threshold: 0.3,
        }
    }
}

/// IR (Internet Ranking) settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrSettings {
    /// Enable IR score submission
    pub enabled: bool,
    /// IR server type
    pub server_type: IrServerType,
    /// Custom server URL (used when server_type is Custom)
    pub server_url: String,
    /// Player ID on the IR server
    pub player_id: String,
    /// Player name for display
    pub player_name: String,
    /// Auto-submit scores after play
    pub auto_submit: bool,
    /// Submit scores even when assist options are used
    pub submit_with_assist: bool,
    /// Secret key for score hash generation
    pub secret_key: String,
}

impl Default for IrSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            server_type: IrServerType::default(),
            server_url: String::new(),
            player_id: String::new(),
            player_name: String::new(),
            auto_submit: true,
            submit_with_assist: false,
            secret_key: String::new(),
        }
    }
}

impl IrSettings {
    /// Get the effective server URL based on server type
    pub fn effective_url(&self) -> String {
        match self.server_type {
            IrServerType::Custom => self.server_url.clone(),
            _ => self.server_type.default_url().to_string(),
        }
    }

    /// Check if IR is properly configured for submission
    pub fn is_configured(&self) -> bool {
        self.enabled && !self.player_id.is_empty() && !self.effective_url().is_empty()
    }
}

/// Convert KeyCode to string for serialization
pub fn keycode_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Space => "Space".to_string(),
        KeyCode::Apostrophe => "Apostrophe".to_string(),
        KeyCode::Comma => "Comma".to_string(),
        KeyCode::Minus => "Minus".to_string(),
        KeyCode::Period => "Period".to_string(),
        KeyCode::Slash => "Slash".to_string(),
        KeyCode::Key0 => "0".to_string(),
        KeyCode::Key1 => "1".to_string(),
        KeyCode::Key2 => "2".to_string(),
        KeyCode::Key3 => "3".to_string(),
        KeyCode::Key4 => "4".to_string(),
        KeyCode::Key5 => "5".to_string(),
        KeyCode::Key6 => "6".to_string(),
        KeyCode::Key7 => "7".to_string(),
        KeyCode::Key8 => "8".to_string(),
        KeyCode::Key9 => "9".to_string(),
        KeyCode::Semicolon => "Semicolon".to_string(),
        KeyCode::Equal => "Equal".to_string(),
        KeyCode::A => "A".to_string(),
        KeyCode::B => "B".to_string(),
        KeyCode::C => "C".to_string(),
        KeyCode::D => "D".to_string(),
        KeyCode::E => "E".to_string(),
        KeyCode::F => "F".to_string(),
        KeyCode::G => "G".to_string(),
        KeyCode::H => "H".to_string(),
        KeyCode::I => "I".to_string(),
        KeyCode::J => "J".to_string(),
        KeyCode::K => "K".to_string(),
        KeyCode::L => "L".to_string(),
        KeyCode::M => "M".to_string(),
        KeyCode::N => "N".to_string(),
        KeyCode::O => "O".to_string(),
        KeyCode::P => "P".to_string(),
        KeyCode::Q => "Q".to_string(),
        KeyCode::R => "R".to_string(),
        KeyCode::S => "S".to_string(),
        KeyCode::T => "T".to_string(),
        KeyCode::U => "U".to_string(),
        KeyCode::V => "V".to_string(),
        KeyCode::W => "W".to_string(),
        KeyCode::X => "X".to_string(),
        KeyCode::Y => "Y".to_string(),
        KeyCode::Z => "Z".to_string(),
        KeyCode::LeftBracket => "LeftBracket".to_string(),
        KeyCode::Backslash => "Backslash".to_string(),
        KeyCode::RightBracket => "RightBracket".to_string(),
        KeyCode::GraveAccent => "GraveAccent".to_string(),
        KeyCode::Escape => "Escape".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::ScrollLock => "ScrollLock".to_string(),
        KeyCode::NumLock => "NumLock".to_string(),
        KeyCode::PrintScreen => "PrintScreen".to_string(),
        KeyCode::Pause => "Pause".to_string(),
        KeyCode::F1 => "F1".to_string(),
        KeyCode::F2 => "F2".to_string(),
        KeyCode::F3 => "F3".to_string(),
        KeyCode::F4 => "F4".to_string(),
        KeyCode::F5 => "F5".to_string(),
        KeyCode::F6 => "F6".to_string(),
        KeyCode::F7 => "F7".to_string(),
        KeyCode::F8 => "F8".to_string(),
        KeyCode::F9 => "F9".to_string(),
        KeyCode::F10 => "F10".to_string(),
        KeyCode::F11 => "F11".to_string(),
        KeyCode::F12 => "F12".to_string(),
        KeyCode::LeftShift => "LeftShift".to_string(),
        KeyCode::LeftControl => "LeftControl".to_string(),
        KeyCode::LeftAlt => "LeftAlt".to_string(),
        KeyCode::LeftSuper => "LeftSuper".to_string(),
        KeyCode::RightShift => "RightShift".to_string(),
        KeyCode::RightControl => "RightControl".to_string(),
        KeyCode::RightAlt => "RightAlt".to_string(),
        KeyCode::RightSuper => "RightSuper".to_string(),
        KeyCode::Kp0 => "Kp0".to_string(),
        KeyCode::Kp1 => "Kp1".to_string(),
        KeyCode::Kp2 => "Kp2".to_string(),
        KeyCode::Kp3 => "Kp3".to_string(),
        KeyCode::Kp4 => "Kp4".to_string(),
        KeyCode::Kp5 => "Kp5".to_string(),
        KeyCode::Kp6 => "Kp6".to_string(),
        KeyCode::Kp7 => "Kp7".to_string(),
        KeyCode::Kp8 => "Kp8".to_string(),
        KeyCode::Kp9 => "Kp9".to_string(),
        KeyCode::KpDecimal => "KpDecimal".to_string(),
        KeyCode::KpDivide => "KpDivide".to_string(),
        KeyCode::KpMultiply => "KpMultiply".to_string(),
        KeyCode::KpSubtract => "KpSubtract".to_string(),
        KeyCode::KpAdd => "KpAdd".to_string(),
        KeyCode::KpEnter => "KpEnter".to_string(),
        KeyCode::KpEqual => "KpEqual".to_string(),
        KeyCode::Menu => "Menu".to_string(),
        KeyCode::Unknown => "Unknown".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Convert string to KeyCode for deserialization
pub fn string_to_keycode(s: &str) -> Option<KeyCode> {
    match s {
        "Space" => Some(KeyCode::Space),
        "Apostrophe" => Some(KeyCode::Apostrophe),
        "Comma" => Some(KeyCode::Comma),
        "Minus" => Some(KeyCode::Minus),
        "Period" => Some(KeyCode::Period),
        "Slash" => Some(KeyCode::Slash),
        "0" => Some(KeyCode::Key0),
        "1" => Some(KeyCode::Key1),
        "2" => Some(KeyCode::Key2),
        "3" => Some(KeyCode::Key3),
        "4" => Some(KeyCode::Key4),
        "5" => Some(KeyCode::Key5),
        "6" => Some(KeyCode::Key6),
        "7" => Some(KeyCode::Key7),
        "8" => Some(KeyCode::Key8),
        "9" => Some(KeyCode::Key9),
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
        "LeftShift" => Some(KeyCode::LeftShift),
        "LeftControl" => Some(KeyCode::LeftControl),
        "LeftAlt" => Some(KeyCode::LeftAlt),
        "LeftSuper" => Some(KeyCode::LeftSuper),
        "RightShift" => Some(KeyCode::RightShift),
        "RightControl" => Some(KeyCode::RightControl),
        "RightAlt" => Some(KeyCode::RightAlt),
        "RightSuper" => Some(KeyCode::RightSuper),
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
        "Menu" => Some(KeyCode::Menu),
        _ => None,
    }
}

/// User settings for the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Judge system type (beatoraja or LR2)
    pub judge_system: JudgeSystemType,
    /// Default gauge type
    pub gauge_type: GaugeType,
    /// Random option (MIRROR/RANDOM/R-RANDOM)
    #[serde(default)]
    pub random_option: RandomOption,
    /// Auto scratch option
    #[serde(default)]
    pub auto_scratch: bool,
    /// Legacy note option (convert LN to normal notes)
    #[serde(default)]
    pub legacy_note: bool,
    /// Expand judge option (widen judgment windows by 1.5x)
    #[serde(default)]
    pub expand_judge: bool,
    /// Battle option (flip layout to 2P side - scratch on right)
    #[serde(default)]
    pub battle: bool,
    /// Default scroll speed
    pub scroll_speed: f32,
    /// Default SUDDEN+ value
    pub sudden: u16,
    /// Default HIDDEN+ value
    pub hidden: u16,
    /// Default LIFT value
    pub lift: u16,
    /// Key bindings for BMS 7-key mode
    #[serde(default)]
    pub key_bindings: KeyBindings,
    /// Key bindings for PMS 9-key mode
    #[serde(default)]
    pub key_bindings_9key: KeyBindings9Key,
    /// Key bindings for DP 14-key mode
    #[serde(default)]
    pub key_bindings_dp: KeyBindingsDp,
    /// Controller bindings (for IIDX-style controllers)
    #[serde(default)]
    pub controller_bindings: ControllerBindings,
    /// IR (Internet Ranking) settings
    #[serde(default)]
    pub ir: IrSettings,
    /// Skin name (directory name in skins/)
    #[serde(default)]
    pub skin_name: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            judge_system: JudgeSystemType::Beatoraja,
            gauge_type: GaugeType::Normal,
            random_option: RandomOption::Off,
            auto_scratch: false,
            legacy_note: false,
            expand_judge: false,
            battle: false,
            scroll_speed: 1.0,
            sudden: 0,
            hidden: 0,
            lift: 0,
            key_bindings: KeyBindings::default(),
            key_bindings_9key: KeyBindings9Key::default(),
            key_bindings_dp: KeyBindingsDp::default(),
            controller_bindings: ControllerBindings::default(),
            ir: IrSettings::default(),
            skin_name: String::new(), // Empty means use built-in default
        }
    }
}

impl GameSettings {
    /// Load settings from disk
    pub fn load() -> Self {
        Self::load_from_file().unwrap_or_default()
    }

    fn load_from_file() -> Result<Self> {
        let path = Self::settings_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::settings_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn settings_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "brs", "brs") {
            Ok(proj_dirs.config_dir().join("settings.json"))
        } else {
            Ok(PathBuf::from(".brs-settings.json"))
        }
    }
}
