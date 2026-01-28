//! beatoraja skin type definitions
//!
//! These types represent the beatoraja JSON skin format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skin type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkinType(pub i32);

impl SkinType {
    /// 7KEYS
    pub const PLAY_7KEYS: SkinType = SkinType(0);
    /// 5KEYS
    pub const PLAY_5KEYS: SkinType = SkinType(1);
    /// 14KEYS (Double Play)
    pub const PLAY_14KEYS: SkinType = SkinType(2);
    /// 10KEYS (Double Play 5KEYS)
    pub const PLAY_10KEYS: SkinType = SkinType(3);
    /// 9KEYS (PMS)
    pub const PLAY_9KEYS: SkinType = SkinType(4);
    /// Music Select
    pub const MUSIC_SELECT: SkinType = SkinType(5);
    /// Decide (loading screen)
    pub const DECIDE: SkinType = SkinType(6);
    /// Result
    pub const RESULT: SkinType = SkinType(7);
    /// Course Result
    pub const COURSE_RESULT: SkinType = SkinType(8);
    /// Key Configuration
    pub const KEY_CONFIG: SkinType = SkinType(10);
    /// Skin Select
    pub const SKIN_SELECT: SkinType = SkinType(11);
}

/// Skin header information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinHeader {
    /// Skin name
    #[serde(default)]
    pub name: String,
    /// Skin author
    #[serde(default)]
    pub author: String,
    /// Skin type
    #[serde(rename = "type", default)]
    pub skin_type: i32,
    /// Base width
    #[serde(default = "default_width")]
    pub w: i32,
    /// Base height
    #[serde(default = "default_height")]
    pub h: i32,
    /// File path (set after loading)
    #[serde(skip)]
    pub path: String,
}

impl Default for SkinHeader {
    fn default() -> Self {
        Self {
            name: String::new(),
            author: String::new(),
            skin_type: 0,
            w: default_width(),
            h: default_height(),
            path: String::new(),
        }
    }
}

fn default_width() -> i32 {
    1280
}

fn default_height() -> i32 {
    720
}

/// Image source definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Source ID
    pub id: i32,
    /// File path relative to skin directory
    pub path: String,
}

/// Image region definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDef {
    /// Image definition ID
    pub id: i32,
    /// Source image ID
    pub src: i32,
    /// X position in source
    #[serde(default)]
    pub x: i32,
    /// Y position in source
    #[serde(default)]
    pub y: i32,
    /// Width (0 = full width)
    #[serde(default)]
    pub w: i32,
    /// Height (0 = full height)
    #[serde(default)]
    pub h: i32,
    /// Horizontal divisions (for frame animation)
    #[serde(default = "default_one")]
    pub divx: i32,
    /// Vertical divisions (for frame animation)
    #[serde(default = "default_one")]
    pub divy: i32,
    /// Reference (for LR2 compatibility)
    #[serde(default)]
    pub refer: i32,
}

fn default_one() -> i32 {
    1
}

/// Image set definition (group of images)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSet {
    /// ImageSet ID
    pub id: i32,
    /// Image IDs in this set
    #[serde(default)]
    pub images: Vec<i32>,
    /// Reference (for conditional selection)
    #[serde(default)]
    pub refer: i32,
}

/// Value definition for number/slider rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueDef {
    /// Value definition ID
    pub id: i32,
    /// Source image ID
    pub src: i32,
    /// X position in source
    #[serde(default)]
    pub x: i32,
    /// Y position in source
    #[serde(default)]
    pub y: i32,
    /// Width of single digit
    #[serde(default)]
    pub w: i32,
    /// Height of single digit
    #[serde(default)]
    pub h: i32,
    /// Horizontal divisions
    #[serde(default = "default_ten")]
    pub divx: i32,
    /// Vertical divisions
    #[serde(default = "default_one")]
    pub divy: i32,
    /// Reference (digit layout)
    #[serde(default)]
    pub refer: i32,
    /// Digit arrangement (0: horizontal, 1: vertical)
    #[serde(default)]
    pub align: i32,
}

fn default_ten() -> i32 {
    10
}

/// Text font definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontDef {
    /// Font definition ID
    pub id: i32,
    /// Font name
    #[serde(default)]
    pub name: String,
    /// Font path (TTF file)
    #[serde(default)]
    pub path: String,
    /// Font size
    #[serde(default = "default_font_size")]
    pub size: i32,
}

fn default_font_size() -> i32 {
    24
}

/// Base element common fields
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementBase {
    /// Element type
    #[serde(rename = "type", default)]
    pub element_type: Option<String>,
    /// Destination X
    #[serde(default)]
    pub x: i32,
    /// Destination Y
    #[serde(default)]
    pub y: i32,
    /// Destination width
    #[serde(default)]
    pub w: i32,
    /// Destination height
    #[serde(default)]
    pub h: i32,
    /// Timer ID for animation (-1 = always visible)
    #[serde(default = "default_timer")]
    pub timer: i32,
    /// Condition operations
    #[serde(default, rename = "op")]
    pub operations: Vec<i32>,
    /// Offset ID for position adjustment
    #[serde(default)]
    pub offset: i32,
    /// Draw priority (higher = drawn later)
    #[serde(default)]
    pub draw: i32,
    /// Blend mode (0: normal, 1: add)
    #[serde(default)]
    pub blend: i32,
    /// Filter mode (0: nearest, 1: linear)
    #[serde(default)]
    pub filter: i32,
    /// Reference for conditional value
    #[serde(default)]
    pub refer: i32,
}

fn default_timer() -> i32 {
    -1
}

/// Animation destination (keyframe)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Destination {
    /// Time in milliseconds
    #[serde(default)]
    pub time: i32,
    /// X position
    #[serde(default)]
    pub x: i32,
    /// Y position
    #[serde(default)]
    pub y: i32,
    /// Width
    #[serde(default)]
    pub w: i32,
    /// Height
    #[serde(default)]
    pub h: i32,
    /// Opacity (0-255)
    #[serde(default = "default_opacity")]
    pub a: i32,
    /// Red (0-255)
    #[serde(default = "default_color")]
    pub r: i32,
    /// Green (0-255)
    #[serde(default = "default_color")]
    pub g: i32,
    /// Blue (0-255)
    #[serde(default = "default_color")]
    pub b: i32,
    /// Angle (degrees)
    #[serde(default)]
    pub angle: i32,
    /// Acceleration type
    #[serde(default)]
    pub acc: i32,
}

fn default_opacity() -> i32 {
    255
}

fn default_color() -> i32 {
    255
}

/// Image element (static or animated)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Image definition ID
    #[serde(default)]
    pub id: i32,
    /// Animation cycle duration (ms)
    #[serde(default)]
    pub cycle: i32,
    /// Loop type (0: once, 1: loop)
    #[serde(rename = "loop", default)]
    pub loop_type: i32,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
    /// Stretch mode (for specific elements)
    #[serde(default)]
    pub stretch: i32,
}

/// Number element
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NumberElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Value definition ID
    #[serde(default)]
    pub id: i32,
    /// Value source reference
    #[serde(default)]
    pub value: i32,
    /// Number of digits
    #[serde(default = "default_digit")]
    pub digit: i32,
    /// Zero padding (0: no, 1: yes)
    #[serde(default)]
    pub padding: i32,
    /// Alignment (0: left, 1: center, 2: right)
    #[serde(default)]
    pub align: i32,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
}

fn default_digit() -> i32 {
    4
}

/// Text element
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Font definition ID
    #[serde(default)]
    pub font: i32,
    /// Text string ID (for predefined texts)
    #[serde(rename = "st", default)]
    pub string_id: i32,
    /// Direct text content
    #[serde(default)]
    pub text: String,
    /// Alignment (0: left, 1: center, 2: right)
    #[serde(default)]
    pub align: i32,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
}

/// Slider/gauge element
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SliderElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Image definition ID
    #[serde(default)]
    pub id: i32,
    /// Value source reference
    #[serde(default)]
    pub value: i32,
    /// Direction (0: right, 1: down, 2: left, 3: up)
    #[serde(default)]
    pub direction: i32,
    /// Range type
    #[serde(default)]
    pub range: i32,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
}

/// Graph element
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Graph type
    #[serde(default)]
    pub id: i32,
    /// Value source reference
    #[serde(default)]
    pub value: i32,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
}

/// Judge element (for judgment display)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JudgeElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Image definitions per judge result
    #[serde(default)]
    pub images: Vec<i32>,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
    /// Index (player side, 0=1P, 1=2P)
    #[serde(default)]
    pub index: i32,
}

/// Note element definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteElement {
    /// Lane index
    #[serde(default)]
    pub lane: i32,
    /// Image definition ID
    #[serde(default)]
    pub id: i32,
    /// Destination positions (for lane placement)
    #[serde(default)]
    pub dst: Vec<Destination>,
}

/// Note skin definition (for play skins)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteSkin {
    /// Normal notes per lane
    #[serde(default)]
    pub note: Vec<NoteElement>,
    /// LN start notes per lane
    #[serde(default)]
    pub lnstart: Vec<NoteElement>,
    /// LN body per lane
    #[serde(default)]
    pub lnbody: Vec<NoteElement>,
    /// LN end notes per lane
    #[serde(default)]
    pub lnend: Vec<NoteElement>,
    /// LN active body per lane
    #[serde(default)]
    pub lnactive: Vec<NoteElement>,
    /// Mine notes per lane
    #[serde(default)]
    pub mine: Vec<NoteElement>,
    /// Hidden notes per lane (HIDDEN+)
    #[serde(default)]
    pub hidden: Vec<NoteElement>,
    /// Processed notes per lane
    #[serde(default)]
    pub processed: Vec<NoteElement>,
}

/// Gauge element definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GaugeSkin {
    /// Gauge node images (filled parts)
    #[serde(default)]
    pub nodes: Vec<ImageElement>,
    /// Gauge parts (background, border, etc.)
    #[serde(default)]
    pub parts: Vec<ImageElement>,
}

/// BGA element definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BgaElement {
    #[serde(flatten)]
    pub base: ElementBase,
    /// Animation destinations
    #[serde(default)]
    pub dst: Vec<Destination>,
}

/// Custom property option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyOption {
    /// Option name
    pub name: String,
    /// Option value (operation code)
    pub value: i32,
}

/// Custom property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProperty {
    /// Property name
    pub name: String,
    /// Property type (0: item, 1: slider, 2: file)
    #[serde(rename = "type", default)]
    pub property_type: i32,
    /// Options (for type 0)
    #[serde(default)]
    pub options: Vec<PropertyOption>,
    /// Minimum value (for type 1)
    #[serde(default)]
    pub min: i32,
    /// Maximum value (for type 1)
    #[serde(default)]
    pub max: i32,
    /// Default value
    #[serde(default)]
    pub def: i32,
    /// Operation code to set
    #[serde(rename = "op", default)]
    pub operation: i32,
}

/// Offset definition for position/size adjustment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OffsetDef {
    /// Offset ID
    pub id: i32,
    /// Name for user configuration
    #[serde(default)]
    pub name: String,
    /// X offset
    #[serde(default)]
    pub x: i32,
    /// Y offset
    #[serde(default)]
    pub y: i32,
    /// Width offset
    #[serde(default)]
    pub w: i32,
    /// Height offset
    #[serde(default)]
    pub h: i32,
    /// Angle offset
    #[serde(default)]
    pub angle: i32,
}

/// Complete beatoraja JSON skin structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeatorajaSkin {
    /// Skin header
    #[serde(flatten)]
    pub header: SkinHeader,
    /// Image sources
    #[serde(default)]
    pub source: Vec<ImageSource>,
    /// Image definitions
    #[serde(default)]
    pub image: Vec<ImageDef>,
    /// Image sets
    #[serde(default)]
    pub imageset: Vec<ImageSet>,
    /// Value (number) definitions
    #[serde(default)]
    pub value: Vec<ValueDef>,
    /// Font definitions
    #[serde(default)]
    pub font: Vec<FontDef>,
    /// Image elements
    #[serde(rename = "destination", default)]
    pub images: Vec<ImageElement>,
    /// Number elements
    #[serde(default)]
    pub number: Vec<NumberElement>,
    /// Text elements
    #[serde(default)]
    pub text: Vec<TextElement>,
    /// Slider elements
    #[serde(default)]
    pub slider: Vec<SliderElement>,
    /// Graph elements
    #[serde(default)]
    pub graph: Vec<GraphElement>,
    /// Judge elements
    #[serde(default)]
    pub judge: Vec<JudgeElement>,
    /// BGA element
    #[serde(default)]
    pub bga: Vec<BgaElement>,
    /// Note skin (play skins only)
    #[serde(default)]
    pub note: Option<NoteSkin>,
    /// Gauge skin (play skins only)
    #[serde(default)]
    pub gauge: Option<GaugeSkin>,
    /// Custom properties
    #[serde(default)]
    pub property: Vec<CustomProperty>,
    /// Offset definitions
    #[serde(default)]
    pub offset: Vec<OffsetDef>,
    /// Custom options (runtime values)
    #[serde(default)]
    pub customoption: HashMap<String, i32>,
    /// Custom timers (runtime values)
    #[serde(default)]
    pub customtimer: HashMap<String, i32>,
}

impl BeatorajaSkin {
    /// Get image source by ID
    pub fn get_source(&self, id: i32) -> Option<&ImageSource> {
        self.source.iter().find(|s| s.id == id)
    }

    /// Get image definition by ID
    pub fn get_image(&self, id: i32) -> Option<&ImageDef> {
        self.image.iter().find(|i| i.id == id)
    }

    /// Get image set by ID
    pub fn get_imageset(&self, id: i32) -> Option<&ImageSet> {
        self.imageset.iter().find(|s| s.id == id)
    }

    /// Get value definition by ID
    pub fn get_value(&self, id: i32) -> Option<&ValueDef> {
        self.value.iter().find(|v| v.id == id)
    }

    /// Get font definition by ID
    pub fn get_font(&self, id: i32) -> Option<&FontDef> {
        self.font.iter().find(|f| f.id == id)
    }

    /// Get offset definition by ID
    pub fn get_offset(&self, id: i32) -> Option<&OffsetDef> {
        self.offset.iter().find(|o| o.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_type_constants() {
        assert_eq!(SkinType::PLAY_7KEYS.0, 0);
        assert_eq!(SkinType::MUSIC_SELECT.0, 5);
        assert_eq!(SkinType::RESULT.0, 7);
    }

    #[test]
    fn test_default_header() {
        let header = SkinHeader::default();
        assert_eq!(header.w, 1280);
        assert_eq!(header.h, 720);
    }

    #[test]
    fn test_deserialize_minimal_skin() {
        let json = r#"{
            "name": "Test Skin",
            "type": 0,
            "w": 1920,
            "h": 1080
        }"#;

        let skin: BeatorajaSkin = serde_json::from_str(json).unwrap();
        assert_eq!(skin.header.name, "Test Skin");
        assert_eq!(skin.header.skin_type, 0);
        assert_eq!(skin.header.w, 1920);
        assert_eq!(skin.header.h, 1080);
    }

    #[test]
    fn test_deserialize_with_sources() {
        let json = r#"{
            "name": "Test",
            "type": 0,
            "w": 1280,
            "h": 720,
            "source": [
                {"id": 0, "path": "bg.png"},
                {"id": 1, "path": "notes.png"}
            ],
            "image": [
                {"id": 0, "src": 0, "x": 0, "y": 0, "w": 1280, "h": 720}
            ]
        }"#;

        let skin: BeatorajaSkin = serde_json::from_str(json).unwrap();
        assert_eq!(skin.source.len(), 2);
        assert_eq!(skin.image.len(), 1);
        assert_eq!(skin.get_source(0).unwrap().path, "bg.png");
        assert_eq!(skin.get_image(0).unwrap().src, 0);
    }
}
