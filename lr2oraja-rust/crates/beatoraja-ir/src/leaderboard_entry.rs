use crate::ir_score_data::IRScoreData;

/// IR type enum
///
/// Translated from: LeaderboardEntry.IRType
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IRType {
    Primary,
    LR2,
}

/// Leaderboard entry
///
/// Translated from: LeaderboardEntry.java
#[derive(Clone, Debug)]
pub struct LeaderboardEntry {
    ir_score: IRScoreData,
    ir_type: IRType,
    lr2_id: i64,
}

impl LeaderboardEntry {
    fn new(ir_score: IRScoreData, ir_type: IRType) -> Self {
        Self {
            ir_score,
            ir_type,
            lr2_id: 0,
        }
    }

    pub fn new_entry_primary_ir(ir_score: IRScoreData) -> Self {
        Self::new(ir_score, IRType::Primary)
    }

    pub fn new_entry_lr2_ir(ir_score: IRScoreData, lr2_id: i64) -> Self {
        let mut entry = Self::new(ir_score, IRType::LR2);
        entry.lr2_id = lr2_id;
        entry
    }

    pub fn get_ir_score(&self) -> &IRScoreData {
        &self.ir_score
    }

    pub fn is_primary_ir(&self) -> bool {
        self.ir_type == IRType::Primary
    }

    pub fn is_lr2_ir(&self) -> bool {
        self.ir_type == IRType::LR2
    }

    pub fn get_lr2_id(&self) -> i64 {
        self.lr2_id
    }
}
