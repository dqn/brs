use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::game::GaugeType;

use super::DanGrade;

/// Requirements for passing a dan course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanRequirements {
    /// Minimum gauge percentage at the end (0.0 = just survive)
    #[serde(default)]
    pub min_gauge: f32,
    /// Maximum allowed BAD + POOR count (None = no limit)
    #[serde(default)]
    pub max_bad_poor: Option<u32>,
    /// Requires full combo to pass
    #[serde(default)]
    pub full_combo: bool,
}

impl Default for DanRequirements {
    fn default() -> Self {
        Self {
            min_gauge: 0.0,
            max_bad_poor: None,
            full_combo: false,
        }
    }
}

/// A dan certification course consisting of multiple charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanCourse {
    /// Course name (e.g., "SP 初段")
    pub name: String,
    /// Grade this course certifies
    pub grade: DanGrade,
    /// Paths to the charts in play order (relative to course file)
    pub charts: Vec<String>,
    /// Gauge type for the course (typically Hard)
    #[serde(default = "default_gauge_type")]
    pub gauge_type: GaugeType,
    /// Requirements to pass the course
    #[serde(default)]
    pub requirements: DanRequirements,
}

fn default_gauge_type() -> GaugeType {
    GaugeType::Hard
}

impl DanCourse {
    /// Load a course from a JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
        let course: DanCourse = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {:?}", path))?;
        Ok(course)
    }

    /// Get the number of stages in this course
    pub fn stage_count(&self) -> usize {
        self.charts.len()
    }

    /// Resolve chart paths relative to the course file directory
    #[allow(dead_code)]
    pub fn resolve_chart_paths(&self, course_dir: &Path) -> Vec<std::path::PathBuf> {
        self.charts
            .iter()
            .map(|chart| course_dir.join(chart))
            .collect()
    }
}

/// Load all courses from a directory
pub fn load_courses(dir: &Path) -> Vec<(std::path::PathBuf, DanCourse)> {
    let mut courses = Vec::new();

    if !dir.exists() {
        return courses;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively load from subdirectories
                courses.extend(load_courses(&path));
            } else if path.extension().is_some_and(|e| e == "json") {
                if let Ok(course) = DanCourse::load(&path) {
                    courses.push((path, course));
                }
            }
        }
    }

    // Sort by grade
    courses.sort_by(|a, b| a.1.grade.cmp(&b.1.grade));

    courses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_deserialization() {
        let json = r#"
        {
            "name": "SP 初段",
            "grade": { "Dan": 1 },
            "charts": ["stage1.bms", "stage2.bms", "stage3.bms", "stage4.bms"],
            "gauge_type": "Hard",
            "requirements": {
                "min_gauge": 0.0
            }
        }
        "#;

        let course: DanCourse = serde_json::from_str(json).unwrap();
        assert_eq!(course.name, "SP 初段");
        assert_eq!(course.grade, DanGrade::Dan(1));
        assert_eq!(course.charts.len(), 4);
        assert_eq!(course.gauge_type, GaugeType::Hard);
        assert_eq!(course.requirements.min_gauge, 0.0);
    }

    #[test]
    fn test_requirements_default() {
        let json = r#"
        {
            "name": "Test",
            "grade": { "Kyu": 7 },
            "charts": ["test.bms"]
        }
        "#;

        let course: DanCourse = serde_json::from_str(json).unwrap();
        assert_eq!(course.requirements.min_gauge, 0.0);
        assert!(course.requirements.max_bad_poor.is_none());
        assert!(!course.requirements.full_combo);
        assert_eq!(course.gauge_type, GaugeType::Hard);
    }
}
