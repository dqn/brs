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
    /// File map overrides (sorted by key length, desc).
    file_map: Vec<(String, String)>,
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
            file_map: Vec::new(),
        }
    }

    /// Set file map overrides for resolving wildcard paths.
    pub fn set_file_map(&mut self, file_map: HashMap<String, String>) {
        let mut entries: Vec<(String, String)> = file_map.into_iter().collect();
        entries.sort_by(|(left, _), (right, _)| right.len().cmp(&left.len()));
        self.file_map = entries;
    }

    /// Load a texture from a source definition.
    pub async fn load_source(&mut self, id: u32, path_pattern: &str) -> Result<()> {
        let Some(resolved_path) = self.resolve_source_path(path_pattern)? else {
            return Ok(());
        };
        if !resolved_path.exists() {
            return Ok(());
        }

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
        let fnt_full_path = self.resolve_path(fnt_path)?;
        // Canonicalize to resolve .. in path
        let fnt_full_path = fnt_full_path
            .canonicalize()
            .unwrap_or_else(|_| fnt_full_path.clone());

        // Read .fnt file as bytes and convert with lossy UTF-8 (font files may contain non-UTF8 chars)
        let fnt_bytes = std::fs::read(&fnt_full_path)
            .with_context(|| format!("Failed to read font file: {}", fnt_full_path.display()))?;
        let fnt_content = String::from_utf8_lossy(&fnt_bytes);

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
        let mapped = self.apply_file_map(pattern);
        // If path contains wildcard, try to find a matching file
        if mapped.contains('*') {
            self.resolve_wildcard_path(&mapped)
        } else {
            let direct = self.base_dir.join(&mapped);
            if direct.exists() {
                return Ok(direct);
            }
            if let Some(found) = self.resolve_case_insensitive_path(&mapped) {
                return Ok(found);
            }
            Ok(direct)
        }
    }

    fn resolve_source_path(&self, pattern: &str) -> Result<Option<PathBuf>> {
        let mapped = self.apply_file_map(pattern);
        if mapped.contains('*') {
            if let Some(path) = self.resolve_wildcard_recursive(&mapped)? {
                return Ok(Some(path));
            }

            let direct_path = self.base_dir.join(mapped.replace("/*", ""));
            if direct_path.exists() {
                return Ok(Some(direct_path));
            }

            return Ok(None);
        }

        let direct = self.base_dir.join(&mapped);
        if direct.exists() {
            return Ok(Some(direct));
        }
        if let Some(found) = self.resolve_case_insensitive_path(&mapped) {
            return Ok(Some(found));
        }
        Ok(Some(direct))
    }

    /// Resolve a wildcard path pattern.
    fn resolve_wildcard_path(&self, pattern: &str) -> Result<PathBuf> {
        if let Some(path) = self.resolve_wildcard_recursive(pattern)? {
            return Ok(path);
        }

        let direct_path = self.base_dir.join(pattern.replace("/*", ""));
        if direct_path.exists() {
            return Ok(direct_path);
        }

        anyhow::bail!(
            "No matching file for pattern: {}",
            self.base_dir.join(pattern).display()
        )
    }

    fn resolve_wildcard_recursive(&self, pattern: &str) -> Result<Option<PathBuf>> {
        let mut candidates = vec![self.base_dir.clone()];
        let components: Vec<String> = Path::new(pattern)
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect();

        for (index, component) in components.iter().enumerate() {
            let is_last = index + 1 == components.len();
            let mut next_candidates = Vec::new();

            for base in &candidates {
                if component.contains('*') {
                    if !base.is_dir() {
                        continue;
                    }
                    for entry in std::fs::read_dir(base)? {
                        let entry = entry?;
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !matches_wildcard(&name, component)
                            && !matches_wildcard_case_insensitive(&name, component)
                        {
                            continue;
                        }
                        let path = entry.path();
                        if is_last {
                            if path.exists() {
                                return Ok(Some(path));
                            }
                        } else if path.is_dir() {
                            next_candidates.push(path);
                        }
                    }
                } else {
                    let mut path = base.join(component);
                    if !path.exists() {
                        if let Some(found) = find_case_insensitive_entry(base, component) {
                            path = found;
                        }
                    }
                    if is_last {
                        if path.exists() {
                            return Ok(Some(path));
                        }
                    } else if path.is_dir() {
                        next_candidates.push(path);
                    }
                }
            }

            if !is_last {
                if next_candidates.is_empty() {
                    return Ok(None);
                }
                candidates = next_candidates;
            }
        }

        Ok(None)
    }

    fn resolve_case_insensitive_path(&self, relative: &str) -> Option<PathBuf> {
        let mut current = self.base_dir.clone();
        for component in Path::new(relative).components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    current = current.parent()?.to_path_buf();
                }
                std::path::Component::Normal(name) => {
                    let name = name.to_string_lossy();
                    let direct = current.join(name.as_ref());
                    if direct.exists() {
                        current = direct;
                        continue;
                    }
                    let found = find_case_insensitive_entry(&current, name.as_ref())?;
                    current = found;
                }
                _ => {
                    return None;
                }
            }
        }
        Some(current)
    }

    fn apply_file_map(&self, pattern: &str) -> String {
        for (key, value) in &self.file_map {
            if pattern.starts_with(key) {
                if let Some(star_pos) = pattern.rfind('*') {
                    let foot = &pattern[key.len()..];
                    let prefix = &pattern[..star_pos];
                    return format!("{}{}{}", prefix, value, foot);
                }
            }
        }
        pattern.to_string()
    }

    /// Unload all textures.
    pub fn unload_all(&mut self) {
        self.textures.clear();
    }

    /// Get the number of loaded texture sources.
    pub fn source_count(&self) -> usize {
        self.textures.len()
    }

    /// Get the number of loaded fonts.
    pub fn font_count(&self) -> usize {
        self.fonts.len()
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

fn matches_wildcard(value: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return value == pattern;
    }

    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');
    let parts: Vec<&str> = pattern.split('*').filter(|part| !part.is_empty()).collect();

    if parts.is_empty() {
        return true;
    }

    let mut remainder = value;
    for (index, part) in parts.iter().enumerate() {
        if let Some(found) = remainder.find(part) {
            if index == 0 && !starts_with_wildcard && found != 0 {
                return false;
            }
            remainder = &remainder[found + part.len()..];
        } else {
            return false;
        }
    }

    if !ends_with_wildcard {
        if let Some(last) = parts.last() {
            return value.ends_with(last);
        }
    }

    true
}

fn matches_wildcard_case_insensitive(value: &str, pattern: &str) -> bool {
    let value = value.to_lowercase();
    let pattern = pattern.to_lowercase();
    matches_wildcard(&value, &pattern)
}

fn find_case_insensitive_entry(dir: &Path, name: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let target = name.to_lowercase();
    let mut matches = Vec::new();
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let entry_name = entry.file_name().to_string_lossy().to_lowercase();
        if entry_name == target {
            matches.push(entry.path());
        }
    }
    matches.sort_by(|left, right| {
        left.file_name()
            .map(|value| value.to_string_lossy().to_lowercase())
            .cmp(
                &right
                    .file_name()
                    .map(|value| value.to_string_lossy().to_lowercase()),
            )
    });
    matches.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_source_manager_creation() {
        let manager = SkinSourceManager::new(PathBuf::from("skins/ECFN/play"));
        assert!(manager.textures.is_empty());
    }

    #[test]
    fn test_resolve_case_insensitive_path() {
        let dir = tempdir().unwrap();
        let base = dir.path().join("skin");
        std::fs::create_dir_all(&base).unwrap();
        let file_path = base.join("Graph.png");
        std::fs::write(&file_path, b"test").unwrap();

        let manager = SkinSourceManager::new(base);
        let resolved = manager.resolve_case_insensitive_path("graph.png").unwrap();
        let resolved = resolved.canonicalize().unwrap();
        let expected = file_path.canonicalize().unwrap();
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_resolve_case_insensitive_wildcard() {
        let dir = tempdir().unwrap();
        let base = dir.path().join("skin");
        let gauge_dir = base.join("gauge");
        std::fs::create_dir_all(&gauge_dir).unwrap();
        let file_path = gauge_dir.join("A.PNG");
        std::fs::write(&file_path, b"test").unwrap();

        let manager = SkinSourceManager::new(base);
        let resolved = manager
            .resolve_wildcard_recursive("gauge/*.png")
            .unwrap()
            .unwrap();
        assert_eq!(resolved, file_path);
    }
}
