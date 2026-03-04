// Fixture types for JudgeManager golden master tests

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JudgeFixtures {
    pub test_cases: Vec<JudgeTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct JudgeTestCase {
    pub group: String,
    pub name: String,
    pub filename: String,
    pub gauge_type: String,
    pub autoplay: bool,
    #[serde(default)]
    pub offset_us: i64,
    #[serde(default)]
    pub input_log: Vec<InputLogEntry>,
    pub expected: ExpectedResult,
}

#[derive(Debug, Deserialize)]
pub struct InputLogEntry {
    pub presstime: i64,
    pub keycode: i32,
    pub pressed: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedResult {
    pub score: ExpectedScore,
    pub maxcombo: i32,
    pub passnotes: i32,
    pub gauge_value: f32,
    pub gauge_qualified: bool,
    pub ghost: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedScore {
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
    pub passnotes: i32,
}

impl JudgeFixtures {
    pub fn load() -> anyhow::Result<Self> {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/judge_manager.json");
        let content = std::fs::read_to_string(&path)?;
        let fixtures: JudgeFixtures = serde_json::from_str(&content)?;
        Ok(fixtures)
    }
}
