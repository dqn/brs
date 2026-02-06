use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin image object that renders a texture or animated texture sequence.
#[derive(Debug, Clone)]
pub struct ImageObject {
    /// Object ID (skin-local).
    pub id: String,
    /// Texture source reference ID.
    pub src: i32,
    /// Source rectangle within the texture.
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    /// Animation divisions.
    pub div_x: i32,
    pub div_y: i32,
    /// Animation cycle time in milliseconds.
    pub cycle: i32,
    /// Animation timer ID.
    pub timer: i32,
    /// Destination set.
    pub dst: DestinationSet,
    /// Loaded texture ID (set at load time).
    pub texture: Option<TextureId>,
}

impl Default for ImageObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            src: -1,
            src_x: 0,
            src_y: 0,
            src_w: 0,
            src_h: 0,
            div_x: 1,
            div_y: 1,
            cycle: 0,
            timer: 0,
            dst: DestinationSet::default(),
            texture: None,
        }
    }
}

impl ImageObject {
    /// Get the current animation frame index.
    pub fn frame_index(&self, elapsed_ms: i64) -> usize {
        let total_frames = (self.div_x * self.div_y) as usize;
        if total_frames <= 1 || self.cycle <= 0 {
            return 0;
        }
        (elapsed_ms as usize / (self.cycle as usize / total_frames)) % total_frames
    }

    /// Get the source rect for a given animation frame.
    pub fn frame_src_rect(&self, frame: usize) -> (f32, f32, f32, f32) {
        let frame_w = if self.div_x > 0 {
            self.src_w / self.div_x
        } else {
            self.src_w
        };
        let frame_h = if self.div_y > 0 {
            self.src_h / self.div_y
        } else {
            self.src_h
        };
        let fx = (frame as i32) % self.div_x;
        let fy = (frame as i32) / self.div_x;
        (
            (self.src_x + fx * frame_w) as f32,
            (self.src_y + fy * frame_h) as f32,
            frame_w as f32,
            frame_h as f32,
        )
    }
}
