use serde::{Deserialize, Serialize};

/// Play mode defining the number of lanes and layout.
/// Corresponds to bms.model.Mode in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayMode {
    Beat5K,
    Beat7K,
    Beat10K,
    Beat14K,
    PopN9K,
    Keyboard24K,
    Keyboard24KDouble,
}

impl PlayMode {
    /// Total number of lanes (playable keys including scratch).
    pub fn lane_count(self) -> usize {
        match self {
            Self::Beat5K => 6,
            Self::Beat7K => 8,
            Self::Beat10K => 12,
            Self::Beat14K => 16,
            Self::PopN9K => 9,
            Self::Keyboard24K => 26,
            Self::Keyboard24KDouble => 52,
        }
    }

    /// Total number of input keys (including duplicate scratch keys).
    pub fn key_count(self) -> usize {
        match self {
            Self::Beat5K => 7,
            Self::Beat7K => 9,
            Self::Beat10K => 14,
            Self::Beat14K => 18,
            Self::PopN9K => 9,
            Self::Keyboard24K => 26,
            Self::Keyboard24KDouble => 52,
        }
    }

    /// Number of players (1 or 2).
    pub fn player_count(self) -> usize {
        match self {
            Self::Beat5K | Self::Beat7K | Self::PopN9K | Self::Keyboard24K => 1,
            Self::Beat10K | Self::Beat14K | Self::Keyboard24KDouble => 2,
        }
    }

    /// Number of scratch lanes.
    pub fn scratch_count(self) -> usize {
        match self {
            Self::Beat5K | Self::Beat7K => 1,
            Self::Beat10K | Self::Beat14K => 2,
            Self::PopN9K | Self::Keyboard24K | Self::Keyboard24KDouble => 0,
        }
    }

    /// Whether a given lane index is a scratch lane.
    pub fn is_scratch(self, lane: usize) -> bool {
        self.scratch_index(lane).is_some()
    }

    /// Returns which scratch number a lane is (0-indexed), or None.
    pub fn scratch_index(self, lane: usize) -> Option<usize> {
        match self {
            Self::Beat5K => {
                if lane == 5 {
                    Some(0)
                } else {
                    None
                }
            }
            Self::Beat7K => {
                if lane == 7 {
                    Some(0)
                } else {
                    None
                }
            }
            Self::Beat10K => match lane {
                5 => Some(0),
                11 => Some(1),
                _ => None,
            },
            Self::Beat14K => match lane {
                7 => Some(0),
                15 => Some(1),
                _ => None,
            },
            Self::PopN9K | Self::Keyboard24K | Self::Keyboard24KDouble => None,
        }
    }

    /// Map lane index to player number (0-indexed).
    pub fn lane_to_player(self, lane: usize) -> usize {
        let lanes_per_player = self.lane_count() / self.player_count();
        lane / lanes_per_player
    }

    /// Map lane index to skin offset (beatoraja LaneProperty.laneToSkinOffset).
    pub fn lane_to_skin_offset(self, lane: usize) -> usize {
        match self {
            Self::Beat5K => [1, 2, 3, 4, 5, 0][lane],
            Self::Beat7K => [1, 2, 3, 4, 5, 6, 7, 0][lane],
            Self::Beat10K => [1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0][lane],
            Self::Beat14K => [1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0][lane],
            Self::PopN9K => [1, 2, 3, 4, 5, 6, 7, 8, 9][lane],
            Self::Keyboard24K => lane + 1,
            Self::Keyboard24KDouble => lane % 26 + 1,
        }
    }
}

/// Type of note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteType {
    /// Normal note (single tap).
    Normal,
    /// Long note (hold and release).
    LongNote,
    /// Charge note (hold, no release timing).
    ChargeNote,
    /// Hell charge note (must hold, taking damage on release).
    HellChargeNote,
    /// Mine note (damage on press).
    Mine,
    /// Invisible note (no visual, plays keysound on press).
    Invisible,
}

/// Long note mode setting from BMS header (#LNTYPE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LongNoteMode {
    /// LN type 1: #LNTYPE 1 (RDM style, paired notes).
    #[default]
    LnType1,
    /// Charge note (#LNTYPE 2).
    ChargeNote,
    /// Hell charge note.
    HellChargeNote,
}

/// A single note in the chart.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// Lane index (0-based).
    pub lane: usize,
    /// Note type.
    pub note_type: NoteType,
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// End time in microseconds (for LN/CN/HCN).
    pub end_time_us: i64,
    /// WAV object ID for keysound.
    pub wav_id: u32,
    /// Damage value for mine notes (percentage of gauge).
    pub damage: f64,
}

impl Note {
    pub fn is_long_note(&self) -> bool {
        matches!(
            self.note_type,
            NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote
        )
    }

    /// Duration of a long note in microseconds.
    pub fn duration_us(&self) -> i64 {
        self.end_time_us - self.time_us
    }
}

/// Judge rank type from BMS header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum JudgeRankType {
    /// BMS #RANK (0-4 index).
    BmsRank,
    /// BMS #DEFEXRANK (percentage).
    #[default]
    BmsDefExRank,
    /// BMSON judge_rank (percentage, default 100).
    BmsonJudgeRank,
}

/// Total type from BMS header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TotalType {
    /// BMS #TOTAL (absolute value).
    #[default]
    Bms,
    /// BMSON total (percentage of default).
    Bmson,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_mode_lane_count() {
        assert_eq!(PlayMode::Beat5K.lane_count(), 6);
        assert_eq!(PlayMode::Beat7K.lane_count(), 8);
        assert_eq!(PlayMode::Beat10K.lane_count(), 12);
        assert_eq!(PlayMode::Beat14K.lane_count(), 16);
        assert_eq!(PlayMode::PopN9K.lane_count(), 9);
        assert_eq!(PlayMode::Keyboard24K.lane_count(), 26);
        assert_eq!(PlayMode::Keyboard24KDouble.lane_count(), 52);
    }

    #[test]
    fn play_mode_key_count() {
        assert_eq!(PlayMode::Beat5K.key_count(), 7);
        assert_eq!(PlayMode::Beat7K.key_count(), 9);
        assert_eq!(PlayMode::Beat10K.key_count(), 14);
        assert_eq!(PlayMode::Beat14K.key_count(), 18);
        assert_eq!(PlayMode::PopN9K.key_count(), 9);
        assert_eq!(PlayMode::Keyboard24K.key_count(), 26);
        assert_eq!(PlayMode::Keyboard24KDouble.key_count(), 52);
    }

    #[test]
    fn play_mode_player_count() {
        assert_eq!(PlayMode::Beat5K.player_count(), 1);
        assert_eq!(PlayMode::Beat7K.player_count(), 1);
        assert_eq!(PlayMode::PopN9K.player_count(), 1);
        assert_eq!(PlayMode::Keyboard24K.player_count(), 1);
        assert_eq!(PlayMode::Beat10K.player_count(), 2);
        assert_eq!(PlayMode::Beat14K.player_count(), 2);
        assert_eq!(PlayMode::Keyboard24KDouble.player_count(), 2);
    }

    #[test]
    fn play_mode_scratch() {
        // Beat5K: lane 5 is scratch
        for i in 0..5 {
            assert!(!PlayMode::Beat5K.is_scratch(i));
        }
        assert!(PlayMode::Beat5K.is_scratch(5));
        assert_eq!(PlayMode::Beat5K.scratch_index(5), Some(0));

        // Beat7K: lane 7 is scratch
        for i in 0..7 {
            assert!(!PlayMode::Beat7K.is_scratch(i));
        }
        assert!(PlayMode::Beat7K.is_scratch(7));

        // Beat14K: lanes 7 and 15 are scratches
        assert_eq!(PlayMode::Beat14K.scratch_index(7), Some(0));
        assert_eq!(PlayMode::Beat14K.scratch_index(15), Some(1));
        assert_eq!(PlayMode::Beat14K.scratch_index(3), None);

        // PopN9K: no scratches
        for i in 0..9 {
            assert!(!PlayMode::PopN9K.is_scratch(i));
        }
        assert_eq!(PlayMode::PopN9K.scratch_count(), 0);

        // Keyboard: no scratches
        assert_eq!(PlayMode::Keyboard24K.scratch_count(), 0);
    }

    #[test]
    fn play_mode_skin_offset() {
        // Beat7K: [1,2,3,4,5,6,7,0]
        assert_eq!(PlayMode::Beat7K.lane_to_skin_offset(0), 1);
        assert_eq!(PlayMode::Beat7K.lane_to_skin_offset(6), 7);
        assert_eq!(PlayMode::Beat7K.lane_to_skin_offset(7), 0); // scratch

        // PopN9K: [1..9]
        for i in 0..9 {
            assert_eq!(PlayMode::PopN9K.lane_to_skin_offset(i), i + 1);
        }
    }

    #[test]
    fn play_mode_lane_to_player() {
        // Single player
        for i in 0..8 {
            assert_eq!(PlayMode::Beat7K.lane_to_player(i), 0);
        }

        // Double player (Beat14K: 16 lanes, 8 per player)
        for i in 0..8 {
            assert_eq!(PlayMode::Beat14K.lane_to_player(i), 0);
        }
        for i in 8..16 {
            assert_eq!(PlayMode::Beat14K.lane_to_player(i), 1);
        }
    }

    #[test]
    fn note_is_long_note() {
        let normal = Note {
            lane: 0,
            note_type: NoteType::Normal,
            time_us: 0,
            end_time_us: 0,
            wav_id: 0,
            damage: 0.0,
        };
        assert!(!normal.is_long_note());

        let ln = Note {
            lane: 0,
            note_type: NoteType::LongNote,
            time_us: 1_000_000,
            end_time_us: 2_000_000,
            wav_id: 0,
            damage: 0.0,
        };
        assert!(ln.is_long_note());
        assert_eq!(ln.duration_us(), 1_000_000);

        let mine = Note {
            lane: 0,
            note_type: NoteType::Mine,
            time_us: 0,
            end_time_us: 0,
            wav_id: 0,
            damage: 10.0,
        };
        assert!(!mine.is_long_note());
    }
}
