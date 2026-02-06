use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin bargraph object that renders a bar proportional to a float value.
#[derive(Debug, Clone)]
pub struct BargraphObject {
    pub id: String,
    /// Float property ID (BARGRAPH_*).
    pub ref_id: i32,
    /// Source texture reference.
    pub src: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    /// Direction: 0=up, 1=right, 2=down, 3=left.
    pub direction: i32,
    pub dst: DestinationSet,
    pub texture: Option<TextureId>,
}

impl Default for BargraphObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            ref_id: 0,
            src: -1,
            src_x: 0,
            src_y: 0,
            src_w: 0,
            src_h: 0,
            direction: 0,
            dst: DestinationSet::default(),
            texture: None,
        }
    }
}
