// Font map: manages loaded font resources for the skin renderer.
//
// Maps font paths to Bevy font handles (TTF) or parsed BMFont data
// with page textures (bitmap fonts).

use std::collections::HashMap;

use bevy::prelude::*;
use bms_skin::bmfont::BmFont;

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
