use serde::{Deserialize, Serialize};

use crate::clear_type::ClearType;
use crate::{JUDGE_BD, JUDGE_GD, JUDGE_GR, JUDGE_MS, JUDGE_PG, JUDGE_PR};

/// Score data for a single play session.
///
/// Stores all judgment counts split into early (e-prefix) and late (l-prefix),
/// along with metadata about the play session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreData {
    /// SHA-256 hash of the chart
    pub sha256: String,
    /// Player name (empty for self)
    pub player: String,
    /// Play mode identifier
    pub mode: i32,
    /// Clear type achieved
    pub clear: ClearType,
    /// Last score date (unix timestamp, seconds)
    pub date: i64,
    /// Total play count
    pub playcount: i32,
    /// Total clear count
    pub clearcount: i32,

    // Early/Late judgment counts (6 pairs)
    /// Early PGREAT count
    pub epg: i32,
    /// Late PGREAT count
    pub lpg: i32,
    /// Early GREAT count
    pub egr: i32,
    /// Late GREAT count
    pub lgr: i32,
    /// Early GOOD count
    pub egd: i32,
    /// Late GOOD count
    pub lgd: i32,
    /// Early BAD count
    pub ebd: i32,
    /// Late BAD count
    pub lbd: i32,
    /// Early POOR count
    pub epr: i32,
    /// Late POOR count
    pub lpr: i32,
    /// Early MISS count
    pub ems: i32,
    /// Late MISS count
    pub lms: i32,

    /// Maximum combo achieved
    pub maxcombo: i32,
    /// Total notes in the chart
    pub notes: i32,
    /// Notes passed (processed) so far
    pub passnotes: i32,
    /// Minimum bad/poor count (best BP)
    pub minbp: i32,
    /// Average judge timing (microseconds)
    pub avgjudge: i64,

    /// Trophy data (option-specific clear history)
    pub trophy: String,
    /// Ghost data (base64-encoded gzip of per-note judgments)
    pub ghost: String,
    /// Random option used
    pub random: i32,
    /// Option flags
    pub option: i32,
    /// Random seed
    pub seed: i64,
    /// Assist option flags
    pub assist: i32,
    /// Gauge type used
    pub gauge: i32,
    /// Play state
    pub state: i32,
    /// Score hash for integrity
    pub scorehash: String,
}

impl Default for ScoreData {
    fn default() -> Self {
        Self {
            sha256: String::new(),
            player: "unknown".to_string(),
            mode: 0,
            clear: ClearType::default(),
            date: 0,
            playcount: 0,
            clearcount: 0,
            epg: 0,
            lpg: 0,
            egr: 0,
            lgr: 0,
            egd: 0,
            lgd: 0,
            ebd: 0,
            lbd: 0,
            epr: 0,
            lpr: 0,
            ems: 0,
            lms: 0,
            maxcombo: 0,
            notes: 0,
            passnotes: 0,
            minbp: i32::MAX,
            avgjudge: i64::MAX,
            trophy: String::new(),
            ghost: String::new(),
            random: 0,
            option: 0,
            seed: -1,
            assist: 0,
            gauge: 0,
            state: 0,
            scorehash: String::new(),
        }
    }
}

impl ScoreData {
    /// Calculate the EX score: PGREAT * 2 + GREAT
    pub fn exscore(&self) -> i32 {
        (self.epg + self.lpg) * 2 + self.egr + self.lgr
    }

    /// Get the total count for a specific judge (early + late).
    ///
    /// `judge` must be one of the JUDGE_* constants (0-5).
    /// Returns 0 for invalid judge indices.
    pub fn judge_count(&self, judge: usize) -> i32 {
        self.judge_count_early(judge) + self.judge_count_late(judge)
    }

    /// Get the early (fast) count for a specific judge.
    ///
    /// Returns 0 for invalid judge indices.
    pub fn judge_count_early(&self, judge: usize) -> i32 {
        match judge {
            JUDGE_PG => self.epg,
            JUDGE_GR => self.egr,
            JUDGE_GD => self.egd,
            JUDGE_BD => self.ebd,
            JUDGE_PR => self.epr,
            JUDGE_MS => self.ems,
            _ => 0,
        }
    }

    /// Get the late (slow) count for a specific judge.
    ///
    /// Returns 0 for invalid judge indices.
    pub fn judge_count_late(&self, judge: usize) -> i32 {
        match judge {
            JUDGE_PG => self.lpg,
            JUDGE_GR => self.lgr,
            JUDGE_GD => self.lgd,
            JUDGE_BD => self.lbd,
            JUDGE_PR => self.lpr,
            JUDGE_MS => self.lms,
            _ => 0,
        }
    }

    /// Add count to a specific judge (early or late).
    ///
    /// `judge` must be one of the JUDGE_* constants (0-5).
    /// No-op for invalid judge indices.
    pub fn add_judge_count(&mut self, judge: usize, is_early: bool, count: i32) {
        let field = match (judge, is_early) {
            (JUDGE_PG, true) => &mut self.epg,
            (JUDGE_PG, false) => &mut self.lpg,
            (JUDGE_GR, true) => &mut self.egr,
            (JUDGE_GR, false) => &mut self.lgr,
            (JUDGE_GD, true) => &mut self.egd,
            (JUDGE_GD, false) => &mut self.lgd,
            (JUDGE_BD, true) => &mut self.ebd,
            (JUDGE_BD, false) => &mut self.lbd,
            (JUDGE_PR, true) => &mut self.epr,
            (JUDGE_PR, false) => &mut self.lpr,
            (JUDGE_MS, true) => &mut self.ems,
            (JUDGE_MS, false) => &mut self.lms,
            _ => return,
        };
        *field += count;
    }

    /// Get the total count across all judgments.
    pub fn total_judge_count(&self) -> i32 {
        self.epg
            + self.lpg
            + self.egr
            + self.lgr
            + self.egd
            + self.lgd
            + self.ebd
            + self.lbd
            + self.epr
            + self.lpr
            + self.ems
            + self.lms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let sd = ScoreData::default();
        assert_eq!(sd.player, "unknown");
        assert_eq!(sd.clear, ClearType::NoPlay);
        assert_eq!(sd.minbp, i32::MAX);
        assert_eq!(sd.avgjudge, i64::MAX);
        assert_eq!(sd.seed, -1);
    }

    #[test]
    fn exscore_calculation() {
        let mut sd = ScoreData::default();
        sd.epg = 100;
        sd.lpg = 50;
        sd.egr = 30;
        sd.lgr = 20;
        // (100 + 50) * 2 + 30 + 20 = 300 + 50 = 350
        assert_eq!(sd.exscore(), 350);
    }

    #[test]
    fn exscore_zero_when_empty() {
        let sd = ScoreData::default();
        assert_eq!(sd.exscore(), 0);
    }

    #[test]
    fn exscore_pgreat_only() {
        let mut sd = ScoreData::default();
        sd.epg = 10;
        sd.lpg = 5;
        // (10 + 5) * 2 = 30
        assert_eq!(sd.exscore(), 30);
    }

    #[test]
    fn judge_count_all_indices() {
        let mut sd = ScoreData::default();
        sd.epg = 10;
        sd.lpg = 5;
        sd.egr = 20;
        sd.lgr = 15;
        sd.egd = 3;
        sd.lgd = 2;
        sd.ebd = 1;
        sd.lbd = 1;
        sd.epr = 4;
        sd.lpr = 3;
        sd.ems = 2;
        sd.lms = 1;

        assert_eq!(sd.judge_count(JUDGE_PG), 15);
        assert_eq!(sd.judge_count(JUDGE_GR), 35);
        assert_eq!(sd.judge_count(JUDGE_GD), 5);
        assert_eq!(sd.judge_count(JUDGE_BD), 2);
        assert_eq!(sd.judge_count(JUDGE_PR), 7);
        assert_eq!(sd.judge_count(JUDGE_MS), 3);
    }

    #[test]
    fn judge_count_early_late_split() {
        let mut sd = ScoreData::default();
        sd.epg = 10;
        sd.lpg = 5;

        assert_eq!(sd.judge_count_early(JUDGE_PG), 10);
        assert_eq!(sd.judge_count_late(JUDGE_PG), 5);
    }

    #[test]
    fn judge_count_invalid_index_returns_zero() {
        let sd = ScoreData::default();
        assert_eq!(sd.judge_count(6), 0);
        assert_eq!(sd.judge_count(100), 0);
        assert_eq!(sd.judge_count_early(99), 0);
        assert_eq!(sd.judge_count_late(99), 0);
    }

    #[test]
    fn add_judge_count_early_and_late() {
        let mut sd = ScoreData::default();

        // Add early PG
        sd.add_judge_count(JUDGE_PG, true, 5);
        assert_eq!(sd.epg, 5);
        assert_eq!(sd.lpg, 0);

        // Add late PG
        sd.add_judge_count(JUDGE_PG, false, 3);
        assert_eq!(sd.epg, 5);
        assert_eq!(sd.lpg, 3);

        // Add more early PG (accumulates)
        sd.add_judge_count(JUDGE_PG, true, 2);
        assert_eq!(sd.epg, 7);
    }

    #[test]
    fn add_judge_count_all_types() {
        let mut sd = ScoreData::default();

        sd.add_judge_count(JUDGE_PG, true, 1);
        sd.add_judge_count(JUDGE_PG, false, 2);
        sd.add_judge_count(JUDGE_GR, true, 3);
        sd.add_judge_count(JUDGE_GR, false, 4);
        sd.add_judge_count(JUDGE_GD, true, 5);
        sd.add_judge_count(JUDGE_GD, false, 6);
        sd.add_judge_count(JUDGE_BD, true, 7);
        sd.add_judge_count(JUDGE_BD, false, 8);
        sd.add_judge_count(JUDGE_PR, true, 9);
        sd.add_judge_count(JUDGE_PR, false, 10);
        sd.add_judge_count(JUDGE_MS, true, 11);
        sd.add_judge_count(JUDGE_MS, false, 12);

        assert_eq!(sd.epg, 1);
        assert_eq!(sd.lpg, 2);
        assert_eq!(sd.egr, 3);
        assert_eq!(sd.lgr, 4);
        assert_eq!(sd.egd, 5);
        assert_eq!(sd.lgd, 6);
        assert_eq!(sd.ebd, 7);
        assert_eq!(sd.lbd, 8);
        assert_eq!(sd.epr, 9);
        assert_eq!(sd.lpr, 10);
        assert_eq!(sd.ems, 11);
        assert_eq!(sd.lms, 12);
    }

    #[test]
    fn add_judge_count_invalid_index_is_noop() {
        let mut sd = ScoreData::default();
        sd.add_judge_count(6, true, 100);
        sd.add_judge_count(99, false, 100);
        assert_eq!(sd.total_judge_count(), 0);
    }

    #[test]
    fn total_judge_count_sums_all() {
        let mut sd = ScoreData::default();
        sd.epg = 1;
        sd.lpg = 2;
        sd.egr = 3;
        sd.lgr = 4;
        sd.egd = 5;
        sd.lgd = 6;
        sd.ebd = 7;
        sd.lbd = 8;
        sd.epr = 9;
        sd.lpr = 10;
        sd.ems = 11;
        sd.lms = 12;
        assert_eq!(sd.total_judge_count(), 78);
    }

    #[test]
    fn total_judge_count_zero_when_empty() {
        let sd = ScoreData::default();
        assert_eq!(sd.total_judge_count(), 0);
    }

    #[test]
    fn serde_round_trip() {
        let mut sd = ScoreData::default();
        sd.sha256 = "abc123".to_string();
        sd.epg = 100;
        sd.lgr = 50;
        sd.clear = ClearType::Hard;

        let json = serde_json::to_string(&sd).unwrap();
        let deserialized: ScoreData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sha256, "abc123");
        assert_eq!(deserialized.epg, 100);
        assert_eq!(deserialized.lgr, 50);
        assert_eq!(deserialized.clear, ClearType::Hard);
    }
}
