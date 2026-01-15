use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::game::{GaugeType, JudgeSystemType};

/// User settings for the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Judge system type (beatoraja or LR2)
    pub judge_system: JudgeSystemType,
    /// Default gauge type
    pub gauge_type: GaugeType,
    /// Default scroll speed
    pub scroll_speed: f32,
    /// Default SUDDEN+ value
    pub sudden: u16,
    /// Default LIFT value
    pub lift: u16,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            judge_system: JudgeSystemType::Beatoraja,
            gauge_type: GaugeType::Normal,
            scroll_speed: 1.0,
            sudden: 0,
            lift: 0,
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
        if let Some(proj_dirs) = ProjectDirs::from("com", "bms-rs", "bms-player") {
            Ok(proj_dirs.config_dir().join("settings.json"))
        } else {
            Ok(PathBuf::from(".bms-player-settings.json"))
        }
    }
}
