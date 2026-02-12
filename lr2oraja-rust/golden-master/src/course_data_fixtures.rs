use serde::Deserialize;

/// Root structure for course data GM test fixtures.
#[derive(Debug, Deserialize)]
pub struct CourseDataFixture {
    pub test_cases: Vec<CourseDataTestCase>,
}

/// A single course data test case exported from Java.
#[derive(Debug, Deserialize)]
pub struct CourseDataTestCase {
    pub source_file: String,
    pub valid: bool,
    pub name: String,
    pub hash: Vec<CourseDataSongFixture>,
    pub constraint: Vec<String>,
    pub trophy: Vec<CourseDataTrophyFixture>,
    pub release: bool,
    pub is_class_course: bool,
}

#[derive(Debug, Deserialize)]
pub struct CourseDataSongFixture {
    pub sha256: String,
    pub md5: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct CourseDataTrophyFixture {
    pub name: String,
    pub missrate: f32,
    pub scorerate: f32,
}
