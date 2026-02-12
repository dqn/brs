// SkinHidden / SkinLiftCover ported from SkinHidden.java / SkinLiftCover.java.
//
// Hidden cover and lift cover layers that obscure notes in the play field.

use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// SkinHidden
// ---------------------------------------------------------------------------

/// Hidden cover layer (covers notes from the top).
#[derive(Debug, Clone)]
pub struct SkinHidden {
    pub base: SkinObjectBase,
    /// Image source references.
    pub images: Vec<i32>,
    /// Target Y coordinate below which notes disappear.
    pub disapear_line: f32,
    /// Whether to adjust line with lift offset.
    pub link_lift: bool,
    /// Animation timer.
    pub timer: Option<i32>,
    /// Animation cycle in ms.
    pub cycle: i32,
}

impl Default for SkinHidden {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            images: Vec::new(),
            disapear_line: 0.0,
            link_lift: false,
            timer: None,
            cycle: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// SkinLiftCover
// ---------------------------------------------------------------------------

/// Lift cover (similar to hidden but from bottom).
#[derive(Debug, Clone)]
pub struct SkinLiftCover {
    pub base: SkinObjectBase,
    /// Image source references.
    pub images: Vec<i32>,
    /// Target Y coordinate below which notes disappear.
    pub disapear_line: f32,
    /// Whether to adjust line with lift offset.
    pub link_lift: bool,
    /// Animation timer.
    pub timer: Option<i32>,
    /// Animation cycle in ms.
    pub cycle: i32,
}

impl Default for SkinLiftCover {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            images: Vec::new(),
            disapear_line: 0.0,
            link_lift: false,
            timer: None,
            cycle: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_hidden_default() {
        let hidden = SkinHidden::default();
        assert!(hidden.images.is_empty());
        assert!((hidden.disapear_line - 0.0).abs() < f32::EPSILON);
        assert!(!hidden.link_lift);
        assert!(hidden.timer.is_none());
        assert_eq!(hidden.cycle, 0);
    }

    #[test]
    fn test_skin_lift_cover_default() {
        let cover = SkinLiftCover::default();
        assert!(cover.images.is_empty());
        assert!((cover.disapear_line - 0.0).abs() < f32::EPSILON);
        assert!(!cover.link_lift);
        assert!(cover.timer.is_none());
        assert_eq!(cover.cycle, 0);
    }

    #[test]
    fn test_skin_hidden_with_images() {
        let hidden = SkinHidden {
            images: vec![1, 2, 3],
            disapear_line: 100.0,
            link_lift: true,
            timer: Some(41),
            cycle: 500,
            ..Default::default()
        };
        assert_eq!(hidden.images.len(), 3);
        assert!((hidden.disapear_line - 100.0).abs() < f32::EPSILON);
        assert!(hidden.link_lift);
        assert_eq!(hidden.timer, Some(41));
        assert_eq!(hidden.cycle, 500);
    }
}
