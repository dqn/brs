// SkinBar ported from SkinBar.java.
//
// Song selection bar object displaying song info, trophies, lamps, etc.

use crate::skin_image::SkinImage;
use crate::skin_number::SkinNumber;
use crate::skin_object::SkinObjectBase;
use crate::skin_text::SkinText;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const BAR_COUNT: usize = 60;
pub const BAR_TROPHY_COUNT: usize = 3;
pub const BAR_TEXT_COUNT: usize = 11;
pub const BAR_LEVEL_COUNT: usize = 7;
pub const BAR_LABEL_COUNT: usize = 5;
pub const BAR_LAMP_COUNT: usize = 11;

// ---------------------------------------------------------------------------
// SkinBar
// ---------------------------------------------------------------------------

/// Song selection bar object.
#[derive(Debug, Clone)]
pub struct SkinBar {
    pub base: SkinObjectBase,
    /// Position index.
    pub position: i32,
    /// Selected bar images [BAR_COUNT].
    pub bar_image_on: Vec<Option<SkinImage>>,
    /// Unselected bar images [BAR_COUNT].
    pub bar_image_off: Vec<Option<SkinImage>>,
    /// Trophy images [BAR_TROPHY_COUNT].
    pub trophy: Vec<Option<SkinImage>>,
    /// Bar text variants [BAR_TEXT_COUNT].
    pub text: Vec<Option<SkinText>>,
    /// Level numbers [BAR_LEVEL_COUNT].
    pub bar_level: Vec<Option<SkinNumber>>,
    /// Difficulty labels [BAR_LABEL_COUNT].
    pub label: Vec<Option<SkinImage>>,
    /// Clear lamps [BAR_LAMP_COUNT].
    pub lamp: Vec<Option<SkinImage>>,
    /// Player lamp [BAR_LAMP_COUNT].
    pub my_lamp: Vec<Option<SkinImage>>,
    /// Rival lamp [BAR_LAMP_COUNT].
    pub rival_lamp: Vec<Option<SkinImage>>,
}

impl Default for SkinBar {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            position: 0,
            bar_image_on: vec![None; BAR_COUNT],
            bar_image_off: vec![None; BAR_COUNT],
            trophy: vec![None; BAR_TROPHY_COUNT],
            text: vec![None; BAR_TEXT_COUNT],
            bar_level: vec![None; BAR_LEVEL_COUNT],
            label: vec![None; BAR_LABEL_COUNT],
            lamp: vec![None; BAR_LAMP_COUNT],
            my_lamp: vec![None; BAR_LAMP_COUNT],
            rival_lamp: vec![None; BAR_LAMP_COUNT],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_bar_default() {
        let bar = SkinBar::default();
        assert_eq!(bar.position, 0);
        assert_eq!(bar.bar_image_on.len(), BAR_COUNT);
        assert_eq!(bar.bar_image_off.len(), BAR_COUNT);
        assert_eq!(bar.trophy.len(), BAR_TROPHY_COUNT);
        assert_eq!(bar.text.len(), BAR_TEXT_COUNT);
        assert_eq!(bar.bar_level.len(), BAR_LEVEL_COUNT);
        assert_eq!(bar.label.len(), BAR_LABEL_COUNT);
        assert_eq!(bar.lamp.len(), BAR_LAMP_COUNT);
        assert_eq!(bar.my_lamp.len(), BAR_LAMP_COUNT);
        assert_eq!(bar.rival_lamp.len(), BAR_LAMP_COUNT);
    }

    #[test]
    fn test_skin_bar_all_none() {
        let bar = SkinBar::default();
        for img in &bar.bar_image_on {
            assert!(img.is_none());
        }
        for img in &bar.bar_image_off {
            assert!(img.is_none());
        }
        for t in &bar.trophy {
            assert!(t.is_none());
        }
    }

    #[test]
    fn test_skin_bar_set_image() {
        let mut bar = SkinBar::default();
        bar.bar_image_on[0] = Some(SkinImage::from_reference(1));
        bar.bar_image_off[0] = Some(SkinImage::from_reference(2));
        bar.trophy[0] = Some(SkinImage::from_reference(3));
        assert!(bar.bar_image_on[0].is_some());
        assert!(bar.bar_image_off[0].is_some());
        assert!(bar.trophy[0].is_some());
    }
}
