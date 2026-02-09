// SkinText ported from SkinText.java.
//
// Displays dynamic text with formatting options including alignment,
// overflow handling, outline, and shadow effects.

use crate::property_id::StringId;
use crate::skin_object::{Color, SkinObjectBase};

// ---------------------------------------------------------------------------
// SkinText
// ---------------------------------------------------------------------------

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left = 0,
    Center = 1,
    Right = 2,
}

impl TextAlign {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Center,
            2 => Self::Right,
            _ => Self::Left,
        }
    }
}

/// Overflow handling mode for text that exceeds the region width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOverflow {
    /// Text overflows beyond the region (no clipping).
    #[default]
    Overflow = 0,
    /// Text is shrunk to fit the region width.
    Shrink = 1,
    /// Text is truncated at the region boundary.
    Truncate = 2,
}

impl TextOverflow {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Shrink,
            2 => Self::Truncate,
            _ => Self::Overflow,
        }
    }
}

/// Shadow effect configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct TextShadow {
    /// Shadow color.
    pub color: Color,
    /// Shadow offset (x, y).
    pub offset_x: f32,
    pub offset_y: f32,
    /// Shadow blur/smoothness.
    pub smoothness: f32,
}

/// A skin text object that displays dynamic string content.
#[derive(Debug, Clone)]
pub struct SkinText {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// String property ID for the text to display.
    pub ref_id: Option<StringId>,
    /// Constant text (used when ref_id is None).
    pub constant_text: Option<String>,
    /// Text alignment.
    pub align: TextAlign,
    /// Whether the text is editable (for input fields).
    pub editable: bool,
    /// Whether text wrapping is enabled.
    pub wrapping: bool,
    /// Overflow handling mode.
    pub overflow: TextOverflow,
    /// Outline color and width (None = no outline).
    pub outline_color: Option<Color>,
    pub outline_width: f32,
    /// Shadow effect (None = no shadow).
    pub shadow: Option<TextShadow>,
    /// Font path or identifier (resolved at load time).
    pub font: Option<String>,
    /// Font size.
    pub font_size: f32,
}

impl Default for SkinText {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            ref_id: None,
            constant_text: None,
            align: TextAlign::Left,
            editable: false,
            wrapping: false,
            overflow: TextOverflow::Overflow,
            outline_color: None,
            outline_width: 0.0,
            shadow: None,
            font: None,
            font_size: 24.0,
        }
    }
}

impl SkinText {
    pub fn new(ref_id: StringId) -> Self {
        Self {
            ref_id: Some(ref_id),
            ..Default::default()
        }
    }

    pub fn with_constant(text: String) -> Self {
        Self {
            constant_text: Some(text),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let text = SkinText::default();
        assert!(text.ref_id.is_none());
        assert!(text.constant_text.is_none());
        assert_eq!(text.align, TextAlign::Left);
        assert!(!text.editable);
        assert!(!text.wrapping);
        assert_eq!(text.overflow, TextOverflow::Overflow);
    }

    #[test]
    fn test_new() {
        let text = SkinText::new(StringId(10));
        assert_eq!(text.ref_id, Some(StringId(10)));
    }

    #[test]
    fn test_with_constant() {
        let text = SkinText::with_constant("Hello".to_string());
        assert_eq!(text.constant_text, Some("Hello".to_string()));
        assert!(text.ref_id.is_none());
    }

    #[test]
    fn test_text_align() {
        assert_eq!(TextAlign::from_i32(0), TextAlign::Left);
        assert_eq!(TextAlign::from_i32(1), TextAlign::Center);
        assert_eq!(TextAlign::from_i32(2), TextAlign::Right);
        assert_eq!(TextAlign::from_i32(99), TextAlign::Left);
    }

    #[test]
    fn test_text_overflow() {
        assert_eq!(TextOverflow::from_i32(0), TextOverflow::Overflow);
        assert_eq!(TextOverflow::from_i32(1), TextOverflow::Shrink);
        assert_eq!(TextOverflow::from_i32(2), TextOverflow::Truncate);
    }
}
