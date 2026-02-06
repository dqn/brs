use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin number object that renders numeric values using digit images.
#[derive(Debug, Clone)]
pub struct NumberObject {
    pub id: String,
    /// Number property ID to read the value from.
    pub ref_id: i32,
    /// Source texture reference.
    pub src: i32,
    /// Source rect for the digit strip.
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    /// Number of digit columns in the source image (typically 10 or 11).
    pub div_x: i32,
    /// Number of digits to display.
    pub digit: i32,
    /// Padding: 0=zero-fill, 1=space, 2=no-pad.
    pub padding: i32,
    /// Reference type for the number value.
    pub ref_type: i32,
    /// Alignment: 0=right, 1=center, 2=left.
    pub align: i32,
    /// Destination set.
    pub dst: DestinationSet,
    /// Loaded texture ID.
    pub texture: Option<TextureId>,
}

impl Default for NumberObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            ref_id: 0,
            src: -1,
            src_x: 0,
            src_y: 0,
            src_w: 0,
            src_h: 0,
            div_x: 10,
            digit: 0,
            padding: 0,
            ref_type: 0,
            align: 0,
            dst: DestinationSet::default(),
            texture: None,
        }
    }
}

impl NumberObject {
    /// Get the source rect for a single digit (0-9, 10=minus, 11=space).
    pub fn digit_src_rect(&self, digit: usize) -> (f32, f32, f32, f32) {
        let digit_w = if self.div_x > 0 {
            self.src_w / self.div_x
        } else {
            self.src_w
        };
        (
            (self.src_x + digit as i32 * digit_w) as f32,
            self.src_y as f32,
            digit_w as f32,
            self.src_h as f32,
        )
    }
}
