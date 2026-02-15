// Font map: manages loaded font resources for the skin renderer.
//
// Maps font paths to Bevy font handles (TTF) or parsed BMFont data
// with page textures (bitmap fonts).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bms_skin::bmfont::BmFont;
use bms_skin::skin::Skin;

/// Entry for a loaded TrueType font.
#[derive(Debug, Clone)]
pub struct TtfFontEntry {
    pub handle: Handle<Font>,
}

/// Entry for a loaded BMFont bitmap font.
#[derive(Debug, Clone)]
pub struct BmFontEntry {
    /// Parsed BMFont data (glyphs, kerning, etc.).
    pub data: BmFont,
    /// Bevy image handles for each page texture.
    pub page_textures: Vec<Handle<Image>>,
    /// Dimensions (width, height) for each page texture.
    pub page_dimensions: Vec<(f32, f32)>,
    /// Bitmap type: 0=standard, 1=distance_field, 2=colored_distance_field.
    pub bitmap_type: i32,
}

/// Resource that maps font paths to loaded font entries.
#[derive(Default, Resource)]
pub struct FontMap {
    pub ttf_fonts: HashMap<String, TtfFontEntry>,
    pub bitmap_fonts: HashMap<String, BmFontEntry>,
}

impl FontMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a TTF font entry.
    pub fn insert_ttf(&mut self, path: String, handle: Handle<Font>) {
        self.ttf_fonts.insert(path, TtfFontEntry { handle });
    }

    /// Insert a BMFont entry.
    pub fn insert_bitmap(
        &mut self,
        path: String,
        data: BmFont,
        page_textures: Vec<Handle<Image>>,
        page_dimensions: Vec<(f32, f32)>,
        bitmap_type: i32,
    ) {
        self.bitmap_fonts.insert(
            path,
            BmFontEntry {
                data,
                page_textures,
                page_dimensions,
                bitmap_type,
            },
        );
    }

    /// Get a TTF font entry by path.
    pub fn get_ttf(&self, path: &str) -> Option<&TtfFontEntry> {
        self.ttf_fonts.get(path)
    }

    /// Get a BMFont entry by path.
    pub fn get_bitmap(&self, path: &str) -> Option<&BmFontEntry> {
        self.bitmap_fonts.get(path)
    }

    /// Loads LR2 bitmap fonts from a skin into this FontMap.
    ///
    /// Reads texture images from disk and uploads them as Bevy Image assets.
    /// Updates the BmFont's scale_w/scale_h to match actual texture dimensions.
    pub fn load_lr2_fonts(&mut self, skin: &Skin, images: &mut Assets<Image>) {
        for font_data in &skin.lr2_fonts {
            let mut bmfont = font_data.bmfont.clone();
            let mut page_textures = Vec::new();
            let mut page_dimensions = Vec::new();

            // Collect texture paths indexed by page ID.
            let page_count = bmfont.pages.len();
            page_textures.resize(page_count, Handle::default());
            page_dimensions.resize(page_count, (1.0_f32, 1.0_f32));

            for &(tex_id, ref path) in &font_data.texture_paths {
                let idx = tex_id as usize;
                if idx >= page_count {
                    continue;
                }

                let Ok(dyn_image) = image::open(path) else {
                    continue;
                };
                let rgba = dyn_image.to_rgba8();
                let (w, h) = (rgba.width(), rgba.height());

                let bevy_image = Image::new(
                    Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    rgba.into_raw(),
                    TextureFormat::Rgba8UnormSrgb,
                    default(),
                );
                let handle = images.add(bevy_image);
                page_textures[idx] = handle;
                page_dimensions[idx] = (w as f32, h as f32);
            }

            // Update BmFont scale_w/scale_h from the first texture page dimensions
            // (used for UV coordinate calculation in layout_bmfont_text).
            if let Some(&(w, h)) = page_dimensions.first() {
                if w > 1.0 {
                    bmfont.scale_w = w;
                }
                if h > 1.0 {
                    bmfont.scale_h = h;
                }
            }

            self.insert_bitmap(
                font_data.key.clone(),
                bmfont,
                page_textures,
                page_dimensions,
                0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let map = FontMap::new();
        assert!(map.ttf_fonts.is_empty());
        assert!(map.bitmap_fonts.is_empty());
    }

    #[test]
    fn test_insert_and_get_ttf() {
        let mut map = FontMap::new();
        map.insert_ttf("test.ttf".to_string(), Handle::default());
        assert!(map.get_ttf("test.ttf").is_some());
        assert!(map.get_ttf("other.ttf").is_none());
    }

    #[test]
    fn test_insert_and_get_bitmap() {
        let mut map = FontMap::new();
        let data = BmFont {
            size: 16.0,
            line_height: 20.0,
            base: 16.0,
            scale_w: 256.0,
            scale_h: 256.0,
            pages: vec!["page0.png".to_string()],
            glyphs: Default::default(),
            kernings: Default::default(),
        };
        map.insert_bitmap(
            "test.fnt".to_string(),
            data,
            vec![Handle::default()],
            vec![(256.0, 256.0)],
            0,
        );
        let entry = map.get_bitmap("test.fnt").unwrap();
        assert_eq!(entry.data.size, 16.0);
        assert_eq!(entry.bitmap_type, 0);
        assert_eq!(entry.page_textures.len(), 1);
    }
}
