use serde::{Deserialize, Serialize};

/// Judge algorithm for selecting between two candidate notes.
/// Corresponds to JudgeAlgorithm enum in beatoraja.
///
/// `compare` returns true if note 2 should be preferred over note 1.
/// Parameters:
/// - `t1_time`: microsecond time of note 1
/// - `t2_time`: microsecond time of note 2
/// - `t2_unjudged`: whether note 2 is unjudged (state == 0)
/// - `ptime`: press time in microseconds
/// - `judge_table`: judge windows &[[i64; 2]], indexed [PG, GR, GD, BD, MS]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgeAlgorithm {
    /// Combo-priority: prefer note 2 if it falls within GD window and note 1 is past GD late.
    Combo,
    /// Duration-priority: prefer the note closer in time to the press.
    Duration,
    /// Lowest-priority: always keep note 1 (first/earliest note).
    Lowest,
    /// Score-priority: prefer note 2 if it falls within GR window and note 1 is past GR late.
    Score,
}

impl JudgeAlgorithm {
    /// Default algorithm set used by beatoraja.
    pub const DEFAULT: [Self; 3] = [Self::Combo, Self::Duration, Self::Lowest];

    /// Compare two notes and return true if note 2 should be preferred.
    ///
    /// - `t1_time`: time of note 1 in microseconds.
    /// - `t2_time`: time of note 2 in microseconds.
    /// - `t2_unjudged`: whether note 2 has not been judged yet.
    /// - `ptime`: press time in microseconds.
    /// - `judge_table`: scaled judge windows, e.g. from `JudgeWindowRule::create`.
    pub fn compare(
        self,
        t1_time: i64,
        t2_time: i64,
        t2_unjudged: bool,
        ptime: i64,
        judge_table: &[[i64; 2]],
    ) -> bool {
        match self {
            Self::Combo => {
                // Prefer note 2 if: note 2 is unjudged, note 1 is past GD late boundary,
                // and note 2 is within GD early boundary.
                t2_unjudged
                    && t1_time < ptime + judge_table[2][0]
                    && t2_time <= ptime + judge_table[2][1]
            }
            Self::Duration => {
                // Prefer the note closer in absolute time difference.
                (t1_time - ptime).abs() > (t2_time - ptime).abs() && t2_unjudged
            }
            Self::Lowest => {
                // Always keep note 1 (the earlier note).
                false
            }
            Self::Score => {
                // Prefer note 2 if: note 2 is unjudged, note 1 is past GR late boundary,
                // and note 2 is within GR early boundary.
                t2_unjudged
                    && t1_time < ptime + judge_table[1][0]
                    && t2_time <= ptime + judge_table[1][1]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Typical SEVENKEYS note judge table at judgerank=100, rate=[100,100,100].
    /// [PG, GR, GD, BD, MS] = [[-20000,20000], [-60000,60000], [-150000,150000], [-280000,220000], [-150000,500000]]
    const SEVENKEYS_TABLE: [[i64; 2]; 5] = [
        [-20000, 20000],
        [-60000, 60000],
        [-150000, 150000],
        [-280000, 220000],
        [-150000, 500000],
    ];

    // =========================================================================
    // Combo algorithm tests
    // =========================================================================

    #[test]
    fn combo_prefers_note2_when_note1_past_gd_late() {
        // Note 1 at time 0, ptime = 200000 (note 1 is 200ms late).
        // GD late boundary: ptime + table[2][0] = 200000 + (-150000) = 50000.
        // t1_time (0) < 50000 => true.
        // Note 2 at time 300000, GD early boundary: ptime + table[2][1] = 200000 + 150000 = 350000.
        // t2_time (300000) <= 350000 => true.
        assert!(JudgeAlgorithm::Combo.compare(0, 300000, true, 200000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn combo_keeps_note1_when_within_gd_late() {
        // Note 1 at time 100000, ptime = 200000.
        // GD late boundary: 200000 + (-150000) = 50000.
        // t1_time (100000) < 50000 => false.
        assert!(!JudgeAlgorithm::Combo.compare(100000, 300000, true, 200000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn combo_keeps_note1_when_note2_judged() {
        assert!(!JudgeAlgorithm::Combo.compare(0, 300000, false, 200000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn combo_keeps_note1_when_note2_past_gd_early() {
        // Note 2 at time 400000, GD early boundary: 200000 + 150000 = 350000.
        // t2_time (400000) <= 350000 => false.
        assert!(!JudgeAlgorithm::Combo.compare(0, 400000, true, 200000, &SEVENKEYS_TABLE));
    }

    // =========================================================================
    // Duration algorithm tests
    // =========================================================================

    #[test]
    fn duration_prefers_closer_note() {
        // Note 1 at 0, note 2 at 180000, ptime = 100000.
        // |0 - 100000| = 100000, |180000 - 100000| = 80000.
        // 100000 > 80000 => true (prefer note 2).
        assert!(JudgeAlgorithm::Duration.compare(0, 180000, true, 100000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn duration_keeps_note1_when_closer() {
        // Note 1 at 90000, note 2 at 200000, ptime = 100000.
        // |90000 - 100000| = 10000, |200000 - 100000| = 100000.
        // 10000 > 100000 => false.
        assert!(!JudgeAlgorithm::Duration.compare(90000, 200000, true, 100000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn duration_keeps_note1_when_note2_judged() {
        assert!(!JudgeAlgorithm::Duration.compare(0, 180000, false, 100000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn duration_keeps_note1_on_tie() {
        // Equal distance: keep note 1.
        // Note 1 at 90000, note 2 at 110000, ptime = 100000.
        // |90000 - 100000| = 10000, |110000 - 100000| = 10000.
        // 10000 > 10000 => false.
        assert!(!JudgeAlgorithm::Duration.compare(90000, 110000, true, 100000, &SEVENKEYS_TABLE));
    }

    // =========================================================================
    // Lowest algorithm tests
    // =========================================================================

    #[test]
    fn lowest_always_keeps_note1() {
        assert!(!JudgeAlgorithm::Lowest.compare(0, 100000, true, 50000, &SEVENKEYS_TABLE));
        assert!(!JudgeAlgorithm::Lowest.compare(1000000, 50000, true, 50000, &SEVENKEYS_TABLE));
    }

    // =========================================================================
    // Score algorithm tests
    // =========================================================================

    #[test]
    fn score_prefers_note2_when_note1_past_gr_late() {
        // Note 1 at 0, ptime = 200000.
        // GR late boundary: ptime + table[1][0] = 200000 + (-60000) = 140000.
        // t1_time (0) < 140000 => true.
        // Note 2 at 250000, GR early boundary: 200000 + 60000 = 260000.
        // t2_time (250000) <= 260000 => true.
        assert!(JudgeAlgorithm::Score.compare(0, 250000, true, 200000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn score_keeps_note1_when_within_gr_late() {
        // Note 1 at 150000, ptime = 200000.
        // GR late boundary: 200000 + (-60000) = 140000.
        // t1_time (150000) < 140000 => false.
        assert!(!JudgeAlgorithm::Score.compare(150000, 250000, true, 200000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn score_keeps_note1_when_note2_judged() {
        assert!(!JudgeAlgorithm::Score.compare(0, 250000, false, 200000, &SEVENKEYS_TABLE));
    }

    #[test]
    fn score_keeps_note1_when_note2_past_gr_early() {
        // Note 2 at 280000, GR early boundary: 200000 + 60000 = 260000.
        // t2_time (280000) <= 260000 => false.
        assert!(!JudgeAlgorithm::Score.compare(0, 280000, true, 200000, &SEVENKEYS_TABLE));
    }

    // =========================================================================
    // Default algorithm set test
    // =========================================================================

    #[test]
    fn default_algorithm_set() {
        assert_eq!(
            JudgeAlgorithm::DEFAULT,
            [
                JudgeAlgorithm::Combo,
                JudgeAlgorithm::Duration,
                JudgeAlgorithm::Lowest
            ]
        );
    }
}
