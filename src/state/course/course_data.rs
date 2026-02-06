use serde::{Deserialize, Serialize};

/// Constraint types for course play.
///
/// Corresponds to beatoraja's CourseDataConstraint.
/// Each constraint has a type category (0-4) where only one
/// constraint per category is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CourseConstraint {
    // Category 0: Grade mode
    /// Dan-i (class) mode - no pattern modification allowed.
    Class,
    /// Dan-i with mirror allowed.
    Mirror,
    /// Dan-i with random allowed.
    Random,

    // Category 1: Speed
    /// Hi-speed change disabled.
    NoSpeed,

    // Category 2: Judge
    /// No GOOD judgment.
    NoGood,
    /// No GREAT judgment.
    NoGreat,

    // Category 3: Gauge
    /// LR2-style gauge.
    GaugeLr2,
    /// 5-key gauge.
    Gauge5Keys,
    /// 7-key gauge.
    Gauge7Keys,
    /// 9-key gauge.
    Gauge9Keys,
    /// 24-key gauge.
    Gauge24Keys,

    // Category 4: LN mode
    /// LN mode.
    Ln,
    /// CN mode.
    Cn,
    /// HCN mode.
    Hcn,
}

impl CourseConstraint {
    /// Get the constraint category (0-4).
    /// Only one constraint per category is allowed.
    pub fn category(self) -> u8 {
        match self {
            Self::Class | Self::Mirror | Self::Random => 0,
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

    /// Parse constraint from string name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "grade" => Some(Self::Class),
            "grade_mirror" => Some(Self::Mirror),
            "grade_random" => Some(Self::Random),
            "no_speed" => Some(Self::NoSpeed),
            "no_good" => Some(Self::NoGood),
            "no_great" => Some(Self::NoGreat),
            "gauge_lr2" => Some(Self::GaugeLr2),
            "gauge_5k" => Some(Self::Gauge5Keys),
            "gauge_7k" => Some(Self::Gauge7Keys),
            "gauge_9k" => Some(Self::Gauge9Keys),
            "gauge_24k" => Some(Self::Gauge24Keys),
            "ln" => Some(Self::Ln),
            "cn" => Some(Self::Cn),
            "hcn" => Some(Self::Hcn),
            _ => None,
        }
    }

    /// Get the string name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Class => "grade",
            Self::Mirror => "grade_mirror",
            Self::Random => "grade_random",
            Self::NoSpeed => "no_speed",
            Self::NoGood => "no_good",
            Self::NoGreat => "no_great",
            Self::GaugeLr2 => "gauge_lr2",
            Self::Gauge5Keys => "gauge_5k",
            Self::Gauge7Keys => "gauge_7k",
            Self::Gauge9Keys => "gauge_9k",
            Self::Gauge24Keys => "gauge_24k",
            Self::Ln => "ln",
            Self::Cn => "cn",
            Self::Hcn => "hcn",
        }
    }
}

/// Trophy condition for a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrophyData {
    /// Trophy name.
    pub name: String,
    /// Maximum miss rate (percent of total notes).
    pub miss_rate: f32,
    /// Minimum score rate (percent of max EX score).
    pub score_rate: f32,
}

impl TrophyData {
    /// Check if the trophy condition is valid.
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.miss_rate > 0.0 && self.score_rate < 100.0
    }
}

/// Song reference within a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSong {
    /// SHA-256 hash of the chart.
    pub sha256: String,
    /// Title (for display).
    pub title: String,
}

/// Course data definition.
///
/// Corresponds to beatoraja's CourseData.
/// A course is a sequence of songs played in order with
/// gauge carryover and optional constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseData {
    /// Course name.
    pub name: String,
    /// Songs in the course (played in order).
    pub songs: Vec<CourseSong>,
    /// Active constraints (at most one per category).
    pub constraints: Vec<CourseConstraint>,
    /// Trophy conditions.
    pub trophies: Vec<TrophyData>,
    /// Whether the course is public (for IR).
    pub release: bool,
}

impl CourseData {
    /// Create a new empty course.
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            constraints: Vec::new(),
            trophies: Vec::new(),
            release: true,
        }
    }

    /// Whether this is a class (dan-i) course.
    pub fn is_class_course(&self) -> bool {
        self.constraints.iter().any(|c| {
            matches!(
                c,
                CourseConstraint::Class | CourseConstraint::Mirror | CourseConstraint::Random
            )
        })
    }

    /// Validate the course data. Deduplicates constraints by category.
    /// Returns false if the course has no songs.
    pub fn validate(&mut self) -> bool {
        if self.songs.is_empty() {
            return false;
        }
        if self.name.is_empty() {
            self.name = "No Course Title".to_string();
        }
        for (i, song) in self.songs.iter_mut().enumerate() {
            if song.title.is_empty() {
                song.title = format!("course {}", i + 1);
            }
        }

        // Deduplicate constraints: keep first per category
        let mut seen = [false; 5];
        self.constraints.retain(|c| {
            let cat = c.category() as usize;
            if cat < 5 && !seen[cat] {
                seen[cat] = true;
                true
            } else {
                false
            }
        });

        self.trophies.retain(|t| t.is_valid());
        true
    }

    /// Number of songs in the course.
    pub fn song_count(&self) -> usize {
        self.songs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_category() {
        assert_eq!(CourseConstraint::Class.category(), 0);
        assert_eq!(CourseConstraint::Mirror.category(), 0);
        assert_eq!(CourseConstraint::NoSpeed.category(), 1);
        assert_eq!(CourseConstraint::NoGood.category(), 2);
        assert_eq!(CourseConstraint::Gauge7Keys.category(), 3);
        assert_eq!(CourseConstraint::Hcn.category(), 4);
    }

    #[test]
    fn constraint_from_name() {
        assert_eq!(
            CourseConstraint::from_name("grade"),
            Some(CourseConstraint::Class)
        );
        assert_eq!(
            CourseConstraint::from_name("gauge_7k"),
            Some(CourseConstraint::Gauge7Keys)
        );
        assert_eq!(CourseConstraint::from_name("unknown"), None);
    }

    #[test]
    fn constraint_round_trip() {
        let constraints = [
            CourseConstraint::Class,
            CourseConstraint::NoSpeed,
            CourseConstraint::NoGood,
            CourseConstraint::Gauge7Keys,
            CourseConstraint::Hcn,
        ];
        for c in &constraints {
            assert_eq!(CourseConstraint::from_name(c.name()), Some(*c));
        }
    }

    #[test]
    fn course_validate_empty_songs_fails() {
        let mut course = CourseData::new("Test".to_string());
        assert!(!course.validate());
    }

    #[test]
    fn course_validate_deduplicates_constraints() {
        let mut course = CourseData::new("Test".to_string());
        course.songs.push(CourseSong {
            sha256: "h1".to_string(),
            title: "Song 1".to_string(),
        });
        // Add two constraints of same category
        course.constraints.push(CourseConstraint::Class);
        course.constraints.push(CourseConstraint::Mirror);
        course.constraints.push(CourseConstraint::NoSpeed);

        assert!(course.validate());
        // Only one of category 0 should remain (Class, the first)
        assert_eq!(course.constraints.len(), 2);
        assert_eq!(course.constraints[0], CourseConstraint::Class);
        assert_eq!(course.constraints[1], CourseConstraint::NoSpeed);
    }

    #[test]
    fn course_validate_fills_empty_names() {
        let mut course = CourseData::new(String::new());
        course.songs.push(CourseSong {
            sha256: "h1".to_string(),
            title: String::new(),
        });

        assert!(course.validate());
        assert_eq!(course.name, "No Course Title");
        assert_eq!(course.songs[0].title, "course 1");
    }

    #[test]
    fn is_class_course() {
        let mut course = CourseData::new("Dan".to_string());
        course.songs.push(CourseSong {
            sha256: "h1".to_string(),
            title: "S1".to_string(),
        });
        assert!(!course.is_class_course());

        course.constraints.push(CourseConstraint::Class);
        assert!(course.is_class_course());
    }

    #[test]
    fn trophy_validation() {
        let valid = TrophyData {
            name: "Gold".to_string(),
            miss_rate: 2.0,
            score_rate: 90.0,
        };
        assert!(valid.is_valid());

        let invalid = TrophyData {
            name: String::new(),
            miss_rate: 2.0,
            score_rate: 90.0,
        };
        assert!(!invalid.is_valid());
    }
}
