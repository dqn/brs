use crate::game::{ClearLamp, PlayResult};

use super::{DanCourse, DanRequirements};

/// Statistics accumulated across all stages
#[derive(Debug, Clone, Default)]
pub struct CourseStats {
    pub ex_score: u32,
    pub max_combo: u32,
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
    pub total_notes: u32,
    pub fast_count: u32,
    pub slow_count: u32,
}

impl CourseStats {
    /// Add a stage result to the accumulated stats
    pub fn add_result(&mut self, result: &PlayResult) {
        self.ex_score += result.ex_score;
        if result.max_combo > self.max_combo {
            self.max_combo = result.max_combo;
        }
        self.pgreat_count += result.pgreat_count;
        self.great_count += result.great_count;
        self.good_count += result.good_count;
        self.bad_count += result.bad_count;
        self.poor_count += result.poor_count;
        self.total_notes += result.total_notes;
        self.fast_count += result.fast_count;
        self.slow_count += result.slow_count;
    }

    /// Calculate accuracy percentage
    pub fn accuracy(&self) -> f64 {
        if self.total_notes == 0 {
            return 0.0;
        }
        let max_ex = self.total_notes * 2;
        self.ex_score as f64 / max_ex as f64 * 100.0
    }

    /// Get rank based on accuracy
    pub fn rank(&self) -> &'static str {
        let acc = self.accuracy();
        if acc >= 100.0 {
            "MAX"
        } else if acc >= 94.44 {
            "AAA"
        } else if acc >= 88.88 {
            "AA"
        } else if acc >= 77.77 {
            "A"
        } else if acc >= 66.66 {
            "B"
        } else if acc >= 55.55 {
            "C"
        } else if acc >= 44.44 {
            "D"
        } else if acc >= 33.33 {
            "E"
        } else {
            "F"
        }
    }
}

/// Result of a single stage
#[derive(Debug, Clone)]
pub struct StageResult {
    #[allow(dead_code)]
    pub chart_path: String,
    pub title: String,
    pub ex_score: u32,
    pub clear_lamp: ClearLamp,
    pub end_gauge_hp: f32,
}

/// State tracking during dan course play
pub struct CourseState {
    /// The course being played
    course: DanCourse,
    /// Current stage index (0-based)
    current_stage: usize,
    /// Current gauge HP (carried between stages)
    gauge_hp: f32,
    /// Whether the course has been failed
    failed: bool,
    /// Results from completed stages
    stage_results: Vec<StageResult>,
    /// Accumulated statistics
    total_stats: CourseStats,
}

impl CourseState {
    /// Create a new course state
    pub fn new(course: DanCourse) -> Self {
        Self {
            course,
            current_stage: 0,
            gauge_hp: 100.0, // Start at full for Hard gauge
            failed: false,
            stage_results: Vec::new(),
            total_stats: CourseStats::default(),
        }
    }

    /// Get the current stage index (0-based)
    pub fn current_stage(&self) -> usize {
        self.current_stage
    }

    /// Get the total number of stages
    pub fn total_stages(&self) -> usize {
        self.course.stage_count()
    }

    /// Get the current gauge HP
    #[allow(dead_code)]
    pub fn gauge_hp(&self) -> f32 {
        self.gauge_hp
    }

    /// Check if the course has been failed
    #[allow(dead_code)]
    pub fn is_failed(&self) -> bool {
        self.failed
    }

    /// Check if all stages are completed
    pub fn is_completed(&self) -> bool {
        self.current_stage >= self.course.stage_count()
    }

    /// Get the course reference
    pub fn course(&self) -> &DanCourse {
        &self.course
    }

    /// Get the accumulated statistics
    pub fn total_stats(&self) -> &CourseStats {
        &self.total_stats
    }

    /// Get the stage results
    pub fn stage_results(&self) -> &[StageResult] {
        &self.stage_results
    }

    /// Set the initial gauge HP for continuing from previous stage
    #[allow(dead_code)]
    pub fn set_initial_gauge_hp(&mut self, hp: f32) {
        self.gauge_hp = hp;
    }

    /// Record a stage completion and advance to next stage
    /// Returns true if the course should continue
    pub fn complete_stage(&mut self, result: &PlayResult, end_gauge_hp: f32) -> bool {
        // Record stage result
        self.stage_results.push(StageResult {
            chart_path: result.chart_path.clone(),
            title: result.title.clone(),
            ex_score: result.ex_score,
            clear_lamp: result.clear_lamp,
            end_gauge_hp,
        });

        // Accumulate stats
        self.total_stats.add_result(result);

        // Update gauge HP for next stage
        self.gauge_hp = end_gauge_hp;

        // Check for failure (gauge depleted)
        if end_gauge_hp <= 0.0 {
            self.failed = true;
            return false;
        }

        // Advance to next stage
        self.current_stage += 1;

        // Continue if more stages remain
        !self.is_completed()
    }

    /// Mark the course as failed (e.g., player quit)
    pub fn mark_failed(&mut self) {
        self.failed = true;
    }

    /// Check if the course requirements are met
    pub fn check_requirements(&self) -> CoursePassResult {
        if self.failed {
            return CoursePassResult::Failed;
        }

        if !self.is_completed() {
            return CoursePassResult::Incomplete;
        }

        let req = &self.course.requirements;

        // Check gauge requirement
        if self.gauge_hp < req.min_gauge {
            return CoursePassResult::Failed;
        }

        // Check BAD + POOR limit
        if let Some(max_bp) = req.max_bad_poor {
            let bp = self.total_stats.bad_count + self.total_stats.poor_count;
            if bp > max_bp {
                return CoursePassResult::Failed;
            }
        }

        // Check full combo requirement
        if req.full_combo && (self.total_stats.bad_count > 0 || self.total_stats.poor_count > 0) {
            return CoursePassResult::Failed;
        }

        CoursePassResult::Passed
    }

    /// Get the requirements for display
    pub fn requirements(&self) -> &DanRequirements {
        &self.course.requirements
    }
}

/// Result of checking course pass requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoursePassResult {
    /// Course passed all requirements
    Passed,
    /// Course failed (gauge depleted or requirements not met)
    Failed,
    /// Course not yet completed
    Incomplete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dan::DanGrade;
    use crate::game::{GaugeType, RandomOption};
    use crate::ir::PlayOptionFlags;

    fn make_test_course() -> DanCourse {
        DanCourse {
            name: "Test Course".to_string(),
            grade: DanGrade::Dan(1),
            charts: vec!["a.bms".to_string(), "b.bms".to_string()],
            gauge_type: GaugeType::Hard,
            requirements: DanRequirements::default(),
        }
    }

    fn make_test_result() -> PlayResult {
        PlayResult {
            chart_path: "test.bms".to_string(),
            title: "Test".to_string(),
            artist: "Artist".to_string(),
            ex_score: 1000,
            max_combo: 100,
            pgreat_count: 400,
            great_count: 100,
            good_count: 10,
            bad_count: 5,
            poor_count: 3,
            total_notes: 500,
            clear_lamp: ClearLamp::Hard,
            random_option: RandomOption::Off,
            fast_count: 30,
            slow_count: 20,
            play_options: PlayOptionFlags {
                random_option: RandomOption::Off,
                gauge_type: GaugeType::Hard,
                auto_scratch: false,
                legacy_note: false,
                expand_judge: false,
                battle: false,
            },
        }
    }

    #[test]
    fn test_course_state_progression() {
        let course = make_test_course();
        let mut state = CourseState::new(course);

        assert_eq!(state.current_stage(), 0);
        assert_eq!(state.total_stages(), 2);
        assert!(!state.is_completed());
        assert!(!state.is_failed());

        // Complete first stage
        let result = make_test_result();
        let should_continue = state.complete_stage(&result, 50.0);
        assert!(should_continue);
        assert_eq!(state.current_stage(), 1);
        assert_eq!(state.gauge_hp(), 50.0);

        // Complete second stage
        let should_continue = state.complete_stage(&result, 30.0);
        assert!(!should_continue);
        assert!(state.is_completed());
        assert_eq!(state.check_requirements(), CoursePassResult::Passed);
    }

    #[test]
    fn test_course_failure() {
        let course = make_test_course();
        let mut state = CourseState::new(course);

        // Fail first stage (gauge depleted)
        let result = make_test_result();
        let should_continue = state.complete_stage(&result, 0.0);
        assert!(!should_continue);
        assert!(state.is_failed());
        assert_eq!(state.check_requirements(), CoursePassResult::Failed);
    }

    #[test]
    fn test_stats_accumulation() {
        let course = make_test_course();
        let mut state = CourseState::new(course);

        let result = make_test_result();
        state.complete_stage(&result, 50.0);
        state.complete_stage(&result, 30.0);

        let stats = state.total_stats();
        assert_eq!(stats.ex_score, 2000);
        assert_eq!(stats.pgreat_count, 800);
        assert_eq!(stats.total_notes, 1000);
    }
}
