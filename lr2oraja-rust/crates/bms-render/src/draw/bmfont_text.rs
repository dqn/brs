// BMFont text layout engine.
//
// Computes glyph positions for rendering bitmap font text.
// Handles alignment, overflow (shrink/truncate), and kerning.

use bms_skin::bmfont::BmFont;
use bms_skin::skin_object::Rect;
use bms_skin::skin_text::{TextAlign, TextOverflow};

/// A single glyph draw command produced by layout.
#[derive(Debug, Clone)]
pub struct GlyphDrawCommand {
    /// Texture page index.
    pub page: u32,
    /// Source region in the texture atlas.
    pub src_x: f32,
    pub src_y: f32,
    pub src_w: f32,
    pub src_h: f32,
    /// Destination position and size in skin coordinates.
    pub dst_x: f32,
    pub dst_y: f32,
    pub dst_w: f32,
    pub dst_h: f32,
}

/// Lays out text using a BMFont and returns glyph draw commands.
///
/// `target_size`: desired font size (scaling is applied relative to font.size).
/// `region`: destination rectangle in skin coordinates.
/// `align`: text alignment within the region.
/// `overflow`: how to handle text wider than the region.
pub fn layout_bmfont_text(
    text: &str,
    font: &BmFont,
    target_size: f32,
    region: &Rect,
    align: TextAlign,
    overflow: TextOverflow,
) -> Vec<GlyphDrawCommand> {
    if text.is_empty() || font.size == 0.0 {
        return Vec::new();
    }

    let base_scale = target_size / font.size;

    // First pass: collect glyph info and compute total width
    let mut glyph_infos: Vec<(u32, &bms_skin::bmfont::BmGlyph, f32)> = Vec::new(); // (codepoint, glyph, kerning)
    let mut total_advance = 0.0_f32;
    let mut prev_char: Option<u32> = None;

    for ch in text.chars() {
        let cp = ch as u32;
        let glyph = match font.glyphs.get(&cp) {
            Some(g) => g,
            None => continue, // Skip unknown characters
        };

        let kern = if let Some(prev) = prev_char {
            font.kernings.get(&(prev, cp)).copied().unwrap_or(0) as f32
        } else {
            0.0
        };

        total_advance += (glyph.xadvance + kern) * base_scale;
        glyph_infos.push((cp, glyph, kern));
        prev_char = Some(cp);
    }

    if glyph_infos.is_empty() {
        return Vec::new();
    }

    // Determine final scale based on overflow mode
    let scale = match overflow {
        TextOverflow::Shrink => {
            if total_advance > region.w && region.w > 0.0 {
                base_scale * (region.w / total_advance)
            } else {
                base_scale
            }
        }
        _ => base_scale,
    };

    // Recompute total advance with final scale if shrunk
    let final_total = if (scale - base_scale).abs() > f32::EPSILON {
        total_advance * (scale / base_scale)
    } else {
        total_advance
    };

    // Alignment offset
    let align_offset_x = match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => (region.w - final_total) * 0.5,
        TextAlign::Right => region.w - final_total,
    };

    // Second pass: generate draw commands
    let mut commands = Vec::with_capacity(glyph_infos.len());
    let mut cursor_x = 0.0_f32;

    for (_cp, glyph, kern) in &glyph_infos {
        cursor_x += kern * scale;

        let dst_x = region.x + align_offset_x + cursor_x + glyph.xoffset * scale;
        let dst_y = region.y + glyph.yoffset * scale;
        let dst_w = glyph.width * scale;
        let dst_h = glyph.height * scale;

        // Truncate: skip glyphs past region width
        if overflow == TextOverflow::Truncate
            && (cursor_x + glyph.xoffset * scale + dst_w) > region.w
        {
            break;
        }

        if glyph.width > 0.0 && glyph.height > 0.0 {
            commands.push(GlyphDrawCommand {
                page: glyph.page,
                src_x: glyph.x,
                src_y: glyph.y,
                src_w: glyph.width,
                src_h: glyph.height,
                dst_x,
                dst_y,
                dst_w,
                dst_h,
            });
        }

        cursor_x += glyph.xadvance * scale;
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_font() -> BmFont {
        let mut glyphs = HashMap::new();
        // 'A' = 65
        glyphs.insert(
            65,
            bms_skin::bmfont::BmGlyph {
                id: 65,
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 24.0,
                xoffset: 1.0,
                yoffset: 2.0,
                xadvance: 22.0,
                page: 0,
            },
        );
        // 'B' = 66
        glyphs.insert(
            66,
            bms_skin::bmfont::BmGlyph {
                id: 66,
                x: 21.0,
                y: 0.0,
                width: 18.0,
                height: 24.0,
                xoffset: 2.0,
                yoffset: 2.0,
                xadvance: 20.0,
                page: 0,
            },
        );
        // ' ' = 32
        glyphs.insert(
            32,
            bms_skin::bmfont::BmGlyph {
                id: 32,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                xoffset: 0.0,
                yoffset: 0.0,
                xadvance: 10.0,
                page: 0,
            },
        );

        BmFont {
            size: 24.0,
            line_height: 28.0,
            base: 22.0,
            scale_w: 256.0,
            scale_h: 256.0,
            pages: vec!["page0.png".to_string()],
            glyphs,
            kernings: HashMap::new(),
        }
    }

    #[test]
    fn empty_text_returns_empty() {
        let font = make_test_font();
        let region = Rect::new(0.0, 0.0, 200.0, 30.0);
        let cmds = layout_bmfont_text(
            "",
            &font,
            24.0,
            &region,
            TextAlign::Left,
            TextOverflow::Overflow,
        );
        assert!(cmds.is_empty());
    }

    #[test]
    fn single_char_left_aligned() {
        let font = make_test_font();
        let region = Rect::new(10.0, 20.0, 200.0, 30.0);
        let cmds = layout_bmfont_text(
            "A",
            &font,
            24.0,
            &region,
            TextAlign::Left,
            TextOverflow::Overflow,
        );
        assert_eq!(cmds.len(), 1);
        // dst_x = region.x + 0 (align) + 0 (cursor) + 1 (xoffset) * 1.0 (scale) = 11
        assert!((cmds[0].dst_x - 11.0).abs() < 0.01);
        // dst_y = region.y + 2 (yoffset) * 1.0 = 22
        assert!((cmds[0].dst_y - 22.0).abs() < 0.01);
        assert_eq!(cmds[0].src_x, 0.0);
        assert_eq!(cmds[0].src_w, 20.0);
    }

    #[test]
    fn center_alignment() {
        let font = make_test_font();
        let region = Rect::new(0.0, 0.0, 200.0, 30.0);
        // "A" advance = 22, region width = 200
        // align_offset = (200 - 22) * 0.5 = 89
        let cmds = layout_bmfont_text(
            "A",
            &font,
            24.0,
            &region,
            TextAlign::Center,
            TextOverflow::Overflow,
        );
        assert_eq!(cmds.len(), 1);
        // dst_x = 0 + 89 + 0 + 1 = 90
        assert!((cmds[0].dst_x - 90.0).abs() < 0.01);
    }

    #[test]
    fn right_alignment() {
        let font = make_test_font();
        let region = Rect::new(0.0, 0.0, 200.0, 30.0);
        // "A" advance = 22
        // align_offset = 200 - 22 = 178
        let cmds = layout_bmfont_text(
            "A",
            &font,
            24.0,
            &region,
            TextAlign::Right,
            TextOverflow::Overflow,
        );
        assert_eq!(cmds.len(), 1);
        assert!((cmds[0].dst_x - 179.0).abs() < 0.01); // 178 + 1 xoffset
    }

    #[test]
    fn scale_factor() {
        let font = make_test_font();
        let region = Rect::new(0.0, 0.0, 400.0, 60.0);
        // target_size = 48, font.size = 24 → scale = 2.0
        let cmds = layout_bmfont_text(
            "A",
            &font,
            48.0,
            &region,
            TextAlign::Left,
            TextOverflow::Overflow,
        );
        assert_eq!(cmds.len(), 1);
        // dst_w = 20 * 2 = 40
        assert!((cmds[0].dst_w - 40.0).abs() < 0.01);
        assert!((cmds[0].dst_h - 48.0).abs() < 0.01);
    }

    #[test]
    fn overflow_shrink() {
        let font = make_test_font();
        // Region only 20px wide, "AB" advance = 22 + 20 = 42
        let region = Rect::new(0.0, 0.0, 21.0, 30.0);
        let cmds = layout_bmfont_text(
            "AB",
            &font,
            24.0,
            &region,
            TextAlign::Left,
            TextOverflow::Shrink,
        );
        assert_eq!(cmds.len(), 2);
        // Scale should be shrunk: 21/42 = 0.5
        // 'A' dst_w = 20 * 0.5 = 10
        assert!((cmds[0].dst_w - 10.0).abs() < 0.01);
    }

    #[test]
    fn overflow_truncate() {
        let font = make_test_font();
        // Region 25px wide, "AB" advance = 22 + 20 = 42
        // 'A' takes up to xoffset+width = 1+20 = 21, fits.
        // 'B' would start at cursor_x=22, xoffset=2, +width=18 → 42 > 25
        let region = Rect::new(0.0, 0.0, 25.0, 30.0);
        let cmds = layout_bmfont_text(
            "AB",
            &font,
            24.0,
            &region,
            TextAlign::Left,
            TextOverflow::Truncate,
        );
        assert_eq!(cmds.len(), 1); // Only 'A' fits
    }

    #[test]
    fn space_char_not_drawn() {
        let font = make_test_font();
        let region = Rect::new(0.0, 0.0, 200.0, 30.0);
        let cmds = layout_bmfont_text(
            " A",
            &font,
            24.0,
            &region,
            TextAlign::Left,
            TextOverflow::Overflow,
        );
        // Space has width=0, so not drawn. Only 'A' is drawn.
        assert_eq!(cmds.len(), 1);
        // 'A' should be offset by space advance (10)
        // dst_x = 0 + 0 + 10 + 1 = 11
        assert!((cmds[0].dst_x - 11.0).abs() < 0.01);
    }

    #[test]
    fn kerning_applied() {
        let mut font = make_test_font();
        font.kernings.insert((65, 66), -3); // A→B kern = -3

        let region = Rect::new(0.0, 0.0, 200.0, 30.0);
        let cmds = layout_bmfont_text(
            "AB",
            &font,
            24.0,
            &region,
            TextAlign::Left,
            TextOverflow::Overflow,
        );
        assert_eq!(cmds.len(), 2);
        // 'A' at cursor_x=0: dst_x = 0 + 1 = 1
        assert!((cmds[0].dst_x - 1.0).abs() < 0.01);
        // After 'A': cursor = 22
        // 'B' kern = -3, cursor = 22 + (-3) = 19
        // dst_x = 19 + 2 (xoffset) = 21
        assert!((cmds[1].dst_x - 21.0).abs() < 0.01);
    }
}
