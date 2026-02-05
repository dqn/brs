use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub max_fps: u32,
    pub player_name: String,
    pub song_folders: Vec<String>,
    pub auto_scan_on_startup: bool,
    pub play_skin_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            vsync: true,
            max_fps: 240,
            player_name: String::new(),
            song_folders: vec!["bms".to_string()],
            auto_scan_on_startup: true,
            play_skin_path: Some("skins/ECFN/play/play7main.lua".to_string()),
        }
    }
}

impl AppConfig {
    /// Loads config from the default config file.
    /// Returns default config if file doesn't exist.
    pub fn load() -> Result<Self> {
        Self::load_from(CONFIG_FILE)
    }

    /// Loads config from a specified path.
    /// Returns default config if file doesn't exist.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Saves config to the default config file.
    pub fn save(&self) -> Result<()> {
        self.save_to(CONFIG_FILE)
    }

    /// Saves config to a specified path.
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_values() {
        let config = AppConfig::default();
        assert_eq!(config.window_width, 1920);
        assert_eq!(config.window_height, 1080);
        assert!(!config.fullscreen);
        assert!(config.vsync);
        assert_eq!(config.max_fps, 240);
        assert!(config.player_name.is_empty());
        assert_eq!(config.song_folders, vec!["bms".to_string()]);
        assert!(config.auto_scan_on_startup);
        assert_eq!(
            config.play_skin_path,
            Some("skins/ECFN/play/play7main.lua".to_string())
        );
    }

    #[test]
    fn test_json_serialization() {
        let config = AppConfig {
            window_width: 1280,
            window_height: 720,
            fullscreen: true,
            vsync: false,
            max_fps: 120,
            player_name: "TestPlayer".to_string(),
            song_folders: vec!["songs".to_string(), "bms".to_string()],
            auto_scan_on_startup: false,
            play_skin_path: Some("skins/custom/play.luaskin".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_file_io() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.json");

        let config = AppConfig {
            window_width: 1600,
            window_height: 900,
            fullscreen: false,
            vsync: true,
            max_fps: 144,
            player_name: "Player1".to_string(),
            song_folders: vec!["bms".to_string()],
            auto_scan_on_startup: true,
            play_skin_path: None,
        };

        config.save_to(&file_path).unwrap();
        let loaded = AppConfig::load_from(&file_path).unwrap();

        assert_eq!(config, loaded);
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.json");

        let config = AppConfig::load_from(&file_path).unwrap();
        assert_eq!(config, AppConfig::default());
    }
}
