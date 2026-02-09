// SkinNumber ported from SkinNumber.java.
//
// Displays integer values as digit glyphs. Supports positive/negative images,
// zero-padding, spacing, and alignment.

use crate::property_id::IntegerId;
use crate::skin_object::{SkinObjectBase, SkinOffset};

// ---------------------------------------------------------------------------
// SkinNumber
// ---------------------------------------------------------------------------

/// Alignment for digit display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberAlign {
    /// Right-aligned (default): leading empty digits are at the left.
    #[default]
    Right = 0,
    /// Left-aligned: digits shift left when leading zeros are hidden.
    Left = 1,
    /// Center-aligned.
    Center = 2,
}

impl NumberAlign {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Left,
            2 => Self::Center,
            _ => Self::Right,
        }
    }
}

/// Zero-padding mode for digit display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZeroPadding {
    /// No padding: leading zeros are hidden.
    #[default]
    None = 0,
    /// Pad with zero glyphs (image index 0).
    Zero = 1,
    /// Pad with space glyphs (image index 10).
    Space = 2,
}

impl ZeroPadding {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Zero,
            2 => Self::Space,
            _ => Self::None,
        }
    }
}

/// A skin number object that displays integer values as digit glyphs.
///
/// Image layout per source set: indices 0-9 = digits, 10 = period/space,
/// 11 = minus sign.
#[derive(Debug, Clone)]
pub struct SkinNumber {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Integer property ID for the value to display.
    pub ref_id: Option<IntegerId>,
    /// Number of digit positions to display.
    pub keta: i32,
    /// Zero-padding mode.
    pub zero_padding: ZeroPadding,
    /// Pixel spacing between digits.
    pub space: i32,
    /// Alignment mode.
    pub align: NumberAlign,
    /// Source image set for positive/unsigned values.
    /// Timer and cycle for animation of the digit images.
    pub image_timer: Option<i32>,
    pub image_cycle: i32,
    /// Source image set for negative values (optional).
    pub has_minus_images: bool,
    /// Per-digit offsets (optional, length should match keta).
    pub digit_offsets: Vec<SkinOffset>,
}

impl Default for SkinNumber {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            ref_id: None,
            keta: 1,
            zero_padding: ZeroPadding::None,
            space: 0,
            align: NumberAlign::Right,
            image_timer: None,
            image_cycle: 0,
            has_minus_images: false,
            digit_offsets: Vec::new(),
        }
    }
}

impl SkinNumber {
    pub fn new(ref_id: IntegerId, keta: i32, zero_padding: i32, space: i32, align: i32) -> Self {
        Self {
            ref_id: Some(ref_id),
            keta,
            zero_padding: ZeroPadding::from_i32(zero_padding),
            space,
            align: NumberAlign::from_i32(align),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let num = SkinNumber::default();
        assert_eq!(num.keta, 1);
        assert_eq!(num.zero_padding, ZeroPadding::None);
        assert_eq!(num.align, NumberAlign::Right);
    }

    #[test]
    fn test_new() {
        let num = SkinNumber::new(IntegerId(71), 5, 2, 1, 1);
        assert_eq!(num.ref_id, Some(IntegerId(71)));
        assert_eq!(num.keta, 5);
        assert_eq!(num.zero_padding, ZeroPadding::Space);
        assert_eq!(num.space, 1);
        assert_eq!(num.align, NumberAlign::Left);
    }

    #[test]
    fn test_number_align() {
        assert_eq!(NumberAlign::from_i32(0), NumberAlign::Right);
        assert_eq!(NumberAlign::from_i32(1), NumberAlign::Left);
        assert_eq!(NumberAlign::from_i32(2), NumberAlign::Center);
        assert_eq!(NumberAlign::from_i32(99), NumberAlign::Right);
    }

    #[test]
    fn test_zero_padding() {
        assert_eq!(ZeroPadding::from_i32(0), ZeroPadding::None);
        assert_eq!(ZeroPadding::from_i32(1), ZeroPadding::Zero);
        assert_eq!(ZeroPadding::from_i32(2), ZeroPadding::Space);
        assert_eq!(ZeroPadding::from_i32(-1), ZeroPadding::None);
    }
}
