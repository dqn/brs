use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin graph object for score graph, BPM graph, etc.
#[derive(Debug, Clone)]
pub struct GraphObject {
    pub id: String,
    /// Graph type.
    pub graph_type: i32,
    /// Source texture reference.
    pub src: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    /// Divisions for the graph line (number of entries).
    pub div_x: i32,
    /// Direction: 0=up, 1=right, 2=down, 3=left.
    pub direction: i32,
    pub dst: DestinationSet,
    pub texture: Option<TextureId>,
}

impl Default for GraphObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            graph_type: 0,
            src: -1,
            src_x: 0,
            src_y: 0,
            src_w: 0,
            src_h: 0,
            div_x: 1,
            direction: 0,
            dst: DestinationSet::default(),
            texture: None,
        }
    }
}
