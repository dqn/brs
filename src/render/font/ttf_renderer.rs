use ab_glyph::{Font, FontRef, ScaleFont};
use anyhow::{Result, anyhow};

/// Rasterizes TrueType font glyphs to RGBA bitmaps.
pub struct TtfRenderer {
    font_data: Vec<u8>,
}

impl TtfRenderer {
    /// Create a new TtfRenderer from font file data.
    pub fn new(font_data: Vec<u8>) -> Result<Self> {
        // Validate by attempting to parse.
        FontRef::try_from_slice(&font_data)
            .map_err(|e| anyhow!("failed to parse font data: {e}"))?;
        Ok(Self { font_data })
    }

    /// Rasterize a string into an RGBA image buffer.
    /// Returns (width, height, rgba_data).
    pub fn rasterize(&self, text: &str, size: f32) -> Result<(u32, u32, Vec<u8>)> {
        let font =
            FontRef::try_from_slice(&self.font_data).map_err(|e| anyhow!("font error: {e}"))?;
        let scaled = font.as_scaled(size);

        if text.is_empty() {
            return Ok((0, 0, Vec::new()));
        }

        // Calculate layout.
        let mut width = 0.0f32;
        let mut prev_glyph_id = None;

        for ch in text.chars() {
            let glyph_id = scaled.glyph_id(ch);
            if let Some(prev) = prev_glyph_id {
                width += scaled.kern(prev, glyph_id);
            }
            width += scaled.h_advance(glyph_id);
            prev_glyph_id = Some(glyph_id);
        }

        let height = scaled.height() + scaled.descent().abs();
        let img_width = width.ceil() as u32;
        let img_height = height.ceil() as u32;

        if img_width == 0 || img_height == 0 {
            return Ok((0, 0, Vec::new()));
        }

        let mut buffer = vec![0u8; (img_width * img_height * 4) as usize];
        let mut cursor_x = 0.0f32;
        prev_glyph_id = None;

        for ch in text.chars() {
            let glyph_id = scaled.glyph_id(ch);
            if let Some(prev) = prev_glyph_id {
                cursor_x += scaled.kern(prev, glyph_id);
            }

            let glyph = glyph_id.with_scale(size);
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                let gx = cursor_x + bounds.min.x;
                let gy = scaled.ascent() + bounds.min.y;

                outlined.draw(|x, y, coverage| {
                    let px = (gx as i32 + x as i32) as u32;
                    let py = (gy as i32 + y as i32) as u32;
                    if px < img_width && py < img_height {
                        let idx = ((py * img_width + px) * 4) as usize;
                        let alpha = (coverage * 255.0) as u8;
                        // Premultiplied white with alpha.
                        buffer[idx] = 255;
                        buffer[idx + 1] = 255;
                        buffer[idx + 2] = 255;
                        buffer[idx + 3] = buffer[idx + 3].saturating_add(alpha);
                    }
                });
            }

            cursor_x += scaled.h_advance(glyph_id);
            prev_glyph_id = Some(glyph_id);
        }

        Ok((img_width, img_height, buffer))
    }
}
