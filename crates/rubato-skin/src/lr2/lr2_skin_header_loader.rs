use std::path::Path;

use crate::offset_capabilities::OffsetCapabilities;

use crate::lr2::lr2_skin_loader::LR2SkinLoaderState;
use crate::reexports::{MainState, Resolution};
use crate::skin_property::{OFFSET_ALL, OFFSET_JUDGE_1P, OFFSET_JUDGEDETAIL_1P, OFFSET_NOTES_1P};

/// LR2 skin header loader
///
/// Translated from LR2SkinHeaderLoader.java
/// Loads LR2 skin header files (.lr2skin) to extract skin metadata,
/// custom options, custom files, and custom offsets.
///
/// Custom option definition
#[derive(Clone, Debug)]
pub struct CustomOption {
    pub name: String,
    pub option: Vec<i32>,
    pub contents: Vec<String>,
    pub selected_option: i32,
}

impl CustomOption {
    pub fn new(name: &str, option: Vec<i32>, contents: Vec<String>) -> Self {
        let selected = option.first().copied().unwrap_or(0);
        Self {
            name: name.to_string(),
            option,
            contents,
            selected_option: selected,
        }
    }

    pub fn selected_option(&self) -> i32 {
        self.selected_option
    }
}

/// Custom file definition
#[derive(Clone, Debug)]
pub struct CustomFile {
    pub name: String,
    pub path: String,
    pub def: Option<String>,
    pub selected_filename: Option<String>,
}

impl CustomFile {
    pub fn new(name: &str, path: &str, def: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            def: def.map(|s| s.to_string()),
            selected_filename: None,
        }
    }

    pub fn selected_filename(&self) -> Option<&str> {
        self.selected_filename.as_deref()
    }
}

/// Custom offset definition
#[derive(Clone, Debug)]
pub struct CustomOffset {
    pub name: String,
    pub id: i32,
    pub caps: OffsetCapabilities,
}

impl CustomOffset {
    pub fn new(name: &str, id: i32, caps: OffsetCapabilities) -> Self {
        Self {
            name: name.to_string(),
            id,
            caps,
        }
    }
}

/// Skin header data
#[derive(Clone, Debug, Default)]
pub struct LR2SkinHeaderData {
    pub path: Option<std::path::PathBuf>,
    pub skin_type: Option<crate::skin_type::SkinType>,
    pub name: String,
    pub author: String,
    pub resolution: Option<Resolution>,
    pub custom_options: Vec<CustomOption>,
    pub custom_files: Vec<CustomFile>,
    pub custom_offsets: Vec<CustomOffset>,
}

/// LR2 skin header loader
pub struct LR2SkinHeaderLoader {
    pub header: LR2SkinHeaderData,
    pub files: Vec<CustomFile>,
    pub options: Vec<CustomOption>,
    pub offsets: Vec<CustomOffset>,
    pub skinpath: String,
    pub base: LR2SkinLoaderState,
}

impl LR2SkinHeaderLoader {
    pub fn new(skinpath: &str) -> Self {
        Self {
            header: LR2SkinHeaderData::default(),
            files: Vec::new(),
            options: Vec::new(),
            offsets: Vec::new(),
            skinpath: skinpath.to_string(),
            base: LR2SkinLoaderState::new(),
        }
    }

    pub fn load_skin(
        &mut self,
        f: &Path,
        _state: Option<&dyn MainState>,
    ) -> anyhow::Result<LR2SkinHeaderData> {
        self.header = LR2SkinHeaderData::default();
        self.files.clear();
        self.options.clear();
        self.offsets.clear();

        self.header.path = Some(f.to_path_buf());

        let raw_bytes = std::fs::read(f)?;
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&raw_bytes);
        let content = decoded.into_owned();

        for line in content.lines() {
            if let Some((cmd, str_parts)) = self.base.process_line_directives(line, _state) {
                self.process_header_command(&cmd, &str_parts);
            }
        }

        self.header.custom_options = self.options.clone();
        self.header.custom_files = self.files.clone();
        self.header.custom_offsets = self.offsets.clone();

        // Set up options in op map
        for option in &self.header.custom_options {
            for &opt in &option.option {
                let val = if option.selected_option() == opt {
                    1
                } else {
                    0
                };
                self.base.op.insert(opt, val);
            }
        }

        Ok(self.header.clone())
    }

    fn process_header_command(&mut self, cmd: &str, str_parts: &[String]) {
        match cmd {
            "INFORMATION" => {
                if str_parts.len() >= 4 {
                    if let Ok(type_id) = str_parts[1].trim().parse::<i32>() {
                        self.header.skin_type =
                            crate::skin_type::SkinType::skin_type_by_id(type_id);
                    }
                    self.header.name = str_parts[2].clone();
                    self.header.author = str_parts[3].clone();

                    // Add default options for play skin types
                    if let Some(
                        crate::skin_type::SkinType::Play5Keys
                        | crate::skin_type::SkinType::Play7Keys
                        | crate::skin_type::SkinType::Play9Keys
                        | crate::skin_type::SkinType::Play10Keys
                        | crate::skin_type::SkinType::Play14Keys
                        | crate::skin_type::SkinType::Play24Keys
                        | crate::skin_type::SkinType::Play24KeysDouble,
                    ) = self.header.skin_type
                    {
                        self.options.push(CustomOption::new(
                            "BGA Size",
                            vec![30, 31],
                            vec!["Normal".to_string(), "Extend".to_string()],
                        ));
                        self.options.push(CustomOption::new(
                            "Ghost",
                            vec![34, 35, 36, 37],
                            vec![
                                "Off".to_string(),
                                "Type A".to_string(),
                                "Type B".to_string(),
                                "Type C".to_string(),
                            ],
                        ));
                        self.options.push(CustomOption::new(
                            "Score Graph",
                            vec![38, 39],
                            vec!["Off".to_string(), "On".to_string()],
                        ));
                        self.options.push(CustomOption::new(
                            "Judge Detail",
                            vec![1997, 1998, 1999],
                            vec![
                                "Off".to_string(),
                                "EARLY/LATE".to_string(),
                                "+-ms".to_string(),
                            ],
                        ));

                        self.offsets.push(CustomOffset::new(
                            "All offset(%)",
                            OFFSET_ALL,
                            OffsetCapabilities {
                                x: true,
                                y: true,
                                w: true,
                                h: true,
                                ..Default::default()
                            },
                        ));
                        self.offsets.push(CustomOffset::new(
                            "Notes offset",
                            OFFSET_NOTES_1P,
                            OffsetCapabilities {
                                h: true,
                                ..Default::default()
                            },
                        ));
                        self.offsets.push(CustomOffset::new(
                            "Judge offset",
                            OFFSET_JUDGE_1P,
                            OffsetCapabilities {
                                x: true,
                                y: true,
                                w: true,
                                h: true,
                                a: true,
                                ..Default::default()
                            },
                        ));
                        self.offsets.push(CustomOffset::new(
                            "Judge Detail offset",
                            OFFSET_JUDGEDETAIL_1P,
                            OffsetCapabilities {
                                x: true,
                                y: true,
                                w: true,
                                h: true,
                                a: true,
                                ..Default::default()
                            },
                        ));
                    }
                }
            }
            "RESOLUTION" => {
                if str_parts.len() > 1 {
                    let res_values = [
                        Resolution {
                            width: 640.0,
                            height: 480.0,
                        }, // SD
                        Resolution {
                            width: 1280.0,
                            height: 720.0,
                        }, // HD
                        Resolution {
                            width: 1920.0,
                            height: 1080.0,
                        }, // FULLHD
                        Resolution {
                            width: 3840.0,
                            height: 2160.0,
                        }, // ULTRAHD
                    ];
                    if let Ok(idx) = str_parts[1].trim().parse::<usize>()
                        && idx < res_values.len()
                    {
                        self.header.resolution = Some(res_values[idx].clone());
                    }
                }
            }
            "CUSTOMOPTION" => {
                if str_parts.len() >= 3 {
                    let mut contents: Vec<String> = Vec::new();
                    for part in &str_parts[3..] {
                        if !part.is_empty() {
                            contents.push(part.clone());
                        }
                    }
                    let base_op: i32 = str_parts[2].trim().parse().unwrap_or(0);
                    let op: Vec<i32> = (0..contents.len()).map(|i| base_op + i as i32).collect();
                    self.options
                        .push(CustomOption::new(&str_parts[1], op, contents));
                }
            }
            "CUSTOMFILE" => {
                if str_parts.len() >= 3 {
                    let path = str_parts[2]
                        .replace("LR2files\\Theme", &self.skinpath)
                        .replace('\\', "/");
                    let def = if str_parts.len() >= 4 {
                        Some(str_parts[3].as_str())
                    } else {
                        None
                    };
                    self.files.push(CustomFile::new(&str_parts[1], &path, def));
                }
            }
            "CUSTOMOFFSET" => {
                if str_parts.len() >= 3 {
                    let mut op = [true; 6];
                    for i in 0..6 {
                        if i + 3 < str_parts.len()
                            && let Ok(v) = str_parts[i + 3].trim().parse::<i32>()
                        {
                            op[i] = v > 0;
                        }
                    }
                    let id: i32 = str_parts[2].trim().parse().unwrap_or(0);
                    self.offsets.push(CustomOffset::new(
                        &str_parts[1],
                        id,
                        OffsetCapabilities {
                            x: op[0],
                            y: op[1],
                            w: op[2],
                            h: op[3],
                            r: op[4],
                            a: op[5],
                        },
                    ));
                }
            }
            "CUSTOMOPTION_ADDITION_SETTING" => {
                // #CUSTOMOPTION_ADDITION_SETTING, BGA Size, Ghost, Score Graph, Judge Detail
                // 0 = No Add, 1 = Add
                let addition_names = ["BGA Size", "Ghost", "Score Graph", "Judge Detail"];
                let mut addition_indices: [Option<usize>; 4] = [None; 4];
                for (idx, co) in self.options.iter().enumerate() {
                    for (i, name) in addition_names.iter().enumerate() {
                        if co.name == *name {
                            addition_indices[i] = Some(idx);
                        }
                    }
                }
                // Remove in reverse order to maintain indices
                let mut to_remove: Vec<usize> = Vec::new();
                for (i, &add_idx) in addition_indices.iter().enumerate() {
                    if i + 1 < str_parts.len() {
                        let cleaned: String = str_parts[i + 1]
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '-')
                            .collect();
                        if cleaned == "0"
                            && let Some(idx) = add_idx
                        {
                            to_remove.push(idx);
                        }
                    }
                }
                to_remove.sort_unstable();
                to_remove.dedup();
                for idx in to_remove.into_iter().rev() {
                    if idx < self.options.len() {
                        self.options.remove(idx);
                    }
                }
            }
            "INCLUDE" => {
                // No-op in header loader
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_header(name: &str, content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("lr2_skin_header_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_load_empty_file_returns_default_header() {
        let path = write_temp_header("empty_header.lr2skin", "");
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert!(header.skin_type.is_none());
        assert!(header.name.is_empty());
        assert!(header.author.is_empty());
        assert!(header.resolution.is_none());
        assert!(header.custom_options.is_empty());
        assert!(header.custom_files.is_empty());
        assert!(header.custom_offsets.is_empty());
    }

    #[test]
    fn test_information_command_parses_fields() {
        let csv = "#INFORMATION,0,TestSkin,TestAuthor\n";
        let path = write_temp_header("info.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.name, "TestSkin");
        assert_eq!(header.author, "TestAuthor");
    }

    #[test]
    fn test_information_too_few_fields_no_panic() {
        let csv = "#INFORMATION,0\n";
        let path = write_temp_header("info_short.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert!(header.name.is_empty());
    }

    #[test]
    fn test_resolution_sd() {
        let csv = "#RESOLUTION,0\n";
        let path = write_temp_header("res_sd.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        let res = header.resolution.unwrap();
        assert_eq!(res.width, 640.0);
        assert_eq!(res.height, 480.0);
    }

    #[test]
    fn test_resolution_hd() {
        let csv = "#RESOLUTION,1\n";
        let path = write_temp_header("res_hd.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        let res = header.resolution.unwrap();
        assert_eq!(res.width, 1280.0);
        assert_eq!(res.height, 720.0);
    }

    #[test]
    fn test_resolution_fullhd() {
        let csv = "#RESOLUTION,2\n";
        let path = write_temp_header("res_fullhd.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        let res = header.resolution.unwrap();
        assert_eq!(res.width, 1920.0);
        assert_eq!(res.height, 1080.0);
    }

    #[test]
    fn test_resolution_ultrahd() {
        let csv = "#RESOLUTION,3\n";
        let path = write_temp_header("res_uhd.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        let res = header.resolution.unwrap();
        assert_eq!(res.width, 3840.0);
        assert_eq!(res.height, 2160.0);
    }

    #[test]
    fn test_resolution_out_of_range_ignored() {
        let csv = "#RESOLUTION,99\n";
        let path = write_temp_header("res_oor.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert!(header.resolution.is_none());
    }

    #[test]
    fn test_resolution_non_numeric_ignored() {
        let csv = "#RESOLUTION,abc\n";
        let path = write_temp_header("res_nan.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert!(header.resolution.is_none());
    }

    #[test]
    fn test_customoption_basic() {
        let csv = "#CUSTOMOPTION,Lane Style,100,Normal,Wide,Narrow\n";
        let path = write_temp_header("customopt.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.custom_options.len(), 1);
        assert_eq!(header.custom_options[0].name, "Lane Style");
        assert_eq!(header.custom_options[0].option, vec![100, 101, 102]);
        assert_eq!(header.custom_options[0].contents.len(), 3);
    }

    #[test]
    fn test_customoption_too_few_fields_no_panic() {
        let csv = "#CUSTOMOPTION,Name\n";
        let path = write_temp_header("customopt_short.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert!(header.custom_options.is_empty());
    }

    #[test]
    fn test_customfile_basic() {
        let csv = "#CUSTOMFILE,Background,LR2files\\Theme/bg*.png,default_bg.png\n";
        let path = write_temp_header("customfile.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.custom_files.len(), 1);
        assert_eq!(header.custom_files[0].name, "Background");
        assert_eq!(
            header.custom_files[0].def,
            Some("default_bg.png".to_string())
        );
    }

    #[test]
    fn test_customfile_no_default() {
        let csv = "#CUSTOMFILE,BG,images/bg.png\n";
        let path = write_temp_header("customfile_nodef.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.custom_files.len(), 1);
        assert!(header.custom_files[0].def.is_none());
    }

    #[test]
    fn test_customoffset_basic() {
        let csv = "#CUSTOMOFFSET,My Offset,50,1,1,0,0,1,0\n";
        let path = write_temp_header("customoffset.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.custom_offsets.len(), 1);
        assert_eq!(header.custom_offsets[0].name, "My Offset");
        assert_eq!(header.custom_offsets[0].id, 50);
        assert!(header.custom_offsets[0].caps.x);
        assert!(header.custom_offsets[0].caps.y);
        assert!(!header.custom_offsets[0].caps.w);
        assert!(!header.custom_offsets[0].caps.h);
        assert!(header.custom_offsets[0].caps.r);
        assert!(!header.custom_offsets[0].caps.a);
    }

    #[test]
    fn test_customoffset_defaults_to_all_true() {
        // When capability fields are missing, they default to true
        let csv = "#CUSTOMOFFSET,Minimal,10\n";
        let path = write_temp_header("customoffset_min.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.custom_offsets.len(), 1);
        let caps = &header.custom_offsets[0].caps;
        assert!(caps.x);
        assert!(caps.y);
        assert!(caps.w);
        assert!(caps.h);
        assert!(caps.r);
        assert!(caps.a);
    }

    #[test]
    fn test_play_skin_type_adds_default_options_and_offsets() {
        // INFORMATION type 0 = Play7Keys (has defaults)
        let csv = "#INFORMATION,0,PlaySkin,Author\n";
        let path = write_temp_header("play_defaults.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        // Play skins add default options: BGA Size, Ghost, Score Graph, Judge Detail
        let option_names: Vec<&str> = header
            .custom_options
            .iter()
            .map(|o| o.name.as_str())
            .collect();
        assert!(option_names.contains(&"BGA Size"));
        assert!(option_names.contains(&"Ghost"));
        assert!(option_names.contains(&"Score Graph"));
        assert!(option_names.contains(&"Judge Detail"));
        // Play skins add default offsets
        assert!(!header.custom_offsets.is_empty());
    }

    #[test]
    fn test_nonexistent_file_returns_error() {
        let path = std::path::PathBuf::from("/nonexistent/skin.lr2skin");
        let mut loader = LR2SkinHeaderLoader::new("/nonexistent");
        assert!(loader.load_skin(&path, None).is_err());
    }

    #[test]
    fn test_multiple_custom_options_accumulate() {
        let csv = "\
#CUSTOMOPTION,Opt1,100,A,B\n\
#CUSTOMOPTION,Opt2,200,X,Y,Z\n";
        let path = write_temp_header("multi_opts.lr2skin", csv);
        let skinpath = path.parent().unwrap().to_str().unwrap();
        let mut loader = LR2SkinHeaderLoader::new(skinpath);
        let header = loader.load_skin(&path, None).unwrap();
        assert_eq!(header.custom_options.len(), 2);
        assert_eq!(header.custom_options[0].name, "Opt1");
        assert_eq!(header.custom_options[1].name, "Opt2");
        assert_eq!(header.custom_options[1].option, vec![200, 201, 202]);
    }

    #[test]
    fn test_custom_option_selected_defaults_to_first() {
        let co = CustomOption::new(
            "Test",
            vec![10, 11, 12],
            vec!["A".into(), "B".into(), "C".into()],
        );
        assert_eq!(co.selected_option(), 10);
    }

    #[test]
    fn test_custom_option_empty_option_vec() {
        let co = CustomOption::new("Test", vec![], vec![]);
        assert_eq!(co.selected_option(), 0); // Falls back to unwrap_or(0)
    }

    #[test]
    fn test_custom_file_selected_filename_default_none() {
        let cf = CustomFile::new("BG", "images/bg.png", Some("default.png"));
        assert!(cf.selected_filename().is_none());
    }
}
