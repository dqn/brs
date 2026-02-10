use serde::{Deserialize, Serialize};

use crate::chart_data::IRChartData;

/// Course constraint types.
///
/// Corresponds to Java `CourseData.CourseDataConstraint` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CourseDataConstraint {
    /// Grade (段位)
    Class,
    /// Grade (mirror OK)
    Mirror,
    /// Grade (random OK)
    Random,
    /// High-speed disabled
    NoSpeed,
    /// No GOOD judgment
    NoGood,
    /// No GREAT judgment
    NoGreat,
    /// LR2 gauge
    GaugeLr2,
    /// 5KEY gauge
    Gauge5Keys,
    /// 7KEY gauge
    Gauge7Keys,
    /// PMS gauge
    Gauge9Keys,
    /// 24KEY gauge
    Gauge24Keys,
    /// LN mode
    Ln,
    /// CN mode
    Cn,
    /// HCN mode
    Hcn,
}

impl CourseDataConstraint {
    /// Get the constraint name string (matches Java `name` field).
    pub fn constraint_name(self) -> &'static str {
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

    /// Get the constraint type (matches Java `type` field).
    /// 0: grade, 1: speed, 2: judgment, 3: gauge, 4: LN mode
    pub fn constraint_type(self) -> i32 {
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

    /// Look up a constraint by its name string.
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

    /// All constraint variants.
    pub const ALL: [CourseDataConstraint; 14] = [
        Self::Class,
        Self::Mirror,
        Self::Random,
        Self::NoSpeed,
        Self::NoGood,
        Self::NoGreat,
        Self::GaugeLr2,
        Self::Gauge5Keys,
        Self::Gauge7Keys,
        Self::Gauge9Keys,
        Self::Gauge24Keys,
        Self::Ln,
        Self::Cn,
        Self::Hcn,
    ];
}

/// IR trophy data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRTrophyData {
    pub name: String,
    pub scorerate: f32,
    pub smissrate: f32,
}

/// IR course data for transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRCourseData {
    pub name: String,
    pub charts: Vec<IRChartData>,
    pub constraint: Vec<CourseDataConstraint>,
    pub trophy: Vec<IRTrophyData>,
    /// LN TYPE (-1: unspecified, 0: LN, 1: CN, 2: HCN)
    pub lntype: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_name_round_trip() {
        for c in CourseDataConstraint::ALL {
            let name = c.constraint_name();
            let parsed = CourseDataConstraint::from_name(name).unwrap();
            assert_eq!(parsed, c);
        }
    }

    #[test]
    fn constraint_types() {
        assert_eq!(CourseDataConstraint::Class.constraint_type(), 0);
        assert_eq!(CourseDataConstraint::Mirror.constraint_type(), 0);
        assert_eq!(CourseDataConstraint::Random.constraint_type(), 0);
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
    fn from_name_unknown_returns_none() {
        assert!(CourseDataConstraint::from_name("unknown").is_none());
        assert!(CourseDataConstraint::from_name("").is_none());
    }

    #[test]
    fn all_has_14_variants() {
        assert_eq!(CourseDataConstraint::ALL.len(), 14);
    }

    #[test]
    fn serde_round_trip() {
        let course = IRCourseData {
            name: "Test Course".to_string(),
            charts: vec![],
            constraint: vec![CourseDataConstraint::Class, CourseDataConstraint::NoSpeed],
            trophy: vec![IRTrophyData {
                name: "Gold".to_string(),
                scorerate: 90.0,
                smissrate: 2.0,
            }],
            lntype: -1,
        };
        let json = serde_json::to_string(&course).unwrap();
        let deserialized: IRCourseData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Course");
        assert_eq!(deserialized.constraint.len(), 2);
        assert_eq!(deserialized.trophy.len(), 1);
    }
}
