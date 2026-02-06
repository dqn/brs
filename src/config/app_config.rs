use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Per-scene skin paths.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkinPaths {
    #[serde(default)]
    pub select: String,
    #[serde(default)]
    pub decide: String,
    #[serde(default)]
    pub play: String,
    #[serde(default)]
    pub result: String,
}

/// Application-level configuration.
/// Controls display, audio driver, and general behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window width in pixels.
    #[serde(default = "default_width")]
    pub width: u32,
    /// Window height in pixels.
    #[serde(default = "default_height")]
    pub height: u32,
    /// Enable vertical sync.
    #[serde(default = "default_true")]
    pub vsync: bool,
    /// Target FPS (0 = unlimited).
    #[serde(default = "default_fps")]
    pub target_fps: u32,
    /// Audio driver name (e.g., "default").
    #[serde(default = "default_audio_driver")]
    pub audio_driver: String,
    /// Audio buffer size in frames.
    #[serde(default = "default_audio_buffer")]
    pub audio_buffer_size: u32,
    /// Skin path.
    #[serde(default)]
    pub skin_path: String,
    /// Per-scene skin paths.
    #[serde(default)]
    pub skin_paths: SkinPaths,
    /// BMS directories to scan for songs.
    #[serde(default)]
    pub bms_directories: Vec<String>,
    /// Whether to run in fullscreen mode.
    #[serde(default)]
    pub fullscreen: bool,
}

fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    720
}
fn default_true() -> bool {
    true
}
fn default_fps() -> u32 {
    240
}
fn default_audio_driver() -> String {
    "default".to_string()
}
fn default_audio_buffer() -> u32 {
    512
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            vsync: true,
            target_fps: default_fps(),
            audio_driver: default_audio_driver(),
            audio_buffer_size: default_audio_buffer(),
            skin_path: String::new(),
            skin_paths: SkinPaths::default(),
            bms_directories: Vec::new(),
            fullscreen: false,
        }
    }
}

impl AppConfig {
    /// Load configuration from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load from default config path, or create default if not found.
    pub fn load_or_default(config_dir: &Path) -> Self {
        let path = Self::default_path(config_dir);
        Self::load(&path).unwrap_or_default()
    }

    /// Default config file path.
    pub fn default_path(config_dir: &Path) -> PathBuf {
        config_dir.join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let config = AppConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(config.vsync);
        assert_eq!(config.target_fps, 240);
        assert_eq!(config.audio_driver, "default");
        assert_eq!(config.audio_buffer_size, 512);
        assert!(!config.fullscreen);
    }

    #[test]
    fn serialization_round_trip() {
        let config = AppConfig {
            width: 1920,
            height: 1080,
            vsync: false,
            target_fps: 120,
            audio_driver: "wasapi".to_string(),
            audio_buffer_size: 256,
            skin_path: "/path/to/skin".to_string(),
            skin_paths: SkinPaths::default(),
            bms_directories: vec!["/bms1".to_string(), "/bms2".to_string()],
            fullscreen: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.width, 1920);
        assert_eq!(restored.height, 1080);
        assert!(!restored.vsync);
        assert_eq!(restored.target_fps, 120);
        assert_eq!(restored.audio_driver, "wasapi");
        assert_eq!(restored.bms_directories.len(), 2);
        assert!(restored.fullscreen);
    }

    #[test]
    fn deserialization_with_defaults() {
        let json = r#"{"width": 800}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 720); // default
        assert!(config.vsync); // default
        assert_eq!(config.target_fps, 240); // default
    }

    #[test]
    fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        let config = AppConfig {
            width: 1600,
            ..Default::default()
        };
        config.save(&path).unwrap();

        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded.width, 1600);
    }
}
