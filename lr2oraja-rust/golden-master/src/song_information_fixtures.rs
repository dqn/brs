use serde::Deserialize;

/// Root structure for song information GM test fixtures.
#[derive(Debug, Deserialize)]
pub struct SongInformationFixture {
    pub test_cases: Vec<SongInformationTestCase>,
}

/// A single SongInformation test case exported from Java.
#[derive(Debug, Deserialize)]
pub struct SongInformationTestCase {
    pub filename: String,
    pub sha256: String,
    pub n: i32,
    pub ln: i32,
    pub s: i32,
    pub ls: i32,
    pub total: f64,
    pub density: f64,
    pub peakdensity: f64,
    pub enddensity: f64,
    pub mainbpm: f64,
    pub distribution: String,
    pub speedchange: String,
    pub lanenotes: String,
}
