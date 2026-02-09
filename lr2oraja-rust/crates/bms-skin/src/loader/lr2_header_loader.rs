// LR2 skin header loader.
//
// Parses .lr2skin header files to extract metadata, custom options, files,
// and offsets for the skin selection UI. Header files use MS932 (Shift_JIS)
// encoding — the caller should decode to UTF-8 before calling these functions.
//
// Ported from LR2SkinHeaderLoader.java.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use bms_config::resolution::Resolution;
use bms_config::skin_type::SkinType;

use crate::property_id::{OFFSET_ALL, OFFSET_JUDGE_1P, OFFSET_JUDGEDETAIL_1P, OFFSET_NOTES_1P};
use crate::skin_header::{CustomFile, CustomOffset, CustomOption, SkinFormat, SkinHeader};

use super::lr2_csv_loader::{parse_field, process_conditional};

/// Resolution lookup table matching Java's RESOLUTION command.
/// Index 0=SD, 1=HD, 2=FullHD, 3=UltraHD.
const LR2_RESOLUTIONS: [Resolution; 4] = [
    Resolution::Sd,
    Resolution::Hd,
    Resolution::Fullhd,
    Resolution::Ultrahd,
];

/// Loads a SkinHeader from LR2 CSV header content.
///
/// `content` should already be decoded from MS932 to UTF-8.
pub fn load_lr2_header(content: &str, path: Option<&Path>) -> Result<SkinHeader> {
    let mut header = SkinHeader {
        format: SkinFormat::Lr2,
        path: path.map(|p| p.to_path_buf()),
        ..Default::default()
    };

    let mut options: Vec<CustomOption> = Vec::new();
    let mut files: Vec<CustomFile> = Vec::new();
    let mut offsets: Vec<CustomOffset> = Vec::new();

    // Option map for conditional evaluation
    let mut op: HashMap<i32, i32> = HashMap::new();

    // Condition state
    let mut skip = false;
    let mut found_true = false;

    for line in content.lines() {
        if !line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.is_empty() {
            continue;
        }

        let cmd = fields[0].trim_start_matches('#').to_uppercase();

        // Handle #IF / #ELSEIF / #ELSE / #ENDIF
        if process_conditional(&cmd, &fields, &op, &mut skip, &mut found_true) {
            continue;
        }

        if skip {
            continue;
        }

        match cmd.as_str() {
            "INFORMATION" => {
                if fields.len() > 3 {
                    let type_id = parse_field(&fields, 1);
                    header.skin_type = SkinType::from_id(type_id);
                    header.name = fields.get(2).unwrap_or(&"").to_string();
                    header.author = fields.get(3).unwrap_or(&"").to_string();

                    // Add standard play-mode options and offsets
                    if is_play_type(header.skin_type) {
                        add_play_options(&mut options);
                        add_play_offsets(&mut offsets);
                    }
                }
            }
            "RESOLUTION" => {
                let idx = parse_field(&fields, 1) as usize;
                if idx < LR2_RESOLUTIONS.len() {
                    header.resolution = LR2_RESOLUTIONS[idx];
                    header.source_resolution = Some(LR2_RESOLUTIONS[idx]);
                }
            }
            "CUSTOMOPTION" => {
                if fields.len() >= 4 {
                    let name = fields[1].to_string();
                    let base_id = parse_field(&fields, 2);
                    let mut op_names = Vec::new();
                    for field in fields.iter().skip(3) {
                        if !field.is_empty() {
                            op_names.push(field.to_string());
                        }
                    }
                    let op_ids: Vec<i32> =
                        (0..op_names.len() as i32).map(|i| base_id + i).collect();
                    options.push(CustomOption::new(name, op_ids, op_names));
                }
            }
            "CUSTOMFILE" => {
                if fields.len() >= 3 {
                    let name = fields[1].to_string();
                    let file_path = fields[2].replace("LR2files\\Theme", "").replace('\\', "/");
                    let def = fields.get(3).map(|s| s.to_string());
                    files.push(CustomFile::new(name, file_path, def));
                }
            }
            "CUSTOMOFFSET" => {
                if fields.len() >= 3 {
                    let name = fields[1].to_string();
                    let id = parse_field(&fields, 2);
                    let mut flags = [true; 6];
                    for (i, flag) in flags.iter_mut().enumerate() {
                        if i + 3 < fields.len() {
                            *flag = parse_field(&fields, i + 3) > 0;
                        }
                    }
                    offsets.push(CustomOffset::new(
                        name, id, flags[0], flags[1], flags[2], flags[3], flags[4], flags[5],
                    ));
                }
            }
            "CUSTOMOPTION_ADDITION_SETTING" => {
                // Remove default play-mode options if the skin says they are not needed
                let addition_names = ["BGA Size", "Ghost", "Score Graph", "Judge Detail"];
                for (i, add_name) in addition_names.iter().enumerate() {
                    if i + 1 < fields.len() {
                        let val: String = fields[i + 1]
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '-')
                            .collect();
                        if val == "0" {
                            options.retain(|o| o.name != *add_name);
                        }
                    }
                }
            }
            "SETOPTION" => {
                if fields.len() >= 3 {
                    let index = parse_field(&fields, 1);
                    let value = if parse_field(&fields, 2) >= 1 { 1 } else { 0 };
                    op.insert(index, value);
                }
            }
            _ => {}
        }
    }

    // Populate the option map for all custom option IDs
    for opt in &options {
        for &id in &opt.option_ids {
            op.entry(id).or_insert(0);
        }
    }

    header.options = options;
    header.files = files;
    header.offsets = offsets;

    Ok(header)
}

/// Returns true if the skin type is a play screen type.
fn is_play_type(skin_type: Option<SkinType>) -> bool {
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

/// Adds standard play-mode options (BGA Size, Ghost, Score Graph, Judge Detail).
fn add_play_options(options: &mut Vec<CustomOption>) {
    options.push(CustomOption::new(
        "BGA Size".to_string(),
        vec![30, 31],
        vec!["Normal".to_string(), "Extend".to_string()],
    ));
    options.push(CustomOption::new(
        "Ghost".to_string(),
        vec![34, 35, 36, 37],
        vec![
            "Off".to_string(),
            "Type A".to_string(),
            "Type B".to_string(),
            "Type C".to_string(),
        ],
    ));
    options.push(CustomOption::new(
        "Score Graph".to_string(),
        vec![38, 39],
        vec!["Off".to_string(), "On".to_string()],
    ));
    options.push(CustomOption::new(
        "Judge Detail".to_string(),
        vec![1997, 1998, 1999],
        vec![
            "Off".to_string(),
            "EARLY/LATE".to_string(),
            "+-ms".to_string(),
        ],
    ));
}

/// Adds standard play-mode offsets.
fn add_play_offsets(offsets: &mut Vec<CustomOffset>) {
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

/// Decodes MS932 (Shift_JIS) bytes to a UTF-8 string.
pub fn decode_ms932(bytes: &[u8]) -> String {
    let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
    cow.into_owned()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_information() {
        let csv = "#INFORMATION,0,My Skin,Author Name\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert_eq!(header.name, "My Skin");
        assert_eq!(header.author, "Author Name");
        assert_eq!(header.format, SkinFormat::Lr2);
        assert_eq!(header.skin_type, Some(SkinType::Play7Keys));
    }

    #[test]
    fn test_header_resolution() {
        let csv = "#INFORMATION,6,Skin,Auth\n#RESOLUTION,1\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert_eq!(header.resolution, Resolution::Hd);
        assert_eq!(header.source_resolution, Some(Resolution::Hd));
    }

    #[test]
    fn test_header_custom_option() {
        let csv = "#INFORMATION,6,S,A\n#CUSTOMOPTION,Lane Style,900,Normal,Mirror,Random\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert_eq!(header.options.len(), 1);
        assert_eq!(header.options[0].name, "Lane Style");
        assert_eq!(header.options[0].option_ids, vec![900, 901, 902]);
        assert_eq!(
            header.options[0].contents,
            vec!["Normal", "Mirror", "Random"]
        );
    }

    #[test]
    fn test_header_custom_file() {
        let csv =
            "#INFORMATION,6,S,A\n#CUSTOMFILE,Notes,LR2files\\Theme\\img\\notes.png,default.png\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert_eq!(header.files.len(), 1);
        assert_eq!(header.files[0].name, "Notes");
        assert_eq!(header.files[0].path, "/img/notes.png");
        assert_eq!(
            header.files[0].default_filename,
            Some("default.png".to_string())
        );
    }

    #[test]
    fn test_header_custom_offset() {
        let csv = "#INFORMATION,6,S,A\n#CUSTOMOFFSET,My Offset,50,1,1,0,0,0,0\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert_eq!(header.offsets.len(), 1);
        assert_eq!(header.offsets[0].name, "My Offset");
        assert_eq!(header.offsets[0].id, 50);
        assert!(header.offsets[0].editable_x);
        assert!(header.offsets[0].editable_y);
        assert!(!header.offsets[0].editable_w);
    }

    #[test]
    fn test_header_play_offsets_added() {
        // Play skin type 0 (5Keys) adds 4 standard offsets
        let csv = "#INFORMATION,0,Play,Auth\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert!(header.offsets.len() >= 4);
        assert_eq!(header.offsets[0].name, "All offset(%)");
    }

    #[test]
    fn test_header_play_options_added() {
        let csv = "#INFORMATION,0,Play,Auth\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert!(header.options.len() >= 4);
        let names: Vec<&str> = header.options.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"BGA Size"));
        assert!(names.contains(&"Ghost"));
    }

    #[test]
    fn test_header_addition_setting_removal() {
        let csv = "#INFORMATION,0,Play,Auth\n#CUSTOMOPTION_ADDITION_SETTING,0,0,0,0\n";
        let header = load_lr2_header(csv, None).unwrap();
        // All 4 default play options should be removed
        let names: Vec<&str> = header.options.iter().map(|o| o.name.as_str()).collect();
        assert!(!names.contains(&"BGA Size"));
        assert!(!names.contains(&"Ghost"));
    }

    #[test]
    fn test_header_if_condition() {
        let csv = "\
#INFORMATION,6,S,A\n\
#CUSTOMOPTION,Mode,900,A,B\n\
#SETOPTION,900,1\n\
#IF,900\n\
#CUSTOMOPTION,Sub,910,X,Y\n\
#ENDIF\n\
#IF,901\n\
#CUSTOMOPTION,Hidden,920,P,Q\n\
#ENDIF\n";
        let header = load_lr2_header(csv, None).unwrap();
        let names: Vec<&str> = header.options.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"Mode"));
        assert!(names.contains(&"Sub"));
        assert!(!names.contains(&"Hidden"));
    }

    #[test]
    fn test_header_non_play_no_offsets() {
        // Decide skin type (6) should not add play offsets
        let csv = "#INFORMATION,6,Decide,Auth\n";
        let header = load_lr2_header(csv, None).unwrap();
        assert!(header.offsets.is_empty());
    }

    #[test]
    fn test_decode_ms932() {
        // ASCII bytes
        let ascii = b"Hello World";
        assert_eq!(decode_ms932(ascii), "Hello World");
    }
}
