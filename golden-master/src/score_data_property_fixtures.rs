use serde::Deserialize;

/// Root structure for ScoreDataProperty GM test fixtures.
#[derive(Debug, Deserialize)]
pub struct ScoreDataPropertyFixture {
    pub test_cases: Vec<ScoreDataPropertyTestCase>,
}

/// A single ScoreDataProperty test case exported from Java.
#[derive(Debug, Deserialize)]
pub struct ScoreDataPropertyTestCase {
    pub mode: i32,
    pub pattern_name: String,
    // Input judge counts
    pub epg: i32,
    pub lpg: i32,
    pub egr: i32,
    pub lgr: i32,
    pub egd: i32,
    pub lgd: i32,
    pub ebd: i32,
    pub lbd: i32,
    pub epr: i32,
    pub lpr: i32,
    pub ems: i32,
    pub lms: i32,
    pub maxcombo: i32,
    pub notes: i32,
    pub totalnotes: i32,
    // Expected outputs
    pub nowpoint: i32,
    pub rate: f32,
    pub rate_int: i32,
    pub rate_after_dot: i32,
    pub nowrate: f32,
    pub nowrate_int: i32,
    pub nowrate_after_dot: i32,
    pub rank: Vec<bool>,
    pub nextrank: i32,
    pub bestscorerate: f32,
}
