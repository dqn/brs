use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin slider object that renders a sliding element based on a float value.
#[derive(Debug, Clone)]
pub struct SliderObject {
    pub id: String,
    /// Slider property ID (SLIDER_*).
    pub ref_id: i32,
    /// Source texture reference.
    pub src: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    /// Slider range in pixels.
    pub range: f32,
    /// Slider direction: 0=up, 1=right, 2=down, 3=left.
    pub direction: i32,
    pub dst: DestinationSet,
    pub texture: Option<TextureId>,
}

impl Default for SliderObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            ref_id: 0,
            src: -1,
            src_x: 0,
            src_y: 0,
            src_w: 0,
            src_h: 0,
            range: 0.0,
            direction: 0,
            dst: DestinationSet::default(),
            texture: None,
        }
    }
}
