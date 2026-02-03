use crate::state::play::JudgeRank;

/// Score tracker for gameplay.
#[derive(Debug, Clone, Default)]
pub struct Score {
    pub pg_count: u32,
    pub gr_count: u32,
    pub gd_count: u32,
    pub bd_count: u32,
    pub pr_count: u32,
    pub ms_count: u32,
    pub combo: u32,
    pub max_combo: u32,
    total_notes: u32,
}

impl Score {
    /// Create a new score tracker.
    pub fn new(total_notes: u32) -> Self {
        Self {
            pg_count: 0,
            gr_count: 0,
            gd_count: 0,
            bd_count: 0,
            pr_count: 0,
            ms_count: 0,
            combo: 0,
            max_combo: 0,
            total_notes,
        }
    }

    /// Update the score based on a judge result.
    pub fn update(&mut self, rank: JudgeRank) {
        match rank {
            JudgeRank::PerfectGreat => {
                self.pg_count += 1;
                self.combo += 1;
            }
            JudgeRank::Great => {
                self.gr_count += 1;
                self.combo += 1;
            }
            JudgeRank::Good => {
                self.gd_count += 1;
                self.combo += 1;
            }
            JudgeRank::Bad => {
                self.bd_count += 1;
                self.combo = 0;
            }
            JudgeRank::Poor => {
                self.pr_count += 1;
                self.combo = 0;
            }
            JudgeRank::Miss => {
                self.ms_count += 1;
                self.combo = 0;
            }
        }

        if self.combo > self.max_combo {
            self.max_combo = self.combo;
        }
    }

    /// Reset score state to the initial values.
    /// スコア状態を初期値に戻す。
    pub fn reset(&mut self) {
        self.pg_count = 0;
        self.gr_count = 0;
        self.gd_count = 0;
        self.bd_count = 0;
        self.pr_count = 0;
        self.ms_count = 0;
        self.combo = 0;
        self.max_combo = 0;
    }

    /// Calculate the EX-SCORE (PG*2 + GR).
    pub fn ex_score(&self) -> u32 {
        self.pg_count * 2 + self.gr_count
    }

    /// Calculate the maximum possible EX-SCORE.
    pub fn max_ex_score(&self) -> u32 {
        self.total_notes * 2
    }

    /// Calculate the clear rate (0.0 - 100.0).
    pub fn clear_rate(&self) -> f64 {
        let max = self.max_ex_score();
        if max == 0 {
            return 0.0;
        }
        (self.ex_score() as f64 / max as f64) * 100.0
    }

    /// Get the total number of judged notes.
    pub fn judged_count(&self) -> u32 {
        self.pg_count
            + self.gr_count
            + self.gd_count
            + self.bd_count
            + self.pr_count
            + self.ms_count
    }

    /// Get the total number of notes.
    pub fn total_notes(&self) -> u32 {
        self.total_notes
    }

    /// Get the BP (Bad + Poor + Miss count).
    pub fn bp(&self) -> u32 {
        self.bd_count + self.pr_count + self.ms_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_initial() {
        let score = Score::new(100);
        assert_eq!(score.ex_score(), 0);
        assert_eq!(score.combo, 0);
        assert_eq!(score.max_combo, 0);
    }

    #[test]
    fn test_ex_score_calculation() {
        let mut score = Score::new(100);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::Great);
        assert_eq!(score.ex_score(), 5); // 2*2 + 1 = 5
    }

    #[test]
    fn test_combo_continues() {
        let mut score = Score::new(100);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::Great);
        score.update(JudgeRank::Good);
        assert_eq!(score.combo, 3);
        assert_eq!(score.max_combo, 3);
    }

    #[test]
    fn test_combo_breaks() {
        let mut score = Score::new(100);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::Bad);
        assert_eq!(score.combo, 0);
        assert_eq!(score.max_combo, 2);
    }

    #[test]
    fn test_max_combo_tracking() {
        let mut score = Score::new(100);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::Bad);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::PerfectGreat);
        score.update(JudgeRank::PerfectGreat);
        assert_eq!(score.combo, 3);
        assert_eq!(score.max_combo, 3);
    }

    #[test]
    fn test_clear_rate() {
        let mut score = Score::new(10);
        for _ in 0..10 {
            score.update(JudgeRank::PerfectGreat);
        }
        assert!((score.clear_rate() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_bp_count() {
        let mut score = Score::new(100);
        score.update(JudgeRank::Bad);
        score.update(JudgeRank::Poor);
        score.update(JudgeRank::Miss);
        assert_eq!(score.bp(), 3);
    }
}
