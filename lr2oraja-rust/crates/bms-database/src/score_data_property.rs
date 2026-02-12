//! Real-time score computation and ranking.
//!
//! Port of Java `ScoreDataProperty.java`.
//! Computes nowpoint, rate, rank, and ghost comparison data
//! during gameplay from incremental score updates.

use bms_model::PlayMode;
use bms_rule::ScoreData;

/// Number of rank thresholds (27 divisions from 0/27 to 26/27).
const RANK_COUNT: usize = 27;

/// Real-time score property calculator.
///
/// Updated incrementally during play to track current score,
/// rates, ranks, and ghost comparisons.
#[derive(Debug, Clone)]
pub struct ScoreDataProperty {
    // Current score reference
    score: Option<ScoreData>,
    rival: Option<ScoreData>,

    // Score values
    nowpoint: i32,
    nowscore: i32,
    bestscore: i32,
    bestscorerate: f32,
    nowbestscore: i32,
    nowbestscorerate: f32,

    // Rate values
    nowrate: f32,
    nowrate_int: i32,
    nowrate_after_dot: i32,
    rate: f32,
    rate_int: i32,
    rate_after_dot: i32,
    bestrate_int: i32,
    bestrate_after_dot: i32,

    // Rival values
    rivalscore: i32,
    rivalscorerate: f32,
    nowrivalscore: i32,
    nowrivalscorerate: f32,
    rivalrate_int: i32,
    rivalrate_after_dot: i32,

    // Rank arrays
    rank: [bool; RANK_COUNT],
    nowrank: [bool; RANK_COUNT],
    bestrank: [bool; RANK_COUNT],
    nextrank: i32,

    // Ghost tracking
    previous_notes: i32,
    best_ghost: Option<Vec<i32>>,
    rival_ghost: Option<Vec<i32>>,
    use_best_ghost: bool,
    use_rival_ghost: bool,

    totalnotes: i32,
}

impl Default for ScoreDataProperty {
    fn default() -> Self {
        Self {
            score: None,
            rival: None,
            nowpoint: 0,
            nowscore: 0,
            bestscore: 0,
            bestscorerate: 0.0,
            nowbestscore: 0,
            nowbestscorerate: 0.0,
            nowrate: 1.0,
            nowrate_int: 0,
            nowrate_after_dot: 0,
            rate: 1.0,
            rate_int: 0,
            rate_after_dot: 0,
            bestrate_int: 0,
            bestrate_after_dot: 0,
            rivalscore: 0,
            rivalscorerate: 0.0,
            nowrivalscore: 0,
            nowrivalscorerate: 0.0,
            rivalrate_int: 0,
            rivalrate_after_dot: 0,
            rank: [false; RANK_COUNT],
            nowrank: [false; RANK_COUNT],
            bestrank: [false; RANK_COUNT],
            nextrank: i32::MIN,
            previous_notes: 0,
            best_ghost: None,
            rival_ghost: None,
            use_best_ghost: false,
            use_rival_ghost: false,
            totalnotes: 0,
        }
    }
}

impl ScoreDataProperty {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update with score data using the score's own note count.
    pub fn update_score(&mut self, score: &ScoreData) {
        self.update(score, score.notes);
    }

    /// Update with both own and rival score data.
    pub fn update_with_rival(&mut self, score: &ScoreData, rival: &ScoreData) {
        self.update_score(score);
        self.rival = Some(rival.clone());

        let exscore = rival.exscore();
        let totalnotes = rival.notes;

        self.rivalscore = exscore;
        self.rivalscorerate = if totalnotes == 0 {
            1.0
        } else {
            exscore as f32 / (totalnotes * 2) as f32
        };
        self.rivalrate_int = (self.rivalscorerate * 100.0) as i32;
        self.rivalrate_after_dot = ((self.rivalscorerate * 10000.0) as i32) % 100;
    }

    /// Update with score data and a specific note count (for in-play updates).
    ///
    /// `notes` is the number of notes processed so far.
    pub fn update(&mut self, score: &ScoreData, notes: i32) {
        self.score = Some(score.clone());
        let exscore = score.exscore();
        let totalnotes = score.notes;

        // Calculate nowpoint based on play mode
        if totalnotes > 0 {
            let play_mode = PlayMode::from_mode_id(score.mode);
            let pgreat = score.judge_count(0) as i64;
            let great = score.judge_count(1) as i64;
            let good = score.judge_count(2) as i64;
            let combo = score.maxcombo as i64;
            let tn = totalnotes as i64;

            self.nowpoint = match play_mode {
                Some(PlayMode::Beat5K | PlayMode::Beat10K) => {
                    ((100000 * pgreat + 100000 * great + 50000 * good) / tn) as i32
                }
                Some(PlayMode::Beat7K | PlayMode::Beat14K) => {
                    ((150000 * pgreat + 100000 * great + 20000 * good) / tn) as i32
                        + (50000 * combo / tn) as i32
                }
                Some(PlayMode::PopN5K | PlayMode::PopN9K) => {
                    ((100000 * pgreat + 70000 * great + 40000 * good) / tn) as i32
                }
                _ => ((1000000 * pgreat + 700000 * great + 400000 * good) / tn) as i32,
            };
        } else {
            self.nowpoint = 0;
        }

        self.nowscore = exscore;

        // Rate calculations
        self.rate = if totalnotes == 0 {
            1.0
        } else {
            exscore as f32 / (totalnotes * 2) as f32
        };
        self.rate_int = (self.rate * 100.0) as i32;
        self.rate_after_dot = ((self.rate * 10000.0) as i32) % 100;

        self.nowrate = if notes == 0 {
            1.0
        } else {
            exscore as f32 / (notes * 2) as f32
        };
        self.nowrate_int = (self.nowrate * 100.0) as i32;
        self.nowrate_after_dot = ((self.nowrate * 10000.0) as i32) % 100;

        // Rank calculation
        self.nextrank = i32::MIN;
        for i in 0..RANK_COUNT {
            self.rank[i] = totalnotes != 0 && self.rate >= i as f32 / RANK_COUNT as f32;
            if i % 3 == 0 && !self.rank[i] && self.nextrank == i32::MIN {
                self.nextrank = (((i as f64) * (notes * 2) as f64 / RANK_COUNT as f64).ceil()
                    - self.rate as f64 * (notes * 2) as f64) as i32;
            }
        }
        if self.nextrank == i32::MIN {
            self.nextrank = (notes * 2) - exscore;
        }

        for i in 0..RANK_COUNT {
            self.nowrank[i] = totalnotes != 0 && self.nowrate >= i as f32 / RANK_COUNT as f32;
        }

        // Ghost-based best score tracking
        if self.use_best_ghost {
            if let Some(ghost) = &self.best_ghost {
                for i in self.previous_notes..notes {
                    if (i as usize) < ghost.len() {
                        self.nowbestscore += ghost_exscore(ghost[i as usize]);
                    }
                }
            }
            self.nowbestscorerate = if totalnotes == 0 {
                0.0
            } else {
                self.nowbestscore as f32 / (totalnotes * 2) as f32
            };
        } else {
            self.nowbestscore = if totalnotes == 0 {
                0
            } else {
                self.bestscore * notes / totalnotes
            };
            self.nowbestscorerate = if totalnotes == 0 {
                0.0
            } else {
                self.bestscore as f32 * notes as f32 / (totalnotes as f32 * totalnotes as f32 * 2.0)
            };
        }

        // Ghost-based rival score tracking
        if self.use_rival_ghost {
            if let Some(ghost) = &self.rival_ghost {
                for i in self.previous_notes..notes {
                    if (i as usize) < ghost.len() {
                        self.nowrivalscore += ghost_exscore(ghost[i as usize]);
                    }
                }
            }
            self.nowrivalscorerate = if totalnotes == 0 {
                0.0
            } else {
                self.nowrivalscore as f32 / (totalnotes * 2) as f32
            };
        } else {
            self.nowrivalscore = if totalnotes == 0 {
                0
            } else {
                self.rivalscore * notes / totalnotes
            };
            self.nowrivalscorerate = if totalnotes == 0 {
                0.0
            } else {
                self.rivalscore as f32 * notes as f32
                    / (totalnotes as f32 * totalnotes as f32 * 2.0)
            };
        }

        self.previous_notes = notes;
    }

    /// Update the target rival score (without ghost).
    pub fn update_target_score(&mut self, rivalscore: i32) {
        self.rivalscore = rivalscore;
        self.rivalscorerate = rivalscore as f32 / (self.totalnotes * 2) as f32;
        self.rivalrate_int = (self.rivalscorerate * 100.0) as i32;
        self.rivalrate_after_dot = ((self.rivalscorerate * 10000.0) as i32) % 100;
    }

    /// Set target scores and optional ghost data for best/rival tracking.
    pub fn set_target_score(
        &mut self,
        bestscore: i32,
        best_ghost: Option<Vec<i32>>,
        rivalscore: i32,
        rival_ghost: Option<Vec<i32>>,
        totalnotes: i32,
    ) {
        self.bestscore = bestscore;
        self.rivalscore = rivalscore;
        self.totalnotes = totalnotes;

        self.bestscorerate = bestscore as f32 / (totalnotes * 2) as f32;
        self.bestrate_int = (self.bestscorerate * 100.0) as i32;
        self.bestrate_after_dot = ((self.bestscorerate * 10000.0) as i32) % 100;

        self.rivalscorerate = rivalscore as f32 / (totalnotes * 2) as f32;
        self.rivalrate_int = (self.rivalscorerate * 100.0) as i32;
        self.rivalrate_after_dot = ((self.rivalscorerate * 10000.0) as i32) % 100;

        for i in 0..RANK_COUNT {
            self.bestrank[i] = self.bestscorerate >= i as f32 / RANK_COUNT as f32;
        }

        // Ghost usage requires matching note counts
        self.use_best_ghost = best_ghost
            .as_ref()
            .is_some_and(|g| g.len() == totalnotes as usize);
        self.use_rival_ghost = rival_ghost
            .as_ref()
            .is_some_and(|g| g.len() == totalnotes as usize);

        self.best_ghost = best_ghost;
        self.rival_ghost = rival_ghost;
    }

    // ---- Accessors ----

    pub fn now_score(&self) -> i32 {
        self.nowpoint
    }

    pub fn now_exscore(&self) -> i32 {
        self.nowscore
    }

    pub fn now_best_score(&self) -> i32 {
        self.nowbestscore
    }

    pub fn now_rival_score(&self) -> i32 {
        self.nowrivalscore
    }

    pub fn qualify_rank(&self, index: usize) -> bool {
        self.rank.get(index).copied().unwrap_or(false)
    }

    pub fn qualify_now_rank(&self, index: usize) -> bool {
        self.nowrank.get(index).copied().unwrap_or(false)
    }

    pub fn qualify_best_rank(&self, index: usize) -> bool {
        self.bestrank.get(index).copied().unwrap_or(false)
    }

    pub fn now_rate(&self) -> f32 {
        self.nowrate
    }

    pub fn now_rate_int(&self) -> i32 {
        self.nowrate_int
    }

    pub fn now_rate_after_dot(&self) -> i32 {
        self.nowrate_after_dot
    }

    pub fn rival_rate_int(&self) -> i32 {
        self.rivalrate_int
    }

    pub fn rival_rate_after_dot(&self) -> i32 {
        self.rivalrate_after_dot
    }

    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn next_rank(&self) -> i32 {
        self.nextrank
    }

    pub fn rate_int(&self) -> i32 {
        self.rate_int
    }

    pub fn rate_after_dot(&self) -> i32 {
        self.rate_after_dot
    }

    pub fn best_score(&self) -> i32 {
        self.bestscore
    }

    pub fn best_score_rate(&self) -> f32 {
        self.bestscorerate
    }

    pub fn best_rate_int(&self) -> i32 {
        self.bestrate_int
    }

    pub fn best_rate_after_dot(&self) -> i32 {
        self.bestrate_after_dot
    }

    pub fn now_best_score_rate(&self) -> f32 {
        self.nowbestscorerate
    }

    pub fn rival_score(&self) -> i32 {
        self.rivalscore
    }

    pub fn rival_score_rate(&self) -> f32 {
        self.rivalscorerate
    }

    pub fn now_rival_score_rate(&self) -> f32 {
        self.nowrivalscorerate
    }

    pub fn score_data(&self) -> Option<&ScoreData> {
        self.score.as_ref()
    }

    pub fn rival_score_data(&self) -> Option<&ScoreData> {
        self.rival.as_ref()
    }
}

/// Convert a ghost judge value to EX score contribution.
///
/// 0 = PGREAT (2 points), 1 = GREAT (1 point), else 0 points.
fn ghost_exscore(judge: i32) -> i32 {
    match judge {
        0 => 2,
        1 => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_rule::ClearType;

    fn make_score(mode: i32, epg: i32, lpg: i32, egr: i32, lgr: i32, notes: i32) -> ScoreData {
        ScoreData {
            mode,
            epg,
            lpg,
            egr,
            lgr,
            notes,
            ..Default::default()
        }
    }

    #[test]
    fn update_basic_7k() {
        let mut prop = ScoreDataProperty::new();
        let score = make_score(7, 100, 50, 30, 20, 200);

        prop.update(&score, 200);

        // EX score = (100+50)*2 + 30+20 = 350
        assert_eq!(prop.now_exscore(), 350);
        // Rate = 350 / 400 = 0.875
        assert!((prop.rate() - 0.875).abs() < 0.001);
        assert_eq!(prop.rate_int(), 87);
    }

    #[test]
    fn update_basic_5k() {
        let mut prop = ScoreDataProperty::new();
        // 5K mode: (100000*PG + 100000*GR + 50000*GD) / totalnotes
        let mut score = make_score(5, 80, 20, 50, 10, 200);
        score.egd = 30;
        score.lgd = 10;

        prop.update(&score, 200);

        let expected_point = ((100000i64 * 100 + 100000 * 60 + 50000 * 40) / 200) as i32;
        assert_eq!(prop.now_score(), expected_point);
    }

    #[test]
    fn update_popn() {
        let mut prop = ScoreDataProperty::new();
        // PopN9K: (100000*PG + 70000*GR + 40000*GD) / totalnotes
        let mut score = make_score(9, 50, 50, 30, 20, 200);
        score.egd = 10;
        score.lgd = 10;

        prop.update(&score, 200);

        let expected_point = ((100000i64 * 100 + 70000 * 50 + 40000 * 20) / 200) as i32;
        assert_eq!(prop.now_score(), expected_point);
    }

    #[test]
    fn rate_100_percent() {
        let mut prop = ScoreDataProperty::new();
        // All PGREAT
        let score = make_score(7, 100, 100, 0, 0, 200);

        prop.update(&score, 200);

        assert!((prop.rate() - 1.0).abs() < f32::EPSILON);
        assert!(prop.qualify_rank(26)); // Should qualify for highest rank
    }

    #[test]
    fn rate_0_notes() {
        let mut prop = ScoreDataProperty::new();
        let score = make_score(7, 0, 0, 0, 0, 0);

        prop.update(&score, 0);

        assert!((prop.rate() - 1.0).abs() < f32::EPSILON);
        assert!(!prop.qualify_rank(0)); // No notes means no rank qualification
    }

    #[test]
    fn rank_thresholds() {
        let mut prop = ScoreDataProperty::new();
        // 50% rate: 200 exscore / 400 max
        let score = make_score(7, 50, 50, 0, 0, 200);

        prop.update(&score, 200);

        // Rate = 200/400 = 0.5
        // rank[i] = rate >= i/27
        // 0.5 >= 0/27 = true
        // 0.5 >= 13/27 ≈ 0.481 = true
        // 0.5 >= 14/27 ≈ 0.519 = false
        assert!(prop.qualify_rank(0));
        assert!(prop.qualify_rank(13));
        assert!(!prop.qualify_rank(14));
    }

    #[test]
    fn set_target_score_basic() {
        let mut prop = ScoreDataProperty::new();
        prop.set_target_score(300, None, 200, None, 500);

        assert_eq!(prop.best_score(), 300);
        assert_eq!(prop.rival_score(), 200);
        assert!(!prop.use_best_ghost);
        assert!(!prop.use_rival_ghost);
    }

    #[test]
    fn set_target_score_with_ghost() {
        let mut prop = ScoreDataProperty::new();
        let ghost = vec![0, 0, 1, 2, 0]; // PG, PG, GR, miss, PG
        prop.set_target_score(300, Some(ghost), 200, None, 5);

        assert!(prop.use_best_ghost);
        assert!(!prop.use_rival_ghost);
    }

    #[test]
    fn ghost_mismatched_length_not_used() {
        let mut prop = ScoreDataProperty::new();
        let ghost = vec![0, 0, 1]; // 3 entries for 5 notes
        prop.set_target_score(300, Some(ghost), 200, None, 5);

        assert!(!prop.use_best_ghost);
    }

    #[test]
    fn ghost_exscore_values() {
        assert_eq!(ghost_exscore(0), 2); // PG
        assert_eq!(ghost_exscore(1), 1); // GR
        assert_eq!(ghost_exscore(2), 0); // GD or worse
        assert_eq!(ghost_exscore(3), 0);
        assert_eq!(ghost_exscore(-1), 0);
    }

    #[test]
    fn incremental_update() {
        let mut prop = ScoreDataProperty::new();
        prop.set_target_score(100, Some(vec![0, 1, 0, 0, 1]), 50, None, 5);

        // First update: 2 notes processed
        let score = make_score(7, 2, 0, 0, 0, 5);
        prop.update(&score, 2);
        // Ghost: note 0 (PG=2) + note 1 (GR=1) = 3
        assert_eq!(prop.now_best_score(), 3);

        // Second update: 4 notes processed
        let mut score2 = make_score(7, 3, 0, 1, 0, 5);
        score2.maxcombo = 4;
        prop.update(&score2, 4);
        // Ghost: +note 2 (PG=2) + note 3 (PG=2) = 3+4 = 7
        assert_eq!(prop.now_best_score(), 7);
    }

    #[test]
    fn update_with_rival() {
        let mut prop = ScoreDataProperty::new();
        let score = make_score(7, 100, 100, 0, 0, 200);
        let rival = ScoreData {
            mode: 7,
            epg: 50,
            lpg: 50,
            egr: 20,
            lgr: 10,
            notes: 200,
            clear: ClearType::Normal,
            ..Default::default()
        };

        prop.update_with_rival(&score, &rival);

        // Rival exscore = (50+50)*2 + 20+10 = 230
        assert_eq!(prop.rival_score(), 230);
        assert!(prop.rival_score_data().is_some());
    }

    #[test]
    fn update_target_score() {
        let mut prop = ScoreDataProperty::new();
        prop.set_target_score(300, None, 200, None, 500);

        prop.update_target_score(350);
        assert_eq!(prop.rival_score(), 350);
    }

    #[test]
    fn nowrate_partial_notes() {
        let mut prop = ScoreDataProperty::new();
        // 100 total notes, only 50 processed so far
        let score = make_score(7, 25, 25, 0, 0, 100);
        prop.update(&score, 50);

        // now_rate = exscore / (notes_processed * 2) = 100 / 100 = 1.0
        assert!((prop.now_rate() - 1.0).abs() < f32::EPSILON);
        // rate = exscore / (totalnotes * 2) = 100 / 200 = 0.5
        assert!((prop.rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn best_rank_tracking() {
        let mut prop = ScoreDataProperty::new();
        // Best score = 800 out of 1000 max (500 notes * 2) = 80%
        prop.set_target_score(800, None, 0, None, 500);

        // 80% = 0.8 >= 21/27 ≈ 0.778 = true
        assert!(prop.qualify_best_rank(21));
        // 80% = 0.8 >= 22/27 ≈ 0.815 = false
        assert!(!prop.qualify_best_rank(22));
    }
}
