use serde::{Deserialize, Serialize};

/// Dan certification grade levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DanGrade {
    /// Kyu grades (7th kyu to 1st kyu)
    Kyu(u8),
    /// Dan grades (1st dan to 10th dan)
    Dan(u8),
    /// Kaiden (master rank)
    Kaiden,
    /// Overjoy (beyond master)
    Overjoy,
}

impl DanGrade {
    /// Returns the display name for this grade
    pub fn display_name(&self) -> String {
        match self {
            DanGrade::Kyu(n) => format!("{}級", n),
            DanGrade::Dan(n) => format!("{}段", n),
            DanGrade::Kaiden => "皆伝".to_string(),
            DanGrade::Overjoy => "OVERJOY".to_string(),
        }
    }

    /// Returns the next higher grade, if any
    #[allow(dead_code)]
    pub fn next(&self) -> Option<DanGrade> {
        match self {
            DanGrade::Kyu(1) => Some(DanGrade::Dan(1)),
            DanGrade::Kyu(n) => Some(DanGrade::Kyu(n - 1)),
            DanGrade::Dan(10) => Some(DanGrade::Kaiden),
            DanGrade::Dan(n) => Some(DanGrade::Dan(n + 1)),
            DanGrade::Kaiden => Some(DanGrade::Overjoy),
            DanGrade::Overjoy => None,
        }
    }

    /// Returns a sort key for ordering grades (lower is easier)
    pub fn sort_key(&self) -> i32 {
        match self {
            DanGrade::Kyu(n) => -((*n as i32) - 8), // 7kyu=-1, 1kyu=7
            DanGrade::Dan(n) => 7 + (*n as i32),    // 1dan=8, 10dan=17
            DanGrade::Kaiden => 18,
            DanGrade::Overjoy => 19,
        }
    }

    /// Creates a grade from a sort key
    #[allow(dead_code)]
    pub fn from_sort_key(key: i32) -> Option<DanGrade> {
        match key {
            -1..=6 => Some(DanGrade::Kyu((8 - key) as u8)),
            8..=17 => Some(DanGrade::Dan((key - 7) as u8)),
            18 => Some(DanGrade::Kaiden),
            19 => Some(DanGrade::Overjoy),
            _ => None,
        }
    }
}

impl PartialOrd for DanGrade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DanGrade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl Default for DanGrade {
    fn default() -> Self {
        DanGrade::Kyu(7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        assert_eq!(DanGrade::Kyu(7).display_name(), "7級");
        assert_eq!(DanGrade::Kyu(1).display_name(), "1級");
        assert_eq!(DanGrade::Dan(1).display_name(), "1段");
        assert_eq!(DanGrade::Dan(10).display_name(), "10段");
        assert_eq!(DanGrade::Kaiden.display_name(), "皆伝");
        assert_eq!(DanGrade::Overjoy.display_name(), "OVERJOY");
    }

    #[test]
    fn test_next() {
        assert_eq!(DanGrade::Kyu(7).next(), Some(DanGrade::Kyu(6)));
        assert_eq!(DanGrade::Kyu(1).next(), Some(DanGrade::Dan(1)));
        assert_eq!(DanGrade::Dan(1).next(), Some(DanGrade::Dan(2)));
        assert_eq!(DanGrade::Dan(10).next(), Some(DanGrade::Kaiden));
        assert_eq!(DanGrade::Kaiden.next(), Some(DanGrade::Overjoy));
        assert_eq!(DanGrade::Overjoy.next(), None);
    }

    #[test]
    fn test_ordering() {
        assert!(DanGrade::Kyu(7) < DanGrade::Kyu(1));
        assert!(DanGrade::Kyu(1) < DanGrade::Dan(1));
        assert!(DanGrade::Dan(1) < DanGrade::Dan(10));
        assert!(DanGrade::Dan(10) < DanGrade::Kaiden);
        assert!(DanGrade::Kaiden < DanGrade::Overjoy);
    }
}
