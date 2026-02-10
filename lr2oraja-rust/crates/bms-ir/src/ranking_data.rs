use bms_rule::ScoreData;

use crate::score_data::IRScoreData;

/// IR access state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RankingState {
    #[default]
    None,
    Access,
    Finish,
    Fail,
}

/// Number of clear type lamp slots (NoPlay=0 through Max=10).
const LAMP_COUNT: usize = 11;

/// IR ranking data for a selected chart/course.
///
/// Corresponds to Java `RankingData`.
#[derive(Debug, Clone)]
pub struct RankingData {
    /// Current IR rank
    irrank: i32,
    /// Previous IR rank
    prevrank: i32,
    /// Estimated IR rank based on local score
    localrank: i32,
    /// Total number of IR players
    irtotal: i32,
    /// Clear lamp counts (indexed by ClearType id)
    lamps: [i32; LAMP_COUNT],
    /// All score data
    scores: Option<Vec<IRScoreData>>,
    /// Per-score rankings
    scorerankings: Option<Vec<i32>>,
    /// Access state
    state: RankingState,
    /// Last update time (epoch ms)
    last_update_time: i64,
}

impl Default for RankingData {
    fn default() -> Self {
        Self {
            irrank: 0,
            prevrank: 0,
            localrank: 0,
            irtotal: 0,
            lamps: [0; LAMP_COUNT],
            scores: None,
            scorerankings: None,
            state: RankingState::None,
            last_update_time: 0,
        }
    }
}

impl RankingData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update scores from IR response.
    ///
    /// Corresponds to Java `RankingData.updateScore()`.
    pub fn update_score(&mut self, scores: &[IRScoreData], local_score: Option<&ScoreData>) {
        let first_update = self.scores.is_none();

        // Sort by exscore descending
        let mut sorted: Vec<IRScoreData> = scores.to_vec();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.exscore()));

        // Calculate per-score rankings (same exscore = same rank)
        let mut rankings = vec![0i32; sorted.len()];
        for i in 0..rankings.len() {
            rankings[i] = if i > 0 && sorted[i].exscore() == sorted[i - 1].exscore() {
                rankings[i - 1]
            } else {
                (i + 1) as i32
            };
        }

        if !first_update {
            self.prevrank = self.irrank;
        }

        self.irtotal = sorted.len() as i32;
        self.lamps = [0; LAMP_COUNT];
        self.irrank = 0;
        self.localrank = 0;

        for i in 0..sorted.len() {
            // irrank: first score with empty player name
            if self.irrank == 0 && sorted[i].player.is_empty() {
                self.irrank = rankings[i];
            }
            // localrank: first score with exscore <= local_score.exscore
            if let Some(ls) = local_score
                && self.localrank == 0
                && sorted[i].exscore() <= ls.exscore()
            {
                self.localrank = rankings[i];
            }
            // Count lamps
            let clear_id = sorted[i].clear.id() as usize;
            if clear_id < LAMP_COUNT {
                self.lamps[clear_id] += 1;
            }
        }

        if first_update && self.localrank != 0 {
            self.prevrank = self.irrank.max(self.localrank);
        }

        self.scores = Some(sorted);
        self.scorerankings = Some(rankings);
        self.state = RankingState::Finish;
    }

    /// Set the last update time.
    pub fn set_last_update_time(&mut self, time: i64) {
        self.last_update_time = time;
    }

    /// Current IR rank.
    pub fn rank(&self) -> i32 {
        self.irrank
    }

    /// Previous IR rank.
    pub fn previous_rank(&self) -> i32 {
        self.prevrank
    }

    /// Local score estimated rank.
    pub fn local_rank(&self) -> i32 {
        self.localrank
    }

    /// Total number of players.
    pub fn total_player(&self) -> i32 {
        self.irtotal
    }

    /// Get score at index.
    pub fn score(&self, index: usize) -> Option<&IRScoreData> {
        self.scores.as_ref()?.get(index)
    }

    /// Get ranking for score at index.
    pub fn score_ranking(&self, index: usize) -> Option<i32> {
        self.scorerankings.as_ref()?.get(index).copied()
    }

    /// Get clear count for a clear type.
    pub fn clear_count(&self, clear_type: usize) -> i32 {
        if clear_type < LAMP_COUNT {
            self.lamps[clear_type]
        } else {
            0
        }
    }

    /// Current state.
    pub fn state(&self) -> RankingState {
        self.state
    }

    /// Set state.
    pub fn set_state(&mut self, state: RankingState) {
        self.state = state;
    }

    /// Last update time.
    pub fn last_update_time(&self) -> i64 {
        self.last_update_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_rule::ClearType;

    fn make_ir_score(player: &str, epg: i32, egr: i32, clear: ClearType) -> IRScoreData {
        let mut sd = ScoreData::default();
        sd.player = player.to_string();
        sd.epg = epg;
        sd.egr = egr;
        sd.clear = clear;
        IRScoreData::from(&sd)
    }

    #[test]
    fn default_state() {
        let rd = RankingData::new();
        assert_eq!(rd.state(), RankingState::None);
        assert_eq!(rd.rank(), 0);
        assert_eq!(rd.previous_rank(), 0);
        assert_eq!(rd.local_rank(), 0);
        assert_eq!(rd.total_player(), 0);
    }

    #[test]
    fn update_score_basic() {
        let mut rd = RankingData::new();
        let scores = vec![
            make_ir_score("Player1", 200, 50, ClearType::Hard), // ex: 450
            make_ir_score("Player2", 150, 30, ClearType::Normal), // ex: 330
            make_ir_score("Player3", 100, 20, ClearType::Easy), // ex: 220
        ];
        rd.update_score(&scores, None);

        assert_eq!(rd.state(), RankingState::Finish);
        assert_eq!(rd.total_player(), 3);
        // No empty-player scores, so irrank = 0
        assert_eq!(rd.rank(), 0);
    }

    #[test]
    fn update_score_with_self_rank() {
        let mut rd = RankingData::new();
        let scores = vec![
            make_ir_score("Player1", 200, 50, ClearType::Hard), // ex: 450
            make_ir_score("", 150, 30, ClearType::Normal),      // ex: 330 (self)
            make_ir_score("Player3", 100, 20, ClearType::Easy), // ex: 220
        ];
        rd.update_score(&scores, None);

        assert_eq!(rd.rank(), 2); // 2nd place
        assert_eq!(rd.total_player(), 3);
    }

    #[test]
    fn update_score_tied_rankings() {
        let mut rd = RankingData::new();
        let scores = vec![
            make_ir_score("Player1", 200, 50, ClearType::Hard), // ex: 450
            make_ir_score("Player2", 200, 50, ClearType::Normal), // ex: 450 (tied)
            make_ir_score("Player3", 100, 20, ClearType::Easy), // ex: 220
        ];
        rd.update_score(&scores, None);

        // Both top scores should have rank 1
        assert_eq!(rd.score_ranking(0), Some(1));
        assert_eq!(rd.score_ranking(1), Some(1));
        assert_eq!(rd.score_ranking(2), Some(3));
    }

    #[test]
    fn update_score_localrank() {
        let mut rd = RankingData::new();
        let scores = vec![
            make_ir_score("Player1", 200, 50, ClearType::Hard), // ex: 450
            make_ir_score("Player2", 150, 30, ClearType::Normal), // ex: 330
            make_ir_score("Player3", 100, 20, ClearType::Easy), // ex: 220
        ];
        let mut local = ScoreData::default();
        local.epg = 170;
        local.egr = 40; // ex: 380
        rd.update_score(&scores, Some(&local));

        // Local score (380) is between Player1 (450) and Player2 (330)
        assert_eq!(rd.local_rank(), 2);
    }

    #[test]
    fn update_score_lamps() {
        let mut rd = RankingData::new();
        let scores = vec![
            make_ir_score("P1", 200, 50, ClearType::Hard),
            make_ir_score("P2", 150, 30, ClearType::Hard),
            make_ir_score("P3", 100, 20, ClearType::Easy),
            make_ir_score("P4", 80, 10, ClearType::Failed),
        ];
        rd.update_score(&scores, None);

        assert_eq!(rd.clear_count(ClearType::Hard.id() as usize), 2);
        assert_eq!(rd.clear_count(ClearType::Easy.id() as usize), 1);
        assert_eq!(rd.clear_count(ClearType::Failed.id() as usize), 1);
        assert_eq!(rd.clear_count(ClearType::Normal.id() as usize), 0);
    }

    #[test]
    fn update_score_prevrank_first_update() {
        let mut rd = RankingData::new();
        let scores = vec![
            make_ir_score("Player1", 200, 50, ClearType::Hard), // ex: 450
            make_ir_score("", 150, 30, ClearType::Normal),      // ex: 330 (self)
        ];
        let mut local = ScoreData::default();
        local.epg = 150;
        local.egr = 30; // ex: 330
        rd.update_score(&scores, Some(&local));

        // First update: prevrank = max(irrank, localrank) = max(2, 2) = 2
        assert_eq!(rd.previous_rank(), 2);
    }

    #[test]
    fn update_score_prevrank_subsequent_update() {
        let mut rd = RankingData::new();

        // First update
        let scores1 = vec![
            make_ir_score("P1", 200, 50, ClearType::Hard),
            make_ir_score("", 100, 20, ClearType::Normal), // self, rank 2
        ];
        rd.update_score(&scores1, None);
        assert_eq!(rd.rank(), 2);

        // Second update
        let scores2 = vec![
            make_ir_score("P1", 200, 50, ClearType::Hard),
            make_ir_score("", 180, 40, ClearType::Hard), // self improved, rank 2
            make_ir_score("P2", 100, 20, ClearType::Normal),
        ];
        rd.update_score(&scores2, None);

        // prevrank should be the old irrank (2)
        assert_eq!(rd.previous_rank(), 2);
    }

    #[test]
    fn score_access() {
        let mut rd = RankingData::new();
        let scores = vec![make_ir_score("P1", 200, 50, ClearType::Hard)];
        rd.update_score(&scores, None);

        assert!(rd.score(0).is_some());
        assert_eq!(rd.score(0).unwrap().epg, 200);
        assert!(rd.score(1).is_none());
    }

    #[test]
    fn score_ranking_out_of_bounds() {
        let rd = RankingData::new();
        assert!(rd.score_ranking(0).is_none());
    }

    #[test]
    fn clear_count_out_of_bounds() {
        let rd = RankingData::new();
        assert_eq!(rd.clear_count(100), 0);
    }
}
