use std::collections::HashMap;

use super::SkinHeader;

/// Loaded skin data.
#[derive(Debug)]
pub struct Skin {
    /// Skin header containing metadata.
    pub header: SkinHeader,
    /// Image sources indexed by ID.
    pub sources: HashMap<u32, SkinSource>,
    /// Skin objects to draw.
    pub objects: Vec<SkinObjectData>,
    /// Image definitions indexed by string ID.
    pub images: HashMap<String, ImageDef>,
    /// Image set definitions indexed by string ID.
    pub image_sets: HashMap<String, ImageSetDef>,
}

impl Skin {
    /// Create a new skin with the given header.
    pub fn new(header: SkinHeader) -> Self {
        Self {
            header,
            sources: HashMap::new(),
            objects: Vec::new(),
            images: HashMap::new(),
            image_sets: HashMap::new(),
        }
    }
}

/// Image source definition.
#[derive(Debug, Clone)]
pub struct SkinSource {
    /// Source ID.
    pub id: u32,
    /// File path pattern.
    pub path: String,
}

/// Image definition parsed from skin.
#[derive(Debug, Clone)]
pub struct ImageDef {
    /// Image ID (string).
    pub id: String,
    /// Source ID to reference.
    pub src: u32,
    /// Source X coordinate.
    pub x: i32,
    /// Source Y coordinate.
    pub y: i32,
    /// Source width.
    pub w: i32,
    /// Source height.
    pub h: i32,
    /// X division count for animation.
    pub divx: i32,
    /// Y division count for animation.
    pub divy: i32,
    /// Timer ID for animation.
    pub timer: i32,
    /// Animation cycle in ms.
    pub cycle: i32,
}

impl Default for ImageDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            src: 0,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: 0,
            cycle: 0,
        }
    }
}

/// Image set definition for grouped images.
#[derive(Debug, Clone, Default)]
pub struct ImageSetDef {
    /// Image set ID (string).
    pub id: String,
    /// Selection mode.
    pub mode: i32,
    /// List of image IDs in this set.
    pub images: Vec<String>,
}

/// Skin object data for rendering.
#[derive(Debug, Clone)]
pub struct SkinObjectData {
    /// Object type.
    pub object_type: SkinObjectType,
    /// Image or image set ID to reference.
    pub id: String,
    /// Display conditions (option IDs).
    pub op: Vec<i32>,
    /// Timer ID for display timing.
    pub timer: i32,
    /// Loop count (-1 for infinite).
    pub loop_count: i32,
    /// Offset ID.
    pub offset: i32,
    /// Blend mode.
    pub blend: i32,
    /// Filter mode.
    pub filter: i32,
    /// Stretch mode.
    pub stretch: i32,
    /// Destination keyframes.
    pub dst: Vec<Destination>,
}

impl Default for SkinObjectData {
    fn default() -> Self {
        Self {
            object_type: SkinObjectType::Image,
            id: String::new(),
            op: Vec::new(),
            timer: 0,
            loop_count: 0,
            offset: 0,
            blend: 0,
            filter: 0,
            stretch: 0,
            dst: Vec::new(),
        }
    }
}

/// Skin object type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinObjectType {
    /// Static image.
    Image,
    /// Animated image set.
    ImageSet,
    /// Number display.
    Number,
    /// Text display.
    Text,
    /// Slider/bar graph.
    Slider,
    /// Graph display.
    Graph,
    /// Note display.
    Note,
    /// Gauge display.
    Gauge,
    /// BGA display.
    Bga,
    /// Judge display.
    Judge,
}

/// Destination keyframe for animation.
#[derive(Debug, Clone)]
pub struct Destination {
    /// Time in ms (for animation keyframes).
    pub time: i32,
    /// X position.
    pub x: f32,
    /// Y position.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
    /// Acceleration type.
    pub acc: i32,
    /// Alpha (opacity).
    pub a: f32,
    /// Red color component.
    pub r: f32,
    /// Green color component.
    pub g: f32,
    /// Blue color component.
    pub b: f32,
    /// Rotation angle in degrees.
    pub angle: f32,
    /// Rotation center.
    pub center: i32,
}

impl Default for Destination {
    fn default() -> Self {
        Self {
            time: 0,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            acc: 0,
            a: 255.0,
            r: 255.0,
            g: 255.0,
            b: 255.0,
            angle: 0.0,
            center: 0,
        }
    }
}
