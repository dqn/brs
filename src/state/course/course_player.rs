//! Course player for managing consecutive song play.

use super::{Course, CourseSong};
use crate::state::play::{PlayResult, Score};

/// State of the course play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseState {
    /// Not started yet.
    NotStarted,
    /// Playing a song.
    Playing,
    /// Transitioning between songs.
    Transition,
    /// Course completed successfully.
    Cleared,
    /// Course failed.
    Failed,
}

/// Accumulated result from course play.
#[derive(Debug, Clone, Default)]
pub struct CourseResult {
    /// Total score across all songs.
    pub total_score: Score,
    /// Final gauge value.
    pub final_gauge: f64,
    /// Number of songs cleared.
    pub songs_cleared: usize,
    /// Results for each song.
    pub song_results: Vec<PlayResult>,
}

impl CourseResult {
    /// Create a new course result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a song result to the accumulated total.
    pub fn add_result(&mut self, result: PlayResult) {
        self.total_score.pg_count += result.score.pg_count;
        self.total_score.gr_count += result.score.gr_count;
        self.total_score.gd_count += result.score.gd_count;
        self.total_score.bd_count += result.score.bd_count;
        self.total_score.pr_count += result.score.pr_count;
        self.total_score.ms_count += result.score.ms_count;

        if result.score.max_combo > self.total_score.max_combo {
            self.total_score.max_combo = result.score.max_combo;
        }

        self.final_gauge = result.gauge_value;

        if result.is_clear {
            self.songs_cleared += 1;
        }

        self.song_results.push(result);
    }

    /// Get total EX score.
    pub fn ex_score(&self) -> u32 {
        self.total_score.ex_score()
    }
}

/// Player for managing course (Dan) play progression.
pub struct CoursePlayer {
    /// The course being played.
    course: Course,
    /// Current song index (0-based).
    current_song_index: usize,
    /// Current state.
    state: CourseState,
    /// Accumulated result.
    result: CourseResult,
    /// Current gauge value (for carry-over).
    current_gauge: f64,
}

impl CoursePlayer {
    /// Create a new course player.
    pub fn new(course: Course) -> Self {
        Self {
            course,
            current_song_index: 0,
            state: CourseState::NotStarted,
            result: CourseResult::new(),
            current_gauge: 100.0, // Start at full gauge for HARD-type
        }
    }

    /// Get the course being played.
    pub fn course(&self) -> &Course {
        &self.course
    }

    /// Get the current state.
    pub fn state(&self) -> CourseState {
        self.state
    }

    /// Get the current song index.
    pub fn current_song_index(&self) -> usize {
        self.current_song_index
    }

    /// Get the current song.
    pub fn current_song(&self) -> Option<&CourseSong> {
        self.course.get_song(self.current_song_index)
    }

    /// Get the accumulated result.
    pub fn result(&self) -> &CourseResult {
        &self.result
    }

    /// Get the current gauge value for carry-over.
    pub fn current_gauge(&self) -> f64 {
        self.current_gauge
    }

    /// Start the course.
    pub fn start(&mut self) {
        self.state = CourseState::Playing;
        self.current_song_index = 0;
    }

    /// Process the result of a completed song.
    /// Returns true if there are more songs, false if course is complete.
    pub fn on_song_complete(&mut self, result: PlayResult) -> bool {
        // Store result
        self.result.add_result(result.clone());

        // Handle gauge carry-over
        if self.course.constraints.gauge_carry {
            self.current_gauge = result.gauge_value;
        } else {
            self.current_gauge = 100.0;
        }

        // Check if failed
        if !result.is_clear {
            self.state = CourseState::Failed;
            return false;
        }

        // Check no_good constraint
        if self.course.constraints.no_good && result.score.gd_count > 0 {
            self.state = CourseState::Failed;
            return false;
        }

        // Move to next song
        self.current_song_index += 1;

        if self.current_song_index >= self.course.song_count() {
            // All songs cleared
            self.state = CourseState::Cleared;
            false
        } else {
            // More songs to play
            self.state = CourseState::Transition;
            true
        }
    }

    /// Continue to the next song (after transition).
    pub fn continue_to_next(&mut self) {
        if self.state == CourseState::Transition {
            self.state = CourseState::Playing;
        }
    }

    /// Check if the course is complete (cleared or failed).
    pub fn is_complete(&self) -> bool {
        matches!(self.state, CourseState::Cleared | CourseState::Failed)
    }

    /// Check if the course was cleared.
    pub fn is_cleared(&self) -> bool {
        self.state == CourseState::Cleared
    }

    /// Get the progress as a string (e.g., "2/4").
    pub fn progress_string(&self) -> String {
        format!(
            "{}/{}",
            self.current_song_index + 1,
            self.course.song_count()
        )
    }

    /// Reset the course player.
    pub fn reset(&mut self) {
        self.current_song_index = 0;
        self.state = CourseState::NotStarted;
        self.result = CourseResult::new();
        self.current_gauge = 100.0;
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::{ChartFormat, JudgeRankType, LongNoteMode, PlayMode};
    use crate::state::course::CourseConstraints;
    use crate::state::play::GaugeType;

    fn create_test_course() -> Course {
        let songs = vec![
            super::super::CourseSong::new(PathBuf::from("song1.bms"), "Song 1".to_string()),
            super::super::CourseSong::new(PathBuf::from("song2.bms"), "Song 2".to_string()),
        ];
        Course::new(
            "Test".to_string(),
            songs,
            GaugeType::Hard,
            CourseConstraints::class_mode(),
        )
    }

    fn create_clear_result() -> PlayResult {
        PlayResult {
            score: Score::new(100),
            gauge_value: 80.0,
            gauge_type: GaugeType::Hard,
            is_clear: true,
            play_time_ms: 60000.0,
            fast_count: 10,
            slow_count: 10,
            play_mode: PlayMode::Beat7K,
            long_note_mode: LongNoteMode::Ln,
            judge_rank: 2,
            judge_rank_type: JudgeRankType::BmsRank,
            total: 200.0,
            source_format: ChartFormat::Bms,
        }
    }

    fn create_failed_result() -> PlayResult {
        PlayResult {
            score: Score::new(100),
            gauge_value: 0.0,
            gauge_type: GaugeType::Hard,
            is_clear: false,
            play_time_ms: 30000.0,
            fast_count: 5,
            slow_count: 5,
            play_mode: PlayMode::Beat7K,
            long_note_mode: LongNoteMode::Ln,
            judge_rank: 2,
            judge_rank_type: JudgeRankType::BmsRank,
            total: 200.0,
            source_format: ChartFormat::Bms,
        }
    }

    #[test]
    fn test_course_player_start() {
        let course = create_test_course();
        let mut player = CoursePlayer::new(course);

        assert_eq!(player.state(), CourseState::NotStarted);

        player.start();
        assert_eq!(player.state(), CourseState::Playing);
        assert_eq!(player.current_song_index(), 0);
    }

    #[test]
    fn test_course_player_clear() {
        let course = create_test_course();
        let mut player = CoursePlayer::new(course);
        player.start();

        // Clear first song
        let has_more = player.on_song_complete(create_clear_result());
        assert!(has_more);
        assert_eq!(player.state(), CourseState::Transition);
        assert_eq!(player.current_song_index(), 1);

        player.continue_to_next();
        assert_eq!(player.state(), CourseState::Playing);

        // Clear second song
        let has_more = player.on_song_complete(create_clear_result());
        assert!(!has_more);
        assert_eq!(player.state(), CourseState::Cleared);
        assert!(player.is_cleared());
    }

    #[test]
    fn test_course_player_fail() {
        let course = create_test_course();
        let mut player = CoursePlayer::new(course);
        player.start();

        // Fail first song
        let has_more = player.on_song_complete(create_failed_result());
        assert!(!has_more);
        assert_eq!(player.state(), CourseState::Failed);
        assert!(!player.is_cleared());
    }

    #[test]
    fn test_gauge_carry_over() {
        let course = create_test_course();
        let mut player = CoursePlayer::new(course);
        player.start();

        let mut result = create_clear_result();
        result.gauge_value = 75.0;

        player.on_song_complete(result);
        assert!((player.current_gauge() - 75.0).abs() < 0.001);
    }
}
