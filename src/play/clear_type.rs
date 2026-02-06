use serde::{Deserialize, Serialize};

use super::gauge::gauge_property::GaugeType;

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

    /// Get ClearType corresponding to a gauge type.
    ///
    /// Gauge type mapping (from beatoraja):
    /// - LightAssistEasy: AssistEasy
    /// - Easy: Easy
    /// - Normal: Normal, Class
    /// - Hard: Hard, ExClass
    /// - ExHard: ExHard, ExHardClass
    /// - FullCombo: Hazard
    pub fn from_gauge_type(gauge_type: GaugeType) -> Self {
        match gauge_type {
            GaugeType::AssistEasy => Self::LightAssistEasy,
            GaugeType::Easy => Self::Easy,
            GaugeType::Normal | GaugeType::Class => Self::Normal,
            GaugeType::Hard | GaugeType::ExClass => Self::Hard,
            GaugeType::ExHard | GaugeType::ExHardClass => Self::ExHard,
            GaugeType::Hazard => Self::FullCombo,
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
        use crate::play::gauge::gauge_property::GaugeType;

        assert_eq!(
            ClearType::from_gauge_type(GaugeType::AssistEasy),
            ClearType::LightAssistEasy
        );
        assert_eq!(ClearType::from_gauge_type(GaugeType::Easy), ClearType::Easy);
        assert_eq!(
            ClearType::from_gauge_type(GaugeType::Normal),
            ClearType::Normal
        );
        assert_eq!(
            ClearType::from_gauge_type(GaugeType::Class),
            ClearType::Normal
        );
        assert_eq!(ClearType::from_gauge_type(GaugeType::Hard), ClearType::Hard);
        assert_eq!(
            ClearType::from_gauge_type(GaugeType::ExClass),
            ClearType::Hard
        );
        assert_eq!(
            ClearType::from_gauge_type(GaugeType::ExHard),
            ClearType::ExHard
        );
        assert_eq!(
            ClearType::from_gauge_type(GaugeType::ExHardClass),
            ClearType::ExHard
        );
        assert_eq!(
            ClearType::from_gauge_type(GaugeType::Hazard),
            ClearType::FullCombo
        );
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
