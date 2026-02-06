use anyhow::Result;

use crate::database::models::ScoreData as DbScoreData;
use crate::database::score_db::ScoreDatabase;
use crate::play::play_result::PlayResult;
use crate::state::game_state::{GameState, StateTransition};

/// Phase of the result screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultPhase {
    /// Displaying the result (score graph animation, etc.).
    Display,
    /// Fading out to transition.
    FadeOut,
}

/// Configuration for the result screen.
pub struct ResultConfig {
    /// Display duration in microseconds before input is accepted.
    pub input_delay_us: i64,
    /// Scene duration in microseconds before auto-advance.
    pub scene_duration_us: i64,
    /// Fade-out duration in microseconds.
    pub fadeout_duration_us: i64,
}

impl Default for ResultConfig {
    fn default() -> Self {
        Self {
            input_delay_us: 500_000,       // 0.5 seconds
            scene_duration_us: 10_000_000, // 10 seconds
            fadeout_duration_us: 500_000,  // 0.5 seconds
        }
    }
}

/// State for the music result screen.
///
/// Corresponds to beatoraja's MusicResult. Displays play results
/// (score, rank, clear type, gauge graph) and saves to DB.
pub struct ResultState {
    phase: ResultPhase,
    config: ResultConfig,
    /// The play result to display.
    play_result: PlayResult,
    /// SHA-256 of the chart.
    sha256: String,
    /// Play mode (for DB storage).
    mode: i32,
    /// Elapsed time in microseconds since entering the state.
    elapsed_us: i64,
    /// Time at which fade-out started (None if not fading).
    fadeout_start_us: Option<i64>,
    /// Whether the user accepted input (pressed a key to advance).
    input_accepted: bool,
    /// Whether the score has been saved to DB.
    score_saved: bool,
    /// Previous best score data from DB (for comparison display).
    old_score: Option<DbScoreData>,
}

impl ResultState {
    pub fn new(play_result: PlayResult, sha256: String, mode: i32, config: ResultConfig) -> Self {
        Self {
            phase: ResultPhase::Display,
            config,
            play_result,
            sha256,
            mode,
            elapsed_us: 0,
            fadeout_start_us: None,
            input_accepted: false,
            score_saved: false,
            old_score: None,
        }
    }

    /// Get the current phase.
    pub fn phase(&self) -> ResultPhase {
        self.phase
    }

    /// Get the play result.
    pub fn play_result(&self) -> &PlayResult {
        &self.play_result
    }

    /// Get the old score for comparison.
    pub fn old_score(&self) -> Option<&DbScoreData> {
        self.old_score.as_ref()
    }

    /// Whether input is currently accepted.
    pub fn is_input_active(&self) -> bool {
        self.elapsed_us >= self.config.input_delay_us && self.phase == ResultPhase::Display
    }

    /// Request advance (user pressed a key).
    pub fn advance(&mut self) {
        if self.is_input_active() {
            self.input_accepted = true;
            self.start_fadeout();
        }
    }

    /// Save the score to the database.
    /// Should be called during `create()` or early in the lifecycle.
    pub fn save_score(&mut self, score_db: &ScoreDatabase) -> Result<()> {
        if self.score_saved {
            return Ok(());
        }

        // Load existing best score
        self.old_score = score_db.get_score(&self.sha256, self.mode)?;

        // Build DB score record from play result
        let new_db_score = self.build_db_score();

        // Merge with existing best
        if let Some(ref mut old) = self.old_score {
            let existing = old.clone();
            old.update(&new_db_score);
            score_db.upsert_score(old)?;
            // Keep original for comparison display
            self.old_score = Some(existing);
        } else {
            score_db.upsert_score(&new_db_score)?;
        }

        self.score_saved = true;
        Ok(())
    }

    /// Build a DB score record from the play result.
    fn build_db_score(&self) -> DbScoreData {
        let s = &self.play_result.score;
        DbScoreData {
            sha256: self.sha256.clone(),
            mode: self.mode,
            clear: self.play_result.clear_type as i32,
            epg: s.early_counts[0] as i32,
            lpg: s.late_counts[0] as i32,
            egr: s.early_counts[1] as i32,
            lgr: s.late_counts[1] as i32,
            egd: s.early_counts[2] as i32,
            lgd: s.late_counts[2] as i32,
            ebd: s.early_counts[3] as i32,
            lbd: s.late_counts[3] as i32,
            epr: s.early_counts[4] as i32,
            lpr: s.late_counts[4] as i32,
            ems: s.early_counts[5] as i32,
            lms: s.late_counts[5] as i32,
            notes: s.total_notes as i32,
            combo: s.max_combo as i32,
            minbp: s.min_bp as i32,
            playcount: 1,
            clearcount: if self.play_result.clear_type as u8 >= 2 {
                1
            } else {
                0
            },
            ..Default::default()
        }
    }

    fn start_fadeout(&mut self) {
        if self.phase != ResultPhase::FadeOut {
            self.phase = ResultPhase::FadeOut;
            self.fadeout_start_us = Some(self.elapsed_us);
        }
    }
}

impl GameState for ResultState {
    fn create(&mut self) -> Result<()> {
        self.phase = ResultPhase::Display;
        self.elapsed_us = 0;
        self.fadeout_start_us = None;
        self.input_accepted = false;
        Ok(())
    }

    fn update(&mut self, dt_us: i64) -> Result<StateTransition> {
        self.elapsed_us += dt_us;

        match self.phase {
            ResultPhase::Display => {
                // Auto-advance after scene duration
                if self.elapsed_us >= self.config.scene_duration_us {
                    self.start_fadeout();
                }
                Ok(StateTransition::None)
            }
            ResultPhase::FadeOut => {
                let fadeout_elapsed =
                    self.elapsed_us - self.fadeout_start_us.unwrap_or(self.elapsed_us);
                if fadeout_elapsed >= self.config.fadeout_duration_us {
                    // Always transition back to select
                    Ok(StateTransition::Back)
                } else {
                    Ok(StateTransition::None)
                }
            }
        }
    }

    fn dispose(&mut self) {
        self.phase = ResultPhase::Display;
        self.fadeout_start_us = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::clear_type::ClearType;
    use crate::play::score::{ScoreData, ScoreRank};

    fn make_play_result() -> PlayResult {
        let mut score = ScoreData::new(100);
        score.early_counts[0] = 80;
        score.early_counts[1] = 15;
        score.early_counts[3] = 5;
        score.max_combo = 50;
        score.min_bp = 5;
        PlayResult {
            score,
            clear_type: ClearType::Normal,
            rank: ScoreRank::AAA,
            gauge_value: 85.0,
            gauge_type: 2,
        }
    }

    fn make_result_state() -> ResultState {
        ResultState::new(
            make_play_result(),
            "sha256_test".to_string(),
            0,
            ResultConfig {
                input_delay_us: 500_000,
                scene_duration_us: 5_000_000,
                fadeout_duration_us: 500_000,
            },
        )
    }

    #[test]
    fn initial_phase_is_display() {
        let state = make_result_state();
        assert_eq!(state.phase(), ResultPhase::Display);
    }

    #[test]
    fn input_not_accepted_during_delay() {
        let mut state = make_result_state();
        state.create().unwrap();
        state.update(100_000).unwrap(); // 0.1s < 0.5s delay
        assert!(!state.is_input_active());
    }

    #[test]
    fn input_accepted_after_delay() {
        let mut state = make_result_state();
        state.create().unwrap();
        state.update(600_000).unwrap(); // 0.6s > 0.5s delay
        assert!(state.is_input_active());
    }

    #[test]
    fn advance_starts_fadeout() {
        let mut state = make_result_state();
        state.create().unwrap();
        state.update(600_000).unwrap();

        state.advance();
        assert_eq!(state.phase(), ResultPhase::FadeOut);
    }

    #[test]
    fn fadeout_transitions_back() {
        let mut state = make_result_state();
        state.create().unwrap();
        state.update(600_000).unwrap();
        state.advance();

        // During fadeout
        let result = state.update(400_000).unwrap();
        assert_eq!(result, StateTransition::None);

        // After fadeout
        let result = state.update(200_000).unwrap();
        assert_eq!(result, StateTransition::Back);
    }

    #[test]
    fn auto_advance_after_scene_duration() {
        let mut state = make_result_state();
        state.create().unwrap();

        // Before scene duration
        state.update(4_000_000).unwrap();
        assert_eq!(state.phase(), ResultPhase::Display);

        // After scene duration
        state.update(1_500_000).unwrap();
        assert_eq!(state.phase(), ResultPhase::FadeOut);
    }

    #[test]
    fn save_score_to_db() {
        let mut state = make_result_state();
        state.create().unwrap();

        let db = ScoreDatabase::open_in_memory().unwrap();
        state.save_score(&db).unwrap();

        let saved = db.get_score("sha256_test", 0).unwrap();
        assert!(saved.is_some());
        let s = saved.unwrap();
        assert_eq!(s.epg, 80);
        assert_eq!(s.egr, 15);
        assert_eq!(s.clear, ClearType::Normal as i32);
    }

    #[test]
    fn save_score_updates_best() {
        let db = ScoreDatabase::open_in_memory().unwrap();

        // Insert an initial score
        let initial = DbScoreData {
            sha256: "sha256_test".to_string(),
            mode: 0,
            clear: ClearType::Easy as i32,
            epg: 50,
            egr: 20,
            notes: 100,
            combo: 30,
            minbp: 10,
            ..Default::default()
        };
        db.upsert_score(&initial).unwrap();

        // New score with better clear
        let mut state = make_result_state();
        state.create().unwrap();
        state.save_score(&db).unwrap();

        let saved = db.get_score("sha256_test", 0).unwrap().unwrap();
        // Clear should be updated to Normal (5) since it's better than Easy (4)
        assert_eq!(saved.clear, ClearType::Normal as i32);
        // Old score is preserved for comparison
        assert!(state.old_score().is_some());
        assert_eq!(state.old_score().unwrap().clear, ClearType::Easy as i32);
    }

    #[test]
    fn play_result_accessible() {
        let state = make_result_state();
        assert_eq!(state.play_result().clear_type, ClearType::Normal);
        assert_eq!(state.play_result().rank, ScoreRank::AAA);
    }
}
