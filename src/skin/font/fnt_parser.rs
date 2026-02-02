use std::collections::HashMap;

use anyhow::{Result, anyhow};

/// Glyph information for a single character.
#[derive(Debug, Clone, Default)]
pub struct GlyphInfo {
    /// X position in texture.
    pub x: i32,
    /// Y position in texture.
    pub y: i32,
    /// Width of glyph.
    pub width: i32,
    /// Height of glyph.
    pub height: i32,
    /// X offset for rendering.
    pub xoffset: i32,
    /// Y offset for rendering.
    pub yoffset: i32,
    /// Advance width (how much to move cursor after drawing).
    pub xadvance: i32,
    /// Texture page index.
    pub page: i32,
}

/// Font information from .fnt file.
#[derive(Debug, Clone, Default)]
pub struct FontInfo {
    /// Character to glyph mapping.
    pub glyphs: HashMap<char, GlyphInfo>,
    /// Texture file name (page 0).
    pub texture_file: String,
    /// Line height.
    pub line_height: i32,
    /// Font size.
    pub size: i32,
    /// Base line.
    pub base: i32,
}

/// Parse a BMFont text format .fnt file.
pub fn parse_fnt(content: &str) -> Result<FontInfo> {
    let mut font_info = FontInfo::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "info" => {
                // Parse font info line
                for part in &parts[1..] {
                    if let Some((key, value)) = parse_key_value(part) {
                        if key == "size" {
                            font_info.size = value.parse().unwrap_or(0);
                        }
                    }
                }
            }
            "common" => {
                // Parse common line
                for part in &parts[1..] {
                    if let Some((key, value)) = parse_key_value(part) {
                        match key {
                            "lineHeight" => font_info.line_height = value.parse().unwrap_or(0),
                            "base" => font_info.base = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }
            }
            "page" => {
                // Parse page line
                for part in &parts[1..] {
                    if let Some((key, value)) = parse_key_value(part) {
                        if key == "file" {
                            // Remove quotes from filename
                            font_info.texture_file = value.trim_matches('"').to_string();
                        }
                    }
                }
            }
            "char" => {
                // Parse character definition
                let mut glyph = GlyphInfo::default();
                let mut char_id: Option<u32> = None;

                for part in &parts[1..] {
                    if let Some((key, value)) = parse_key_value(part) {
                        match key {
                            "id" => char_id = value.parse().ok(),
                            "x" => glyph.x = value.parse().unwrap_or(0),
                            "y" => glyph.y = value.parse().unwrap_or(0),
                            "width" => glyph.width = value.parse().unwrap_or(0),
                            "height" => glyph.height = value.parse().unwrap_or(0),
                            "xoffset" => glyph.xoffset = value.parse().unwrap_or(0),
                            "yoffset" => glyph.yoffset = value.parse().unwrap_or(0),
                            "xadvance" => glyph.xadvance = value.parse().unwrap_or(0),
                            "page" => glyph.page = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }

                if let Some(id) = char_id {
                    if let Some(c) = char::from_u32(id) {
                        font_info.glyphs.insert(c, glyph);
                    }
                }
            }
            _ => {}
        }
    }

    if font_info.texture_file.is_empty() {
        return Err(anyhow!("No texture file specified in font"));
    }

    Ok(font_info)
}

/// Parse a key=value pair.
fn parse_key_value(s: &str) -> Option<(&str, &str)> {
    s.split_once('=')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fnt() {
        let content = r#"
info face="Arial" size=32 bold=0 italic=0
common lineHeight=36 base=28 scaleW=256 scaleH=256 pages=1
page id=0 file="font.png"
char id=65 x=0 y=0 width=20 height=32 xoffset=0 yoffset=0 xadvance=22 page=0
char id=66 x=20 y=0 width=18 height=32 xoffset=1 yoffset=0 xadvance=20 page=0
"#;

        let font_info = parse_fnt(content).unwrap();

        assert_eq!(font_info.size, 32);
        assert_eq!(font_info.line_height, 36);
        assert_eq!(font_info.base, 28);
        assert_eq!(font_info.texture_file, "font.png");

        // Check glyph for 'A' (id=65)
        let glyph_a = font_info.glyphs.get(&'A').unwrap();
        assert_eq!(glyph_a.x, 0);
        assert_eq!(glyph_a.y, 0);
        assert_eq!(glyph_a.width, 20);
        assert_eq!(glyph_a.height, 32);
        assert_eq!(glyph_a.xadvance, 22);

        // Check glyph for 'B' (id=66)
        let glyph_b = font_info.glyphs.get(&'B').unwrap();
        assert_eq!(glyph_b.x, 20);
        assert_eq!(glyph_b.width, 18);
    }

    #[test]
    fn test_parse_key_value() {
        assert_eq!(parse_key_value("id=65"), Some(("id", "65")));
        assert_eq!(
            parse_key_value("file=\"font.png\""),
            Some(("file", "\"font.png\""))
        );
        assert_eq!(parse_key_value("invalid"), None);
    }
}
