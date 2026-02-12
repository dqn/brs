use serde::Deserialize;

/// Root structure for pattern modifier detail GM test fixtures.
#[derive(Debug, Deserialize)]
pub struct PatternModifierDetailFixture {
    pub test_cases: Vec<PatternModifierTestCase>,
}

/// A single pattern modifier test case from the Java exporter.
#[derive(Debug, Deserialize)]
pub struct PatternModifierTestCase {
    pub modifier_type: String,
    pub bms_file: String,
    pub config: serde_json::Value,
    pub notes_before: Vec<ModifierNote>,
    pub notes_after: Vec<ModifierNote>,
    pub assist_level: String,
    /// BPM/stop/scroll state after modification (scroll_speed_remove only)
    #[serde(default)]
    pub bpm_after: Option<Vec<BpmStateEntry>>,
}

/// A note as captured before/after modifier application.
#[derive(Debug, Deserialize)]
pub struct ModifierNote {
    pub lane: usize,
    pub time_ms: i32,
    pub note_type: String,
    #[serde(default)]
    pub end_time_ms: Option<i32>,
}

/// BPM/stop/scroll state for a timeline entry (scroll_speed_remove).
#[derive(Debug, Deserialize)]
pub struct BpmStateEntry {
    pub time_ms: i32,
    pub bpm: f64,
    pub stop_ms: i32,
    pub scroll: f64,
}
