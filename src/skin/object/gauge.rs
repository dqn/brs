use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin gauge object for groove gauge display.
#[derive(Debug, Clone)]
pub struct GaugeObject {
    pub id: String,
    /// Source textures for different gauge parts.
    /// [0]=empty_normal, [1]=filled_normal, [2]=empty_hard, [3]=filled_hard,
    /// [4]=empty_border, [5]=filled_border
    pub textures: Vec<GaugeTextureEntry>,
    /// Number of gauge parts (typically 50).
    pub parts: i32,
    /// Animation cycle time in ms.
    pub cycle: i32,
    pub dst: DestinationSet,
}

/// A single gauge texture entry.
#[derive(Debug, Clone)]
pub struct GaugeTextureEntry {
    pub src: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    pub div_x: i32,
    pub div_y: i32,
    pub texture: Option<TextureId>,
}

impl Default for GaugeObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            textures: Vec::new(),
            parts: 50,
            cycle: 0,
            dst: DestinationSet::default(),
        }
    }
}
