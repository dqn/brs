// BG image processor for BGA static image management.
//
// Ported from Java BGImageProcessor.java (~139 lines).
// Manages loading and caching of BGA image resources as Bevy textures.

use std::collections::HashMap;
use std::path::Path;

use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Supported image file extensions for BGA.
pub const PIC_EXTENSIONS: &[&str] = &["jpg", "jpeg", "gif", "bmp", "png", "tga"];

/// Manages BGA image loading and texture caching.
///
/// Images are loaded from disk using the `image` crate, converted to
/// Bevy Image assets, and cached by BMP ID for efficient lookup during
/// playback.
pub struct BgImageProcessor {
    /// BMP ID -> Bevy image handle
    cache: HashMap<i32, Handle<Image>>,
    /// BMP ID -> image dimensions (width, height)
    dimensions: HashMap<i32, (f32, f32)>,
}

impl BgImageProcessor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            dimensions: HashMap::new(),
        }
    }

    /// Load an image from disk and cache it under the given BMP ID.
    ///
    /// Returns true if the image was loaded successfully.
    pub fn load(&mut self, bmp_id: i32, path: &Path, images: &mut Assets<Image>) -> bool {
        let img = match image::open(path) {
            Ok(img) => img.into_rgba8(),
            Err(_) => return false,
        };

        let (width, height) = img.dimensions();
        let raw = img.into_raw();

        let mut bevy_image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            raw,
            TextureFormat::Rgba8UnormSrgb,
            default(),
        );
        bevy_image.sampler = ImageSampler::linear();

        let handle = images.add(bevy_image);
        self.dimensions
            .insert(bmp_id, (width as f32, height as f32));
        self.cache.insert(bmp_id, handle);
        true
    }

    /// Get the cached image handle for the given BMP ID.
    pub fn get(&self, bmp_id: i32) -> Option<&Handle<Image>> {
        self.cache.get(&bmp_id)
    }

    /// Get the dimensions of a cached image.
    pub fn dimensions(&self, bmp_id: i32) -> Option<(f32, f32)> {
        self.dimensions.get(&bmp_id).copied()
    }

    /// Returns true if the given BMP ID has a cached image.
    pub fn contains(&self, bmp_id: i32) -> bool {
        self.cache.contains_key(&bmp_id)
    }

    /// Number of cached images.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear all cached images. Handles are released back to Bevy's asset system.
    pub fn dispose(&mut self) {
        self.cache.clear();
        self.dimensions.clear();
    }
}

impl Default for BgImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let proc = BgImageProcessor::new();
        assert!(proc.is_empty());
        assert_eq!(proc.len(), 0);
    }

    #[test]
    fn pic_extensions_contains_common_formats() {
        assert!(PIC_EXTENSIONS.contains(&"png"));
        assert!(PIC_EXTENSIONS.contains(&"jpg"));
        assert!(PIC_EXTENSIONS.contains(&"bmp"));
    }

    #[test]
    fn get_missing_returns_none() {
        let proc = BgImageProcessor::new();
        assert!(proc.get(0).is_none());
        assert!(proc.get(999).is_none());
    }

    #[test]
    fn contains_missing_returns_false() {
        let proc = BgImageProcessor::new();
        assert!(!proc.contains(0));
    }

    #[test]
    fn dispose_clears_cache() {
        let mut proc = BgImageProcessor::new();
        // No images loaded, but dispose should still work
        proc.dispose();
        assert!(proc.is_empty());
    }

    #[test]
    fn dimensions_missing_returns_none() {
        let proc = BgImageProcessor::new();
        assert!(proc.dimensions(42).is_none());
    }
}
