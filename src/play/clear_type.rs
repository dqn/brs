use serde::{Deserialize, Serialize};

/// Clear type for a play result.
/// Corresponds to ClearType enum in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ClearType {
    NoPlay = 0,
    Failed = 1,
    AssistEasy = 2,
    LightAssistEasy = 3,
    Easy = 4,
    Normal = 5,
    Hard = 6,
    ExHard = 7,
    FullCombo = 8,
    Perfect = 9,
    Max = 10,
}

impl ClearType {
    /// Get ClearType from integer ID.
    /// Returns NoPlay for unknown IDs.
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Self::NoPlay,
            1 => Self::Failed,
            2 => Self::AssistEasy,
            3 => Self::LightAssistEasy,
            4 => Self::Easy,
            5 => Self::Normal,
            6 => Self::Hard,
            7 => Self::ExHard,
            8 => Self::FullCombo,
            9 => Self::Perfect,
            10 => Self::Max,
            _ => Self::NoPlay,
        }
    }

    /// Get ClearType corresponding to a gauge type index.
    /// Returns None if no clear type maps to the given gauge type.
    ///
    /// Gauge type mapping (from beatoraja):
    /// - LightAssistEasy: [0] (ASSIST_EASY)
    /// - Easy: [1] (EASY)
    /// - Normal: [2, 6] (NORMAL, CLASS)
    /// - Hard: [3, 7] (HARD, EXCLASS)
    /// - ExHard: [4, 8] (EXHARD, EXHARDCLASS)
    /// - FullCombo: [5] (HAZARD)
    pub fn from_gauge_type(gauge_type: usize) -> Option<Self> {
        match gauge_type {
            0 => Some(Self::LightAssistEasy),
            1 => Some(Self::Easy),
            2 | 6 => Some(Self::Normal),
            3 | 7 => Some(Self::Hard),
            4 | 8 => Some(Self::ExHard),
            5 => Some(Self::FullCombo),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_id_all_valid() {
        assert_eq!(ClearType::from_id(0), ClearType::NoPlay);
        assert_eq!(ClearType::from_id(1), ClearType::Failed);
        assert_eq!(ClearType::from_id(2), ClearType::AssistEasy);
        assert_eq!(ClearType::from_id(3), ClearType::LightAssistEasy);
        assert_eq!(ClearType::from_id(4), ClearType::Easy);
        assert_eq!(ClearType::from_id(5), ClearType::Normal);
        assert_eq!(ClearType::from_id(6), ClearType::Hard);
        assert_eq!(ClearType::from_id(7), ClearType::ExHard);
        assert_eq!(ClearType::from_id(8), ClearType::FullCombo);
        assert_eq!(ClearType::from_id(9), ClearType::Perfect);
        assert_eq!(ClearType::from_id(10), ClearType::Max);
    }

    #[test]
    fn from_id_unknown_returns_noplay() {
        assert_eq!(ClearType::from_id(11), ClearType::NoPlay);
        assert_eq!(ClearType::from_id(255), ClearType::NoPlay);
    }

    #[test]
    fn from_gauge_type_matches_beatoraja() {
        // LightAssistEasy: gauge type 0 (ASSIST_EASY)
        assert_eq!(
            ClearType::from_gauge_type(0),
            Some(ClearType::LightAssistEasy)
        );
        // Easy: gauge type 1
        assert_eq!(ClearType::from_gauge_type(1), Some(ClearType::Easy));
        // Normal: gauge type 2 (NORMAL) and 6 (CLASS)
        assert_eq!(ClearType::from_gauge_type(2), Some(ClearType::Normal));
        assert_eq!(ClearType::from_gauge_type(6), Some(ClearType::Normal));
        // Hard: gauge type 3 (HARD) and 7 (EXCLASS)
        assert_eq!(ClearType::from_gauge_type(3), Some(ClearType::Hard));
        assert_eq!(ClearType::from_gauge_type(7), Some(ClearType::Hard));
        // ExHard: gauge type 4 (EXHARD) and 8 (EXHARDCLASS)
        assert_eq!(ClearType::from_gauge_type(4), Some(ClearType::ExHard));
        assert_eq!(ClearType::from_gauge_type(8), Some(ClearType::ExHard));
        // FullCombo: gauge type 5 (HAZARD)
        assert_eq!(ClearType::from_gauge_type(5), Some(ClearType::FullCombo));
    }

    #[test]
    fn from_gauge_type_unknown() {
        assert_eq!(ClearType::from_gauge_type(9), None);
        assert_eq!(ClearType::from_gauge_type(100), None);
    }

    #[test]
    fn ordering_matches_id() {
        assert!(ClearType::NoPlay < ClearType::Failed);
        assert!(ClearType::Failed < ClearType::AssistEasy);
        assert!(ClearType::AssistEasy < ClearType::LightAssistEasy);
        assert!(ClearType::LightAssistEasy < ClearType::Easy);
        assert!(ClearType::Easy < ClearType::Normal);
        assert!(ClearType::Normal < ClearType::Hard);
        assert!(ClearType::Hard < ClearType::ExHard);
        assert!(ClearType::ExHard < ClearType::FullCombo);
        assert!(ClearType::FullCombo < ClearType::Perfect);
        assert!(ClearType::Perfect < ClearType::Max);
    }
}
