// SkinFloat ported from SkinFloatNumber.java.
//
// Displays floating-point values as digit glyphs with integer and
// fractional parts.

use crate::property_id::FloatId;
use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// SkinFloat
// ---------------------------------------------------------------------------

/// Floating-point number display object.
#[derive(Debug, Clone)]
pub struct SkinFloat {
    pub base: SkinObjectBase,
    /// Positive digit image sources.
    pub image_sources: Vec<i32>,
    /// Negative digit image sources (optional).
    pub minus_image_sources: Vec<i32>,
    /// Float property reference.
    pub ref_id: Option<FloatId>,
    /// Integer digit count.
    pub iketa: i32,
    /// Fractional digit count.
    pub fketa: i32,
    /// Sign visibility.
    pub sign_visible: bool,
    /// Value multiplier.
    pub gain: f32,
    /// Zero padding: 0=none, 1=front, 2=rear.
    pub zero_padding: i32,
    /// Alignment: 0=left, 1=right, 2=center.
    pub align: i32,
}

impl Default for SkinFloat {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            image_sources: Vec::new(),
            minus_image_sources: Vec::new(),
            ref_id: None,
            iketa: 1,
            fketa: 0,
            sign_visible: false,
            gain: 1.0,
            zero_padding: 0,
            align: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let f = SkinFloat::default();
        assert!(f.image_sources.is_empty());
        assert!(f.minus_image_sources.is_empty());
        assert!(f.ref_id.is_none());
        assert_eq!(f.iketa, 1);
        assert_eq!(f.fketa, 0);
        assert!(!f.sign_visible);
        assert!((f.gain - 1.0).abs() < f32::EPSILON);
        assert_eq!(f.zero_padding, 0);
        assert_eq!(f.align, 0);
    }

    #[test]
    fn test_custom() {
        let f = SkinFloat {
            ref_id: Some(FloatId(102)),
            iketa: 3,
            fketa: 2,
            gain: 100.0,
            sign_visible: true,
            ..Default::default()
        };
        assert_eq!(f.ref_id, Some(FloatId(102)));
        assert_eq!(f.iketa, 3);
        assert_eq!(f.fketa, 2);
        assert!((f.gain - 100.0).abs() < f32::EPSILON);
        assert!(f.sign_visible);
    }
}
