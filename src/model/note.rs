/// Total number of lanes supported (Scratch + 14 keys).
pub const LANE_COUNT: usize = 16;

/// Represents a lane in the play area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lane {
    // 1P side
    Scratch,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    // 2P side (for DP mode)
    Scratch2,
    Key8,
    Key9,
    Key10,
    Key11,
    Key12,
    Key13,
    Key14,
}

impl Lane {
    /// Returns all lanes in order.
    pub fn all() -> &'static [Lane] {
        &[
            Lane::Scratch,
            Lane::Key1,
            Lane::Key2,
            Lane::Key3,
            Lane::Key4,
            Lane::Key5,
            Lane::Key6,
            Lane::Key7,
            Lane::Scratch2,
            Lane::Key8,
            Lane::Key9,
            Lane::Key10,
            Lane::Key11,
            Lane::Key12,
            Lane::Key13,
            Lane::Key14,
        ]
    }
    /// Returns all lanes in order for 7-key mode (1P).
    pub fn all_7k() -> &'static [Lane] {
        &[
            Lane::Scratch,
            Lane::Key1,
            Lane::Key2,
            Lane::Key3,
            Lane::Key4,
            Lane::Key5,
            Lane::Key6,
            Lane::Key7,
        ]
    }

    /// Returns all lanes in order for 14-key mode (DP).
    pub fn all_14k() -> &'static [Lane] {
        &[
            Lane::Scratch,
            Lane::Key1,
            Lane::Key2,
            Lane::Key3,
            Lane::Key4,
            Lane::Key5,
            Lane::Key6,
            Lane::Key7,
            Lane::Scratch2,
            Lane::Key8,
            Lane::Key9,
            Lane::Key10,
            Lane::Key11,
            Lane::Key12,
            Lane::Key13,
            Lane::Key14,
        ]
    }

    /// Returns all key lanes (excluding scratch) for 7-key mode.
    pub fn keys_7k() -> &'static [Lane] {
        &[
            Lane::Key1,
            Lane::Key2,
            Lane::Key3,
            Lane::Key4,
            Lane::Key5,
            Lane::Key6,
            Lane::Key7,
        ]
    }

    /// Returns all 2P side lanes for 14-key mode.
    pub fn all_2p() -> &'static [Lane] {
        &[
            Lane::Scratch2,
            Lane::Key8,
            Lane::Key9,
            Lane::Key10,
            Lane::Key11,
            Lane::Key12,
            Lane::Key13,
            Lane::Key14,
        ]
    }

    /// Returns the lane index (0-based).
    pub fn index(self) -> usize {
        match self {
            Lane::Scratch => 0,
            Lane::Key1 => 1,
            Lane::Key2 => 2,
            Lane::Key3 => 3,
            Lane::Key4 => 4,
            Lane::Key5 => 5,
            Lane::Key6 => 6,
            Lane::Key7 => 7,
            Lane::Scratch2 => 8,
            Lane::Key8 => 9,
            Lane::Key9 => 10,
            Lane::Key10 => 11,
            Lane::Key11 => 12,
            Lane::Key12 => 13,
            Lane::Key13 => 14,
            Lane::Key14 => 15,
        }
    }

    /// Create a lane from a 0-based index.
    pub fn from_index(index: usize) -> Option<Lane> {
        match index {
            0 => Some(Lane::Scratch),
            1 => Some(Lane::Key1),
            2 => Some(Lane::Key2),
            3 => Some(Lane::Key3),
            4 => Some(Lane::Key4),
            5 => Some(Lane::Key5),
            6 => Some(Lane::Key6),
            7 => Some(Lane::Key7),
            8 => Some(Lane::Scratch2),
            9 => Some(Lane::Key8),
            10 => Some(Lane::Key9),
            11 => Some(Lane::Key10),
            12 => Some(Lane::Key11),
            13 => Some(Lane::Key12),
            14 => Some(Lane::Key13),
            15 => Some(Lane::Key14),
            _ => None,
        }
    }

    /// Returns true if this lane is a key (not scratch).
    pub fn is_key(self) -> bool {
        !matches!(self, Lane::Scratch | Lane::Scratch2)
    }

    /// Returns true if this lane is a scratch lane.
    pub fn is_scratch(self) -> bool {
        matches!(self, Lane::Scratch | Lane::Scratch2)
    }

    /// Returns true if this lane is on the 1P side.
    pub fn is_1p(self) -> bool {
        matches!(
            self,
            Lane::Scratch
                | Lane::Key1
                | Lane::Key2
                | Lane::Key3
                | Lane::Key4
                | Lane::Key5
                | Lane::Key6
                | Lane::Key7
        )
    }

    /// Returns true if this lane is on the 2P side.
    pub fn is_2p(self) -> bool {
        !self.is_1p()
    }
}

/// Type of note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Normal,
    LongStart,
    LongEnd,
    Invisible,
    Mine,
}

/// A single note in the BMS chart.
#[derive(Debug, Clone)]
pub struct Note {
    pub lane: Lane,
    pub start_time_ms: f64,
    pub end_time_ms: Option<f64>,
    pub wav_id: u16,
    pub note_type: NoteType,
    pub mine_damage: Option<f64>,
}

impl Note {
    /// Create a new normal note.
    pub fn normal(lane: Lane, time_ms: f64, wav_id: u16) -> Self {
        Self {
            lane,
            start_time_ms: time_ms,
            end_time_ms: None,
            wav_id,
            note_type: NoteType::Normal,
            mine_damage: None,
        }
    }

    /// Create a new long note start.
    pub fn long_start(lane: Lane, start_ms: f64, end_ms: f64, wav_id: u16) -> Self {
        Self {
            lane,
            start_time_ms: start_ms,
            end_time_ms: Some(end_ms),
            wav_id,
            note_type: NoteType::LongStart,
            mine_damage: None,
        }
    }

    /// Create a new mine note.
    pub fn mine(lane: Lane, time_ms: f64, damage: f64) -> Self {
        Self {
            lane,
            start_time_ms: time_ms,
            end_time_ms: None,
            wav_id: 0,
            note_type: NoteType::Mine,
            mine_damage: Some(damage),
        }
    }

    /// Create a new invisible note.
    pub fn invisible(lane: Lane, time_ms: f64, wav_id: u16) -> Self {
        Self {
            lane,
            start_time_ms: time_ms,
            end_time_ms: None,
            wav_id,
            note_type: NoteType::Invisible,
            mine_damage: None,
        }
    }

    /// Create a new long note end.
    pub fn long_end(lane: Lane, time_ms: f64, wav_id: u16) -> Self {
        Self {
            lane,
            start_time_ms: time_ms,
            end_time_ms: None,
            wav_id,
            note_type: NoteType::LongEnd,
            mine_damage: None,
        }
    }

    /// Returns true if this is a long note.
    pub fn is_long(&self) -> bool {
        matches!(self.note_type, NoteType::LongStart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lane_all_returns_all_16_lanes() {
        let all = Lane::all();
        assert_eq!(all.len(), 16);
        assert_eq!(all[0], Lane::Scratch);
        assert_eq!(all[15], Lane::Key14);
    }

    #[test]
    fn test_lane_all_7k_returns_1p_lanes() {
        let lanes = Lane::all_7k();
        assert_eq!(lanes.len(), 8);
        assert_eq!(lanes[0], Lane::Scratch);
        assert_eq!(lanes[7], Lane::Key7);

        for lane in lanes {
            assert!(lane.is_1p());
            assert!(!lane.is_2p());
        }
    }

    #[test]
    fn test_lane_all_14k_returns_all_lanes() {
        let lanes = Lane::all_14k();
        assert_eq!(lanes.len(), 16);

        // First 8 are 1P
        for lane in &lanes[0..8] {
            assert!(lane.is_1p(), "{:?} should be 1P", lane);
        }
        // Last 8 are 2P
        for lane in &lanes[8..16] {
            assert!(lane.is_2p(), "{:?} should be 2P", lane);
        }
    }

    #[test]
    fn test_lane_all_2p_returns_2p_lanes() {
        let lanes = Lane::all_2p();
        assert_eq!(lanes.len(), 8);
        assert_eq!(lanes[0], Lane::Scratch2);
        assert_eq!(lanes[7], Lane::Key14);

        for lane in lanes {
            assert!(lane.is_2p());
            assert!(!lane.is_1p());
        }
    }

    #[test]
    fn test_lane_index_round_trip() {
        for lane in Lane::all() {
            let index = lane.index();
            let recovered = Lane::from_index(index);
            assert_eq!(recovered, Some(*lane), "Round-trip failed for {:?}", lane);
        }
    }

    #[test]
    fn test_lane_from_index_invalid() {
        assert_eq!(Lane::from_index(16), None);
        assert_eq!(Lane::from_index(100), None);
        assert_eq!(Lane::from_index(usize::MAX), None);
    }

    #[test]
    fn test_lane_is_key() {
        assert!(!Lane::Scratch.is_key());
        assert!(!Lane::Scratch2.is_key());

        for lane in Lane::keys_7k() {
            assert!(lane.is_key(), "{:?} should be a key", lane);
        }

        // 2P keys should also be keys
        assert!(Lane::Key8.is_key());
        assert!(Lane::Key14.is_key());
    }

    #[test]
    fn test_lane_is_scratch() {
        assert!(Lane::Scratch.is_scratch());
        assert!(Lane::Scratch2.is_scratch());

        for lane in Lane::keys_7k() {
            assert!(!lane.is_scratch(), "{:?} should not be scratch", lane);
        }
    }

    #[test]
    fn test_lane_keys_7k() {
        let keys = Lane::keys_7k();
        assert_eq!(keys.len(), 7);

        for lane in keys {
            assert!(lane.is_key());
            assert!(!lane.is_scratch());
            assert!(lane.is_1p());
        }

        // Should not include scratch
        assert!(!keys.contains(&Lane::Scratch));
    }

    #[test]
    fn test_lane_indices_are_sequential() {
        let all = Lane::all();
        for (expected_index, lane) in all.iter().enumerate() {
            assert_eq!(
                lane.index(),
                expected_index,
                "{:?} should have index {}",
                lane,
                expected_index
            );
        }
    }

    #[test]
    fn test_lane_ordering() {
        // Lanes should be orderable (used for sorting)
        assert!(Lane::Scratch < Lane::Key1);
        assert!(Lane::Key7 < Lane::Scratch2);
        assert!(Lane::Scratch2 < Lane::Key8);
    }

    #[test]
    fn test_note_normal_creation() {
        let note = Note::normal(Lane::Key1, 1000.0, 42);
        assert_eq!(note.lane, Lane::Key1);
        assert_eq!(note.start_time_ms, 1000.0);
        assert_eq!(note.end_time_ms, None);
        assert_eq!(note.wav_id, 42);
        assert_eq!(note.note_type, NoteType::Normal);
        assert!(!note.is_long());
    }

    #[test]
    fn test_note_long_start_creation() {
        let note = Note::long_start(Lane::Key2, 1000.0, 2000.0, 42);
        assert_eq!(note.lane, Lane::Key2);
        assert_eq!(note.start_time_ms, 1000.0);
        assert_eq!(note.end_time_ms, Some(2000.0));
        assert_eq!(note.note_type, NoteType::LongStart);
        assert!(note.is_long());
    }

    #[test]
    fn test_note_long_end_creation() {
        let note = Note::long_end(Lane::Key2, 2000.0, 42);
        assert_eq!(note.lane, Lane::Key2);
        assert_eq!(note.start_time_ms, 2000.0);
        assert_eq!(note.end_time_ms, None);
        assert_eq!(note.note_type, NoteType::LongEnd);
        assert!(!note.is_long()); // LongEnd is not considered "long" itself
    }

    #[test]
    fn test_note_mine_creation() {
        let note = Note::mine(Lane::Key3, 1500.0, 50.0);
        assert_eq!(note.lane, Lane::Key3);
        assert_eq!(note.start_time_ms, 1500.0);
        assert_eq!(note.note_type, NoteType::Mine);
        assert_eq!(note.mine_damage, Some(50.0));
        assert_eq!(note.wav_id, 0);
    }

    #[test]
    fn test_note_invisible_creation() {
        let note = Note::invisible(Lane::Key4, 1200.0, 42);
        assert_eq!(note.lane, Lane::Key4);
        assert_eq!(note.start_time_ms, 1200.0);
        assert_eq!(note.note_type, NoteType::Invisible);
        assert_eq!(note.wav_id, 42);
    }

    #[test]
    fn test_note_type_equality() {
        assert_eq!(NoteType::Normal, NoteType::Normal);
        assert_ne!(NoteType::Normal, NoteType::LongStart);
        assert_ne!(NoteType::LongStart, NoteType::LongEnd);
    }
}
