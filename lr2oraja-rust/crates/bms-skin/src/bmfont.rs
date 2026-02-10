// BMFont text format (.fnt) parser.
//
// Parses the BMFont text format produced by tools like BMFont, Hiero, etc.
// Supports the info, common, page, char, and kerning blocks.
//
// Reference: https://www.angelcode.com/products/bmfont/doc/file_format.html

use std::collections::HashMap;

use anyhow::Result;

/// Parsed BMFont data.
#[derive(Debug, Clone)]
pub struct BmFont {
    /// Original font size (from info block).
    pub size: f32,
    /// Line height in pixels (from common block).
    pub line_height: f32,
    /// Baseline offset from top of line (from common block).
    pub base: f32,
    /// Texture atlas width (from common block).
    pub scale_w: f32,
    /// Texture atlas height (from common block).
    pub scale_h: f32,
    /// Page texture file names, indexed by page ID.
    pub pages: Vec<String>,
    /// Glyph definitions keyed by Unicode codepoint.
    pub glyphs: HashMap<u32, BmGlyph>,
    /// Kerning pairs keyed by (first, second) codepoint.
    pub kernings: HashMap<(u32, u32), i32>,
}

/// A single glyph in the BMFont.
#[derive(Debug, Clone)]
pub struct BmGlyph {
    /// Unicode codepoint.
    pub id: u32,
    /// X position in the texture atlas.
    pub x: f32,
    /// Y position in the texture atlas.
    pub y: f32,
    /// Width in the texture atlas.
    pub width: f32,
    /// Height in the texture atlas.
    pub height: f32,
    /// X offset when rendering.
    pub xoffset: f32,
    /// Y offset when rendering.
    pub yoffset: f32,
    /// Horizontal advance after rendering this glyph.
    pub xadvance: f32,
    /// Texture page index.
    pub page: u32,
}

/// Parses BMFont text format content into a `BmFont`.
pub fn parse_bmfont(content: &str) -> Result<BmFont> {
    let mut size = 0.0_f32;
    let mut line_height = 0.0_f32;
    let mut base = 0.0_f32;
    let mut scale_w = 0.0_f32;
    let mut scale_h = 0.0_f32;
    let mut pages: Vec<(u32, String)> = Vec::new();
    let mut glyphs = HashMap::new();
    let mut kernings = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("info ") {
            let attrs = parse_attrs(rest);
            size = get_f32(&attrs, "size").unwrap_or(0.0).abs();
        } else if let Some(rest) = line.strip_prefix("common ") {
            let attrs = parse_attrs(rest);
            line_height = get_f32(&attrs, "lineHeight").unwrap_or(0.0);
            base = get_f32(&attrs, "base").unwrap_or(0.0);
            scale_w = get_f32(&attrs, "scaleW").unwrap_or(0.0);
            scale_h = get_f32(&attrs, "scaleH").unwrap_or(0.0);
        } else if let Some(rest) = line.strip_prefix("page ") {
            let attrs = parse_attrs(rest);
            let id = get_u32(&attrs, "id").unwrap_or(0);
            let file = attrs
                .get("file")
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_default();
            pages.push((id, file));
        } else if let Some(rest) = line.strip_prefix("char ") {
            let attrs = parse_attrs(rest);
            let glyph = BmGlyph {
                id: get_u32(&attrs, "id").unwrap_or(0),
                x: get_f32(&attrs, "x").unwrap_or(0.0),
                y: get_f32(&attrs, "y").unwrap_or(0.0),
                width: get_f32(&attrs, "width").unwrap_or(0.0),
                height: get_f32(&attrs, "height").unwrap_or(0.0),
                xoffset: get_f32(&attrs, "xoffset").unwrap_or(0.0),
                yoffset: get_f32(&attrs, "yoffset").unwrap_or(0.0),
                xadvance: get_f32(&attrs, "xadvance").unwrap_or(0.0),
                page: get_u32(&attrs, "page").unwrap_or(0),
            };
            glyphs.insert(glyph.id, glyph);
        } else if let Some(rest) = line.strip_prefix("kerning ") {
            let attrs = parse_attrs(rest);
            let first = get_u32(&attrs, "first").unwrap_or(0);
            let second = get_u32(&attrs, "second").unwrap_or(0);
            let amount = get_i32(&attrs, "amount").unwrap_or(0);
            kernings.insert((first, second), amount);
        }
        // Skip "chars count=..." and "kernings count=..." lines
    }

    if scale_w == 0.0 || scale_h == 0.0 {
        anyhow::bail!("Invalid BMFont: missing or zero scaleW/scaleH in common block");
    }

    // Sort pages by id and extract file names
    pages.sort_by_key(|(id, _)| *id);
    let page_files: Vec<String> = pages.into_iter().map(|(_, f)| f).collect();

    Ok(BmFont {
        size,
        line_height,
        base,
        scale_w,
        scale_h,
        pages: page_files,
        glyphs,
        kernings,
    })
}

/// Parses key=value pairs from a BMFont line.
/// Handles quoted values (e.g., file="font.png").
fn parse_attrs(input: &str) -> HashMap<&str, &str> {
    let mut result = HashMap::new();
    let mut rest = input;

    while !rest.is_empty() {
        // Skip whitespace
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }

        // Find key
        let eq_pos = match rest.find('=') {
            Some(p) => p,
            None => break,
        };
        let key = rest[..eq_pos].trim();
        rest = &rest[eq_pos + 1..];

        // Find value
        if rest.starts_with('"') {
            // Quoted value
            rest = &rest[1..]; // skip opening quote
            let end = rest.find('"').unwrap_or(rest.len());
            let value = &rest[..end];
            result.insert(key, value);
            rest = if end < rest.len() {
                &rest[end + 1..]
            } else {
                ""
            };
        } else {
            // Unquoted value (ends at space or end of string)
            let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
            let value = &rest[..end];
            result.insert(key, value);
            rest = &rest[end..];
        }
    }

    result
}

fn get_f32(attrs: &HashMap<&str, &str>, key: &str) -> Option<f32> {
    attrs.get(key).and_then(|v| v.parse::<f32>().ok())
}

fn get_u32(attrs: &HashMap<&str, &str>, key: &str) -> Option<u32> {
    attrs.get(key).and_then(|v| v.parse::<u32>().ok())
}

fn get_i32(attrs: &HashMap<&str, &str>, key: &str) -> Option<i32> {
    attrs.get(key).and_then(|v| v.parse::<i32>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FNT: &str = r#"info face="Arial" size=32 bold=0 italic=0 charset="" unicode=1 stretchH=100 smooth=1 aa=1 padding=0,0,0,0 spacing=1,1 outline=0
common lineHeight=36 base=29 scaleW=256 scaleH=256 pages=1 packed=0 alphaChnl=1 redChnl=0 greenChnl=0 blueChnl=0
page id=0 file="arial.png"
chars count=3
char id=65   x=0    y=0    width=22   height=24  xoffset=0    yoffset=5    xadvance=21   page=0  chnl=15
char id=66   x=23   y=0    width=18   height=24  xoffset=2    yoffset=5    xadvance=20   page=0  chnl=15
char id=32   x=42   y=0    width=0    height=0   xoffset=0    yoffset=0    xadvance=8    page=0  chnl=15
kerning first=65 second=66 amount=-1
"#;

    #[test]
    fn parse_info_block() {
        let font = parse_bmfont(SAMPLE_FNT).unwrap();
        assert_eq!(font.size, 32.0);
    }

    #[test]
    fn parse_common_block() {
        let font = parse_bmfont(SAMPLE_FNT).unwrap();
        assert_eq!(font.line_height, 36.0);
        assert_eq!(font.base, 29.0);
        assert_eq!(font.scale_w, 256.0);
        assert_eq!(font.scale_h, 256.0);
    }

    #[test]
    fn parse_page_block() {
        let font = parse_bmfont(SAMPLE_FNT).unwrap();
        assert_eq!(font.pages.len(), 1);
        assert_eq!(font.pages[0], "arial.png");
    }

    #[test]
    fn parse_char_block() {
        let font = parse_bmfont(SAMPLE_FNT).unwrap();
        assert_eq!(font.glyphs.len(), 3);

        let a = font.glyphs.get(&65).unwrap();
        assert_eq!(a.x, 0.0);
        assert_eq!(a.y, 0.0);
        assert_eq!(a.width, 22.0);
        assert_eq!(a.height, 24.0);
        assert_eq!(a.xoffset, 0.0);
        assert_eq!(a.yoffset, 5.0);
        assert_eq!(a.xadvance, 21.0);
        assert_eq!(a.page, 0);

        let space = font.glyphs.get(&32).unwrap();
        assert_eq!(space.width, 0.0);
        assert_eq!(space.xadvance, 8.0);
    }

    #[test]
    fn parse_kerning_block() {
        let font = parse_bmfont(SAMPLE_FNT).unwrap();
        assert_eq!(font.kernings.len(), 1);
        assert_eq!(font.kernings.get(&(65, 66)), Some(&-1));
    }

    #[test]
    fn parse_missing_common_fails() {
        let content = "info face=\"Test\" size=16\npage id=0 file=\"test.png\"\n";
        assert!(parse_bmfont(content).is_err());
    }

    #[test]
    fn parse_negative_size() {
        // Some tools export negative size for italic/bold
        let content = "info face=\"Test\" size=-24\ncommon lineHeight=28 base=22 scaleW=128 scaleH=128 pages=1\npage id=0 file=\"test.png\"\n";
        let font = parse_bmfont(content).unwrap();
        assert_eq!(font.size, 24.0);
    }

    #[test]
    fn parse_multiple_pages() {
        let content = "info face=\"Test\" size=16\ncommon lineHeight=20 base=16 scaleW=256 scaleH=256 pages=2\npage id=0 file=\"page0.png\"\npage id=1 file=\"page1.png\"\n";
        let font = parse_bmfont(content).unwrap();
        assert_eq!(font.pages.len(), 2);
        assert_eq!(font.pages[0], "page0.png");
        assert_eq!(font.pages[1], "page1.png");
    }

    #[test]
    fn parse_empty_lines_ignored() {
        let content = "\n\ninfo face=\"Test\" size=16\n\ncommon lineHeight=20 base=16 scaleW=128 scaleH=128 pages=1\n\npage id=0 file=\"test.png\"\n\n";
        let font = parse_bmfont(content).unwrap();
        assert_eq!(font.size, 16.0);
    }
}
