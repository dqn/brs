use bms_model::note::Note;

/// Judge algorithm
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgeAlgorithm {
    /// Combo priority
    Combo,
    /// Duration priority
    Duration,
    /// Lowest note priority
    Lowest,
    /// Score priority
    Score,
}

pub static DEFAULT_ALGORITHM: &[JudgeAlgorithm] = &[
    JudgeAlgorithm::Combo,
    JudgeAlgorithm::Duration,
    JudgeAlgorithm::Lowest,
];

impl JudgeAlgorithm {
    /// Compare two notes. Returns true if t2 is preferred over t1.
    pub fn compare(&self, t1: &Note, t2: &Note, ptime: i64, judgetable: &[Vec<i64>]) -> bool {
        match self {
            JudgeAlgorithm::Combo => {
                t2.get_state() == 0
                    && t1.get_micro_time() < ptime + judgetable[2][0]
                    && t2.get_micro_time() <= ptime + judgetable[2][1]
            }
            JudgeAlgorithm::Duration => {
                (t1.get_micro_time() - ptime).abs() > (t2.get_micro_time() - ptime).abs()
                    && t2.get_state() == 0
            }
            JudgeAlgorithm::Lowest => false,
            JudgeAlgorithm::Score => {
                t2.get_state() == 0
                    && t1.get_micro_time() < ptime + judgetable[1][0]
                    && t2.get_micro_time() <= ptime + judgetable[1][1]
            }
        }
    }

    pub fn values() -> &'static [JudgeAlgorithm] {
        &[
            JudgeAlgorithm::Combo,
            JudgeAlgorithm::Duration,
            JudgeAlgorithm::Lowest,
            JudgeAlgorithm::Score,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            JudgeAlgorithm::Combo => "Combo",
            JudgeAlgorithm::Duration => "Duration",
            JudgeAlgorithm::Lowest => "Lowest",
            JudgeAlgorithm::Score => "Score",
        }
    }

    pub fn get_index(algorithm: &str) -> i32 {
        for (i, v) in Self::values().iter().enumerate() {
            if v.name() == algorithm {
                return i as i32;
            }
        }
        -1
    }

    pub fn from_name(name: &str) -> Option<JudgeAlgorithm> {
        for v in Self::values() {
            if v.name() == name {
                return Some(*v);
            }
        }
        None
    }
}
