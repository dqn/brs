//! Skin definition for JSON serialization
//!
//! This struct represents the complete skin.json file format.

use serde::{Deserialize, Serialize};

use super::layout::LayoutConfig;
use super::theme::{EffectConfig, SkinTheme};

/// Skin metadata
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    /// Skin name
    pub name: String,
    /// Skin author
    pub author: String,
    /// Skin version (optional)
    #[serde(default)]
    pub version: String,
    /// Skin description (optional)
    #[serde(default)]
    pub description: String,
}

impl Default for SkinInfo {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            author: "bms-rs".to_string(),
            version: "1.0.0".to_string(),
            description: "Default built-in skin".to_string(),
        }
    }
}

/// Resolution configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
        }
    }
}

/// Complete skin definition
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkinDefinition {
    /// Skin metadata
    #[serde(default)]
    pub info: SkinInfo,
    /// Base resolution for layout calculations
    #[serde(default)]
    pub resolution: Resolution,
    /// Visual theme settings
    #[serde(default)]
    pub theme: SkinTheme,
    /// UI layout settings
    #[serde(default)]
    pub layout: LayoutConfig,
    /// Effect settings
    #[serde(default)]
    pub effects: EffectConfig,
}

impl SkinDefinition {
    /// Get the default built-in skin
    #[allow(dead_code)]
    pub fn default_skin() -> Self {
        Self::default()
    }

    /// Scale layout values for a different resolution
    #[allow(dead_code)]
    pub fn scale_for_resolution(&mut self, target_width: f32, target_height: f32) {
        let scale_x = target_width / self.resolution.width as f32;
        let scale_y = target_height / self.resolution.height as f32;

        // Scale BGA layout
        self.layout.bga.position.x *= scale_x;
        self.layout.bga.position.y *= scale_y;
        self.layout.bga.width *= scale_x.min(scale_y);
        self.layout.bga.height *= scale_x.min(scale_y);

        // Scale gauge bar
        self.layout.gauge.bar_width *= scale_x;
        self.layout.gauge.bar_height *= scale_y;

        // Note: Font sizes are kept as-is for readability
        // Individual skins can override if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_skin() {
        let skin = SkinDefinition::default_skin();
        assert_eq!(skin.info.name, "Default");
        assert_eq!(skin.resolution.width, 1920);
        assert_eq!(skin.resolution.height, 1080);
    }

    #[test]
    fn test_serialize_deserialize() {
        let skin = SkinDefinition::default_skin();
        let json = serde_json::to_string_pretty(&skin).unwrap();
        let deserialized: SkinDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.info.name, skin.info.name);
    }
}
