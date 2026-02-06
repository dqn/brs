use crate::play::play_result::PlayResult;
use crate::play::score::ScoreData;
use crate::state::course::course_data::CourseData;

/// Course play progress tracking.
///
/// Manages gauge carryover between songs and accumulates
/// course-wide score data.
pub struct CoursePlayer {
    /// The course definition.
    course: CourseData,
    /// Current song index (0-based).
    current_index: usize,
    /// Per-song play results (populated as songs are completed).
    song_results: Vec<PlayResult>,
    /// Cumulative gauge values at the end of each song.
    gauge_history: Vec<f32>,
    /// Whether the course has been failed (gauge death mid-course).
    failed: bool,
}

impl CoursePlayer {
    /// Create a new course player.
    pub fn new(course: CourseData) -> Self {
        Self {
            course,
            current_index: 0,
            song_results: Vec::new(),
            gauge_history: Vec::new(),
            failed: false,
        }
    }

    /// Get the course definition.
    pub fn course(&self) -> &CourseData {
        &self.course
    }

    /// Get the current song index.
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the total number of songs.
    pub fn song_count(&self) -> usize {
        self.course.song_count()
    }

    /// Whether there are more songs to play.
    pub fn has_next(&self) -> bool {
        self.current_index < self.course.song_count()
    }

    /// Whether the course has been failed.
    pub fn is_failed(&self) -> bool {
        self.failed
    }

    /// Whether all songs have been completed successfully.
    pub fn is_complete(&self) -> bool {
        self.song_results.len() == self.course.song_count() && !self.failed
    }

    /// Get the SHA-256 of the current song.
    pub fn current_sha256(&self) -> Option<&str> {
        self.course
            .songs
            .get(self.current_index)
            .map(|s| s.sha256.as_str())
    }

    /// Get per-song results.
    pub fn song_results(&self) -> &[PlayResult] {
        &self.song_results
    }

    /// Get gauge history (final gauge value per completed song).
    pub fn gauge_history(&self) -> &[f32] {
        &self.gauge_history
    }

    /// Get the gauge value to carry over to the next song.
    /// Returns None if no songs have been completed yet.
    pub fn carryover_gauge(&self) -> Option<f32> {
        self.gauge_history.last().copied()
    }

    /// Record a song result and advance to the next song.
    /// Returns true if there are more songs remaining.
    pub fn record_result(&mut self, result: PlayResult) -> bool {
        let gauge = result.gauge_value;
        let is_clear = result.clear_type as u8 >= 2; // AssistEasy or better

        self.gauge_history.push(gauge);
        self.song_results.push(result);

        // If gauge dropped to 0 on a hard-type gauge, course is failed
        if gauge <= 0.0 && !is_clear {
            self.failed = true;
            return false;
        }

        self.current_index += 1;
        self.has_next()
    }

    /// Build cumulative score data across all completed songs.
    pub fn cumulative_score(&self) -> ScoreData {
        let total_notes: u32 = self.song_results.iter().map(|r| r.score.total_notes).sum();
        let mut cumulative = ScoreData::new(total_notes);

        for result in &self.song_results {
            for i in 0..6 {
                cumulative.early_counts[i] += result.score.early_counts[i];
                cumulative.late_counts[i] += result.score.late_counts[i];
            }
            cumulative.max_combo = cumulative.max_combo.max(result.score.max_combo);
            cumulative.pass_notes += result.score.pass_notes;
            cumulative.min_bp += result.score.min_bp;
        }

        cumulative
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::clear_type::ClearType;
    use crate::play::gauge::gauge_property::GaugeType;
    use crate::play::score::ScoreRank;
    use crate::state::course::course_data::CourseSong;

    fn make_course(count: usize) -> CourseData {
        let mut course = CourseData::new("Test Course".to_string());
        for i in 0..count {
            course.songs.push(CourseSong {
                sha256: format!("hash_{}", i),
                title: format!("Song {}", i),
            });
        }
        course
    }

    fn make_result(clear: ClearType, gauge: f32, pg: u32, total: u32) -> PlayResult {
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
            gauge_type: GaugeType::Normal,
        }
    }

    #[test]
    fn new_course_player() {
        let player = CoursePlayer::new(make_course(4));
        assert_eq!(player.current_index(), 0);
        assert_eq!(player.song_count(), 4);
        assert!(player.has_next());
        assert!(!player.is_failed());
        assert!(!player.is_complete());
    }

    #[test]
    fn current_sha256() {
        let player = CoursePlayer::new(make_course(3));
        assert_eq!(player.current_sha256(), Some("hash_0"));
    }

    #[test]
    fn record_result_advances() {
        let mut player = CoursePlayer::new(make_course(3));
        let has_next = player.record_result(make_result(ClearType::Normal, 80.0, 100, 100));
        assert!(has_next);
        assert_eq!(player.current_index(), 1);
        assert_eq!(player.current_sha256(), Some("hash_1"));
    }

    #[test]
    fn gauge_carryover() {
        let mut player = CoursePlayer::new(make_course(3));
        assert_eq!(player.carryover_gauge(), None);

        player.record_result(make_result(ClearType::Normal, 75.0, 80, 100));
        assert_eq!(player.carryover_gauge(), Some(75.0));

        player.record_result(make_result(ClearType::Hard, 50.0, 90, 100));
        assert_eq!(player.carryover_gauge(), Some(50.0));
    }

    #[test]
    fn course_complete() {
        let mut player = CoursePlayer::new(make_course(2));
        player.record_result(make_result(ClearType::Normal, 80.0, 90, 100));
        assert!(!player.is_complete());

        player.record_result(make_result(ClearType::Normal, 85.0, 95, 100));
        assert!(player.is_complete());
        assert!(!player.is_failed());
    }

    #[test]
    fn course_failed_on_gauge_death() {
        let mut player = CoursePlayer::new(make_course(3));
        player.record_result(make_result(ClearType::Normal, 80.0, 90, 100));

        let has_next = player.record_result(make_result(ClearType::Failed, 0.0, 30, 100));
        assert!(!has_next);
        assert!(player.is_failed());
        assert!(!player.is_complete());
    }

    #[test]
    fn cumulative_score() {
        let mut player = CoursePlayer::new(make_course(2));
        player.record_result(make_result(ClearType::Normal, 80.0, 90, 100));
        player.record_result(make_result(ClearType::Normal, 85.0, 95, 100));

        let cumulative = player.cumulative_score();
        assert_eq!(cumulative.total_notes, 200);
        assert_eq!(cumulative.early_counts[0], 185); // 90 + 95
        assert_eq!(cumulative.max_combo, 95); // max of individual combos
        assert_eq!(cumulative.min_bp, 15); // (100-90) + (100-95)
    }

    #[test]
    fn gauge_history() {
        let mut player = CoursePlayer::new(make_course(3));
        player.record_result(make_result(ClearType::Normal, 80.0, 90, 100));
        player.record_result(make_result(ClearType::Normal, 75.0, 85, 100));

        let history = player.gauge_history();
        assert_eq!(history.len(), 2);
        assert!((history[0] - 80.0).abs() < 0.001);
        assert!((history[1] - 75.0).abs() < 0.001);
    }
}
