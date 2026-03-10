use super::deserializers::{deserialize_i32_lenient, deserialize_optional_string_from_int};
use serde::{Deserialize, Serialize};

/// Corresponds to JsonSkin.Property
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Property {
    pub category: Option<String>,
    pub name: Option<String>,
    pub item: Vec<PropertyItem>,
    pub def: Option<String>,
}

/// Corresponds to JsonSkin.PropertyItem
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PropertyItem {
    pub name: Option<String>,
    pub op: i32,
}

/// Corresponds to JsonSkin.Filepath
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Filepath {
    pub category: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub def: Option<String>,
}

/// Corresponds to JsonSkin.Offset
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Offset {
    pub category: Option<String>,
    pub name: Option<String>,
    #[serde(deserialize_with = "deserialize_i32_lenient", default)]
    pub id: i32,
    pub x: bool,
    pub y: bool,
    pub w: bool,
    pub h: bool,
    pub r: bool,
    pub a: bool,
}

/// Corresponds to JsonSkin.Category
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Category {
    pub name: Option<String>,
    pub item: Vec<String>,
}

/// Corresponds to JsonSkin.Source
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Source {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub path: Option<String>,
}

/// Corresponds to JsonSkin.Font
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Font {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "type")]
    pub font_type: i32,
}
