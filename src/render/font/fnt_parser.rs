use std::collections::HashMap;

use anyhow::{Result, anyhow};

/// A single character glyph in a bitmap font.
#[derive(Debug, Clone)]
pub struct BitmapGlyph {
    /// Character code point.
    pub id: u32,
    /// X position in the texture page.
    pub x: u32,
    /// Y position in the texture page.
    pub y: u32,
    /// Width of the glyph in the texture.
    pub width: u32,
    /// Height of the glyph in the texture.
    pub height: u32,
    /// X offset for rendering.
    pub xoffset: i32,
    /// Y offset for rendering.
    pub yoffset: i32,
    /// Advance width after rendering this glyph.
    pub xadvance: i32,
    /// Texture page index.
    pub page: u32,
}

/// Parsed bitmap font data (AngelCode BMFont text format).
#[derive(Debug, Clone)]
pub struct BitmapFont {
    /// Font face name.
    pub face: String,
    /// Font size.
    pub size: u32,
    /// Line height.
    pub line_height: u32,
    /// Base offset from top of line.
    pub base: u32,
    /// Texture page file names.
    pub pages: Vec<String>,
    /// Character glyphs indexed by code point.
    pub glyphs: HashMap<u32, BitmapGlyph>,
    /// Kerning pairs: (first, second) -> amount.
    pub kernings: HashMap<(u32, u32), i32>,
}

/// Parse a BMFont text format (.fnt) file.
pub fn parse_fnt(content: &str) -> Result<BitmapFont> {
    let mut face = String::new();
    let mut size = 0u32;
    let mut line_height = 0u32;
    let mut base = 0u32;
    let mut pages = Vec::new();
    let mut glyphs = HashMap::new();
    let mut kernings = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("info ") {
            face = extract_quoted(rest, "face").unwrap_or_default();
            size = extract_value(rest, "size").unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("common ") {
            line_height = extract_value(rest, "lineHeight").unwrap_or(0);
            base = extract_value(rest, "base").unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("page ") {
            let file = extract_quoted(rest, "file").unwrap_or_default();
            pages.push(file);
        } else if let Some(rest) = line.strip_prefix("char ") {
            let glyph = BitmapGlyph {
                id: extract_value(rest, "id").unwrap_or(0),
                x: extract_value(rest, "x").unwrap_or(0),
                y: extract_value(rest, "y").unwrap_or(0),
                width: extract_value(rest, "width").unwrap_or(0),
                height: extract_value(rest, "height").unwrap_or(0),
                xoffset: extract_value_signed(rest, "xoffset").unwrap_or(0),
                yoffset: extract_value_signed(rest, "yoffset").unwrap_or(0),
                xadvance: extract_value_signed(rest, "xadvance").unwrap_or(0),
                page: extract_value(rest, "page").unwrap_or(0),
            };
            glyphs.insert(glyph.id, glyph);
        } else if let Some(rest) = line.strip_prefix("kerning ") {
            let first: u32 = extract_value(rest, "first").unwrap_or(0);
            let second: u32 = extract_value(rest, "second").unwrap_or(0);
            let amount: i32 = extract_value_signed(rest, "amount").unwrap_or(0);
            kernings.insert((first, second), amount);
        }
        // Skip "chars count=N" and other unknown lines.
    }

    if pages.is_empty() {
        return Err(anyhow!("no page definitions found in FNT file"));
    }

    Ok(BitmapFont {
        face,
        size,
        line_height,
        base,
        pages,
        glyphs,
        kernings,
    })
}

/// Extract a quoted value like `face="Arial"`.
fn extract_quoted(s: &str, key: &str) -> Option<String> {
    let pattern = format!("{key}=\"");
    let start = s.find(&pattern)? + pattern.len();
    let end = s[start..].find('"')? + start;
    Some(s[start..end].to_string())
}

/// Extract an unsigned integer value like `size=32`.
fn extract_value<T: std::str::FromStr>(s: &str, key: &str) -> Option<T> {
    let pattern = format!("{key}=");
    let start = s.find(&pattern)? + pattern.len();
    let rest = &s[start..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Extract a signed integer value like `xoffset=-5`.
fn extract_value_signed(s: &str, key: &str) -> Option<i32> {
    let pattern = format!("{key}=");
    let start = s.find(&pattern)? + pattern.len();
    let rest = &s[start..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FNT: &str = r#"info face="TestFont" size=32 bold=0 italic=0
common lineHeight=36 base=28 scaleW=256 scaleH=256 pages=1
page id=0 file="test_font.png"
chars count=3
char id=65 x=0 y=0 width=20 height=30 xoffset=1 yoffset=2 xadvance=22 page=0
char id=66 x=20 y=0 width=18 height=30 xoffset=1 yoffset=2 xadvance=20 page=0
char id=67 x=38 y=0 width=19 height=30 xoffset=1 yoffset=2 xadvance=21 page=0
kerning first=65 second=66 amount=-1
kerning first=66 second=67 amount=-2
"#;

    #[test]
    fn test_parse_fnt_info() {
        let font = parse_fnt(SAMPLE_FNT).unwrap();
        assert_eq!(font.face, "TestFont");
        assert_eq!(font.size, 32);
    }

    #[test]
    fn test_parse_fnt_common() {
        let font = parse_fnt(SAMPLE_FNT).unwrap();
        assert_eq!(font.line_height, 36);
        assert_eq!(font.base, 28);
    }

    #[test]
    fn test_parse_fnt_pages() {
        let font = parse_fnt(SAMPLE_FNT).unwrap();
        assert_eq!(font.pages.len(), 1);
        assert_eq!(font.pages[0], "test_font.png");
    }

    #[test]
    fn test_parse_fnt_glyphs() {
        let font = parse_fnt(SAMPLE_FNT).unwrap();
        assert_eq!(font.glyphs.len(), 3);

        let a = font.glyphs.get(&65).unwrap();
        assert_eq!(a.x, 0);
        assert_eq!(a.y, 0);
        assert_eq!(a.width, 20);
        assert_eq!(a.height, 30);
        assert_eq!(a.xoffset, 1);
        assert_eq!(a.yoffset, 2);
        assert_eq!(a.xadvance, 22);
        assert_eq!(a.page, 0);
    }

    #[test]
    fn test_parse_fnt_kerning() {
        let font = parse_fnt(SAMPLE_FNT).unwrap();
        assert_eq!(font.kernings.len(), 2);
        assert_eq!(font.kernings[&(65, 66)], -1);
        assert_eq!(font.kernings[&(66, 67)], -2);
    }

    #[test]
    fn test_parse_fnt_empty_pages_error() {
        let content = "info face=\"Test\" size=16\ncommon lineHeight=20 base=16\n";
        let result = parse_fnt(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fnt_negative_offsets() {
        let content = r#"info face="Test" size=16
common lineHeight=20 base=16 scaleW=128 scaleH=128 pages=1
page id=0 file="test.png"
char id=48 x=0 y=0 width=10 height=14 xoffset=-2 yoffset=-1 xadvance=12 page=0
"#;
        let font = parse_fnt(content).unwrap();
        let glyph = font.glyphs.get(&48).unwrap();
        assert_eq!(glyph.xoffset, -2);
        assert_eq!(glyph.yoffset, -1);
    }
}
