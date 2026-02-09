// bmson JSON format data structures (serde deserialization)
// Reference: Java bms.model.bmson package
//
// Many fields are used only by serde deserialization or reserved for
// future phases (BGA: Phase 9, BarLine: Phase 10, names: Phase 8).

use serde::Deserialize;

/// Root bmson object
#[derive(Debug, Deserialize)]
pub struct Bmson {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub info: BmsInfo,
    #[serde(default)]
    pub lines: Vec<BarLine>,
    #[serde(default)]
    pub bpm_events: Vec<BpmEvent>,
    #[serde(default)]
    pub stop_events: Vec<StopEvent>,
    #[serde(default)]
    pub scroll_events: Vec<ScrollEvent>,
    #[serde(default)]
    pub sound_channels: Vec<SoundChannel>,
    #[serde(default)]
    pub bga: Option<Bga>,
    #[serde(default)]
    pub mine_channels: Vec<MineChannel>,
    #[serde(default)]
    pub key_channels: Vec<MineChannel>,
}

/// Chart information
#[derive(Debug, Deserialize)]
pub struct BmsInfo {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub subartists: Vec<String>,
    #[serde(default = "default_mode_hint")]
    pub mode_hint: String,
    #[serde(default)]
    pub chart_name: String,
    #[serde(default = "default_judge_rank")]
    pub judge_rank: i32,
    #[serde(default = "default_total")]
    pub total: f64,
    #[serde(default)]
    pub init_bpm: f64,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub back_image: String,
    #[serde(default)]
    pub eyecatch_image: String,
    #[serde(default)]
    pub banner_image: String,
    #[serde(default)]
    pub preview_music: String,
    #[serde(default = "default_resolution")]
    pub resolution: i32,
    #[serde(default)]
    pub ln_type: i32,
}

impl Default for BmsInfo {
    fn default() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            genre: String::new(),
            artist: String::new(),
            subartists: Vec::new(),
            mode_hint: "beat-7k".to_string(),
            chart_name: String::new(),
            judge_rank: 100,
            total: 100.0,
            init_bpm: 0.0,
            level: 0,
            back_image: String::new(),
            eyecatch_image: String::new(),
            banner_image: String::new(),
            preview_music: String::new(),
            resolution: 240,
            ln_type: 0,
        }
    }
}

fn default_mode_hint() -> String {
    "beat-7k".to_string()
}

fn default_judge_rank() -> i32 {
    100
}

fn default_total() -> f64 {
    100.0
}

fn default_resolution() -> i32 {
    240
}

/// Sound note within a SoundChannel
#[derive(Debug, Deserialize)]
pub struct SoundNote {
    /// Lane (0 = BGM, 1+ = playable)
    #[serde(default)]
    pub x: i32,
    /// Y position (pulse location)
    #[serde(default)]
    pub y: i32,
    /// Length (0 = normal, >0 = long note)
    #[serde(default)]
    pub l: i32,
    /// Continue flag (true = continue previous sound)
    #[serde(default)]
    pub c: bool,
    /// Note type (0 = default, 1-3 = LN type override)
    #[serde(default)]
    pub t: i32,
    /// LN end sound definition
    #[serde(default)]
    pub up: bool,
}

/// Mine note
#[derive(Debug, Deserialize)]
pub struct BmsonMineNote {
    /// Lane
    #[serde(default)]
    pub x: i32,
    /// Y position
    #[serde(default)]
    pub y: i32,
    /// Damage value
    #[serde(default)]
    pub damage: f64,
}

/// Sound channel containing notes for one sound file
#[derive(Debug, Deserialize)]
pub struct SoundChannel {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub notes: Vec<SoundNote>,
}

/// Mine/Key channel
#[derive(Debug, Deserialize)]
pub struct MineChannel {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub notes: Vec<BmsonMineNote>,
}

/// Bar line position
#[derive(Debug, Deserialize)]
pub struct BarLine {
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub k: i32,
}

/// BPM change event
#[derive(Debug, Deserialize)]
pub struct BpmEvent {
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub bpm: f64,
}

/// Stop event
#[derive(Debug, Deserialize)]
pub struct StopEvent {
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub duration: i64,
}

/// Scroll speed event
#[derive(Debug, Deserialize)]
pub struct ScrollEvent {
    #[serde(default)]
    pub y: i32,
    #[serde(default = "default_scroll_rate")]
    pub rate: f64,
}

fn default_scroll_rate() -> f64 {
    1.0
}

/// BGA container (structure only, conversion deferred to Phase 9)
#[derive(Debug, Deserialize)]
pub struct Bga {
    #[serde(default)]
    pub bga_header: Vec<BgaHeader>,
    #[serde(default)]
    pub bga_sequence: Vec<BgaSequence>,
    #[serde(default)]
    pub bga_events: Vec<BNote>,
    #[serde(default)]
    pub layer_events: Vec<BNote>,
    #[serde(default)]
    pub poor_events: Vec<BNote>,
}

#[derive(Debug, Deserialize)]
pub struct BgaHeader {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BgaSequence {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub sequence: Vec<BgaSequenceEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BgaSequenceEntry {
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct BNote {
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub id_set: Option<Vec<i32>>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub interval: i32,
}
