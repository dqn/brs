// Play state skin configuration.
//
// Aggregates play-specific skin objects (notes, covers, judges, BGA).

use crate::skin_bga::SkinBga;
use crate::skin_hidden::{SkinHidden, SkinLiftCover};
use crate::skin_judge::SkinJudge;
use crate::skin_note::SkinNote;

/// Play state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct PlaySkinConfig {
    pub note: Option<SkinNote>,
    pub hidden_cover: Option<SkinHidden>,
    pub lift_cover: Option<SkinLiftCover>,
    pub judges: Vec<SkinJudge>,
    pub bga: Option<SkinBga>,
    /// PLAYSTART command value (ms).
    pub playstart: i32,
    /// LOADSTART command value (ms).
    pub loadstart: i32,
    /// LOADEND command value (ms).
    pub loadend: i32,
    /// FINISHMARGIN command value (ms).
    pub finish_margin: i32,
    /// JUDGETIMER command value (ms).
    pub judge_timer: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = PlaySkinConfig::default();
        assert!(config.note.is_none());
        assert!(config.hidden_cover.is_none());
        assert!(config.lift_cover.is_none());
        assert!(config.judges.is_empty());
        assert!(config.bga.is_none());
    }

    #[test]
    fn test_with_note() {
        let config = PlaySkinConfig {
            note: Some(SkinNote::default()),
            ..Default::default()
        };
        assert!(config.note.is_some());
    }
}
