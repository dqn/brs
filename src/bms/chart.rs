use fraction::Fraction;
use serde::{Deserialize, Serialize};

/// Play mode for the chart
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum PlayMode {
    #[default]
    Bms7Key, // 7 keys + scratch (8 lanes)
    Pms9Key, // 9 keys (9 lanes)
    Dp14Key, // DP mode: 14 keys + 2 scratches (16 lanes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub metadata: Metadata,
    pub timing_data: TimingData,
    pub notes: Vec<Note>,
    pub bgm_events: Vec<BgmEvent>,
    pub bga_events: Vec<BgaEvent>,
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
    /// Play mode (BMS 7-key or PMS 9-key)
    #[serde(default)]
    pub play_mode: PlayMode,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgaEvent {
    pub time_ms: f64,
    pub bga_id: u32,
    pub layer: BgaLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BgaLayer {
    Base,    // Channel 04
    Poor,    // Channel 06
    Overlay, // Channel 07
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteChannel {
    // P1 side
    Scratch, // P1 Scratch
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    // P2 side (also used for PMS Key8-9)
    Key8,     // PMS lane 8, or DP P2 Key1
    Key9,     // PMS lane 9, or DP P2 Key2
    Key10,    // DP P2 Key3
    Key11,    // DP P2 Key4
    Key12,    // DP P2 Key5
    Key13,    // DP P2 Key6
    Key14,    // DP P2 Key7
    Scratch2, // DP P2 Scratch
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

    /// Get lane index for BMS 7-key mode (8 lanes: scratch + 7 keys)
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
            Self::Key8 => 8,
            Self::Key9 => 9,
            Self::Key10 => 10,
            Self::Key11 => 11,
            Self::Key12 => 12,
            Self::Key13 => 13,
            Self::Key14 => 14,
            Self::Scratch2 => 15,
        }
    }

    /// Get lane index based on play mode
    /// - BMS 7-key: lanes 0-7 (scratch + 7 keys)
    /// - PMS 9-key: lanes 0-8 (9 keys, no scratch)
    /// - DP 14-key: lanes 0-15 (P1: scratch + 7 keys, P2: 7 keys + scratch)
    pub fn lane_index_for_mode(&self, mode: PlayMode) -> usize {
        match mode {
            PlayMode::Bms7Key => self.lane_index(),
            PlayMode::Pms9Key => match self {
                Self::Key1 => 0,
                Self::Key2 => 1,
                Self::Key3 => 2,
                Self::Key4 => 3,
                Self::Key5 => 4,
                Self::Key6 => 5,
                Self::Key7 => 6,
                Self::Key8 => 7,
                Self::Key9 => 8,
                Self::Scratch => 0, // Should not occur in PMS
                _ => 0,             // Other channels not used in PMS
            },
            PlayMode::Dp14Key => {
                // DP layout: P1(S 1 2 3 4 5 6 7) + P2(1 2 3 4 5 6 7 S)
                // Lanes: 0-7 for P1, 8-15 for P2
                match self {
                    Self::Scratch => 0, // P1 Scratch
                    Self::Key1 => 1,    // P1 Key1
                    Self::Key2 => 2,
                    Self::Key3 => 3,
                    Self::Key4 => 4,
                    Self::Key5 => 5,
                    Self::Key6 => 6,
                    Self::Key7 => 7,      // P1 Key7
                    Self::Key8 => 8,      // P2 Key1
                    Self::Key9 => 9,      // P2 Key2
                    Self::Key10 => 10,    // P2 Key3
                    Self::Key11 => 11,    // P2 Key4
                    Self::Key12 => 12,    // P2 Key5
                    Self::Key13 => 13,    // P2 Key6
                    Self::Key14 => 14,    // P2 Key7
                    Self::Scratch2 => 15, // P2 Scratch
                }
            }
        }
    }

    /// Convert lane index (1-7) to NoteChannel for keys (BMS 7-key)
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

    /// Convert lane index (0-8) to NoteChannel for PMS 9-key mode
    #[allow(dead_code)]
    pub fn from_pms_lane(lane: usize) -> Option<Self> {
        match lane {
            0 => Some(Self::Key1),
            1 => Some(Self::Key2),
            2 => Some(Self::Key3),
            3 => Some(Self::Key4),
            4 => Some(Self::Key5),
            5 => Some(Self::Key6),
            6 => Some(Self::Key7),
            7 => Some(Self::Key8),
            8 => Some(Self::Key9),
            _ => None,
        }
    }

    /// Check if this channel is a key (not scratch)
    #[allow(dead_code)]
    pub fn is_key(&self) -> bool {
        !matches!(self, Self::Scratch | Self::Scratch2)
    }

    /// Check if this channel is a scratch
    #[allow(dead_code)]
    pub fn is_scratch(&self) -> bool {
        matches!(self, Self::Scratch | Self::Scratch2)
    }

    /// Check if this channel belongs to P2 side (DP mode)
    #[allow(dead_code)]
    pub fn is_p2(&self) -> bool {
        matches!(
            self,
            Self::Key8
                | Self::Key9
                | Self::Key10
                | Self::Key11
                | Self::Key12
                | Self::Key13
                | Self::Key14
                | Self::Scratch2
        )
    }

    /// Convert lane index (0-15) to NoteChannel for DP 14-key mode
    #[allow(dead_code)]
    pub fn from_dp_lane(lane: usize) -> Option<Self> {
        match lane {
            0 => Some(Self::Scratch),
            1 => Some(Self::Key1),
            2 => Some(Self::Key2),
            3 => Some(Self::Key3),
            4 => Some(Self::Key4),
            5 => Some(Self::Key5),
            6 => Some(Self::Key6),
            7 => Some(Self::Key7),
            8 => Some(Self::Key8),
            9 => Some(Self::Key9),
            10 => Some(Self::Key10),
            11 => Some(Self::Key11),
            12 => Some(Self::Key12),
            13 => Some(Self::Key13),
            14 => Some(Self::Key14),
            15 => Some(Self::Scratch2),
            _ => None,
        }
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

/// Lane count for BMS 7-key mode (scratch + 7 keys)
pub const LANE_COUNT_BMS: usize = 8;

/// Lane count for PMS 9-key mode (9 keys)
#[allow(dead_code)]
pub const LANE_COUNT_PMS: usize = 9;

/// Lane count for DP 14-key mode (P1: scratch + 7 keys, P2: 7 keys + scratch)
pub const LANE_COUNT_DP: usize = 16;

/// Maximum lane count across all modes
pub const MAX_LANE_COUNT: usize = 16;

/// Legacy constant for backward compatibility
#[allow(dead_code)]
pub const LANE_COUNT: usize = LANE_COUNT_BMS;

/// Get lane count for a specific play mode
#[allow(dead_code)]
pub fn lane_count(mode: PlayMode) -> usize {
    match mode {
        PlayMode::Bms7Key => LANE_COUNT_BMS,
        PlayMode::Pms9Key => LANE_COUNT_PMS,
        PlayMode::Dp14Key => LANE_COUNT_DP,
    }
}

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

    /// Build lane index for BMS 7-key mode (legacy)
    #[allow(dead_code)]
    pub fn build_lane_index(&self) -> [Vec<usize>; LANE_COUNT_BMS] {
        let mut index: [Vec<usize>; LANE_COUNT_BMS] = Default::default();
        for (i, note) in self.notes.iter().enumerate() {
            let lane = note.channel.lane_index();
            if lane < LANE_COUNT_BMS {
                index[lane].push(i);
            }
        }
        index
    }

    /// Build lane index that supports all play modes
    #[allow(dead_code)]
    pub fn build_lane_index_for_mode(&self) -> [Vec<usize>; MAX_LANE_COUNT] {
        let mode = self.metadata.play_mode;
        let mut index: [Vec<usize>; MAX_LANE_COUNT] = Default::default();
        for (i, note) in self.notes.iter().enumerate() {
            let lane = note.channel.lane_index_for_mode(mode);
            if lane < MAX_LANE_COUNT {
                index[lane].push(i);
            }
        }
        index
    }
}
