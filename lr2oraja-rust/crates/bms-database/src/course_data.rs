//! Course data structures.
//!
//! Port of Java `CourseData.java`.

use serde::{Deserialize, Serialize};

/// Course constraint types.
///
/// Each constraint has a `constraint_type()` returning 0-4, used for deduplication
/// (only one constraint per type is kept during validation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourseDataConstraint {
    /// Class grade (段位) — type 0
    #[serde(rename = "grade")]
    Class,
    /// Class grade with mirror allowed — type 0
    GradeMirror,
    /// Class grade with random allowed — type 0
    GradeRandom,
    /// No speed change allowed — type 1
    NoSpeed,
    /// No GOOD judgement — type 2
    NoGood,
    /// No GREAT judgement — type 2
    NoGreat,
    /// LR2 gauge — type 3
    GaugeLr2,
    /// 5-key gauge — type 3
    #[serde(rename = "gauge_5k")]
    Gauge5Keys,
    /// 7-key gauge — type 3
    #[serde(rename = "gauge_7k")]
    Gauge7Keys,
    /// 9-key (PMS) gauge — type 3
    #[serde(rename = "gauge_9k")]
    Gauge9Keys,
    /// 24-key gauge — type 3
    #[serde(rename = "gauge_24k")]
    Gauge24Keys,
    /// LN mode — type 4
    Ln,
    /// CN mode — type 4
    Cn,
    /// HCN mode — type 4
    Hcn,
}

impl CourseDataConstraint {
    /// Returns the constraint type (0-4).
    /// During validation, only one constraint per type is kept.
    pub fn constraint_type(self) -> i32 {
        match self {
            Self::Class | Self::GradeMirror | Self::GradeRandom => 0,
            Self::NoSpeed => 1,
            Self::NoGood | Self::NoGreat => 2,
            Self::GaugeLr2
            | Self::Gauge5Keys
            | Self::Gauge7Keys
            | Self::Gauge9Keys
            | Self::Gauge24Keys => 3,
            Self::Ln | Self::Cn | Self::Hcn => 4,
        }
    }

    /// Returns true if this is a class-course constraint.
    pub fn is_class(self) -> bool {
        matches!(self, Self::Class | Self::GradeMirror | Self::GradeRandom)
    }
}

/// Trophy condition for a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrophyData {
    pub name: String,
    pub missrate: f32,
    pub scorerate: f32,
}

impl TrophyData {
    /// Validate that the trophy data is valid.
    /// Matches Java: `name != null && missrate > 0 && scorerate < 100`.
    pub fn validate(&self) -> bool {
        !self.name.is_empty() && self.missrate > 0.0 && self.scorerate < 100.0
    }
}

/// Song hash reference for course stages (lightweight, no full SongData).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CourseSongData {
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub md5: String,
    #[serde(default)]
    pub title: String,
}

impl CourseSongData {
    /// Validate that the song reference has at least one hash.
    pub fn validate(&self) -> bool {
        !self.md5.is_empty() || !self.sha256.is_empty()
    }

    /// Clear transient fields (shrink for serialization).
    pub fn shrink(&mut self) {
        // CourseSongData is already minimal; nothing to clear.
    }
}

fn default_release() -> bool {
    true
}

/// Course data containing song references, constraints, and trophy conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseData {
    pub name: String,
    #[serde(default)]
    pub hash: Vec<CourseSongData>,
    #[serde(default)]
    pub constraint: Vec<CourseDataConstraint>,
    #[serde(default)]
    pub trophy: Vec<TrophyData>,
    #[serde(default = "default_release")]
    pub release: bool,
}

impl Default for CourseData {
    fn default() -> Self {
        Self {
            name: String::new(),
            hash: Vec::new(),
            constraint: Vec::new(),
            trophy: Vec::new(),
            release: true,
        }
    }
}

impl CourseData {
    /// Returns true if this course has a class-course constraint.
    pub fn is_class_course(&self) -> bool {
        self.constraint.iter().any(|c| c.is_class())
    }

    /// Validate course data.
    ///
    /// - Ensures at least one song reference exists.
    /// - Sets default name if empty.
    /// - Sets default title for songs with empty titles.
    /// - Deduplicates constraints by type (keeps first per type).
    /// - Removes invalid trophies.
    pub fn validate(&mut self) -> bool {
        if self.hash.is_empty() {
            return false;
        }

        if self.name.is_empty() {
            self.name = "No Course Title".to_string();
        }

        for (i, song) in self.hash.iter_mut().enumerate() {
            if song.title.is_empty() {
                song.title = format!("course {}", i + 1);
            }
            if !song.validate() {
                return false;
            }
        }

        // Deduplicate constraints by type (keep first per type).
        // Java uses a 5-slot array indexed by constraint type.
        let mut seen = [false; 5];
        let mut deduped = Vec::new();
        for &c in &self.constraint {
            let ct = c.constraint_type() as usize;
            if ct < seen.len() && !seen[ct] {
                seen[ct] = true;
                deduped.push(c);
            }
        }
        self.constraint = deduped;

        // Remove invalid trophies
        self.trophy.retain(|t| t.validate());

        true
    }

    /// Clear transient fields from song references.
    pub fn shrink(&mut self) {
        for song in &mut self.hash {
            song.shrink();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_song(sha: &str) -> CourseSongData {
        CourseSongData {
            sha256: sha.to_string(),
            md5: String::new(),
            title: String::new(),
        }
    }

    #[test]
    fn constraint_types() {
        assert_eq!(CourseDataConstraint::Class.constraint_type(), 0);
        assert_eq!(CourseDataConstraint::GradeMirror.constraint_type(), 0);
        assert_eq!(CourseDataConstraint::GradeRandom.constraint_type(), 0);
        assert_eq!(CourseDataConstraint::NoSpeed.constraint_type(), 1);
        assert_eq!(CourseDataConstraint::NoGood.constraint_type(), 2);
        assert_eq!(CourseDataConstraint::NoGreat.constraint_type(), 2);
        assert_eq!(CourseDataConstraint::GaugeLr2.constraint_type(), 3);
        assert_eq!(CourseDataConstraint::Gauge5Keys.constraint_type(), 3);
        assert_eq!(CourseDataConstraint::Gauge7Keys.constraint_type(), 3);
        assert_eq!(CourseDataConstraint::Gauge9Keys.constraint_type(), 3);
        assert_eq!(CourseDataConstraint::Gauge24Keys.constraint_type(), 3);
        assert_eq!(CourseDataConstraint::Ln.constraint_type(), 4);
        assert_eq!(CourseDataConstraint::Cn.constraint_type(), 4);
        assert_eq!(CourseDataConstraint::Hcn.constraint_type(), 4);
    }

    #[test]
    fn is_class() {
        assert!(CourseDataConstraint::Class.is_class());
        assert!(CourseDataConstraint::GradeMirror.is_class());
        assert!(CourseDataConstraint::GradeRandom.is_class());
        assert!(!CourseDataConstraint::NoSpeed.is_class());
        assert!(!CourseDataConstraint::Ln.is_class());
    }

    #[test]
    fn trophy_validate() {
        let valid = TrophyData {
            name: "Gold".to_string(),
            missrate: 5.0,
            scorerate: 90.0,
        };
        assert!(valid.validate());

        let no_name = TrophyData {
            name: String::new(),
            missrate: 5.0,
            scorerate: 90.0,
        };
        assert!(!no_name.validate());

        let zero_miss = TrophyData {
            name: "Test".to_string(),
            missrate: 0.0,
            scorerate: 90.0,
        };
        assert!(!zero_miss.validate());

        let high_score = TrophyData {
            name: "Test".to_string(),
            missrate: 5.0,
            scorerate: 100.0,
        };
        assert!(!high_score.validate());
    }

    #[test]
    fn course_song_data_validate() {
        let valid = CourseSongData {
            sha256: "abc".to_string(),
            md5: String::new(),
            title: String::new(),
        };
        assert!(valid.validate());

        let valid_md5 = CourseSongData {
            sha256: String::new(),
            md5: "def".to_string(),
            title: String::new(),
        };
        assert!(valid_md5.validate());

        let invalid = CourseSongData {
            sha256: String::new(),
            md5: String::new(),
            title: String::new(),
        };
        assert!(!invalid.validate());
    }

    #[test]
    fn course_data_validate_empty_songs() {
        let mut course = CourseData::default();
        assert!(!course.validate());
    }

    #[test]
    fn course_data_validate_sets_defaults() {
        let mut course = CourseData {
            name: String::new(),
            hash: vec![sample_song("abc123")],
            constraint: Vec::new(),
            trophy: Vec::new(),
            release: true,
        };
        assert!(course.validate());
        assert_eq!(course.name, "No Course Title");
        assert_eq!(course.hash[0].title, "course 1");
    }

    #[test]
    fn course_data_validate_deduplicates_constraints() {
        let mut course = CourseData {
            name: "Test".to_string(),
            hash: vec![sample_song("abc")],
            constraint: vec![
                CourseDataConstraint::Class,
                CourseDataConstraint::GradeMirror, // same type 0, should be dropped
                CourseDataConstraint::NoSpeed,
                CourseDataConstraint::NoGood,
                CourseDataConstraint::NoGreat, // same type 2, should be dropped
                CourseDataConstraint::Ln,
            ],
            trophy: Vec::new(),
            release: true,
        };
        assert!(course.validate());
        assert_eq!(course.constraint.len(), 4);
        assert_eq!(course.constraint[0], CourseDataConstraint::Class);
        assert_eq!(course.constraint[1], CourseDataConstraint::NoSpeed);
        assert_eq!(course.constraint[2], CourseDataConstraint::NoGood);
        assert_eq!(course.constraint[3], CourseDataConstraint::Ln);
    }

    #[test]
    fn course_data_validate_removes_invalid_trophies() {
        let mut course = CourseData {
            name: "Test".to_string(),
            hash: vec![sample_song("abc")],
            constraint: Vec::new(),
            trophy: vec![
                TrophyData {
                    name: "Gold".to_string(),
                    missrate: 5.0,
                    scorerate: 90.0,
                },
                TrophyData {
                    name: String::new(),
                    missrate: 5.0,
                    scorerate: 90.0,
                },
            ],
            release: true,
        };
        assert!(course.validate());
        assert_eq!(course.trophy.len(), 1);
        assert_eq!(course.trophy[0].name, "Gold");
    }

    #[test]
    fn course_data_validate_invalid_song_hash() {
        let mut course = CourseData {
            name: "Test".to_string(),
            hash: vec![CourseSongData {
                sha256: String::new(),
                md5: String::new(),
                title: "test".to_string(),
            }],
            constraint: Vec::new(),
            trophy: Vec::new(),
            release: true,
        };
        assert!(!course.validate());
    }

    #[test]
    fn is_class_course() {
        let course = CourseData {
            name: "Dan".to_string(),
            hash: vec![sample_song("a")],
            constraint: vec![CourseDataConstraint::Class, CourseDataConstraint::NoSpeed],
            trophy: Vec::new(),
            release: true,
        };
        assert!(course.is_class_course());

        let non_class = CourseData {
            name: "Regular".to_string(),
            hash: vec![sample_song("b")],
            constraint: vec![CourseDataConstraint::NoSpeed],
            trophy: Vec::new(),
            release: true,
        };
        assert!(!non_class.is_class_course());
    }

    #[test]
    fn default_release_is_true() {
        let course = CourseData::default();
        assert!(course.release);
    }

    #[test]
    fn serde_roundtrip() {
        let course = CourseData {
            name: "Test Course".to_string(),
            hash: vec![CourseSongData {
                sha256: "abc".to_string(),
                md5: "def".to_string(),
                title: "Song 1".to_string(),
            }],
            constraint: vec![CourseDataConstraint::Class, CourseDataConstraint::Ln],
            trophy: vec![TrophyData {
                name: "Gold".to_string(),
                missrate: 3.0,
                scorerate: 85.0,
            }],
            release: false,
        };

        let json = serde_json::to_string_pretty(&course).unwrap();
        let parsed: CourseData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test Course");
        assert_eq!(parsed.hash.len(), 1);
        assert_eq!(parsed.constraint.len(), 2);
        assert_eq!(parsed.trophy.len(), 1);
        assert!(!parsed.release);
    }

    #[test]
    fn serde_deserialize_constraint_names() {
        // Verify the serde rename_all works for JSON strings
        let json = r#""grade""#;
        let c: CourseDataConstraint = serde_json::from_str(json).unwrap();
        assert_eq!(c, CourseDataConstraint::Class);

        let json = r#""grade_mirror""#;
        let c: CourseDataConstraint = serde_json::from_str(json).unwrap();
        assert_eq!(c, CourseDataConstraint::GradeMirror);

        let json = r#""gauge_lr2""#;
        let c: CourseDataConstraint = serde_json::from_str(json).unwrap();
        assert_eq!(c, CourseDataConstraint::GaugeLr2);

        let json = r#""gauge_5k""#;
        let c: CourseDataConstraint = serde_json::from_str(json).unwrap();
        assert_eq!(c, CourseDataConstraint::Gauge5Keys);

        let json = r#""ln""#;
        let c: CourseDataConstraint = serde_json::from_str(json).unwrap();
        assert_eq!(c, CourseDataConstraint::Ln);
    }
}
