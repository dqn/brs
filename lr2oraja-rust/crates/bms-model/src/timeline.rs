use serde::{Deserialize, Serialize};

/// BPM change event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BpmChange {
    /// Time in microseconds
    pub time_us: i64,
    /// New BPM value
    pub bpm: f64,
}

/// STOP/freeze event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopEvent {
    /// Time in microseconds when STOP starts
    pub time_us: i64,
    /// Duration of STOP in ticks (192 ticks = 1 measure at 4/4)
    pub duration_ticks: i64,
    /// Duration of STOP in microseconds (computed from BPM)
    pub duration_us: i64,
}

/// BGA event layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BgaLayer {
    /// Base BGA layer (channel 04)
    Bga,
    /// Overlay layer (channel 06)
    Layer,
    /// Poor/miss layer (channel 07)
    Poor,
}

/// A single BGA event parsed from channels 04/06/07
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgaEvent {
    /// Time in microseconds
    pub time_us: i64,
    /// Which BGA layer this event targets
    pub layer: BgaLayer,
    /// BMP definition ID (index into bmp_defs / bga list)
    pub id: i32,
}

/// A point in time within the chart that may contain notes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeLine {
    /// Time in microseconds
    pub time_us: i64,
    /// Measure number
    pub measure: u32,
    /// Position within measure (0.0 to 1.0)
    pub position: f64,
    /// BPM at this point
    pub bpm: f64,
}
