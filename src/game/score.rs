use super::JudgeResult;

#[derive(Debug, Clone, Default)]
pub struct ScoreManager {
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
    pub combo: u32,
    pub max_combo: u32,
}

impl ScoreManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_judgment(&mut self, result: JudgeResult) {
        match result {
            JudgeResult::PGreat => {
                self.pgreat_count += 1;
                self.combo += 1;
            }
            JudgeResult::Great => {
                self.great_count += 1;
                self.combo += 1;
            }
            JudgeResult::Good => {
                self.good_count += 1;
                self.combo += 1;
            }
            JudgeResult::Bad => {
                self.bad_count += 1;
                self.combo = 0;
            }
            JudgeResult::Poor => {
                self.poor_count += 1;
                self.combo = 0;
            }
        }

        self.max_combo = self.max_combo.max(self.combo);
    }

    /// Calculate total EX score from judgment counts.
    /// Uses the same scoring rules as `JudgeResult::ex_score()`:
    /// PGREAT = 2, GREAT = 1, others = 0.
    pub fn ex_score(&self) -> u32 {
        self.pgreat_count * 2 + self.great_count
    }

    // Public API for querying total note count
    #[allow(dead_code)]
    pub fn total_notes(&self) -> u32 {
        self.pgreat_count + self.great_count + self.good_count + self.bad_count + self.poor_count
    }

    // Public API for calculating accuracy percentage
    #[allow(dead_code)]
    pub fn accuracy(&self) -> f64 {
        let total = self.total_notes();
        if total == 0 {
            return 100.0;
        }
        let max_ex = total * 2;
        (self.ex_score() as f64 / max_ex as f64) * 100.0
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
