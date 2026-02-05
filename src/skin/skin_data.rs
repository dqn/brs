use std::collections::{HashMap, HashSet};

use super::SkinHeader;

/// Loaded skin data.
#[derive(Debug)]
pub struct Skin {
    /// Skin header containing metadata.
    pub header: SkinHeader,
    /// Path overrides for wildcard file mappings.
    pub file_map: HashMap<String, String>,
    /// Offset values indexed by offset ID.
    pub offsets: HashMap<i32, SkinOffset>,
    /// Image sources indexed by ID.
    pub sources: HashMap<u32, SkinSource>,
    /// Skin objects to draw.
    pub objects: Vec<SkinObjectData>,
    /// Image definitions indexed by string ID.
    pub images: HashMap<String, ImageDef>,
    /// Image set definitions indexed by string ID.
    pub image_sets: HashMap<String, ImageSetDef>,
    /// Slider definitions indexed by string ID.
    pub sliders: HashMap<String, SliderDef>,
    /// BGA object IDs.
    pub bga_ids: HashSet<String>,
    /// Number definitions indexed by string ID.
    pub numbers: HashMap<String, NumberDef>,
    /// Font definitions indexed by numeric ID.
    pub fonts: HashMap<u32, FontDef>,
    /// Text definitions indexed by string ID.
    pub texts: HashMap<String, TextDef>,
}

impl Skin {
    /// Create a new skin with the given header.
    pub fn new(header: SkinHeader) -> Self {
        Self {
            header,
            file_map: HashMap::new(),
            offsets: HashMap::new(),
            sources: HashMap::new(),
            objects: Vec::new(),
            images: HashMap::new(),
            image_sets: HashMap::new(),
            sliders: HashMap::new(),
            bga_ids: HashSet::new(),
            numbers: HashMap::new(),
            fonts: HashMap::new(),
            texts: HashMap::new(),
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
    /// Integer property ID for selection.
    pub ref_id: i32,
    /// List of image IDs in this set.
    pub images: Vec<String>,
}

/// Slider definition for movable UI elements.
#[derive(Debug, Clone)]
pub struct SliderDef {
    /// Slider ID (string).
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
    /// Slider direction (0=up, 1=right, 2=down, 3=left).
    pub angle: i32,
    /// Slider range (pixels).
    pub range: i32,
    /// Slider type (float property ID).
    pub slider_type: i32,
    /// Optional minimum value.
    pub min: Option<i32>,
    /// Optional maximum value.
    pub max: Option<i32>,
}

impl Default for SliderDef {
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
            angle: 0,
            range: 0,
            slider_type: 0,
            min: None,
            max: None,
        }
    }
}

/// Number definition for digit-based number display.
#[derive(Debug, Clone, Default)]
pub struct NumberDef {
    /// Number ID (string).
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
    /// Horizontal division count (10=0-9, 11=+minus, 12=+space).
    pub divx: i32,
    /// Vertical division count.
    pub divy: i32,
    /// Number of digits to display.
    pub digit: i32,
    /// IntegerProperty ID to reference.
    pub ref_id: i32,
    /// Alignment (0=RIGHT, 1=LEFT, 2=CENTER).
    pub align: i32,
    /// Zero padding flag.
    pub zeropadding: i32,
    /// Space between digits.
    pub space: i32,
    /// Animation cycle in ms.
    pub cycle: i32,
}

/// Font definition for bitmap fonts.
#[derive(Debug, Clone, Default)]
pub struct FontDef {
    /// Font ID (numeric).
    pub id: u32,
    /// Path to .fnt file (relative to skin directory).
    pub path: String,
}

/// Text definition for text display.
#[derive(Debug, Clone, Default)]
pub struct TextDef {
    /// Text ID (string).
    pub id: String,
    /// Font ID to reference.
    pub font: u32,
    /// Font size.
    pub size: i32,
    /// Alignment (0=LEFT, 1=CENTER, 2=RIGHT).
    pub align: i32,
    /// Overflow handling (0=OVERFLOW, 1=SHRINK, 2=TRUNCATE).
    pub overflow: i32,
    /// StringProperty ID to reference.
    pub ref_id: i32,
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
    /// Offset IDs.
    pub offsets: Vec<i32>,
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
            offsets: Vec::new(),
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

/// Offset values for skin rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct SkinOffset {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub r: f32,
    pub a: f32,
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
