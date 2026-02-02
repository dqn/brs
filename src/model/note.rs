/// Represents a lane in the play area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lane {
    Scratch,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
}

impl Lane {
    /// Returns all lanes in order for 7-key mode.
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
        }
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
        }
    }

    /// Create a new mine note.
    pub fn mine(lane: Lane, time_ms: f64, wav_id: u16) -> Self {
        Self {
            lane,
            start_time_ms: time_ms,
            end_time_ms: None,
            wav_id,
            note_type: NoteType::Mine,
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
        }
    }

    /// Returns true if this is a long note.
    pub fn is_long(&self) -> bool {
        matches!(self.note_type, NoteType::LongStart)
    }
}
