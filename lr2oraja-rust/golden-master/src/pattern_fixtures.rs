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
