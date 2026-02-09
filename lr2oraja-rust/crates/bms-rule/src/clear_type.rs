use serde::{Deserialize, Serialize};

/// Clear type representing the player's achievement on a chart.
///
/// Ordered from lowest (NoPlay) to highest (Max) achievement.
/// Each variant maps to a numeric ID for database storage.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[repr(u8)]
pub enum ClearType {
    #[default]
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
    /// Convert a numeric ID to the corresponding ClearType.
    ///
    /// Returns `None` if the ID does not correspond to any variant.
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::NoPlay),
            1 => Some(Self::Failed),
            2 => Some(Self::AssistEasy),
            3 => Some(Self::LightAssistEasy),
            4 => Some(Self::Easy),
            5 => Some(Self::Normal),
            6 => Some(Self::Hard),
            7 => Some(Self::ExHard),
            8 => Some(Self::FullCombo),
            9 => Some(Self::Perfect),
            10 => Some(Self::Max),
            _ => None,
        }
    }

    /// Get the numeric ID for this clear type.
    pub fn id(self) -> u8 {
        self as u8
    }

    /// Get the ClearType that corresponds to a gauge type index.
    ///
    /// Gauge type mapping (from Java ClearType.gaugetype):
    /// - 0 => LightAssistEasy
    /// - 1 => Easy
    /// - 2 => Normal
    /// - 3 => Hard
    /// - 4 => ExHard
    /// - 5 => FullCombo
    /// - 6 => Normal
    /// - 7 => Hard
    /// - 8 => ExHard
    ///
    /// Returns `None` for unmapped gauge types.
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
    fn from_id_round_trip_all_variants() {
        let variants = [
            (0, ClearType::NoPlay),
            (1, ClearType::Failed),
            (2, ClearType::AssistEasy),
            (3, ClearType::LightAssistEasy),
            (4, ClearType::Easy),
            (5, ClearType::Normal),
            (6, ClearType::Hard),
            (7, ClearType::ExHard),
            (8, ClearType::FullCombo),
            (9, ClearType::Perfect),
            (10, ClearType::Max),
        ];

        for (id, expected) in variants {
            let ct = ClearType::from_id(id).unwrap();
            assert_eq!(ct, expected);
            assert_eq!(ct.id(), id);
        }
    }

    #[test]
    fn from_id_invalid_returns_none() {
        assert_eq!(ClearType::from_id(11), None);
        assert_eq!(ClearType::from_id(255), None);
    }

    #[test]
    fn from_gauge_type_mapping() {
        assert_eq!(
            ClearType::from_gauge_type(0),
            Some(ClearType::LightAssistEasy)
        );
        assert_eq!(ClearType::from_gauge_type(1), Some(ClearType::Easy));
        assert_eq!(ClearType::from_gauge_type(2), Some(ClearType::Normal));
        assert_eq!(ClearType::from_gauge_type(3), Some(ClearType::Hard));
        assert_eq!(ClearType::from_gauge_type(4), Some(ClearType::ExHard));
        assert_eq!(ClearType::from_gauge_type(5), Some(ClearType::FullCombo));
        // Alternate gauge types (6, 7, 8) map to Normal, Hard, ExHard
        assert_eq!(ClearType::from_gauge_type(6), Some(ClearType::Normal));
        assert_eq!(ClearType::from_gauge_type(7), Some(ClearType::Hard));
        assert_eq!(ClearType::from_gauge_type(8), Some(ClearType::ExHard));
    }

    #[test]
    fn from_gauge_type_unmapped_returns_none() {
        assert_eq!(ClearType::from_gauge_type(9), None);
        assert_eq!(ClearType::from_gauge_type(100), None);
    }

    #[test]
    fn default_is_no_play() {
        assert_eq!(ClearType::default(), ClearType::NoPlay);
    }

    #[test]
    fn ordering_is_ascending() {
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

    #[test]
    fn serde_round_trip() {
        for id in 0..=10 {
            let ct = ClearType::from_id(id).unwrap();
            let json = serde_json::to_string(&ct).unwrap();
            let deserialized: ClearType = serde_json::from_str(&json).unwrap();
            assert_eq!(ct, deserialized);
        }
    }
}
