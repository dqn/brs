use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use macroquad::prelude::*;

use crate::skin::font::{FontInfo, parse_fnt};

/// Loaded font with texture.
#[derive(Debug)]
pub struct LoadedFont {
    /// Font information from .fnt file.
    pub info: FontInfo,
    /// Font texture.
    pub texture: LoadedTexture,
}

/// Manager for skin image sources.
pub struct SkinSourceManager {
    /// Loaded textures indexed by source ID.
    textures: HashMap<u32, LoadedTexture>,
    /// Loaded fonts indexed by font ID.
    fonts: HashMap<u32, LoadedFont>,
    /// Base directory for the skin.
    base_dir: PathBuf,
}

/// A loaded texture with metadata.
#[derive(Debug)]
pub struct LoadedTexture {
    /// The texture data.
    pub texture: Texture2D,
    /// Original width.
    pub width: u32,
    /// Original height.
    pub height: u32,
}

impl SkinSourceManager {
    /// Create a new source manager.
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            textures: HashMap::new(),
            fonts: HashMap::new(),
            base_dir,
        }
    }

    /// Load a texture from a source definition.
    pub async fn load_source(&mut self, id: u32, path_pattern: &str) -> Result<()> {
        let resolved_path = self.resolve_path(path_pattern)?;

        let texture = load_texture(&resolved_path.to_string_lossy())
            .await
            .with_context(|| format!("Failed to load texture: {}", resolved_path.display()))?;

        // Set texture filter to nearest for pixel-perfect rendering
        texture.set_filter(FilterMode::Nearest);

        let loaded = LoadedTexture {
            width: texture.width() as u32,
            height: texture.height() as u32,
            texture,
        };

        self.textures.insert(id, loaded);
        Ok(())
    }

    /// Get a loaded texture by ID.
    pub fn get(&self, id: u32) -> Option<&LoadedTexture> {
        self.textures.get(&id)
    }

    /// Check if a source is loaded.
    pub fn is_loaded(&self, id: u32) -> bool {
        self.textures.contains_key(&id)
    }

    /// Load a font from a .fnt file.
    pub async fn load_font(&mut self, id: u32, fnt_path: &str) -> Result<()> {
        let fnt_full_path = self.base_dir.join(fnt_path);

        // Read and parse .fnt file
        let fnt_content = std::fs::read_to_string(&fnt_full_path)
            .with_context(|| format!("Failed to read font file: {}", fnt_full_path.display()))?;

        let font_info = parse_fnt(&fnt_content)
            .with_context(|| format!("Failed to parse font file: {}", fnt_full_path.display()))?;

        // Load the texture (relative to .fnt file location)
        let fnt_dir = fnt_full_path.parent().unwrap_or(Path::new("."));
        let texture_path = fnt_dir.join(&font_info.texture_file);

        let texture = load_texture(&texture_path.to_string_lossy())
            .await
            .with_context(|| format!("Failed to load font texture: {}", texture_path.display()))?;

        texture.set_filter(FilterMode::Nearest);

        let loaded_texture = LoadedTexture {
            width: texture.width() as u32,
            height: texture.height() as u32,
            texture,
        };

        let loaded_font = LoadedFont {
            info: font_info,
            texture: loaded_texture,
        };

        self.fonts.insert(id, loaded_font);
        Ok(())
    }

    /// Get font info and texture by ID.
    pub fn get_font_info(&self, id: u32) -> Option<(&FontInfo, &LoadedTexture)> {
        self.fonts.get(&id).map(|f| (&f.info, &f.texture))
    }

    /// Check if a font is loaded.
    pub fn is_font_loaded(&self, id: u32) -> bool {
        self.fonts.contains_key(&id)
    }

    /// Resolve a path pattern to an actual file path.
    fn resolve_path(&self, pattern: &str) -> Result<PathBuf> {
        // If path contains wildcard, try to find a matching file
        if pattern.contains('*') {
            self.resolve_wildcard_path(pattern)
        } else {
            Ok(self.base_dir.join(pattern))
        }
    }

    /// Resolve a wildcard path pattern.
    fn resolve_wildcard_path(&self, pattern: &str) -> Result<PathBuf> {
        // Split pattern into directory and file parts
        let path = Path::new(pattern);
        let parent = path.parent().unwrap_or(Path::new(""));
        let file_pattern = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let search_dir = self.base_dir.join(parent);

        if !search_dir.exists() {
            // Try without wildcard directory
            let direct_path = self.base_dir.join(pattern.replace("/*", ""));
            if direct_path.exists() {
                return Ok(direct_path);
            }
            anyhow::bail!("Directory not found: {}", search_dir.display());
        }

        // Find first matching file
        let glob_pattern = file_pattern.replace('*', "");
        for entry in std::fs::read_dir(&search_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(&glob_pattern) || glob_pattern.is_empty() || name.ends_with(".png") {
                return Ok(entry.path());
            }
        }

        // Fallback: return the pattern path
        Ok(search_dir.join(file_pattern))
    }

    /// Unload all textures.
    pub fn unload_all(&mut self) {
        self.textures.clear();
    }
}

/// Parameters for drawing a portion of a texture.
#[derive(Debug, Clone)]
pub struct DrawParams {
    /// Source rectangle (x, y, w, h) in texture coordinates.
    pub src_rect: Option<Rect>,
    /// Destination position and size.
    pub dest_rect: Rect,
    /// Rotation in degrees.
    pub rotation: f32,
    /// Color tint (RGBA).
    pub color: Color,
}

impl Default for DrawParams {
    fn default() -> Self {
        Self {
            src_rect: None,
            dest_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            rotation: 0.0,
            color: WHITE,
        }
    }
}

/// Draw a texture with the given parameters.
pub fn draw_texture_params(texture: &Texture2D, params: &DrawParams) {
    let dest_size = Some(vec2(params.dest_rect.w, params.dest_rect.h));

    let draw_params = DrawTextureParams {
        dest_size,
        source: params.src_rect,
        rotation: params.rotation.to_radians(),
        flip_x: false,
        flip_y: false,
        pivot: None,
    };

    draw_texture_ex(
        texture,
        params.dest_rect.x,
        params.dest_rect.y,
        params.color,
        draw_params,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_manager_creation() {
        let manager = SkinSourceManager::new(PathBuf::from("skins/ECFN/play"));
        assert!(manager.textures.is_empty());
    }
}
