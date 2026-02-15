// LR2 bitmap font (.lr2font) parser.
//
// Parses .lr2font files and converts them to BmFont format for rendering
// through the existing bitmap font pipeline.
//
// Ported from LR2FontLoader.java.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::bmfont::{BmFont, BmGlyph};

/// Parsed LR2 font data, ready for loading into FontMap.
#[derive(Debug, Clone)]
pub struct Lr2FontData {
    /// Key for FontMap registration (the .lr2font path).
    pub key: String,
    /// Converted BmFont data.
    pub bmfont: BmFont,
    /// Texture paths: (texture_id, absolute_path).
    pub texture_paths: Vec<(i32, PathBuf)>,
}

/// Parses a `.lr2font` file and converts to BmFont format.
///
/// `content` should already be decoded to UTF-8.
/// `font_dir` is the directory containing the .lr2font file (for resolving texture paths).
pub fn parse_lr2font(content: &str, font_dir: &Path) -> Result<Lr2FontData> {
    let mut size: i32 = 0;
    let mut margin: i32 = 0;
    let mut pages: Vec<String> = Vec::new();
    let mut texture_paths: Vec<(i32, PathBuf)> = Vec::new();
    let mut glyphs: HashMap<u32, BmGlyph> = HashMap::new();
    // Track max texture ID to size the pages vec correctly.
    let mut max_page_id: i32 = -1;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line[1..].split(',').collect();
        if fields.is_empty() {
            continue;
        }

        let cmd = fields[0].trim();
        match cmd {
            "S" => {
                if fields.len() >= 2 {
                    size = fields[1].trim().parse::<i32>().unwrap_or(0);
                }
            }
            "M" => {
                if fields.len() >= 2 {
                    margin = fields[1].trim().parse::<i32>().unwrap_or(0);
                }
            }
            "T" => {
                if fields.len() >= 3 {
                    let tex_id = fields[1].trim().parse::<i32>().unwrap_or(0);
                    let tex_path = fields[2].trim();
                    let abs_path = font_dir.join(tex_path);
                    texture_paths.push((tex_id, abs_path));
                    if tex_id > max_page_id {
                        max_page_id = tex_id;
                    }
                }
            }
            "R" => {
                if fields.len() >= 7 {
                    let code = fields[1].trim().parse::<i32>().unwrap_or(0);
                    let tex_id = fields[2].trim().parse::<i32>().unwrap_or(0);
                    let x = fields[3].trim().parse::<i32>().unwrap_or(0);
                    let y = fields[4].trim().parse::<i32>().unwrap_or(0);
                    let w = fields[5].trim().parse::<i32>().unwrap_or(0);
                    let h = fields[6].trim().parse::<i32>().unwrap_or(0);

                    let codepoints = map_code(code);
                    for cp in codepoints {
                        glyphs.insert(
                            cp,
                            BmGlyph {
                                id: cp,
                                x: x as f32,
                                y: y as f32,
                                width: w as f32,
                                height: h as f32,
                                xoffset: 0.0,
                                yoffset: 0.0,
                                xadvance: w as f32 + margin as f32,
                                page: tex_id as u32,
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }

    // Build pages vec indexed by page ID.
    let page_count = if max_page_id >= 0 {
        (max_page_id + 1) as usize
    } else {
        0
    };
    pages.resize(page_count, String::new());
    for &(id, ref path) in &texture_paths {
        if (id as usize) < pages.len() {
            pages[id as usize] = path.to_string_lossy().to_string();
        }
    }

    let line_height = if size > 0 { size } else { 32 };

    // Use a nominal texture size; actual dimensions come from loaded images.
    let bmfont = BmFont {
        size: line_height as f32,
        line_height: line_height as f32,
        base: line_height as f32,
        scale_w: 1.0,
        scale_h: 1.0,
        pages,
        glyphs,
        kernings: HashMap::new(),
    };

    Ok(Lr2FontData {
        key: String::new(),
        bmfont,
        texture_paths,
    })
}

/// Converts an LR2 character code (Shift-JIS based) to Unicode codepoints.
///
/// Ported from LR2FontLoader.java `mapCode()`.
pub fn map_code(code: i32) -> Vec<u32> {
    // Special case: wave dash (both forms)
    if code == 288 {
        return vec![0x301C, 0xFF5E];
    }

    let sjis_bytes: Vec<u8> = if code >= 8127 {
        let sjis_code = (code + 49281) as u16;
        vec![(sjis_code >> 8) as u8, (sjis_code & 0xFF) as u8]
    } else if code >= 256 {
        let sjis_code = (code + 32832) as u16;
        vec![(sjis_code >> 8) as u8, (sjis_code & 0xFF) as u8]
    } else {
        vec![(code & 0xFF) as u8]
    };

    let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&sjis_bytes);
    let s = decoded.as_ref();

    let codepoints: Vec<u32> = s.chars().map(|c| c as u32).collect();
    // Filter out replacement characters (invalid Shift-JIS sequences)
    let filtered: Vec<u32> = codepoints.into_iter().filter(|&cp| cp != 0xFFFD).collect();
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- map_code tests --

    #[test]
    fn map_code_ascii() {
        // Code 65 = 'A' in ASCII/Shift-JIS
        let result = map_code(65);
        assert_eq!(result, vec![0x41]); // 'A'
    }

    #[test]
    fn map_code_ascii_range() {
        // Code 48 = '0'
        let result = map_code(48);
        assert_eq!(result, vec![0x30]); // '0'

        // Code 90 = 'Z'
        let result = map_code(90);
        assert_eq!(result, vec![0x5A]); // 'Z'
    }

    #[test]
    fn map_code_wave_dash() {
        // Code 288 → both wave dash forms
        let result = map_code(288);
        assert_eq!(result, vec![0x301C, 0xFF5E]);
    }

    #[test]
    fn map_code_hiragana() {
        // Hiragana 'あ' is at Shift-JIS 0x82A0.
        // code + 32832 = sjis_code → code = 0x82A0 - 32832 = 33440 - 32832 = 608
        // Wait, let me recalculate. 0x82A0 = 33440. 33440 - 32832 = 608.
        // So code 608 should map to 'あ' (U+3042).
        let result = map_code(608);
        assert_eq!(result, vec![0x3042]); // 'あ'
    }

    #[test]
    fn map_code_katakana() {
        // Katakana 'ア' is at Shift-JIS 0x8341.
        // 0x8341 = 33601. 33601 - 32832 = 769.
        let result = map_code(769);
        assert_eq!(result, vec![0x30A2]); // 'ア'
    }

    #[test]
    fn map_code_high_range() {
        // Test code >= 8127 (adds 49281).
        // Shift-JIS 0xE040 = 57408. 57408 - 49281 = 8127.
        let result = map_code(8127);
        // Shift-JIS 0xE040 maps to a specific kanji.
        assert!(!result.is_empty());
    }

    // -- parse_lr2font tests --

    #[test]
    fn parse_basic_lr2font() {
        let content = "\
#S,32
#M,2
#T,0,test_font.png
#R,65,0,0,0,16,32
#R,66,0,16,0,14,32
#R,67,0,30,0,16,32
";
        let font_dir = Path::new("/skin/fonts");
        let data = parse_lr2font(content, font_dir).unwrap();

        assert_eq!(data.bmfont.size, 32.0);
        assert_eq!(data.bmfont.line_height, 32.0);
        assert_eq!(data.bmfont.pages.len(), 1);
        assert_eq!(data.texture_paths.len(), 1);
        assert_eq!(data.texture_paths[0].0, 0);
        assert_eq!(
            data.texture_paths[0].1,
            PathBuf::from("/skin/fonts/test_font.png")
        );

        // Check glyph 'A' (code 65 → U+0041)
        let glyph_a = data.bmfont.glyphs.get(&0x41).unwrap();
        assert_eq!(glyph_a.x, 0.0);
        assert_eq!(glyph_a.y, 0.0);
        assert_eq!(glyph_a.width, 16.0);
        assert_eq!(glyph_a.height, 32.0);
        assert_eq!(glyph_a.xadvance, 18.0); // width(16) + margin(2)
        assert_eq!(glyph_a.page, 0);

        // Check glyph 'B' (code 66 → U+0042)
        let glyph_b = data.bmfont.glyphs.get(&0x42).unwrap();
        assert_eq!(glyph_b.x, 16.0);
        assert_eq!(glyph_b.width, 14.0);
        assert_eq!(glyph_b.xadvance, 16.0); // width(14) + margin(2)

        // Check glyph 'C' (code 67 → U+0043)
        let glyph_c = data.bmfont.glyphs.get(&0x43).unwrap();
        assert_eq!(glyph_c.x, 30.0);
    }

    #[test]
    fn parse_multiple_textures() {
        let content = "\
#S,48
#M,0
#T,0,ascii.png
#T,1,kanji.png
#R,65,0,0,0,24,48
#R,66,1,0,0,24,48
";
        let font_dir = Path::new("/skin");
        let data = parse_lr2font(content, font_dir).unwrap();

        assert_eq!(data.bmfont.pages.len(), 2);
        assert_eq!(data.texture_paths.len(), 2);

        let glyph_a = data.bmfont.glyphs.get(&0x41).unwrap();
        assert_eq!(glyph_a.page, 0);

        let glyph_b = data.bmfont.glyphs.get(&0x42).unwrap();
        assert_eq!(glyph_b.page, 1);
    }

    #[test]
    fn parse_empty_content() {
        let data = parse_lr2font("", Path::new("/skin")).unwrap();
        assert!(data.bmfont.glyphs.is_empty());
        assert!(data.bmfont.pages.is_empty());
        // Default size when no #S command
        assert_eq!(data.bmfont.size, 32.0);
    }

    #[test]
    fn parse_invalid_commands_ignored() {
        let content = "\
#S,32
#UNKNOWN,1,2,3
#R,65,0,0,0,16,32
not a command
";
        let data = parse_lr2font(content, Path::new("/skin")).unwrap();
        assert_eq!(data.bmfont.glyphs.len(), 1);
    }

    #[test]
    fn parse_zero_margin() {
        let content = "\
#S,24
#M,0
#T,0,font.png
#R,65,0,0,0,12,24
";
        let data = parse_lr2font(content, Path::new("/skin")).unwrap();
        let glyph = data.bmfont.glyphs.get(&0x41).unwrap();
        assert_eq!(glyph.xadvance, 12.0); // width(12) + margin(0)
    }

    #[test]
    fn parse_wave_dash_creates_two_glyphs() {
        let content = "\
#S,32
#M,0
#T,0,font.png
#R,288,0,0,0,16,32
";
        let data = parse_lr2font(content, Path::new("/skin")).unwrap();
        // Wave dash maps to two codepoints
        assert!(data.bmfont.glyphs.contains_key(&0x301C));
        assert!(data.bmfont.glyphs.contains_key(&0xFF5E));
    }
}
