use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use macroquad::prelude::{KeyCode, is_key_pressed};
use serde::{Deserialize, Serialize};

use crate::input::key_config::SerializableKeyCode;

const HOTKEY_CONFIG_FILE: &str = "hotkeys.json";

/// Hotkey actions available in the select screen.
/// 選曲画面で利用するホットキーのアクション。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectHotkey {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Decide,
    Cancel,
    SortNext,
    SortPrev,
    FilterNext,
    FilterPrev,
    TogglePreview,
    ToggleDetails,
    ToggleFavorite,
    OpenConfig,
    RandomSelect,
    ReloadDatabase,
}

impl SelectHotkey {
    pub fn all() -> &'static [SelectHotkey] {
        &[
            SelectHotkey::MoveUp,
            SelectHotkey::MoveDown,
            SelectHotkey::PageUp,
            SelectHotkey::PageDown,
            SelectHotkey::Decide,
            SelectHotkey::Cancel,
            SelectHotkey::SortNext,
            SelectHotkey::SortPrev,
            SelectHotkey::FilterNext,
            SelectHotkey::FilterPrev,
            SelectHotkey::TogglePreview,
            SelectHotkey::ToggleDetails,
            SelectHotkey::ToggleFavorite,
            SelectHotkey::OpenConfig,
            SelectHotkey::RandomSelect,
            SelectHotkey::ReloadDatabase,
        ]
    }
}

/// Hotkey actions available in the play screen.
/// プレイ画面で利用するホットキーのアクション。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayHotkey {
    HiSpeedUp,
    HiSpeedDown,
    SpeedReset,
    SpeedDown,
    SpeedUp,
    PracticeStart,
    PracticeEnd,
    PracticeClear,
    ToggleBga,
}

impl PlayHotkey {
    pub fn all() -> &'static [PlayHotkey] {
        &[
            PlayHotkey::HiSpeedUp,
            PlayHotkey::HiSpeedDown,
            PlayHotkey::SpeedReset,
            PlayHotkey::SpeedDown,
            PlayHotkey::SpeedUp,
            PlayHotkey::PracticeStart,
            PlayHotkey::PracticeEnd,
            PlayHotkey::PracticeClear,
            PlayHotkey::ToggleBga,
        ]
    }
}

/// Hotkey actions available in config screens.
/// 設定画面で利用するホットキーのアクション。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigHotkey {
    Up,
    Down,
    Confirm,
    Cancel,
}

impl ConfigHotkey {
    pub fn all() -> &'static [ConfigHotkey] {
        &[
            ConfigHotkey::Up,
            ConfigHotkey::Down,
            ConfigHotkey::Confirm,
            ConfigHotkey::Cancel,
        ]
    }
}

/// Configurable hotkey mappings.
/// 設定可能なホットキーマッピング。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub select: HashMap<SelectHotkey, Vec<SerializableKeyCode>>,
    pub play: HashMap<PlayHotkey, Vec<SerializableKeyCode>>,
    pub config: HashMap<ConfigHotkey, Vec<SerializableKeyCode>>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        let mut select = HashMap::new();
        select.insert(
            SelectHotkey::MoveUp,
            vec![SerializableKeyCode::from_keycode(KeyCode::Up)],
        );
        select.insert(
            SelectHotkey::MoveDown,
            vec![SerializableKeyCode::from_keycode(KeyCode::Down)],
        );
        select.insert(
            SelectHotkey::PageUp,
            vec![SerializableKeyCode::from_keycode(KeyCode::PageUp)],
        );
        select.insert(
            SelectHotkey::PageDown,
            vec![SerializableKeyCode::from_keycode(KeyCode::PageDown)],
        );
        select.insert(
            SelectHotkey::Decide,
            vec![SerializableKeyCode::from_keycode(KeyCode::Enter)],
        );
        select.insert(
            SelectHotkey::Cancel,
            vec![SerializableKeyCode::from_keycode(KeyCode::Escape)],
        );
        select.insert(
            SelectHotkey::SortNext,
            vec![SerializableKeyCode::from_keycode(KeyCode::F3)],
        );
        select.insert(
            SelectHotkey::SortPrev,
            vec![SerializableKeyCode::from_keycode(KeyCode::F2)],
        );
        select.insert(
            SelectHotkey::FilterNext,
            vec![SerializableKeyCode::from_keycode(KeyCode::F5)],
        );
        select.insert(
            SelectHotkey::FilterPrev,
            vec![SerializableKeyCode::from_keycode(KeyCode::F4)],
        );
        select.insert(
            SelectHotkey::TogglePreview,
            vec![SerializableKeyCode::from_keycode(KeyCode::P)],
        );
        select.insert(
            SelectHotkey::ToggleDetails,
            vec![SerializableKeyCode::from_keycode(KeyCode::Tab)],
        );
        select.insert(
            SelectHotkey::ToggleFavorite,
            vec![SerializableKeyCode::from_keycode(KeyCode::F)],
        );
        select.insert(
            SelectHotkey::OpenConfig,
            vec![SerializableKeyCode::from_keycode(KeyCode::F1)],
        );
        select.insert(
            SelectHotkey::RandomSelect,
            vec![SerializableKeyCode::from_keycode(KeyCode::R)],
        );
        select.insert(
            SelectHotkey::ReloadDatabase,
            vec![SerializableKeyCode::from_keycode(KeyCode::F12)],
        );

        let mut play = HashMap::new();
        play.insert(
            PlayHotkey::HiSpeedUp,
            vec![SerializableKeyCode::from_keycode(KeyCode::Up)],
        );
        play.insert(
            PlayHotkey::HiSpeedDown,
            vec![SerializableKeyCode::from_keycode(KeyCode::Down)],
        );
        play.insert(
            PlayHotkey::SpeedReset,
            vec![SerializableKeyCode::from_keycode(KeyCode::F2)],
        );
        play.insert(
            PlayHotkey::SpeedDown,
            vec![SerializableKeyCode::from_keycode(KeyCode::F3)],
        );
        play.insert(
            PlayHotkey::SpeedUp,
            vec![SerializableKeyCode::from_keycode(KeyCode::F4)],
        );
        play.insert(
            PlayHotkey::PracticeStart,
            vec![SerializableKeyCode::from_keycode(KeyCode::F5)],
        );
        play.insert(
            PlayHotkey::PracticeEnd,
            vec![SerializableKeyCode::from_keycode(KeyCode::F6)],
        );
        play.insert(
            PlayHotkey::PracticeClear,
            vec![SerializableKeyCode::from_keycode(KeyCode::F7)],
        );
        play.insert(
            PlayHotkey::ToggleBga,
            vec![SerializableKeyCode::from_keycode(KeyCode::B)],
        );

        let mut config = HashMap::new();
        config.insert(
            ConfigHotkey::Up,
            vec![SerializableKeyCode::from_keycode(KeyCode::Up)],
        );
        config.insert(
            ConfigHotkey::Down,
            vec![SerializableKeyCode::from_keycode(KeyCode::Down)],
        );
        config.insert(
            ConfigHotkey::Confirm,
            vec![SerializableKeyCode::from_keycode(KeyCode::Enter)],
        );
        config.insert(
            ConfigHotkey::Cancel,
            vec![SerializableKeyCode::from_keycode(KeyCode::Escape)],
        );

        Self {
            select,
            play,
            config,
        }
    }
}

impl HotkeyConfig {
    /// Load hotkey configuration from default file.
    /// 既定のファイルからホットキー設定を読み込む。
    pub fn load() -> Result<Self> {
        Self::load_from(HOTKEY_CONFIG_FILE)
    }

    /// Load hotkey configuration from a path.
    /// 指定パスからホットキー設定を読み込む。
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

    /// Save hotkey configuration to default file.
    /// 既定のファイルへホットキー設定を保存する。
    pub fn save(&self) -> Result<()> {
        self.save_to(HOTKEY_CONFIG_FILE)
    }

    /// Save hotkey configuration to a path.
    /// 指定パスへホットキー設定を保存する。
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn pressed_select(&self, action: SelectHotkey) -> bool {
        self.resolve_select(action)
            .iter()
            .any(|key| is_key_pressed(*key))
    }

    pub fn pressed_play(&self, action: PlayHotkey) -> bool {
        self.resolve_play(action)
            .iter()
            .any(|key| is_key_pressed(*key))
    }

    pub fn pressed_config(&self, action: ConfigHotkey) -> bool {
        self.resolve_config(action)
            .iter()
            .any(|key| is_key_pressed(*key))
    }

    fn resolve_select(&self, action: SelectHotkey) -> Vec<KeyCode> {
        self.select
            .get(&action)
            .map(|keys| keys.iter().filter_map(|k| k.to_keycode()).collect())
            .unwrap_or_default()
    }

    fn resolve_play(&self, action: PlayHotkey) -> Vec<KeyCode> {
        self.play
            .get(&action)
            .map(|keys| keys.iter().filter_map(|k| k.to_keycode()).collect())
            .unwrap_or_default()
    }

    fn resolve_config(&self, action: ConfigHotkey) -> Vec<KeyCode> {
        self.config
            .get(&action)
            .map(|keys| keys.iter().filter_map(|k| k.to_keycode()).collect())
            .unwrap_or_default()
    }

    fn normalize(&mut self) {
        let defaults = Self::default();

        for action in SelectHotkey::all() {
            self.select
                .entry(*action)
                .or_insert_with(|| defaults.select.get(action).cloned().unwrap_or_default());
        }
        for action in PlayHotkey::all() {
            self.play
                .entry(*action)
                .or_insert_with(|| defaults.play.get(action).cloned().unwrap_or_default());
        }
        for action in ConfigHotkey::all() {
            self.config
                .entry(*action)
                .or_insert_with(|| defaults.config.get(action).cloned().unwrap_or_default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_defaults_have_actions() {
        let config = HotkeyConfig::default();
        for action in SelectHotkey::all() {
            assert!(config.select.contains_key(action));
        }
        for action in PlayHotkey::all() {
            assert!(config.play.contains_key(action));
        }
        for action in ConfigHotkey::all() {
            assert!(config.config.contains_key(action));
        }
    }
}
