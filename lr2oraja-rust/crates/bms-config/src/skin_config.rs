use serde::{Deserialize, Serialize};

use crate::skin_type::SkinType;

/// A named option with an integer value.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkinOption {
    pub name: String,
    pub value: i32,
}

impl SkinOption {
    fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

/// A named file path.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FilePath {
    pub name: String,
    pub path: String,
}

impl FilePath {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.path.is_empty()
    }
}

/// A named offset with position, size, rotation and alpha.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Offset {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub r: i32,
    pub a: i32,
}

impl Offset {
    fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}

/// Skin property containing options, file paths, and offsets.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Property {
    pub option: Vec<SkinOption>,
    pub file: Vec<FilePath>,
    pub offset: Vec<Offset>,
}

impl Property {
    pub fn validate(&mut self) {
        self.option.retain(|o| o.is_valid());
        self.file.retain(|f| f.is_valid());
        self.offset.retain(|o| o.is_valid());
    }
}

/// Skin configuration with file path and properties.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkinConfig {
    pub path: Option<String>,
    pub properties: Option<Property>,
}

impl SkinConfig {
    /// Validates this skin config. Returns false if path is empty or missing.
    pub fn validate(&mut self) -> bool {
        match &self.path {
            Some(p) if !p.is_empty() => {}
            _ => return false,
        }
        if self.properties.is_none() {
            self.properties = Some(Property::default());
        }
        if let Some(ref mut props) = self.properties {
            props.validate();
        }
        true
    }

    /// Returns a default skin config for the given skin type ID.
    pub fn get_default(id: i32) -> Self {
        let path = default_skin_path(id);
        let mut skin = Self {
            path: path.map(String::from),
            properties: None,
        };
        skin.validate();
        skin
    }
}

/// Returns the default skin path for the given skin type ID.
fn default_skin_path(id: i32) -> Option<&'static str> {
    let skin_type = SkinType::from_id(id)?;
    match skin_type {
        SkinType::Play7Keys => Some("skin/default/play/play7.luaskin"),
        SkinType::Play5Keys => Some("skin/default/play5.json"),
        SkinType::Play14Keys => Some("skin/default/play14.json"),
        SkinType::Play10Keys => Some("skin/default/play10.json"),
        SkinType::Play9Keys => Some("skin/default/play9.json"),
        SkinType::MusicSelect => Some("skin/default/select.json"),
        SkinType::Decide => Some("skin/default/decide/decide.luaskin"),
        SkinType::Result => Some("skin/default/result/result.luaskin"),
        SkinType::CourseResult => Some("skin/default/graderesult.json"),
        SkinType::Play24Keys => Some("skin/default/play24.json"),
        SkinType::Play24KeysDouble => Some("skin/default/play24double.json"),
        SkinType::KeyConfig => Some("skin/default/keyconfig/keyconfig.luaskin"),
        SkinType::SkinSelect => Some("skin/default/skinselect/skinselect.luaskin"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_path() {
        let mut sc = SkinConfig::default();
        assert!(!sc.validate());
    }

    #[test]
    fn test_validate_with_path() {
        let mut sc = SkinConfig {
            path: Some("test/path.json".to_string()),
            properties: None,
        };
        assert!(sc.validate());
        assert!(sc.properties.is_some());
    }

    #[test]
    fn test_validate_removes_invalid_options() {
        let mut sc = SkinConfig {
            path: Some("test.json".to_string()),
            properties: Some(Property {
                option: vec![
                    SkinOption {
                        name: "valid".to_string(),
                        value: 1,
                    },
                    SkinOption {
                        name: String::new(),
                        value: 2,
                    },
                ],
                file: vec![
                    FilePath {
                        name: "good".to_string(),
                        path: "path.png".to_string(),
                    },
                    FilePath {
                        name: "bad".to_string(),
                        path: String::new(),
                    },
                ],
                offset: vec![Offset {
                    name: String::new(),
                    ..Default::default()
                }],
            }),
        };
        sc.validate();
        let props = sc.properties.as_ref().unwrap();
        assert_eq!(props.option.len(), 1);
        assert_eq!(props.file.len(), 1);
        assert_eq!(props.offset.len(), 0);
    }

    #[test]
    fn test_get_default() {
        let sc = SkinConfig::get_default(0);
        assert_eq!(sc.path.as_deref(), Some("skin/default/play/play7.luaskin"));
        assert!(sc.properties.is_some());

        let sc5 = SkinConfig::get_default(1);
        assert_eq!(sc5.path.as_deref(), Some("skin/default/play5.json"));
    }

    #[test]
    fn test_get_default_no_default_skin() {
        // SoundSet (id=10) and Theme (id=11) have no default
        let sc = SkinConfig::get_default(10);
        assert!(sc.path.is_none());

        let sc = SkinConfig::get_default(11);
        assert!(sc.path.is_none());
    }

    #[test]
    fn test_get_default_invalid_id() {
        let sc = SkinConfig::get_default(99);
        assert!(sc.path.is_none());
    }

    #[test]
    fn test_serde_round_trip() {
        let sc = SkinConfig {
            path: Some("test.json".to_string()),
            properties: Some(Property {
                option: vec![SkinOption {
                    name: "opt".to_string(),
                    value: 42,
                }],
                file: Vec::new(),
                offset: Vec::new(),
            }),
        };
        let json = serde_json::to_string(&sc).unwrap();
        let back: SkinConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, sc.path);
        assert_eq!(back.properties.as_ref().unwrap().option.len(), 1);
    }

    #[test]
    fn test_deserialize_from_empty() {
        let sc: SkinConfig = serde_json::from_str("{}").unwrap();
        assert!(sc.path.is_none());
        assert!(sc.properties.is_none());
    }
}
