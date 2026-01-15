use fraction::Fraction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub metadata: Metadata,
    pub timing_data: TimingData,
    pub notes: Vec<Note>,
    pub bgm_events: Vec<BgmEvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub artist: String,
    pub genre: String,
    pub bpm: f64,
    pub play_level: u32,
    pub rank: u32,
    pub total: f64,
    /// Long note type (#LNTYPE)
    pub ln_type: LnType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingData {
    pub initial_bpm: f64,
    pub bpm_changes: Vec<BpmChange>,
    pub stops: Vec<StopEvent>,
    pub measure_lengths: Vec<MeasureLength>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmChange {
    pub measure: u32,
    pub position: Fraction,
    pub bpm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopEvent {
    pub measure: u32,
    pub position: Fraction,
    pub duration_192: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureLength {
    pub measure: u32,
    pub length: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub measure: u32,
    pub position: Fraction,
    pub time_ms: f64,
    pub channel: NoteChannel,
    pub keysound_id: u32,
    pub note_type: NoteType,
    /// For LongStart: the end time in ms
    pub long_end_time_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgmEvent {
    pub measure: u32,
    pub position: Fraction,
    pub time_ms: f64,
    pub keysound_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteChannel {
    Scratch,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
}

impl NoteChannel {
    // Public API for converting BMS channel numbers to NoteChannel
    #[allow(dead_code)]
    pub fn from_bms_channel(channel: u32) -> Option<Self> {
        match channel {
            16 => Some(Self::Scratch),
            11 => Some(Self::Key1),
            12 => Some(Self::Key2),
            13 => Some(Self::Key3),
            14 => Some(Self::Key4),
            15 => Some(Self::Key5),
            18 => Some(Self::Key6),
            19 => Some(Self::Key7),
            _ => None,
        }
    }

    pub fn lane_index(&self) -> usize {
        match self {
            Self::Scratch => 0,
            Self::Key1 => 1,
            Self::Key2 => 2,
            Self::Key3 => 3,
            Self::Key4 => 4,
            Self::Key5 => 5,
            Self::Key6 => 6,
            Self::Key7 => 7,
        }
    }

    /// Convert lane index (1-7) to NoteChannel for keys
    /// Returns None for scratch (0) or invalid indices
    #[allow(dead_code)]
    pub fn from_key_lane(lane: usize) -> Option<Self> {
        match lane {
            1 => Some(Self::Key1),
            2 => Some(Self::Key2),
            3 => Some(Self::Key3),
            4 => Some(Self::Key4),
            5 => Some(Self::Key5),
            6 => Some(Self::Key6),
            7 => Some(Self::Key7),
            _ => None,
        }
    }

    /// Check if this channel is a key (not scratch)
    #[allow(dead_code)]
    pub fn is_key(&self) -> bool {
        !matches!(self, Self::Scratch)
    }
}

/// Long note type (LN/CN/HCN)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LnType {
    /// Ez2DJ style - no release judgment
    #[default]
    Ln,
    /// IIDX style - with release judgment (wider window)
    Cn,
    /// Hell Charge Note - damage while released
    Hcn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteType {
    Normal,
    LongStart,
    LongEnd,
    Invisible,
    Landmine,
}

pub const LANE_COUNT: usize = 8;

impl Chart {
    // Public API for querying chart's maximum measure
    #[allow(dead_code)]
    pub fn max_measure(&self) -> u32 {
        let note_max = self.notes.iter().map(|n| n.measure).max().unwrap_or(0);
        let bgm_max = self.bgm_events.iter().map(|b| b.measure).max().unwrap_or(0);
        note_max.max(bgm_max)
    }

    pub fn note_count(&self) -> usize {
        self.notes
            .iter()
            .filter(|n| matches!(n.note_type, NoteType::Normal | NoteType::LongStart))
            .count()
    }

    pub fn build_lane_index(&self) -> [Vec<usize>; LANE_COUNT] {
        let mut index: [Vec<usize>; LANE_COUNT] = Default::default();
        for (i, note) in self.notes.iter().enumerate() {
            index[note.channel.lane_index()].push(i);
        }
        index
    }
}
