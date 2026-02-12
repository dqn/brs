// SkinNote ported from SkinNote.java.
//
// Displays note objects on the play field including normal notes, long notes,
// mine notes, hidden notes, and measure/BPM/stop/time lines.

use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// LN texture indices
// ---------------------------------------------------------------------------

/// LN texture indices (0-9) matching Java SkinNote.SkinLane.longnote array.
pub const LN_END: usize = 0;
pub const LN_START: usize = 1;
pub const LN_BODY_ACTIVE: usize = 2;
pub const LN_BODY_INACTIVE: usize = 3;
pub const HCN_END: usize = 4;
pub const HCN_START: usize = 5;
pub const HCN_BODY_ACTIVE: usize = 6;
pub const HCN_BODY_INACTIVE: usize = 7;
pub const HCN_BODY_REACTIVE: usize = 8;
pub const HCN_BODY_DAMAGE: usize = 9;
pub const LN_TYPE_COUNT: usize = 10;

// ---------------------------------------------------------------------------
// SkinLane
// ---------------------------------------------------------------------------

/// Per-lane note graphics.
#[derive(Debug, Clone)]
pub struct SkinLane {
    /// Normal note source image reference.
    pub note: Option<i32>,
    /// LN type texture references [0-9].
    pub longnote: [Option<i32>; LN_TYPE_COUNT],
    /// Mine note source.
    pub mine_note: Option<i32>,
    /// Hidden note source.
    pub hidden_note: Option<i32>,
    /// Processed note source.
    pub processed_note: Option<i32>,
    /// Note scale multiplier.
    pub scale: f32,
    /// Secondary note region height offset.
    pub dst_note2: i32,
}

impl Default for SkinLane {
    fn default() -> Self {
        Self {
            note: None,
            longnote: [None; LN_TYPE_COUNT],
            mine_note: None,
            hidden_note: None,
            processed_note: None,
            scale: 1.0,
            dst_note2: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// SkinNote
// ---------------------------------------------------------------------------

/// Note rendering object for play state.
#[derive(Debug, Clone, Default)]
pub struct SkinNote {
    pub base: SkinObjectBase,
    /// Per-lane configurations.
    pub lanes: Vec<SkinLane>,
    /// Line image source (measure lines).
    pub line_image: Option<i32>,
    /// BPM line image source.
    pub bpm_line_image: Option<i32>,
    /// Stop line image source.
    pub stop_line_image: Option<i32>,
    /// Time line image source.
    pub time_line_image: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_lane_default() {
        let lane = SkinLane::default();
        assert!(lane.note.is_none());
        assert_eq!(lane.longnote, [None; LN_TYPE_COUNT]);
        assert!(lane.mine_note.is_none());
        assert!(lane.hidden_note.is_none());
        assert!(lane.processed_note.is_none());
        assert!((lane.scale - 1.0).abs() < f32::EPSILON);
        assert_eq!(lane.dst_note2, 0);
    }

    #[test]
    fn test_skin_note_default() {
        let note = SkinNote::default();
        assert!(note.lanes.is_empty());
        assert!(note.line_image.is_none());
        assert!(note.bpm_line_image.is_none());
        assert!(note.stop_line_image.is_none());
        assert!(note.time_line_image.is_none());
    }

    #[test]
    fn test_skin_lane_with_longnote() {
        let mut lane = SkinLane::default();
        lane.longnote[LN_START] = Some(1);
        lane.longnote[LN_END] = Some(2);
        lane.longnote[LN_BODY_ACTIVE] = Some(3);
        assert_eq!(lane.longnote[LN_START], Some(1));
        assert_eq!(lane.longnote[LN_END], Some(2));
        assert_eq!(lane.longnote[LN_BODY_ACTIVE], Some(3));
        assert!(lane.longnote[HCN_END].is_none());
    }

    #[test]
    fn test_skin_note_with_lanes() {
        let mut note = SkinNote::default();
        note.lanes.push(SkinLane {
            note: Some(10),
            ..Default::default()
        });
        note.lanes.push(SkinLane {
            note: Some(20),
            scale: 2.0,
            ..Default::default()
        });
        assert_eq!(note.lanes.len(), 2);
        assert_eq!(note.lanes[0].note, Some(10));
        assert_eq!(note.lanes[1].note, Some(20));
        assert!((note.lanes[1].scale - 2.0).abs() < f32::EPSILON);
    }
}
