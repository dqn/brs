/// Judge display skin object
///
/// Moved from rubato-play::skin::judge to break the rubato-skin -> rubato-play dependency.
pub struct SkinJudge {
    /// Judge images present (7 types: PG, GR, GD, BD, PR, MS, PG+MAX)
    judge: [bool; 7],
    /// Judge count numbers present (7 types)
    count: [bool; 7],
    /// Player index
    player: i32,
    /// Whether to shift position based on count length
    shift: bool,
    /// Currently active judge
    _now_judge: Option<usize>,
    /// Currently active count
    _now_count: Option<usize>,
}

impl SkinJudge {
    pub fn new(player: i32, shift: bool) -> Self {
        SkinJudge {
            judge: [false; 7],
            count: [false; 7],
            player,
            shift,
            _now_judge: None,
            _now_count: None,
        }
    }

    pub fn judge(&self, index: usize) -> bool {
        index < self.judge.len() && self.judge[index]
    }

    pub fn set_judge(&mut self, index: usize) {
        if index < self.judge.len() {
            self.judge[index] = true;
        }
    }

    pub fn judge_count(&self, index: usize) -> bool {
        index < self.count.len() && self.count[index]
    }

    pub fn set_judge_count(&mut self, index: usize) {
        if index < self.count.len() {
            self.count[index] = true;
        }
    }

    pub fn player(&self) -> i32 {
        self.player
    }

    pub fn is_shift(&self) -> bool {
        self.shift
    }

    pub fn prepare(&mut self, _time: i64) {
        // Prepare logic is handled by SkinJudgeObject in rubato-skin.
    }

    pub fn draw(&self) {
        // Drawing is handled by SkinJudgeObject in rubato-skin.
    }

    pub fn dispose(&mut self) {
        // no GPU resources in Rust translation
    }
}
