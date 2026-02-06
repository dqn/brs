use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::render::font::fnt_parser::{self, BitmapFont};
use crate::render::font::ttf_renderer::TtfRenderer;
use crate::traits::render::FontId;

/// Loaded font data (either TrueType or bitmap).
pub enum FontData {
    TrueType(TtfRenderer),
    Bitmap(BitmapFont),
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

    /// Load a font file. Supports .ttf/.otf (TrueType) and .fnt (BMFont text format).
    pub fn load_font(&mut self, path: &Path) -> Result<FontId> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let font_data = match ext.as_str() {
            "ttf" | "otf" => {
                let data = std::fs::read(path)
                    .map_err(|e| anyhow!("failed to read font {}: {}", path.display(), e))?;
                FontData::TrueType(TtfRenderer::new(data)?)
            }
            "fnt" => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| anyhow!("failed to read FNT {}: {}", path.display(), e))?;
                FontData::Bitmap(fnt_parser::parse_fnt(&content)?)
            }
            _ => return Err(anyhow!("unsupported font format: .{ext}")),
        };

        let id = FontId(self.next_id);
        self.next_id += 1;
        self.fonts.insert(id, font_data);
        Ok(id)
    }

    /// Get the loaded font data.
    pub fn get(&self, id: FontId) -> Option<&FontData> {
        self.fonts.get(&id)
    }
}
