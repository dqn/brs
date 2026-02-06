/// Ghost data for comparing current play against a previous attempt.
/// Ghost stores the judge level per note as a byte array.
#[derive(Debug, Clone)]
pub struct Ghost {
    /// Judge level per note (0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS).
    data: Vec<u8>,
    /// Total number of notes in the chart.
    total_notes: usize,
}

impl Ghost {
    /// Create a new ghost from recorded data.
    pub fn new(data: Vec<u8>, total_notes: usize) -> Self {
        Self { data, total_notes }
    }

    /// Create an empty ghost with default values (PR=4).
    pub fn empty(total_notes: usize) -> Self {
        Self {
            data: vec![4; total_notes],
            total_notes,
        }
    }

    /// Get the judge level for a note index.
    pub fn judge_at(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    /// Get the ghost data slice.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Total number of notes.
    pub fn total_notes(&self) -> usize {
        self.total_notes
    }

    /// Calculate EX score difference between current play and ghost up to a note index.
    /// Returns (current_exscore - ghost_exscore).
    pub fn exscore_diff(&self, current_ghost: &[u8], up_to: usize) -> i32 {
        let mut current_ex = 0i32;
        let mut ghost_ex = 0i32;

        let limit = up_to.min(self.data.len()).min(current_ghost.len());
        for (current, ghost) in current_ghost.iter().zip(self.data.iter()).take(limit) {
            current_ex += judge_to_exscore(*current);
            ghost_ex += judge_to_exscore(*ghost);
        }

        current_ex - ghost_ex
    }
}

/// Convert a judge level to EX score contribution.
fn judge_to_exscore(judge: u8) -> i32 {
    match judge {
        0 => 2, // PG
        1 => 1, // GR
        _ => 0, // GD, BD, PR, MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_empty() {
        let ghost = Ghost::empty(10);
        assert_eq!(ghost.total_notes(), 10);
        for i in 0..10 {
            assert_eq!(ghost.judge_at(i), Some(4)); // PR default
        }
    }

    #[test]
    fn ghost_judge_at() {
        let ghost = Ghost::new(vec![0, 1, 2, 3, 4, 5], 6);
        assert_eq!(ghost.judge_at(0), Some(0)); // PG
        assert_eq!(ghost.judge_at(1), Some(1)); // GR
        assert_eq!(ghost.judge_at(5), Some(5)); // MS
        assert_eq!(ghost.judge_at(6), None);
    }

    #[test]
    fn exscore_diff_all_pg() {
        let ghost = Ghost::new(vec![0, 0, 0], 3);
        let current = vec![0, 0, 0];
        assert_eq!(ghost.exscore_diff(&current, 3), 0);
    }

    #[test]
    fn exscore_diff_current_better() {
        let ghost = Ghost::new(vec![1, 1, 1], 3); // All GR = 3 EX
        let current = vec![0, 0, 0]; // All PG = 6 EX
        assert_eq!(ghost.exscore_diff(&current, 3), 3); // 6 - 3 = 3
    }

    #[test]
    fn exscore_diff_ghost_better() {
        let ghost = Ghost::new(vec![0, 0, 0], 3); // All PG = 6 EX
        let current = vec![2, 2, 2]; // All GD = 0 EX
        assert_eq!(ghost.exscore_diff(&current, 3), -6); // 0 - 6 = -6
    }

    #[test]
    fn exscore_diff_partial() {
        let ghost = Ghost::new(vec![0, 1, 2, 3], 4);
        let current = vec![0, 0, 0, 0];
        // Ghost up to 2: PG(2) + GR(1) = 3
        // Current up to 2: PG(2) + PG(2) = 4
        assert_eq!(ghost.exscore_diff(&current, 2), 1);
    }

    #[test]
    fn judge_to_exscore_values() {
        assert_eq!(judge_to_exscore(0), 2);
        assert_eq!(judge_to_exscore(1), 1);
        assert_eq!(judge_to_exscore(2), 0);
        assert_eq!(judge_to_exscore(3), 0);
        assert_eq!(judge_to_exscore(4), 0);
        assert_eq!(judge_to_exscore(5), 0);
    }
}
