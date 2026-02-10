use serde::{Deserialize, Serialize};

use crate::score_data::IRScoreData;

/// Type of IR source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IRType {
    Primary,
    LR2,
}

/// Leaderboard entry wrapping an IR score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub ir_score: IRScoreData,
    pub ir_type: IRType,
    pub lr2_id: i64,
}

impl LeaderboardEntry {
    /// Create a new primary IR entry.
    pub fn new_primary(ir_score: IRScoreData) -> Self {
        Self {
            ir_score,
            ir_type: IRType::Primary,
            lr2_id: 0,
        }
    }

    /// Create a new LR2 IR entry.
    pub fn new_lr2(ir_score: IRScoreData, lr2_id: i64) -> Self {
        Self {
            ir_score,
            ir_type: IRType::LR2,
            lr2_id,
        }
    }

    pub fn is_primary_ir(&self) -> bool {
        self.ir_type == IRType::Primary
    }

    pub fn is_lr2_ir(&self) -> bool {
        self.ir_type == IRType::LR2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_rule::ScoreData;

    fn sample_ir_score() -> IRScoreData {
        let mut sd = ScoreData::default();
        sd.epg = 100;
        sd.lpg = 50;
        sd.egr = 30;
        sd.lgr = 20;
        IRScoreData::from(&sd)
    }

    #[test]
    fn new_primary_entry() {
        let entry = LeaderboardEntry::new_primary(sample_ir_score());
        assert!(entry.is_primary_ir());
        assert!(!entry.is_lr2_ir());
        assert_eq!(entry.lr2_id, 0);
    }

    #[test]
    fn new_lr2_entry() {
        let entry = LeaderboardEntry::new_lr2(sample_ir_score(), 12345);
        assert!(!entry.is_primary_ir());
        assert!(entry.is_lr2_ir());
        assert_eq!(entry.lr2_id, 12345);
    }

    #[test]
    fn exscore_through_entry() {
        let entry = LeaderboardEntry::new_primary(sample_ir_score());
        assert_eq!(entry.ir_score.exscore(), 350);
    }
}
