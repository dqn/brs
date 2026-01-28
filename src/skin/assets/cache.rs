//! Texture cache for skin assets
//!
//! Uses an LRU cache to manage loaded textures with a configurable memory limit.

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lru::LruCache;
use macroquad::prelude::*;

/// Texture identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

/// Texture entry in cache
#[derive(Debug)]
pub struct TextureEntry {
    /// The loaded texture
    pub texture: Texture2D,
    /// Texture width
    pub width: u32,
    /// Texture height
    pub height: u32,
    /// Estimated memory usage in bytes
    pub memory_bytes: usize,
}

impl TextureEntry {
    /// Create a new texture entry
    pub fn new(texture: Texture2D) -> Self {
        let width = texture.width() as u32;
        let height = texture.height() as u32;
        // Estimate 4 bytes per pixel (RGBA)
        let memory_bytes = (width * height * 4) as usize;

        Self {
            texture,
            width,
            height,
            memory_bytes,
        }
    }
}

/// Texture cache with LRU eviction
pub struct TextureCache {
    /// Loaded textures by ID
    textures: HashMap<TextureId, TextureEntry>,
    /// Path to texture ID mapping
    path_to_id: HashMap<PathBuf, TextureId>,
    /// LRU tracking for eviction (just keys, values stored in textures map)
    lru: LruCache<TextureId, ()>,
    /// Next texture ID
    next_id: u32,
    /// Current memory usage in bytes
    current_memory: usize,
    /// Maximum memory usage in bytes
    max_memory: usize,
    /// Base path for relative resource loading
    base_path: PathBuf,
}

impl TextureCache {
    /// Default max cache size (100 MB)
    pub const DEFAULT_MAX_MEMORY: usize = 100 * 1024 * 1024;

    /// Create a new texture cache
    pub fn new(base_path: PathBuf) -> Self {
        Self::with_max_memory(base_path, Self::DEFAULT_MAX_MEMORY)
    }

    /// Create a new texture cache with custom memory limit
    pub fn with_max_memory(base_path: PathBuf, max_memory: usize) -> Self {
        // Use a reasonable default for LRU capacity
        let capacity = NonZeroUsize::new(1000).unwrap();

        Self {
            textures: HashMap::new(),
            path_to_id: HashMap::new(),
            lru: LruCache::new(capacity),
            next_id: 0,
            current_memory: 0,
            max_memory,
            base_path,
        }
    }

    /// Get base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Set base path for relative resource loading
    pub fn set_base_path(&mut self, path: PathBuf) {
        self.base_path = path;
    }

    /// Load a texture from path, returning cached version if available
    pub async fn load(&mut self, path: &str) -> Result<TextureId> {
        let full_path = self.resolve_path(path);

        // Check if already loaded
        if let Some(&id) = self.path_to_id.get(&full_path) {
            // Update LRU
            self.lru.get(&id);
            return Ok(id);
        }

        // Load the texture
        let texture = load_texture(full_path.to_str().unwrap_or(path))
            .await
            .with_context(|| format!("Failed to load texture: {}", path))?;

        // Set texture filtering
        texture.set_filter(FilterMode::Nearest);

        let entry = TextureEntry::new(texture);
        let memory = entry.memory_bytes;

        // Evict if necessary
        self.evict_until_fits(memory);

        // Add to cache
        let id = TextureId(self.next_id);
        self.next_id += 1;

        self.textures.insert(id, entry);
        self.path_to_id.insert(full_path, id);
        self.lru.put(id, ());
        self.current_memory += memory;

        Ok(id)
    }

    /// Load a texture synchronously from image data
    pub fn load_from_image(&mut self, path: &str, image: &image::RgbaImage) -> TextureId {
        let full_path = self.resolve_path(path);

        // Check if already loaded
        if let Some(&id) = self.path_to_id.get(&full_path) {
            self.lru.get(&id);
            return id;
        }

        // Create texture from image data
        let texture =
            Texture2D::from_rgba8(image.width() as u16, image.height() as u16, image.as_raw());
        texture.set_filter(FilterMode::Nearest);

        let entry = TextureEntry::new(texture);
        let memory = entry.memory_bytes;

        // Evict if necessary
        self.evict_until_fits(memory);

        // Add to cache
        let id = TextureId(self.next_id);
        self.next_id += 1;

        self.textures.insert(id, entry);
        self.path_to_id.insert(full_path, id);
        self.lru.put(id, ());
        self.current_memory += memory;

        id
    }

    /// Get a texture by ID
    pub fn get(&mut self, id: TextureId) -> Option<&TextureEntry> {
        // Update LRU
        self.lru.get(&id);
        self.textures.get(&id)
    }

    /// Get texture without updating LRU (for read-only access)
    pub fn peek(&self, id: TextureId) -> Option<&TextureEntry> {
        self.textures.get(&id)
    }

    /// Check if a texture is loaded
    pub fn contains(&self, id: TextureId) -> bool {
        self.textures.contains_key(&id)
    }

    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Get number of loaded textures
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Clear all cached textures
    pub fn clear(&mut self) {
        self.textures.clear();
        self.path_to_id.clear();
        self.lru.clear();
        self.current_memory = 0;
    }

    /// Resolve a path relative to base path
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        }
    }

    /// Evict textures until we have room for new_bytes
    fn evict_until_fits(&mut self, new_bytes: usize) {
        while self.current_memory + new_bytes > self.max_memory {
            if let Some((id, _)) = self.lru.pop_lru() {
                if let Some(entry) = self.textures.remove(&id) {
                    self.current_memory = self.current_memory.saturating_sub(entry.memory_bytes);
                    // Remove from path mapping
                    self.path_to_id.retain(|_, &mut v| v != id);
                }
            } else {
                break;
            }
        }
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new(PathBuf::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_id() {
        let id1 = TextureId(0);
        let id2 = TextureId(0);
        let id3 = TextureId(1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_resolve_path_relative() {
        let cache = TextureCache::new(PathBuf::from("/skins/myskin"));
        let resolved = cache.resolve_path("images/note.png");
        assert_eq!(resolved, PathBuf::from("/skins/myskin/images/note.png"));
    }

    #[test]
    fn test_resolve_path_absolute() {
        let cache = TextureCache::new(PathBuf::from("/skins/myskin"));
        let resolved = cache.resolve_path("/absolute/path.png");
        assert_eq!(resolved, PathBuf::from("/absolute/path.png"));
    }

    #[test]
    fn test_cache_empty() {
        let cache = TextureCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.memory_usage(), 0);
    }

    // Note: Tests that call load_from_image require macroquad graphics context
    // which is not available in unit tests. These should be integration tests.
}
