//! beatoraja JSON skin parser

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::types::BeatorajaSkin;

/// Parse a beatoraja JSON skin file
pub fn parse_json_skin(path: &Path) -> Result<BeatorajaSkin> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read skin file: {}", path.display()))?;

    parse_json_skin_str(&content, path)
}

/// Parse a beatoraja JSON skin from a string
pub fn parse_json_skin_str(content: &str, path: &Path) -> Result<BeatorajaSkin> {
    let mut skin: BeatorajaSkin = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse skin JSON: {}", path.display()))?;

    // Store the skin directory path for relative resource resolution
    if let Some(parent) = path.parent() {
        skin.header.path = parent.to_string_lossy().to_string();
    }

    Ok(skin)
}

/// Check if a file is a beatoraja JSON skin
pub fn is_beatoraja_json(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if extension.to_lowercase() != "json" {
        return false;
    }

    // Try to peek at the file to check for beatoraja skin markers
    if let Ok(content) = fs::read_to_string(path) {
        // beatoraja skins typically have "type" field at top level
        // and often have "source" and "image" arrays
        return content.contains("\"type\"")
            && (content.contains("\"source\"")
                || content.contains("\"image\"")
                || content.contains("\"destination\""));
    }

    false
}

/// Detect beatoraja skin format from directory
pub enum SkinFormat {
    /// Standard JSON format
    Json(std::path::PathBuf),
    /// Lua skin format (.luaskin wrapper + .lua main)
    Lua {
        wrapper: std::path::PathBuf,
        main: std::path::PathBuf,
    },
    /// Not a beatoraja skin
    None,
}

/// Detect skin format in a directory
pub fn detect_skin_format(dir: &Path) -> SkinFormat {
    if !dir.is_dir() {
        return SkinFormat::None;
    }

    // Check for .luaskin wrapper first (Lua skins)
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "luaskin" {
                    // Try to find the main Lua file
                    if let Some(lua_main) = find_lua_main(&path) {
                        return SkinFormat::Lua {
                            wrapper: path,
                            main: lua_main,
                        };
                    }
                }
            }
        }
    }

    // Check for JSON skins
    // Common naming patterns: skin.json, main.json, *.json
    let json_candidates = ["skin.json", "main.json"];
    for name in &json_candidates {
        let path = dir.join(name);
        if is_beatoraja_json(&path) {
            return SkinFormat::Json(path);
        }
    }

    // Try any .json file in the directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_beatoraja_json(&path) {
                return SkinFormat::Json(path);
            }
        }
    }

    SkinFormat::None
}

/// Find the main Lua file referenced by a .luaskin wrapper
fn find_lua_main(wrapper_path: &Path) -> Option<std::path::PathBuf> {
    let content = fs::read_to_string(wrapper_path).ok()?;

    // .luaskin files are JSON that reference the main Lua file
    #[derive(serde::Deserialize)]
    struct LuaSkinWrapper {
        #[serde(default)]
        main: String,
    }

    let wrapper: LuaSkinWrapper = serde_json::from_str(&content).ok()?;

    if wrapper.main.is_empty() {
        // Default to skin.lua in the same directory
        let parent = wrapper_path.parent()?;
        let default_main = parent.join("skin.lua");
        if default_main.exists() {
            return Some(default_main);
        }
        return None;
    }

    let parent = wrapper_path.parent()?;
    let main_path = parent.join(&wrapper.main);

    if main_path.exists() {
        Some(main_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_minimal_skin() {
        let json = r#"{
            "name": "Test",
            "type": 0,
            "w": 1920,
            "h": 1080,
            "source": [],
            "image": []
        }"#;

        let dir = tempdir().unwrap();
        let path = dir.path().join("skin.json");
        fs::write(&path, json).unwrap();

        let skin = parse_json_skin(&path).unwrap();
        assert_eq!(skin.header.name, "Test");
        assert_eq!(skin.header.skin_type, 0);
    }

    #[test]
    fn test_is_beatoraja_json() {
        let dir = tempdir().unwrap();

        // Valid beatoraja skin
        let valid_path = dir.path().join("valid.json");
        fs::write(&valid_path, r#"{"type": 0, "source": [], "image": []}"#).unwrap();
        assert!(is_beatoraja_json(&valid_path));

        // Not a beatoraja skin (missing markers)
        let invalid_path = dir.path().join("invalid.json");
        fs::write(&invalid_path, r#"{"name": "something else"}"#).unwrap();
        assert!(!is_beatoraja_json(&invalid_path));

        // Not a JSON file
        let txt_path = dir.path().join("test.txt");
        fs::write(&txt_path, "not json").unwrap();
        assert!(!is_beatoraja_json(&txt_path));
    }

    #[test]
    fn test_detect_skin_format_json() {
        let dir = tempdir().unwrap();

        let skin_path = dir.path().join("skin.json");
        fs::write(&skin_path, r#"{"type": 0, "source": [], "image": []}"#).unwrap();

        match detect_skin_format(dir.path()) {
            SkinFormat::Json(path) => {
                assert_eq!(path.file_name().unwrap(), "skin.json");
            }
            _ => panic!("Expected JSON format"),
        }
    }

    #[test]
    fn test_detect_skin_format_none() {
        let dir = tempdir().unwrap();

        // Empty directory
        match detect_skin_format(dir.path()) {
            SkinFormat::None => {}
            _ => panic!("Expected None format"),
        }
    }
}
