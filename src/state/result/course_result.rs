use anyhow::Result;

use crate::play::clear_type::ClearType;
use crate::play::play_result::PlayResult;
use crate::state::course::course_player::CoursePlayer;
use crate::state::game_state::{GameState, StateTransition};

/// Phase of the course result screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseResultPhase {
    /// Displaying the result.
    Display,
    /// Fading out.
    FadeOut,
}

/// Configuration for the course result screen.
pub struct CourseResultConfig {
    /// Input delay before accepting user input (microseconds).
    pub input_delay_us: i64,
    /// Scene duration before auto-advance (microseconds).
    pub scene_duration_us: i64,
    /// Fade-out duration (microseconds).
    pub fadeout_duration_us: i64,
}

impl Default for CourseResultConfig {
    fn default() -> Self {
        Self {
            input_delay_us: 500_000,
            scene_duration_us: 15_000_000, // 15 seconds (courses show more data)
            fadeout_duration_us: 500_000,
        }
    }
}

/// State for the course result screen.
///
/// Corresponds to beatoraja's CourseResult. Displays cumulative
/// results across all songs in the course.
pub struct CourseResultState {
    phase: CourseResultPhase,
    config: CourseResultConfig,
    /// Cumulative course result.
    course_result: PlayResult,
    /// Per-song results for display.
    song_results: Vec<PlayResult>,
    /// Whether the course was completed or failed.
    course_cleared: bool,
    /// Elapsed time.
    elapsed_us: i64,
    /// Fade-out start time.
    fadeout_start_us: Option<i64>,
}

impl CourseResultState {
    /// Create a course result state from a completed (or failed) course player.
    pub fn new(player: &CoursePlayer, config: CourseResultConfig) -> Self {
        let cumulative = player.cumulative_score();
        let course_cleared = player.is_complete();
        let clear_type = if course_cleared {
            // Use the lowest clear type among all songs
            player
                .song_results()
                .iter()
                .map(|r| r.clear_type)
                .min()
                .unwrap_or(ClearType::Failed)
        } else {
            ClearType::Failed
        };

        let rank = cumulative.rank();
        let gauge_value = player.carryover_gauge().unwrap_or(0.0);

        let course_result = PlayResult {
            score: cumulative,
            clear_type,
            rank,
            gauge_value,
            gauge_type: player
                .song_results()
                .last()
                .map(|r| r.gauge_type)
                .unwrap_or(2),
        };

        Self {
            phase: CourseResultPhase::Display,
            config,
            course_result,
            song_results: player.song_results().to_vec(),
            course_cleared,
            elapsed_us: 0,
            fadeout_start_us: None,
        }
    }

    /// Get the current phase.
    pub fn phase(&self) -> CourseResultPhase {
        self.phase
    }

    /// Get the cumulative course result.
    pub fn course_result(&self) -> &PlayResult {
        &self.course_result
    }

    /// Get per-song results.
    pub fn song_results(&self) -> &[PlayResult] {
        &self.song_results
    }

    /// Whether the course was cleared.
    pub fn is_cleared(&self) -> bool {
        self.course_cleared
    }

    /// Whether input is currently accepted.
    pub fn is_input_active(&self) -> bool {
        self.elapsed_us >= self.config.input_delay_us && self.phase == CourseResultPhase::Display
    }

    /// Request advance (user input).
    pub fn advance(&mut self) {
        if self.is_input_active() {
            self.start_fadeout();
        }
    }

    fn start_fadeout(&mut self) {
        if self.phase != CourseResultPhase::FadeOut {
            self.phase = CourseResultPhase::FadeOut;
            self.fadeout_start_us = Some(self.elapsed_us);
        }
    }
}

impl GameState for CourseResultState {
    fn create(&mut self) -> Result<()> {
        self.phase = CourseResultPhase::Display;
        self.elapsed_us = 0;
        self.fadeout_start_us = None;
        Ok(())
    }

    fn update(&mut self, dt_us: i64) -> Result<StateTransition> {
        self.elapsed_us += dt_us;

        match self.phase {
            CourseResultPhase::Display => {
                if self.elapsed_us >= self.config.scene_duration_us {
                    self.start_fadeout();
                }
                Ok(StateTransition::None)
            }
            CourseResultPhase::FadeOut => {
                let fadeout_elapsed =
                    self.elapsed_us - self.fadeout_start_us.unwrap_or(self.elapsed_us);
                if fadeout_elapsed >= self.config.fadeout_duration_us {
                    // Always return to select after course result
                    Ok(StateTransition::Back)
                } else {
                    Ok(StateTransition::None)
                }
            }
        }
    }

    fn dispose(&mut self) {
        self.phase = CourseResultPhase::Display;
        self.fadeout_start_us = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::score::{ScoreData, ScoreRank};
    use crate::state::course::course_data::{CourseData, CourseSong};
    use crate::state::course::course_player::CoursePlayer;

    fn make_course(count: usize) -> CourseData {
        let mut course = CourseData::new("Test".to_string());
        for i in 0..count {
            course.songs.push(CourseSong {
                sha256: format!("h{}", i),
                title: format!("S{}", i),
            });
        }
        course
    }

    fn make_play_result(clear: ClearType, gauge: f32, pg: u32, total: u32) -> PlayResult {
        let mut score = ScoreData::new(total);
        score.early_counts[0] = pg;
        score.max_combo = pg;
        score.pass_notes = total;
        score.min_bp = total.saturating_sub(pg);
        PlayResult {
            score,
            clear_type: clear,
            rank: ScoreRank::AAA,
            gauge_value: gauge,
            gauge_type: 2,
        }
    }

    #[test]
    fn course_result_from_cleared_course() {
        let mut player = CoursePlayer::new(make_course(2));
        player.record_result(make_play_result(ClearType::Normal, 80.0, 90, 100));
        player.record_result(make_play_result(ClearType::Hard, 60.0, 95, 100));

        let state = CourseResultState::new(&player, CourseResultConfig::default());
        assert!(state.is_cleared());
        // Clear type should be the lowest: Normal
        assert_eq!(state.course_result().clear_type, ClearType::Normal);
        assert_eq!(state.song_results().len(), 2);
    }

    #[test]
    fn course_result_from_failed_course() {
        let mut player = CoursePlayer::new(make_course(3));
        player.record_result(make_play_result(ClearType::Normal, 80.0, 90, 100));
        player.record_result(make_play_result(ClearType::Failed, 0.0, 30, 100));

        let state = CourseResultState::new(&player, CourseResultConfig::default());
        assert!(!state.is_cleared());
        assert_eq!(state.course_result().clear_type, ClearType::Failed);
    }

    #[test]
    fn cumulative_score_displayed() {
        let mut player = CoursePlayer::new(make_course(2));
        player.record_result(make_play_result(ClearType::Normal, 80.0, 90, 100));
        player.record_result(make_play_result(ClearType::Normal, 85.0, 95, 100));

        let state = CourseResultState::new(&player, CourseResultConfig::default());
        assert_eq!(state.course_result().score.total_notes, 200);
        assert_eq!(state.course_result().score.early_counts[0], 185);
    }

    #[test]
    fn advance_and_fadeout() {
        let mut player = CoursePlayer::new(make_course(1));
        player.record_result(make_play_result(ClearType::Normal, 80.0, 90, 100));

        let mut state = CourseResultState::new(
            &player,
            CourseResultConfig {
                input_delay_us: 100_000,
                scene_duration_us: 5_000_000,
                fadeout_duration_us: 200_000,
            },
        );
        state.create().unwrap();

        // Not yet input active
        state.update(50_000).unwrap();
        assert!(!state.is_input_active());

        // Input active
        state.update(100_000).unwrap();
        assert!(state.is_input_active());

        state.advance();
        assert_eq!(state.phase(), CourseResultPhase::FadeOut);

        // After fadeout
        let result = state.update(300_000).unwrap();
        assert_eq!(result, StateTransition::Back);
    }
}
