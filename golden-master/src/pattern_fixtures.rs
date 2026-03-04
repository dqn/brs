// Fixture types for Phase 3 pattern shuffle golden master testing

use serde::Deserialize;

// =========================================================================
// Lane Shuffle Mappings
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct LaneShuffleFixture {
    pub test_cases: Vec<LaneShuffleTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct LaneShuffleTestCase {
    pub modifier_type: String,
    pub mode: String,
    pub seed: Option<i64>,
    pub contains_scratch: bool,
    pub player: usize,
    pub keys: Vec<usize>,
    pub key_count: usize,
    pub mapping: Vec<usize>,
}

// =========================================================================
// Playable Random
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct PlayableRandomFixture {
    pub test_cases: Vec<PlayableRandomTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct PlayableRandomTestCase {
    pub mode: String,
    pub seed: i64,
    pub chord_patterns: Vec<u32>,
    pub candidate_count: usize,
    pub mapping: Vec<usize>,
    pub is_fallback: bool,
}

// =========================================================================
// Battle
// =========================================================================

#[derive(Debug, Deserialize)]
pub struct BattleFixture {
    pub test_cases: Vec<BattleTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct BattleTestCase {
    pub name: String,
    pub mode: String,
    pub input_notes: Vec<BattleNote>,
    pub output_notes: Vec<BattleNote>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BattleNote {
    pub lane: usize,
    pub time_us: i64,
    #[serde(rename = "type")]
    pub note_type: String,
    pub wav_id: u32,
    #[serde(default)]
    pub end_time_us: Option<i64>,
}
