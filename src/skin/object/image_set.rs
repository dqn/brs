use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// An image set object that selects from multiple images based on a value.
#[derive(Debug, Clone, Default)]
pub struct ImageSetObject {
    pub id: String,
    /// Reference property ID to determine which image to display.
    pub ref_id: i32,
    /// Source entries (one per image in the set).
    pub images: Vec<ImageSetEntry>,
    pub dst: DestinationSet,
}

/// A single entry in an image set.
#[derive(Debug, Clone)]
pub struct ImageSetEntry {
    pub src: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    pub texture: Option<TextureId>,
}
