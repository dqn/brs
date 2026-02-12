// SkinJudge ported from SkinJudge.java.
//
// Displays judgment feedback (PERFECT, GREAT, etc.) with combo counters.

use crate::skin_image::SkinImage;
use crate::skin_number::SkinNumber;
use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// Judgment display indices
// ---------------------------------------------------------------------------

pub const JUDGE_PERFECT: usize = 0;
pub const JUDGE_GREAT: usize = 1;
pub const JUDGE_GOOD: usize = 2;
pub const JUDGE_BAD: usize = 3;
pub const JUDGE_POOR: usize = 4;
pub const JUDGE_MISS: usize = 5;
pub const JUDGE_MAX: usize = 6;
pub const JUDGE_COUNT: usize = 7;

// ---------------------------------------------------------------------------
// SkinJudge
// ---------------------------------------------------------------------------

/// Judgment display object.
#[derive(Debug, Clone, Default)]
pub struct SkinJudge {
    pub base: SkinObjectBase,
    /// Judgment images [0-6]: PG, GR, GD, BD, PR, MS, MAX.
    pub judge_images: [Option<SkinImage>; JUDGE_COUNT],
    /// Combo counter numbers [0-6].
    pub judge_counts: [Option<SkinNumber>; JUDGE_COUNT],
    /// Player index (0=1P, 1=2P).
    pub player: i32,
    /// Center count relative to judge image.
    pub shift: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_judge_default() {
        let judge = SkinJudge::default();
        assert_eq!(judge.player, 0);
        assert!(!judge.shift);
        for img in &judge.judge_images {
            assert!(img.is_none());
        }
        for cnt in &judge.judge_counts {
            assert!(cnt.is_none());
        }
    }

    #[test]
    fn test_skin_judge_with_images() {
        let mut judge = SkinJudge::default();
        judge.judge_images[JUDGE_PERFECT] = Some(SkinImage::from_reference(1));
        judge.judge_images[JUDGE_GREAT] = Some(SkinImage::from_reference(2));
        judge.player = 1;
        judge.shift = true;

        assert!(judge.judge_images[JUDGE_PERFECT].is_some());
        assert!(judge.judge_images[JUDGE_GREAT].is_some());
        assert!(judge.judge_images[JUDGE_GOOD].is_none());
        assert_eq!(judge.player, 1);
        assert!(judge.shift);
    }

    #[test]
    fn test_judge_indices() {
        assert_eq!(JUDGE_PERFECT, 0);
        assert_eq!(JUDGE_MAX, 6);
        assert_eq!(JUDGE_COUNT, 7);
    }
}
