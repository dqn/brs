use serde::{Deserialize, Serialize};

/// Score rank (beatoraja 27-division rank system).
/// Rank boundaries: rate = exscore / (total_notes * 2).
/// F: rate < 2/9, E: >= 2/9, D: >= 4/9, C: >= 6/9, B: >= 8/9,
/// A: >= 10/9... but normalized to 27 divisions:
/// F=0/27, E=3/27, D=6/27, C=9/27, B=12/27, A=15/27, AA=18/27, AAA=21/27, Max=24/27.
///
/// Beatoraja uses 27 boolean rank array where rank[i] = (rate >= i/27).
/// The discrete rank levels correspond to every 3rd index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ScoreRank {
    F,
    E,
    D,
    C,
    B,
    A,
    AA,
    AAA,
    Max,
}

impl ScoreRank {
    /// Determine rank from EX score and total notes.
    /// `exscore`: EX score = PG * 2 + GR.
    /// `total_notes`: total playable notes in the chart.
    pub fn from_score(exscore: u32, total_notes: u32) -> Self {
        if total_notes == 0 {
            return Self::F;
        }
        // rate = exscore / (total_notes * 2)
        // Rank boundaries: i * (total_notes * 2) / 27 for i = 0,3,6,...,24
        // We use integer math: exscore * 27 >= i * total_notes * 2
        let scaled = exscore as u64 * 27;
        let max_score = total_notes as u64 * 2;

        if scaled >= 24 * max_score {
            Self::Max
        } else if scaled >= 21 * max_score {
            Self::AAA
        } else if scaled >= 18 * max_score {
            Self::AA
        } else if scaled >= 15 * max_score {
            Self::A
        } else if scaled >= 12 * max_score {
            Self::B
        } else if scaled >= 9 * max_score {
            Self::C
        } else if scaled >= 6 * max_score {
            Self::D
        } else if scaled >= 3 * max_score {
            Self::E
        } else {
            Self::F
        }
    }

    /// Get the rank index in the 27-division system (0-8 = F through Max).
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Score data accumulated during play.
/// Corresponds to beatoraja's ScoreData fields relevant to play.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreData {
    /// Judge counts: [PG, GR, GD, BD, PR, MS] for early (fast) hits.
    pub early_counts: [u32; 6],
    /// Judge counts: [PG, GR, GD, BD, PR, MS] for late (slow) hits.
    pub late_counts: [u32; 6],
    /// Maximum combo achieved.
    pub max_combo: u32,
    /// Total notes in the chart.
    pub total_notes: u32,
    /// Number of notes that have been judged.
    pub pass_notes: u32,
    /// Minimum bad+poor count (miss count).
    pub min_bp: u32,
}

impl ScoreData {
    /// Create a new ScoreData with zero counts.
    pub fn new(total_notes: u32) -> Self {
        Self {
            early_counts: [0; 6],
            late_counts: [0; 6],
            max_combo: 0,
            total_notes,
            pass_notes: 0,
            min_bp: 0,
        }
    }

    /// Get total judge count for a level (early + late).
    pub fn judge_count(&self, judge: usize) -> u32 {
        if judge < 6 {
            self.early_counts[judge] + self.late_counts[judge]
        } else {
            0
        }
    }

    /// EX score = PG * 2 + GR.
    pub fn exscore(&self) -> u32 {
        self.judge_count(0) * 2 + self.judge_count(1)
    }

    /// Score rate = exscore / (total_notes * 2).
    pub fn rate(&self) -> f32 {
        if self.total_notes == 0 {
            return 1.0;
        }
        self.exscore() as f32 / (self.total_notes as f32 * 2.0)
    }

    /// Current score rate (based on pass_notes instead of total_notes).
    pub fn current_rate(&self) -> f32 {
        if self.pass_notes == 0 {
            return 1.0;
        }
        self.exscore() as f32 / (self.pass_notes as f32 * 2.0)
    }

    /// Get the score rank.
    pub fn rank(&self) -> ScoreRank {
        ScoreRank::from_score(self.exscore(), self.total_notes)
    }

    /// Whether the score represents a perfect (all PG).
    pub fn is_perfect(&self) -> bool {
        self.judge_count(0) == self.total_notes
            && self.judge_count(1) == 0
            && self.judge_count(2) == 0
            && self.judge_count(3) == 0
            && self.judge_count(4) == 0
            && self.judge_count(5) == 0
    }

    /// Whether the score represents a full combo (no BD/PR/MS).
    pub fn is_full_combo(&self) -> bool {
        self.judge_count(3) == 0 && self.judge_count(4) == 0 && self.judge_count(5) == 0
    }

    /// Whether the score represents max (all PG, full combo, max EX score).
    pub fn is_max(&self) -> bool {
        self.is_perfect() && self.max_combo == self.total_notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // ScoreRank tests
    // =========================================================================

    #[test]
    fn rank_f() {
        // rate < 3/27 = 1/9 ≈ 0.111
        // exscore=1, total=10 -> rate = 1/20 = 0.05
        assert_eq!(ScoreRank::from_score(1, 10), ScoreRank::F);
    }

    #[test]
    fn rank_e() {
        // rate >= 3/27 = 1/9 ≈ 0.111, < 6/27 = 2/9 ≈ 0.222
        // exscore=3, total=10 -> rate = 3/20 = 0.15
        assert_eq!(ScoreRank::from_score(3, 10), ScoreRank::E);
    }

    #[test]
    fn rank_d() {
        // rate >= 6/27 = 2/9 ≈ 0.222, < 9/27 = 1/3 ≈ 0.333
        // exscore=5, total=10 -> rate = 5/20 = 0.25
        assert_eq!(ScoreRank::from_score(5, 10), ScoreRank::D);
    }

    #[test]
    fn rank_c() {
        // rate >= 9/27 = 1/3, < 12/27 = 4/9
        // exscore=7, total=10 -> rate = 7/20 = 0.35
        assert_eq!(ScoreRank::from_score(7, 10), ScoreRank::C);
    }

    #[test]
    fn rank_b() {
        // rate >= 12/27, < 15/27
        // exscore=10, total=10 -> rate = 10/20 = 0.5
        assert_eq!(ScoreRank::from_score(10, 10), ScoreRank::B);
    }

    #[test]
    fn rank_a() {
        // rate >= 15/27, < 18/27
        // exscore=12, total=10 -> rate = 12/20 = 0.6
        assert_eq!(ScoreRank::from_score(12, 10), ScoreRank::A);
    }

    #[test]
    fn rank_aa() {
        // rate >= 18/27, < 21/27
        // exscore=14, total=10 -> rate = 14/20 = 0.7
        assert_eq!(ScoreRank::from_score(14, 10), ScoreRank::AA);
    }

    #[test]
    fn rank_aaa() {
        // rate >= 21/27, < 24/27
        // exscore=16, total=10 -> rate = 16/20 = 0.8
        assert_eq!(ScoreRank::from_score(16, 10), ScoreRank::AAA);
    }

    #[test]
    fn rank_max() {
        // rate >= 24/27 = 8/9 ≈ 0.889
        // exscore=18, total=10 -> rate = 18/20 = 0.9
        assert_eq!(ScoreRank::from_score(18, 10), ScoreRank::Max);
        // Perfect score
        assert_eq!(ScoreRank::from_score(20, 10), ScoreRank::Max);
    }

    #[test]
    fn rank_zero_notes() {
        assert_eq!(ScoreRank::from_score(0, 0), ScoreRank::F);
    }

    #[test]
    fn rank_boundary_exact() {
        // Exact boundary: rate = 3/27 = 1/9
        // exscore * 27 == 3 * total_notes * 2
        // For total=27: exscore * 27 >= 3 * 54 = 162 -> exscore >= 6
        // rate = 6/54 = 1/9
        assert_eq!(ScoreRank::from_score(6, 27), ScoreRank::E);
        assert_eq!(ScoreRank::from_score(5, 27), ScoreRank::F);
    }

    // =========================================================================
    // ScoreData tests
    // =========================================================================

    #[test]
    fn score_data_new() {
        let sd = ScoreData::new(100);
        assert_eq!(sd.total_notes, 100);
        assert_eq!(sd.exscore(), 0);
        assert_eq!(sd.judge_count(0), 0);
    }

    #[test]
    fn exscore_calculation() {
        let mut sd = ScoreData::new(100);
        // 50 PG early + 20 PG late = 70 PG
        sd.early_counts[0] = 50;
        sd.late_counts[0] = 20;
        // 10 GR early + 5 GR late = 15 GR
        sd.early_counts[1] = 10;
        sd.late_counts[1] = 5;
        // EXSCORE = 70 * 2 + 15 = 155
        assert_eq!(sd.exscore(), 155);
    }

    #[test]
    fn rate_calculation() {
        let mut sd = ScoreData::new(100);
        sd.early_counts[0] = 100; // 100 PG
        // exscore = 200, rate = 200 / 200 = 1.0
        assert!((sd.rate() - 1.0).abs() < f32::EPSILON);

        sd.early_counts[0] = 50;
        sd.early_counts[1] = 50;
        // exscore = 100 + 50 = 150, rate = 150/200 = 0.75
        assert!((sd.rate() - 0.75).abs() < 0.001);
    }

    #[test]
    fn rate_zero_notes() {
        let sd = ScoreData::new(0);
        assert!((sd.rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn current_rate() {
        let mut sd = ScoreData::new(100);
        sd.early_counts[0] = 10; // 10 PG
        sd.pass_notes = 10;
        // exscore = 20, current_rate = 20 / 20 = 1.0
        assert!((sd.current_rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn is_perfect() {
        let mut sd = ScoreData::new(10);
        sd.early_counts[0] = 10;
        assert!(sd.is_perfect());

        sd.early_counts[1] = 1;
        assert!(!sd.is_perfect());
    }

    #[test]
    fn is_full_combo() {
        let mut sd = ScoreData::new(10);
        sd.early_counts[0] = 5;
        sd.early_counts[1] = 3;
        sd.early_counts[2] = 2;
        assert!(sd.is_full_combo());

        sd.early_counts[3] = 1; // BD
        assert!(!sd.is_full_combo());
    }

    #[test]
    fn is_max() {
        let mut sd = ScoreData::new(10);
        sd.early_counts[0] = 10;
        sd.max_combo = 10;
        assert!(sd.is_max());

        sd.max_combo = 9;
        assert!(!sd.is_max());
    }

    #[test]
    fn rank_integration() {
        let mut sd = ScoreData::new(100);
        // All PG: rate = 1.0 -> Max
        sd.early_counts[0] = 100;
        assert_eq!(sd.rank(), ScoreRank::Max);

        // 75 PG + 25 GR: exscore = 150+25 = 175, rate = 175/200 = 0.875
        // 0.875 >= 24/27 ≈ 0.889? No. 0.875 * 27 = 23.625 < 24
        // So AAA
        sd.early_counts[0] = 75;
        sd.early_counts[1] = 25;
        assert_eq!(sd.rank(), ScoreRank::AAA);
    }

    #[test]
    fn judge_count_out_of_range() {
        let sd = ScoreData::new(10);
        assert_eq!(sd.judge_count(6), 0);
        assert_eq!(sd.judge_count(100), 0);
    }
}
