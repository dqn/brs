use super::deserializers::{
    deserialize_animations_with_conditionals, deserialize_optional_i32_or_string,
    deserialize_optional_string_from_int,
};
use serde::{Deserialize, Serialize};

/// Corresponds to JsonSkin.Destination
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Destination {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub blend: i32,
    pub filter: i32,
    pub timer: Option<i32>,
    #[serde(rename = "loop")]
    pub loop_val: i32,
    pub center: i32,
    pub offset: i32,
    pub offsets: Vec<i32>,
    #[serde(default = "default_neg_one")]
    pub stretch: i32,
    pub op: Vec<i32>,
    #[serde(deserialize_with = "deserialize_optional_i32_or_string", default)]
    pub draw: Option<i32>,
    #[serde(deserialize_with = "deserialize_animations_with_conditionals", default)]
    pub dst: Vec<Animation>,
    #[serde(rename = "mouseRect")]
    pub mouse_rect: Option<Rect>,
}

fn default_neg_one() -> i32 {
    -1
}

/// Corresponds to JsonSkin.Rect
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Corresponds to JsonSkin.Animation
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Animation {
    pub time: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub acc: i32,
    pub a: i32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub angle: i32,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            time: i32::MIN,
            x: i32::MIN,
            y: i32::MIN,
            w: i32::MIN,
            h: i32::MIN,
            acc: i32::MIN,
            a: i32::MIN,
            r: i32::MIN,
            g: i32::MIN,
            b: i32::MIN,
            angle: i32::MIN,
        }
    }
}
