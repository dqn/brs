// Mechanical translation of JSONSkinLoader.java
// Main JSON skin loader

mod builder;
pub mod parser;

pub use builder::*;
pub use parser::*;
#[cfg(test)]
use parser::{coerce_json_numbers_to_strings, fix_lenient_json};

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid skin JSON that parses successfully via parse_skin_json.
    const MINIMAL_SKIN_JSON: &str = r#"{"type":5,"name":"test","w":1920,"h":1080}"#;

    /// Skin JSON with a line comment — valid in Gson but rejected by serde_json.
    const SKIN_WITH_COMMENT: &str = r#"{
        // This is a line comment
        "type": 5,
        "name": "test"
    }"#;

    #[test]
    fn test_fix_lenient_json_trailing_comma() {
        let input = r#"[1, 2, 3,]"#;
        let fixed = fix_lenient_json(input);
        assert_eq!(fixed, "[1, 2, 3]");
    }

    #[test]
    fn test_fix_lenient_json_missing_comma() {
        let input = "[\n\t{\"a\":1}\n\t{\"b\":2}\n]";
        let fixed = fix_lenient_json(input);
        let parsed: serde_json::Value = serde_json::from_str(&fixed).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_parse_skin_json_numeric_id_and_src() {
        let input =
            r#"{"type":5,"source":[{"id":0,"path":"a.png"}],"image":[{"id":"bg","src":1}]}"#;
        let skin = parse_skin_json(input).unwrap();
        assert_eq!(skin.source[0].id, Some("0".to_string()));
        assert_eq!(skin.image[0].src, Some("1".to_string()));
    }

    #[test]
    fn test_parse_default_select_skin() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("skin/default/select.json");
        if !path.exists() {
            // Skip if skin file not present in CI
            return;
        }
        let content = std::fs::read_to_string(&path).unwrap();
        let skin = parse_skin_json(&content);
        assert!(
            skin.is_ok(),
            "Failed to parse select.json: {:?}",
            skin.err()
        );
        let skin = skin.unwrap();
        assert_eq!(skin.skin_type, 5);
        assert_eq!(skin.name, Some("beatoraja default".to_string()));
        assert!(!skin.source.is_empty());
        assert!(!skin.image.is_empty());
    }

    #[test]
    fn test_parse_default_play24_skin() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("skin/default/play24.json");
        if !path.exists() {
            return;
        }
        let content = std::fs::read_to_string(&path).unwrap();
        let skin = parse_skin_json(&content).unwrap();
        assert_eq!(skin.skin_type, 16);
        assert!(!skin.destination.is_empty());
        assert!(!skin.text.is_empty());
    }

    // ---- Phase 45b: verify MINIMAL_SKIN_JSON round-trips ----

    #[test]
    fn test_minimal_skin_json_parses() {
        let skin = parse_skin_json(MINIMAL_SKIN_JSON).unwrap();
        assert_eq!(skin.skin_type, 5);
        assert_eq!(skin.name, Some("test".to_string()));
        assert_eq!(skin.w, 1920);
        assert_eq!(skin.h, 1080);
    }

    // ---- Line comments are now stripped by fix_lenient_json ----

    #[test]
    fn test_line_comment_stripped() {
        let skin = parse_skin_json(SKIN_WITH_COMMENT).unwrap();
        assert_eq!(skin.skin_type, 5);
        assert_eq!(skin.name, Some("test".to_string()));
    }

    // ---- Block comments are now stripped by fix_lenient_json ----

    #[test]
    fn test_block_comment_stripped() {
        let input = r#"{ /* block comment */ "type": 5, "name": "test" }"#;
        let skin = parse_skin_json(input).unwrap();
        assert_eq!(skin.skin_type, 5);
        assert_eq!(skin.name, Some("test".to_string()));
    }

    // ---- BOM handling ----

    #[test]
    fn test_bom_prefix_stripped() {
        let input = format!("\u{FEFF}{}", MINIMAL_SKIN_JSON);
        let skin = parse_skin_json(&input).unwrap();
        assert_eq!(skin.skin_type, 5);
        assert_eq!(skin.name, Some("test".to_string()));
    }

    // ---- Comment stripping: string-safety ----

    #[test]
    fn test_comments_inside_string_not_stripped() {
        // `//` and `/* */` inside a JSON string value must be preserved
        let input = r#"{"type":5,"name":"a // b /* c */ d","w":1920,"h":1080}"#;
        let skin = parse_skin_json(input).unwrap();
        assert_eq!(skin.name, Some("a // b /* c */ d".to_string()));
    }

    #[test]
    fn test_nested_block_comment_edge_case() {
        // Nested `/*` inside a block comment: the first `*/` ends the comment
        // (same behavior as Java/Gson)
        let input = r#"{ /* outer /* inner */ "type": 5, "name": "test" }"#;
        let skin = parse_skin_json(input).unwrap();
        assert_eq!(skin.skin_type, 5);
        assert_eq!(skin.name, Some("test".to_string()));
    }

    // ---- Phase 48c fix: fix_lenient_json preserves `}{` inside string literals ----
    // The string-aware state machine skips braces inside quoted strings.

    #[test]
    fn test_fix_lenient_json_preserves_braces_in_strings() {
        let input = r#"{"path":"a}{b"}"#;
        let fixed = fix_lenient_json(input);
        assert_eq!(
            fixed, input,
            "fix_lenient_json must not modify braces inside string literals"
        );
    }

    // ---- Phase 48d: M3 — numeric `path` not coerced to string ----
    // The coercion whitelist is `id | src | font`; `path` is not included.
    // Gson coerces ALL numeric values when the target field is String, but
    // our coercion only handles the whitelisted keys.

    #[test]
    fn test_numeric_path_not_coerced() {
        let input = r#"{"type":5,"source":[{"id":"0","path":42}]}"#;
        let cleaned = fix_lenient_json(input);
        let mut value: serde_json::Value = serde_json::from_str(&cleaned).unwrap();
        coerce_json_numbers_to_strings(&mut value);

        let path_val = &value["source"][0]["path"];
        assert!(
            path_val.is_number(),
            "path should remain numeric (not in coercion whitelist), got: {}",
            path_val
        );
        assert_eq!(path_val.as_i64(), Some(42));
    }

    // ---- get_skin_for_type dispatch tests ----

    #[test]
    fn test_get_skin_for_type_play7keys() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData {
            skin_type: SkinType::Play7Keys.id(),
            ..Default::default()
        };
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::Play7Keys, &header);
        assert_eq!(skin.skin_type, Some(SkinType::Play7Keys));
        assert!(skin.header.is_some());
    }

    #[test]
    fn test_get_skin_for_type_music_select() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData {
            skin_type: SkinType::MusicSelect.id(),
            ..Default::default()
        };
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::MusicSelect, &header);
        assert_eq!(skin.skin_type, Some(SkinType::MusicSelect));
    }

    #[test]
    fn test_get_skin_for_type_decide() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData::default();
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::Decide, &header);
        assert_eq!(skin.skin_type, Some(SkinType::Decide));
    }

    #[test]
    fn test_get_skin_for_type_result() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData::default();
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::Result, &header);
        assert_eq!(skin.skin_type, Some(SkinType::Result));
    }

    #[test]
    fn test_get_skin_for_type_course_result() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData::default();
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::CourseResult, &header);
        assert_eq!(skin.skin_type, Some(SkinType::CourseResult));
    }

    #[test]
    fn test_get_skin_for_type_skin_select() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData::default();
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::SkinSelect, &header);
        assert_eq!(skin.skin_type, Some(SkinType::SkinSelect));
    }

    #[test]
    fn test_get_skin_for_type_key_config_default() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData::default();
        // SoundSet falls through to KeyConfig default branch
        let skin = JSONSkinLoader::get_skin_for_type(&SkinType::SoundSet, &header);
        assert_eq!(skin.skin_type, Some(SkinType::KeyConfig));
    }

    #[test]
    fn test_skin_data_from_header_stores_header() {
        use crate::skin_type::SkinType;
        let header = SkinHeaderData {
            skin_type: SkinType::Play7Keys.id(),
            name: "My Skin".to_string(),
            author: "Author".to_string(),
            ..Default::default()
        };
        let skin = SkinData::from_header(&header, SkinType::Play7Keys);
        assert_eq!(skin.skin_type, Some(SkinType::Play7Keys));
        let stored = skin.header.unwrap();
        assert_eq!(stored.name, "My Skin");
        assert_eq!(stored.author, "Author");
    }

    #[test]
    fn test_skin_data_new_has_none_skin_type() {
        let skin = SkinData::new();
        assert_eq!(skin.skin_type, None);
        assert!(skin.header.is_none());
    }

    #[test]
    fn test_is_skin_customize_button_in_range() {
        // BUTTON_SKIN_CUSTOMIZE1 = 220, BUTTON_SKIN_CUSTOMIZE10 = 229
        // Range is [220, 229) — 220..228 are customize buttons
        assert!(is_skin_customize_button(220)); // CUSTOMIZE1
        assert!(is_skin_customize_button(224)); // CUSTOMIZE5
        assert!(is_skin_customize_button(228)); // CUSTOMIZE9
    }

    #[test]
    fn test_is_skin_customize_button_out_of_range() {
        assert!(!is_skin_customize_button(219)); // below range
        assert!(!is_skin_customize_button(229)); // CUSTOMIZE10 is the exclusive upper bound
        assert!(!is_skin_customize_button(230)); // above range
        assert!(!is_skin_customize_button(0));
        assert!(!is_skin_customize_button(-1));
    }

    #[test]
    fn test_get_skin_customize_index() {
        // Index is relative to BUTTON_SKIN_CUSTOMIZE1 (220)
        assert_eq!(skin_customize_index(220), 0);
        assert_eq!(skin_customize_index(221), 1);
        assert_eq!(skin_customize_index(228), 8);
    }
}
