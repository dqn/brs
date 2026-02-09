// SkinSlider ported from SkinSlider.java.
//
// Interactive slider control with direction-based movement and optional
// mouse input handling. Value read from FloatProperty, optionally writable.

use crate::image_handle::ImageHandle;
use crate::property_id::FloatId;
use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// SkinSlider
// ---------------------------------------------------------------------------

/// Movement direction for slider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliderDirection {
    /// Value increases upward.
    #[default]
    Up = 0,
    /// Value increases rightward.
    Right = 1,
    /// Value increases downward.
    Down = 2,
    /// Value increases leftward.
    Left = 3,
}

impl SliderDirection {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Right,
            2 => Self::Down,
            3 => Self::Left,
            _ => Self::Up,
        }
    }
}

/// A skin slider object that displays a thumb image moving along a range.
#[derive(Debug, Clone)]
pub struct SkinSlider {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Source image for the slider thumb.
    pub source_images: Vec<ImageHandle>,
    pub source_timer: Option<i32>,
    pub source_cycle: i32,
    /// Movement direction.
    pub direction: SliderDirection,
    /// Maximum pixel range of movement.
    pub range: i32,
    /// Float property ID for the current value (0.0-1.0).
    pub ref_id: Option<FloatId>,
    /// Whether the slider is interactive (mouse-writable).
    pub changeable: bool,
    /// Optional min/max range for RateProperty conversion.
    pub range_min: Option<i32>,
    pub range_max: Option<i32>,
}

impl Default for SkinSlider {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            source_images: Vec::new(),
            source_timer: None,
            source_cycle: 0,
            direction: SliderDirection::Up,
            range: 0,
            ref_id: None,
            changeable: false,
            range_min: None,
            range_max: None,
        }
    }
}

impl SkinSlider {
    pub fn new(ref_id: FloatId, direction: i32, range: i32, changeable: bool) -> Self {
        Self {
            ref_id: Some(ref_id),
            direction: SliderDirection::from_i32(direction),
            range,
            changeable,
            ..Default::default()
        }
    }

    /// Creates a slider with RateProperty conversion (min/max integer range).
    pub fn with_rate(ref_id: FloatId, direction: i32, range: i32, min: i32, max: i32) -> Self {
        Self {
            ref_id: Some(ref_id),
            direction: SliderDirection::from_i32(direction),
            range,
            range_min: Some(min),
            range_max: Some(max),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let slider = SkinSlider::default();
        assert_eq!(slider.direction, SliderDirection::Up);
        assert_eq!(slider.range, 0);
        assert!(!slider.changeable);
    }

    #[test]
    fn test_new() {
        let slider = SkinSlider::new(FloatId(17), 1, 200, true);
        assert_eq!(slider.ref_id, Some(FloatId(17)));
        assert_eq!(slider.direction, SliderDirection::Right);
        assert_eq!(slider.range, 200);
        assert!(slider.changeable);
    }

    #[test]
    fn test_with_rate() {
        let slider = SkinSlider::with_rate(FloatId(6), 2, 150, 0, 100);
        assert_eq!(slider.direction, SliderDirection::Down);
        assert_eq!(slider.range_min, Some(0));
        assert_eq!(slider.range_max, Some(100));
        assert!(!slider.changeable);
    }

    #[test]
    fn test_slider_direction() {
        assert_eq!(SliderDirection::from_i32(0), SliderDirection::Up);
        assert_eq!(SliderDirection::from_i32(1), SliderDirection::Right);
        assert_eq!(SliderDirection::from_i32(2), SliderDirection::Down);
        assert_eq!(SliderDirection::from_i32(3), SliderDirection::Left);
        assert_eq!(SliderDirection::from_i32(99), SliderDirection::Up);
    }
}
