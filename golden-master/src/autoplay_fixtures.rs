use serde::Deserialize;

/// Root fixture structure for autoplay golden master tests.
#[derive(Debug, Deserialize)]
pub struct AutoplayFixture {
    pub test_cases: Vec<AutoplayTestCase>,
}

/// A single autoplay test case from the Java exporter.
#[derive(Debug, Deserialize)]
pub struct AutoplayTestCase {
    pub filename: String,
    pub mode: String,
    pub ln_type: i32,
    pub key_count: usize,
    /// All timeline times from Java's getAllTimeLines() (in microseconds).
    /// Used to seed the autoplay algorithm with the same timelines as Java.
    pub timeline_times: Vec<i64>,
    pub log: Vec<AutoplayLogEntry>,
}

/// A single key input log entry in the autoplay fixture.
#[derive(Debug, Deserialize)]
pub struct AutoplayLogEntry {
    pub presstime: i64,
    pub keycode: i32,
    pub pressed: bool,
}
