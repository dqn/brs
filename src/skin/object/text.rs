use crate::skin::destination::DestinationSet;
use crate::traits::render::FontId;

/// Skin text object that renders strings.
#[derive(Debug, Clone)]
pub struct TextObject {
    pub id: String,
    /// String property ID (STRING_*).
    pub ref_id: i32,
    /// Font file path (or font reference).
    pub font: String,
    /// Font size.
    pub size: f32,
    /// Text alignment: 0=left, 1=center, 2=right.
    pub align: i32,
    /// Whether to wrap text if too wide.
    pub wrapping: bool,
    /// Overflow behavior.
    pub overflow: i32,
    pub dst: DestinationSet,
    /// Loaded font ID.
    pub font_id: Option<FontId>,
    /// Static text content (for non-property text).
    pub static_text: Option<String>,
}

impl Default for TextObject {
    fn default() -> Self {
        Self {
            id: String::new(),
            ref_id: 0,
            font: String::new(),
            size: 24.0,
            align: 0,
            wrapping: false,
            overflow: 0,
            dst: DestinationSet::default(),
            font_id: None,
            static_text: None,
        }
    }
}
