// Fixture types for Phase 2 rule engine golden master testing

use serde::Deserialize;

// =========================================================================
// Judge Windows
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct JudgeWindowFixture {
    pub test_cases: Vec<JudgeWindowTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct JudgeWindowTestCase {
    pub mode: String,
    pub note_type: String,
    pub judgerank: i32,
    pub judge_window_rate: Vec<i32>,
    pub windows: Vec<Vec<i64>>,
}

// =========================================================================
// Gauge Properties
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct GaugePropertyFixture {
    pub test_cases: Vec<GaugePropertyTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct GaugePropertyTestCase {
    pub mode: String,
    pub gauge_type: String,
    pub gauge_type_index: usize,
    pub total: f64,
    pub total_notes: usize,
    pub min: f32,
    pub max: f32,
    pub init: f32,
    pub border: f32,
    pub death: f32,
    pub base_values: Vec<f32>,
    pub modified_values: Vec<f32>,
    pub guts: Vec<GutsFixtureEntry>,
}

#[derive(Debug, Deserialize)]
pub struct GutsFixtureEntry {
    pub threshold: f32,
    pub multiplier: f32,
}

// =========================================================================
// Gauge Sequences
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct GaugeSequenceFixture {
    pub test_cases: Vec<GaugeSequenceTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct GaugeSequenceTestCase {
    pub mode: String,
    pub sequence_name: String,
    pub total: f64,
    pub total_notes: usize,
    pub sequence: Vec<SequenceStep>,
    pub values_after_each_step: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
pub struct SequenceStep {
    pub judge: usize,
    pub rate_x100: i32,
}
