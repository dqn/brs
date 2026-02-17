// Header loading for JSON skin files.
//
// Handles JSON pre-processing (lenient → strict), header extraction,
// and skin metadata building.

use std::path::Path;

use anyhow::{Context, Result};

use bms_config::resolution::Resolution;
use bms_config::skin_type::SkinType;

use crate::property_id::{OFFSET_ALL, OFFSET_JUDGE_1P, OFFSET_JUDGEDETAIL_1P, OFFSET_NOTES_1P};
use crate::skin_header::{
    CustomCategory, CustomCategoryItem, CustomFile, CustomOffset, CustomOption, SkinFormat,
    SkinHeader,
};

use crate::loader::json_skin::JsonSkinData;

/// All Resolution variants for dimension-based lookup.
const ALL_RESOLUTIONS: [Resolution; 15] = [
    Resolution::Sd,
    Resolution::Svga,
    Resolution::Xga,
    Resolution::Hd,
    Resolution::Quadvga,
    Resolution::Fwxga,
    Resolution::Sxgaplus,
    Resolution::Hdplus,
    Resolution::Uxga,
    Resolution::Wsxgaplus,
    Resolution::Fullhd,
    Resolution::Wuxga,
    Resolution::Qxga,
    Resolution::Wqhd,
    Resolution::Ultrahd,
];

/// Pre-processes lenient JSON (as used by beatoraja skins) into strict JSON.
///
/// Handles:
/// - Missing commas between objects/arrays: `}  {` → `}, {`
/// - Trailing commas before `}` or `]`: `, }` → `}`
pub fn preprocess_json(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = input.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        if escape_next {
            escape_next = false;
            result.push(c);
            continue;
        }
        if c == '\\' && in_string {
            escape_next = true;
            result.push(c);
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }
        if in_string {
            result.push(c);
            continue;
        }

        // Remove trailing commas: skip comma if next non-whitespace is } or ]
        if c == ',' {
            let next_nonws = chars[i + 1..].iter().find(|ch| !ch.is_ascii_whitespace());
            if matches!(next_nonws, Some('}') | Some(']')) {
                continue; // skip trailing comma
            }
        }

        // Insert missing commas: after } or ] if next non-whitespace is { or [ or " or digit/minus
        if c == '}' || c == ']' {
            result.push(c);
            let next_nonws = chars[i + 1..].iter().find(|ch| !ch.is_ascii_whitespace());
            if matches!(
                next_nonws,
                Some('{') | Some('[') | Some('"') | Some('0'..='9') | Some('-')
            ) {
                result.push(',');
            }
            continue;
        }

        result.push(c);
    }

    result
}

/// Loads only the skin header from a JSON skin file.
///
/// This is used for the skin selection UI — it reads metadata and
/// custom options without loading the full skin.
pub fn load_header(json_str: &str) -> Result<SkinHeader> {
    let preprocessed = preprocess_json(json_str);
    let data: JsonSkinData =
        serde_json::from_str(&preprocessed).context("Failed to parse JSON skin")?;
    build_header(&data, None)
}

/// Builds a SkinHeader from parsed JSON skin data.
pub fn build_header(data: &JsonSkinData, path: Option<&Path>) -> Result<SkinHeader> {
    if data.skin_type == -1 {
        anyhow::bail!("Skin type not specified (type = -1)");
    }

    let skin_type = SkinType::from_id(data.skin_type);

    // Build custom options
    let mut options = Vec::new();
    for prop in &data.property {
        let mut op_ids = Vec::new();
        let mut op_names = Vec::new();
        for item in &prop.item {
            op_ids.push(item.op);
            op_names.push(item.name.clone().unwrap_or_default());
        }
        let mut opt = CustomOption::new(prop.name.clone().unwrap_or_default(), op_ids, op_names);
        if let Some(def) = &prop.def {
            opt.default_label = Some(def.clone());
        }
        options.push(opt);
    }

    // Build custom files
    let parent_dir = path
        .and_then(|p| p.parent())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut files = Vec::new();
    for fp in &data.filepath {
        let full_path = if parent_dir.is_empty() {
            fp.path.clone().unwrap_or_default()
        } else {
            format!("{}/{}", parent_dir, fp.path.as_deref().unwrap_or(""))
        };
        files.push(CustomFile::new(
            fp.name.clone().unwrap_or_default(),
            full_path,
            fp.def.clone(),
        ));
    }

    // Build custom offsets
    let mut offsets = Vec::new();
    for off in &data.offset {
        offsets.push(CustomOffset::new(
            off.name.clone().unwrap_or_default(),
            off.id,
            off.x,
            off.y,
            off.w,
            off.h,
            off.r,
            off.a,
        ));
    }

    // Add standard play-mode offsets
    if is_play_type(skin_type) {
        offsets.push(CustomOffset::new(
            "All offset(%)".to_string(),
            OFFSET_ALL,
            true,
            true,
            true,
            true,
            false,
            false,
        ));
        offsets.push(CustomOffset::new(
            "Notes offset".to_string(),
            OFFSET_NOTES_1P,
            false,
            false,
            false,
            true,
            false,
            false,
        ));
        offsets.push(CustomOffset::new(
            "Judge offset".to_string(),
            OFFSET_JUDGE_1P,
            true,
            true,
            true,
            true,
            false,
            true,
        ));
        offsets.push(CustomOffset::new(
            "Judge Detail offset".to_string(),
            OFFSET_JUDGEDETAIL_1P,
            true,
            true,
            true,
            true,
            false,
            true,
        ));
    }

    // Build categories
    let mut categories = Vec::new();
    for cat in &data.category {
        let mut items = Vec::new();
        for cat_item_name in &cat.item {
            // Find matching option
            for (i, prop) in data.property.iter().enumerate() {
                if prop.category.as_deref() == Some(cat_item_name) {
                    items.push(CustomCategoryItem::Option(i));
                }
            }
            // Find matching file
            for (i, fp) in data.filepath.iter().enumerate() {
                if fp.category.as_deref() == Some(cat_item_name) {
                    items.push(CustomCategoryItem::File(i));
                }
            }
            // Find matching offset
            for (i, off) in data.offset.iter().enumerate() {
                if off.category.as_deref() == Some(cat_item_name) {
                    items.push(CustomCategoryItem::Offset(i));
                }
            }
        }
        categories.push(CustomCategory {
            name: cat.name.clone().unwrap_or_default(),
            items,
        });
    }

    // Detect source resolution from skin dimensions
    let source_resolution = ALL_RESOLUTIONS
        .iter()
        .find(|r| r.width() == data.w && r.height() == data.h)
        .copied();

    Ok(SkinHeader {
        format: SkinFormat::Beatoraja,
        path: path.map(|p| p.to_path_buf()),
        skin_type,
        name: data.name.clone().unwrap_or_default(),
        author: data.author.clone().unwrap_or_default(),
        options,
        files,
        offsets,
        categories,
        resolution: source_resolution.unwrap_or(Resolution::Hd),
        source_resolution,
        destination_resolution: None,
    })
}

/// Returns true if the skin type is a play screen type.
pub(super) fn is_play_type(skin_type: Option<SkinType>) -> bool {
    matches!(
        skin_type,
        Some(
            SkinType::Play5Keys
                | SkinType::Play7Keys
                | SkinType::Play9Keys
                | SkinType::Play10Keys
                | SkinType::Play14Keys
                | SkinType::Play24Keys
                | SkinType::Play24KeysDouble
        )
    )
}
