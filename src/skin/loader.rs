//! Skin file loading utilities

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::definition::SkinDefinition;

/// Skin loader utility
#[allow(dead_code)]
pub struct SkinLoader;

#[allow(dead_code)]
impl SkinLoader {
    /// Default skins directory name
    const SKINS_DIR: &'static str = "skins";

    /// Default skin directory name
    const DEFAULT_SKIN: &'static str = "default";

    /// Skin definition filename
    const SKIN_FILENAME: &'static str = "skin.json";

    /// Load a skin from a directory path
    pub fn load_from_directory(skin_dir: &Path) -> Result<SkinDefinition> {
        let skin_path = skin_dir.join(Self::SKIN_FILENAME);
        Self::load_from_file(&skin_path)
    }

    /// Load a skin from a JSON file path
    pub fn load_from_file(path: &Path) -> Result<SkinDefinition> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;

        let skin: SkinDefinition = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse skin file: {}", path.display()))?;

        Ok(skin)
    }

    /// Load skin by name from skins directory
    pub fn load_by_name(skin_name: &str, skins_base_dir: Option<&Path>) -> Result<SkinDefinition> {
        let base_dir = skins_base_dir
            .map(PathBuf::from)
            .unwrap_or_else(Self::default_skins_directory);

        let skin_dir = base_dir.join(skin_name);
        Self::load_from_directory(&skin_dir)
    }

    /// Get the default skins directory path
    pub fn default_skins_directory() -> PathBuf {
        // Try executable directory first
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let skins_dir = exe_dir.join(Self::SKINS_DIR);
                if skins_dir.exists() {
                    return skins_dir;
                }
            }
        }

        // Fall back to current directory
        PathBuf::from(Self::SKINS_DIR)
    }

    /// Load the default skin
    /// Returns built-in default if no custom default skin is found
    pub fn load_default(skins_base_dir: Option<&Path>) -> SkinDefinition {
        // Try to load custom default skin first
        if let Ok(skin) = Self::load_by_name(Self::DEFAULT_SKIN, skins_base_dir) {
            return skin;
        }

        // Fall back to built-in default
        SkinDefinition::default()
    }

    /// List available skins in the skins directory
    pub fn list_skins(skins_base_dir: Option<&Path>) -> Vec<String> {
        let base_dir = skins_base_dir
            .map(PathBuf::from)
            .unwrap_or_else(Self::default_skins_directory);

        if !base_dir.exists() {
            return vec![];
        }

        let mut skins = Vec::new();

        if let Ok(entries) = fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skin_json = path.join(Self::SKIN_FILENAME);
                    if skin_json.exists() {
                        if let Some(name) = path.file_name() {
                            skins.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        skins.sort();
        skins
    }

    /// Save a skin definition to a file
    pub fn save_to_file(skin: &SkinDefinition, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(skin).context("Failed to serialize skin definition")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write skin file: {}", path.display()))?;

        Ok(())
    }

    /// Export the default skin as a JSON file (for users to customize)
    pub fn export_default_skin(output_dir: &Path) -> Result<PathBuf> {
        let skin_dir = output_dir.join(Self::DEFAULT_SKIN);
        let skin_path = skin_dir.join(Self::SKIN_FILENAME);

        let default_skin = SkinDefinition::default();
        Self::save_to_file(&default_skin, &skin_path)?;

        Ok(skin_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_default() {
        let skin = SkinLoader::load_default(None);
        assert_eq!(skin.info.name, "Default");
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let skin = SkinDefinition::default();
        let skin_dir = dir.path().join("test_skin");
        let skin_path = skin_dir.join("skin.json");

        SkinLoader::save_to_file(&skin, &skin_path).unwrap();

        let loaded = SkinLoader::load_from_directory(&skin_dir).unwrap();
        assert_eq!(loaded.info.name, skin.info.name);
    }

    #[test]
    fn test_list_skins_empty() {
        let dir = tempdir().unwrap();
        let skins = SkinLoader::list_skins(Some(dir.path()));
        assert!(skins.is_empty());
    }

    #[test]
    fn test_export_default_skin() {
        let dir = tempdir().unwrap();
        let path = SkinLoader::export_default_skin(dir.path()).unwrap();
        assert!(path.exists());

        let skin = SkinLoader::load_from_file(&path).unwrap();
        assert_eq!(skin.info.name, "Default");
    }
}
