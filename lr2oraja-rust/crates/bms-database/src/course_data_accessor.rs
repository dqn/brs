//! File-based JSON I/O for course data.
//!
//! Port of Java course file access patterns.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::course_data::CourseData;

/// File-based accessor for course data stored as JSON files.
pub struct CourseDataAccessor {
    course_dir: PathBuf,
}

impl CourseDataAccessor {
    /// Create a new accessor for the given directory, creating it if needed.
    pub fn new(course_dir: impl Into<PathBuf>) -> Result<Self> {
        let course_dir = course_dir.into();
        if !course_dir.exists() {
            fs::create_dir_all(&course_dir)?;
        }
        Ok(Self { course_dir })
    }

    /// Read all course data from all .json files in the directory.
    pub fn read_all(&self) -> Result<Vec<CourseData>> {
        let mut results = Vec::new();
        let entries = fs::read_dir(&self.course_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                match self.read_file(&path) {
                    Ok(courses) => results.extend(courses),
                    Err(_) => continue,
                }
            }
        }
        Ok(results)
    }

    /// Read all file stems (names without .json extension) from the directory.
    pub fn read_all_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        let entries = fs::read_dir(&self.course_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json")
                && let Some(stem) = path.file_stem()
            {
                names.push(stem.to_string_lossy().to_string());
            }
        }
        Ok(names)
    }

    /// Read course data from a named file (`{dir}/{name}.json`).
    ///
    /// Tries to parse as an array first, then as a single object.
    /// Validates each course after reading.
    pub fn read(&self, name: &str) -> Result<Vec<CourseData>> {
        let path = self.course_dir.join(format!("{name}.json"));
        self.read_file(&path)
    }

    /// Write course data as a JSON array to `{dir}/{name}.json`.
    pub fn write(&self, name: &str, courses: &[CourseData]) -> Result<()> {
        let path = self.course_dir.join(format!("{name}.json"));
        let json = serde_json::to_string_pretty(courses)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Read and validate courses from a file path.
    /// Tries array parse first, then single object.
    fn read_file(&self, path: &Path) -> Result<Vec<CourseData>> {
        let content = fs::read_to_string(path)?;

        // Try parsing as array first
        let mut courses: Vec<CourseData> =
            if let Ok(arr) = serde_json::from_str::<Vec<CourseData>>(&content) {
                arr
            } else if let Ok(single) = serde_json::from_str::<CourseData>(&content) {
                vec![single]
            } else {
                return Err(anyhow!(
                    "failed to parse course data from {}",
                    path.display()
                ));
            };

        // Validate each course, keeping only valid ones
        courses.retain_mut(|c| c.validate());

        Ok(courses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course_data::{CourseDataConstraint, CourseSongData, TrophyData};

    fn sample_course(name: &str) -> CourseData {
        CourseData {
            name: name.to_string(),
            hash: vec![CourseSongData {
                sha256: "abc123".to_string(),
                md5: String::new(),
                title: "Song 1".to_string(),
            }],
            constraint: vec![CourseDataConstraint::Class],
            trophy: vec![TrophyData {
                name: "Gold".to_string(),
                missrate: 5.0,
                scorerate: 85.0,
            }],
            release: true,
        }
    }

    #[test]
    fn write_and_read_all() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = CourseDataAccessor::new(dir.path()).unwrap();

        let courses = vec![sample_course("Course A"), sample_course("Course B")];
        accessor.write("test_courses", &courses).unwrap();

        let read = accessor.read_all().unwrap();
        assert_eq!(read.len(), 2);
    }

    #[test]
    fn read_all_names() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = CourseDataAccessor::new(dir.path()).unwrap();

        accessor.write("alpha", &[sample_course("A")]).unwrap();
        accessor.write("beta", &[sample_course("B")]).unwrap();

        let mut names = accessor.read_all_names().unwrap();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn read_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = CourseDataAccessor::new(dir.path()).unwrap();

        let courses = vec![sample_course("My Course")];
        accessor.write("my_file", &courses).unwrap();

        let read = accessor.read("my_file").unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].name, "My Course");
    }

    #[test]
    fn read_single_object_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("single.json");
        let course = sample_course("Single");
        let json = serde_json::to_string_pretty(&course).unwrap();
        fs::write(&path, json).unwrap();

        let accessor = CourseDataAccessor::new(dir.path()).unwrap();
        let read = accessor.read("single").unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].name, "Single");
    }

    #[test]
    fn read_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let accessor = CourseDataAccessor::new(dir.path()).unwrap();
        assert!(accessor.read("nonexistent").is_err());
    }

    #[test]
    fn read_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        fs::write(&path, "not valid json").unwrap();

        let accessor = CourseDataAccessor::new(dir.path()).unwrap();
        assert!(accessor.read("bad").is_err());
    }

    #[test]
    fn creates_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        let accessor = CourseDataAccessor::new(&nested).unwrap();
        assert!(nested.exists());
        let courses = accessor.read_all().unwrap();
        assert!(courses.is_empty());
    }

    #[test]
    fn filters_invalid_courses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mixed.json");
        // One valid, one invalid (no songs)
        let json = r#"[
            {
                "name": "Valid",
                "hash": [{"sha256": "abc", "md5": "", "title": "S1"}],
                "constraint": [],
                "trophy": [],
                "release": true
            },
            {
                "name": "Invalid",
                "hash": [],
                "constraint": [],
                "trophy": [],
                "release": true
            }
        ]"#;
        fs::write(&path, json).unwrap();

        let accessor = CourseDataAccessor::new(dir.path()).unwrap();
        let read = accessor.read("mixed").unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].name, "Valid");
    }

    #[test]
    fn skips_non_json_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("readme.txt"), "not json").unwrap();
        let accessor = CourseDataAccessor::new(dir.path()).unwrap();
        accessor.write("real", &[sample_course("Real")]).unwrap();

        let read = accessor.read_all().unwrap();
        assert_eq!(read.len(), 1);
    }
}
