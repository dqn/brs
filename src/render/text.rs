use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::render::font::fnt_parser::BitmapFont;
use crate::render::font::ttf_renderer::TtfRenderer;
use crate::traits::render::{FontId, TextureId};

/// Loaded font data (either TrueType or bitmap).
pub enum FontData {
    TrueType(TtfRenderer),
    Bitmap(BitmapFontData),
}

/// Bitmap font with associated page textures.
pub struct BitmapFontData {
    pub font: BitmapFont,
    /// GPU texture IDs for each page, indexed by page number.
    pub page_textures: Vec<TextureId>,
}

/// Manages font loading and text rendering.
pub struct TextManager {
    fonts: HashMap<FontId, FontData>,
    next_id: u64,
}

impl Default for TextManager {
    fn default() -> Self {
        Self {
            fonts: HashMap::new(),
            next_id: 1,
        }
    }
}

impl TextManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a TrueType font file (.ttf/.otf).
    pub fn load_ttf(&mut self, path: &Path) -> Result<FontId> {
        let data = std::fs::read(path)
            .map_err(|e| anyhow!("failed to read font {}: {}", path.display(), e))?;
        let font_data = FontData::TrueType(TtfRenderer::new(data)?);
        let id = FontId(self.next_id);
        self.next_id += 1;
        self.fonts.insert(id, font_data);
        Ok(id)
    }

    /// Register a bitmap font with its pre-loaded page textures.
    pub fn register_bitmap_font(
        &mut self,
        font: BitmapFont,
        page_textures: Vec<TextureId>,
    ) -> FontId {
        let id = FontId(self.next_id);
        self.next_id += 1;
        self.fonts.insert(
            id,
            FontData::Bitmap(BitmapFontData {
                font,
                page_textures,
            }),
        );
        id
    }

    /// Get the loaded font data.
    pub fn get(&self, id: FontId) -> Option<&FontData> {
        self.fonts.get(&id)
    }
}
