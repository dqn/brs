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
