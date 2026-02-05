use macroquad::prelude::*;

use crate::skin::font::FontInfo;
use crate::skin::object::{
    SkinObject, apply_offsets, check_option_visibility, get_timer_elapsed, interpolate_destinations,
};
use crate::skin::{MainState, SkinObjectData, SkinSourceManager, TextDef};

/// Alignment mode for text display.
const ALIGN_LEFT: i32 = 0;
const ALIGN_CENTER: i32 = 1;
const ALIGN_RIGHT: i32 = 2;

/// Skin object that renders text using bitmap fonts.
pub struct TextObject {
    /// Object data from skin definition.
    pub data: SkinObjectData,
    /// Text definition.
    pub text_def: Option<TextDef>,
    /// Whether the object is prepared.
    prepared: bool,
}

impl TextObject {
    /// Create a new text object.
    pub fn new(data: SkinObjectData, text_def: Option<TextDef>) -> Self {
        Self {
            data,
            text_def,
            prepared: false,
        }
    }

    /// Calculate text width.
    fn calculate_text_width(&self, text: &str, font_info: &FontInfo, scale: f32) -> f32 {
        let mut width = 0.0;
        for c in text.chars() {
            if let Some(glyph) = font_info.glyphs.get(&c) {
                width += glyph.xadvance as f32 * scale;
            }
        }
        width
    }

    /// Draw text using bitmap font.
    #[allow(clippy::too_many_arguments)]
    fn draw_text(
        &self,
        text: &str,
        x: f32,
        y: f32,
        font_info: &FontInfo,
        texture: &Texture2D,
        scale: f32,
        color: Color,
        dst_w: f32,
    ) {
        let Some(ref text_def) = self.text_def else {
            return;
        };

        // Calculate text width for alignment
        let text_width = self.calculate_text_width(text, font_info, scale);

        // Determine starting X position based on alignment
        let start_x = match text_def.align {
            ALIGN_CENTER => x + (dst_w - text_width) / 2.0,
            ALIGN_RIGHT => x + dst_w - text_width,
            ALIGN_LEFT => x,
            _ => x,
        };

        let mut cursor_x = start_x;
        let base_y = y;

        for c in text.chars() {
            if let Some(glyph) = font_info.glyphs.get(&c) {
                // Calculate source rectangle
                let src_rect = Rect::new(
                    glyph.x as f32,
                    glyph.y as f32,
                    glyph.width as f32,
                    glyph.height as f32,
                );

                // Calculate destination position with offsets
                let dest_x = cursor_x + glyph.xoffset as f32 * scale;
                let dest_y = base_y + glyph.yoffset as f32 * scale;
                let dest_w = glyph.width as f32 * scale;
                let dest_h = glyph.height as f32 * scale;

                draw_texture_ex(
                    texture,
                    dest_x,
                    dest_y,
                    color,
                    DrawTextureParams {
                        dest_size: Some(vec2(dest_w, dest_h)),
                        source: Some(src_rect),
                        rotation: 0.0,
                        flip_x: false,
                        flip_y: false,
                        pivot: None,
                    },
                );

                cursor_x += glyph.xadvance as f32 * scale;
            }
        }
    }
}

impl SkinObject for TextObject {
    fn prepare(&mut self, _sources: &SkinSourceManager) {
        self.prepared = true;
    }

    fn draw(&self, state: &MainState, sources: &SkinSourceManager, now_time_us: i64) {
        if !self.is_visible(state) {
            return;
        }

        let Some(ref text_def) = self.text_def else {
            return;
        };

        // Get font info and texture
        let Some((font_info, texture)) = sources.get_font_info(text_def.font) else {
            return;
        };

        // Calculate elapsed time
        let elapsed_us = get_timer_elapsed(self.data.timer, state, now_time_us);
        if elapsed_us < 0 {
            return; // Timer not active
        }

        // Get interpolated destination
        let elapsed_ms = elapsed_us / 1000;
        let Some(dst) = interpolate_destinations(&self.data.dst, elapsed_ms, self.data.loop_count)
        else {
            return;
        };
        let dst = apply_offsets(dst, &self.data, state);

        // Skip if invisible
        if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }

        // Get text from state
        let text = state.text(text_def.ref_id);
        if text.is_empty() {
            return;
        }

        // Calculate scale based on destination height and font size
        let scale = if font_info.line_height > 0 && text_def.size > 0 {
            dst.h / font_info.line_height as f32
        } else {
            1.0
        };

        let color = Color::new(dst.r / 255.0, dst.g / 255.0, dst.b / 255.0, dst.a / 255.0);

        self.draw_text(
            &text,
            dst.x,
            dst.y,
            font_info,
            &texture.texture,
            scale,
            color,
            dst.w,
        );
    }

    fn is_visible(&self, state: &MainState) -> bool {
        check_option_visibility(&self.data.op, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::Destination;

    #[test]
    fn test_text_object_creation() {
        let data = SkinObjectData {
            id: "test".to_string(),
            dst: vec![Destination {
                x: 100.0,
                y: 100.0,
                w: 200.0,
                h: 32.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let obj = TextObject::new(data, None);
        assert!(!obj.prepared);
    }
}
