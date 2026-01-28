//! beatoraja skin compatibility layer
//!
//! This module provides support for loading and rendering beatoraja skins.
//! Both JSON and Lua skin formats are supported.
//!
//! # Supported Features
//!
//! - JSON skin format (`.json`)
//! - Lua skin format (`.luaskin` + `.lua`)
//! - Image sources and regions
//! - Animation with keyframes
//! - Conditional rendering (if/op system)
//! - Timer-based effects
//! - Custom properties
//!
//! # Usage
//!
//! ```ignore
//! use skin::beatoraja::{load_skin, SkinFormat};
//!
//! let skin = load_skin("/path/to/skin")?;
//! ```

#![allow(dead_code)]
#![allow(unused_imports)]

mod conditions;
mod converter;
mod lua_parser;
mod parser;
mod properties;
mod renderer;
pub mod scene;
mod timers;
mod types;

use std::path::Path;

use anyhow::{Result, bail};

pub use conditions::{
    ClearType, DjRank, GameState, GaugeType, JudgeType, TimingType, evaluate_condition,
    evaluate_conditions, opcodes, timers as timer_ids, values,
};
pub use converter::{
    ImageRegion, ResolvedDestination, SkinScale, SkinTypeInfo, resolve_destination, validate_skin,
};
pub use parser::{SkinFormat, detect_skin_format, is_beatoraja_json, parse_json_skin};
pub use properties::{
    ParsedOption, ParsedProperty, PropertyManager, PropertyType, build_property,
    build_slider_property, parse_properties,
};
pub use renderer::{SkinAssets, SkinRenderer};
pub use timers::{JudgeTimerType, TimerManager, calculate_frame, interpolate_destination};
pub use types::{
    BeatorajaSkin, BgaElement, CustomProperty, Destination, ElementBase, FontDef, GaugeSkin,
    GraphElement, ImageDef, ImageElement, ImageSet, ImageSource, JudgeElement, NoteSkin,
    NumberElement, OffsetDef, SkinHeader, SkinType, SliderElement, TextElement, ValueDef,
};

/// Load a beatoraja skin from a directory or file
pub fn load_skin(path: &Path) -> Result<BeatorajaSkin> {
    if path.is_file() {
        // Direct file path
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "json" => return parse_json_skin(path),
                "luaskin" => {
                    if let Some(lua_main) = find_lua_main(path) {
                        return lua_parser::parse_lua_skin(path, &lua_main);
                    }
                    bail!("Could not find main Lua file for skin: {}", path.display());
                }
                _ => bail!("Unknown skin file type: {}", ext),
            }
        }
        bail!("Skin file has no extension: {}", path.display());
    }

    if path.is_dir() {
        match detect_skin_format(path) {
            SkinFormat::Json(json_path) => parse_json_skin(&json_path),
            SkinFormat::Lua { wrapper, main } => lua_parser::parse_lua_skin(&wrapper, &main),
            SkinFormat::None => bail!("No beatoraja skin found in: {}", path.display()),
        }
    } else {
        bail!("Path does not exist: {}", path.display());
    }
}

/// Find the main Lua file for a .luaskin wrapper
fn find_lua_main(wrapper_path: &Path) -> Option<std::path::PathBuf> {
    let content = std::fs::read_to_string(wrapper_path).ok()?;

    #[derive(serde::Deserialize)]
    struct LuaSkinWrapper {
        #[serde(default)]
        main: String,
    }

    let wrapper: LuaSkinWrapper = serde_json::from_str(&content).ok()?;

    let parent = wrapper_path.parent()?;

    if wrapper.main.is_empty() {
        let default_main = parent.join("skin.lua");
        if default_main.exists() {
            return Some(default_main);
        }
        return None;
    }

    let main_path = parent.join(&wrapper.main);
    if main_path.exists() {
        Some(main_path)
    } else {
        None
    }
}

/// Skin compatibility report
#[derive(Debug, Clone, Default)]
pub struct CompatibilityReport {
    /// Skin name
    pub name: String,
    /// Skin type
    pub skin_type: i32,
    /// Base resolution
    pub resolution: (i32, i32),
    /// Number of image sources
    pub source_count: usize,
    /// Number of image definitions
    pub image_count: usize,
    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,
    /// Unsupported features used
    pub unsupported: Vec<String>,
}

impl CompatibilityReport {
    /// Generate a compatibility report for a skin
    pub fn from_skin(skin: &BeatorajaSkin) -> Self {
        // Check for unsupported features
        // (Add checks as features are identified as unsupported)

        Self {
            name: skin.header.name.clone(),
            skin_type: skin.header.skin_type,
            resolution: (skin.header.w, skin.header.h),
            source_count: skin.source.len(),
            image_count: skin.image.len(),
            warnings: validate_skin(skin),
            unsupported: Vec::new(),
        }
    }

    /// Check if the skin is likely to work
    pub fn is_compatible(&self) -> bool {
        self.unsupported.is_empty()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "{} ({}x{}, type {}): {} sources, {} images, {} warnings",
            self.name,
            self.resolution.0,
            self.resolution.1,
            self.skin_type,
            self.source_count,
            self.image_count,
            self.warnings.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_skin_nonexistent() {
        let result = load_skin(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_skin_json() {
        let dir = tempdir().unwrap();
        let skin_path = dir.path().join("skin.json");

        std::fs::write(
            &skin_path,
            r#"{"name": "Test", "type": 0, "w": 1920, "h": 1080, "source": [], "image": []}"#,
        )
        .unwrap();

        let skin = load_skin(&skin_path).unwrap();
        assert_eq!(skin.header.name, "Test");
    }

    #[test]
    fn test_load_skin_directory() {
        let dir = tempdir().unwrap();
        let skin_path = dir.path().join("skin.json");

        std::fs::write(
            &skin_path,
            r#"{"name": "DirTest", "type": 0, "w": 1280, "h": 720, "source": [], "image": []}"#,
        )
        .unwrap();

        let skin = load_skin(dir.path()).unwrap();
        assert_eq!(skin.header.name, "DirTest");
    }

    #[test]
    fn test_compatibility_report() {
        let skin = BeatorajaSkin {
            header: SkinHeader {
                name: "Test Skin".to_string(),
                author: "Test Author".to_string(),
                skin_type: 0,
                w: 1920,
                h: 1080,
                path: String::new(),
            },
            source: vec![ImageSource {
                id: 0,
                path: "bg.png".to_string(),
            }],
            image: vec![],
            ..Default::default()
        };

        let report = CompatibilityReport::from_skin(&skin);
        assert_eq!(report.name, "Test Skin");
        assert_eq!(report.resolution, (1920, 1080));
        assert_eq!(report.source_count, 1);
        assert!(report.is_compatible());
    }
}
