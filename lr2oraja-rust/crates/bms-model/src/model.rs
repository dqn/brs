use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::mode::PlayMode;
use crate::note::{LnType, Note};
use crate::timeline::{BpmChange, StopEvent, TimeLine};

/// Complete BMS chart model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmsModel {
    // Metadata
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub sub_artist: String,
    pub genre: String,
    pub banner: String,
    pub stage_file: String,
    pub back_bmp: String,
    pub preview: String,

    // Difficulty
    pub play_level: i32,
    pub judge_rank: i32,
    pub total: f64,
    pub difficulty: i32,

    // Mode
    pub mode: PlayMode,
    pub ln_type: LnType,
    pub player: i32,

    // BPM / Timing
    pub initial_bpm: f64,
    pub bpm_changes: Vec<BpmChange>,
    pub stop_events: Vec<StopEvent>,
    pub timelines: Vec<TimeLine>,

    // Notes
    pub notes: Vec<Note>,

    // WAV/BMP definitions
    #[serde(skip)]
    pub wav_defs: HashMap<u16, PathBuf>,
    #[serde(skip)]
    pub bmp_defs: HashMap<u16, PathBuf>,

    // Hashes (computed after parsing)
    pub md5: String,
    pub sha256: String,

    // Total measure count
    pub total_measures: u32,

    // Total play time in microseconds
    pub total_time_us: i64,
}

impl Default for BmsModel {
    fn default() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            artist: String::new(),
            sub_artist: String::new(),
            genre: String::new(),
            banner: String::new(),
            stage_file: String::new(),
            back_bmp: String::new(),
            preview: String::new(),
            play_level: 0,
            judge_rank: 100,
            total: 300.0,
            difficulty: 0,
            mode: PlayMode::Beat7K,
            ln_type: LnType::LongNote,
            player: 1,
            initial_bpm: 130.0,
            bpm_changes: Vec::new(),
            stop_events: Vec::new(),
            timelines: Vec::new(),
            notes: Vec::new(),
            wav_defs: HashMap::new(),
            bmp_defs: HashMap::new(),
            md5: String::new(),
            sha256: String::new(),
            total_measures: 0,
            total_time_us: 0,
        }
    }
}

impl BmsModel {
    /// Number of playable notes (excludes mines and invisible)
    pub fn total_notes(&self) -> usize {
        self.notes.iter().filter(|n| n.is_playable()).count()
    }

    /// Number of long notes
    pub fn total_long_notes(&self) -> usize {
        self.notes.iter().filter(|n| n.is_long_note()).count()
    }

    /// Get notes for a specific lane, sorted by time
    pub fn lane_notes(&self, lane: usize) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.notes.iter().filter(|n| n.lane == lane).collect();
        notes.sort_by_key(|n| n.time_us);
        notes
    }

    /// Get all playable notes sorted by time
    pub fn playable_notes(&self) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.notes.iter().filter(|n| n.is_playable()).collect();
        notes.sort_by_key(|n| n.time_us);
        notes
    }

    /// Minimum BPM in the chart
    pub fn min_bpm(&self) -> f64 {
        self.bpm_changes
            .iter()
            .map(|c| c.bpm)
            .fold(self.initial_bpm, f64::min)
    }

    /// Maximum BPM in the chart
    pub fn max_bpm(&self) -> f64 {
        self.bpm_changes
            .iter()
            .map(|c| c.bpm)
            .fold(self.initial_bpm, f64::max)
    }
}
