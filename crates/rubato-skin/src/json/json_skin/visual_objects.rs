use super::deserializers::{
    deserialize_optional_string_from_int, deserialize_vec_string_from_ints,
};
use serde::{Deserialize, Serialize};

/// Corresponds to JsonSkin.Image
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Image {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub src: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default = "default_one")]
    pub divx: i32,
    #[serde(default = "default_one")]
    pub divy: i32,
    pub timer: Option<i32>,
    pub cycle: i32,
    pub len: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub act: Option<i32>,
    pub click: i32,
}

pub(super) fn default_one() -> i32 {
    1
}

/// Corresponds to JsonSkin.ImageSet
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageSet {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<i32>,
    #[serde(deserialize_with = "deserialize_vec_string_from_ints", default)]
    pub images: Vec<String>,
    pub act: Option<i32>,
    pub click: i32,
}

/// Corresponds to JsonSkin.Value
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Value {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub src: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default = "default_one")]
    pub divx: i32,
    #[serde(default = "default_one")]
    pub divy: i32,
    pub timer: Option<i32>,
    pub cycle: i32,
    pub align: i32,
    pub digit: i32,
    pub padding: i32,
    pub zeropadding: i32,
    pub space: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<i32>,
    pub offset: Option<Vec<Value>>,
}

/// Corresponds to JsonSkin.FloatValue
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FloatValue {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub src: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default = "default_one")]
    pub divx: i32,
    #[serde(default = "default_one")]
    pub divy: i32,
    pub timer: Option<i32>,
    pub cycle: i32,
    pub align: i32,
    pub fketa: i32,
    pub iketa: i32,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(rename = "isSignvisible")]
    pub is_signvisible: bool,
    pub padding: i32,
    pub zeropadding: i32,
    pub space: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<i32>,
    pub offset: Option<Vec<Value>>,
}

fn default_gain() -> f32 {
    1.0
}

/// Corresponds to JsonSkin.Text
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Text {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub font: Option<String>,
    pub size: i32,
    pub align: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<i32>,
    #[serde(rename = "constantText")]
    pub constant_text: Option<String>,
    pub wrapping: bool,
    pub overflow: i32,
    #[serde(rename = "outlineColor", default = "default_outline_color")]
    pub outline_color: String,
    #[serde(rename = "outlineWidth")]
    pub outline_width: f32,
    #[serde(rename = "shadowColor", default = "default_shadow_color")]
    pub shadow_color: String,
    #[serde(rename = "shadowOffsetX")]
    pub shadow_offset_x: f32,
    #[serde(rename = "shadowOffsetY")]
    pub shadow_offset_y: f32,
    #[serde(rename = "shadowSmoothness")]
    pub shadow_smoothness: f32,
}

fn default_outline_color() -> String {
    "ffffff00".to_string()
}

fn default_shadow_color() -> String {
    "ffffff00".to_string()
}
