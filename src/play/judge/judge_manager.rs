use serde::{Deserialize, Serialize};

use super::judge_algorithm::JudgeAlgorithm;
use super::judge_property::{JudgeNoteType, JudgeProperty, MissCondition};

/// Judge result levels.
/// PG=0, GR=1, GD=2, BD=3, PR(poor)=4, MS(miss)=5.
/// EmptyPoor(6) is a press with no valid target note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum JudgeLevel {
    PerfectGreat = 0,
    Great = 1,
    Good = 2,
    Bad = 3,
    Poor = 4,
    Miss = 5,
    EmptyPoor = 6,
}

impl JudgeLevel {
    /// Convert from integer index (beatoraja convention).
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::PerfectGreat),
            1 => Some(Self::Great),
            2 => Some(Self::Good),
            3 => Some(Self::Bad),
            4 => Some(Self::Poor),
            5 => Some(Self::Miss),
            6 => Some(Self::EmptyPoor),
            _ => None,
        }
    }

    /// Whether this judge level continues combo.
    pub fn continues_combo(self, combo_cond: &[bool]) -> bool {
        let idx = self as usize;
        idx < combo_cond.len() && combo_cond[idx]
    }

    /// Whether this judge level causes the note to vanish.
    pub fn causes_vanish(self, judge_vanish: &[bool]) -> bool {
        let idx = self as usize;
        idx < judge_vanish.len() && judge_vanish[idx]
    }
}

/// Result of judging a single note press.
#[derive(Debug, Clone, PartialEq)]
pub struct JudgeResult {
    /// The judge level.
    pub level: JudgeLevel,
    /// Time difference in microseconds (note_time - press_time).
    /// Positive = early press, negative = late press.
    pub time_diff_us: i64,
    /// Whether the note should be consumed (vanish).
    pub vanish: bool,
}

/// Determine the judge level for a time difference against a judge table.
/// Returns the judge index (0=PG, 1=GR, 2=GD, 3=BD), or None if outside all windows.
///
/// `dmtime` = note_time - press_time (positive = early, negative = late).
/// `judge_table` = scaled judge windows from JudgeWindowRule::create.
pub fn determine_judge_index(dmtime: i64, judge_table: &[[i64; 2]]) -> Option<usize> {
    for (i, window) in judge_table.iter().enumerate() {
        if dmtime >= window[0] && dmtime <= window[1] {
            return Some(i);
        }
    }
    None
}

/// Determine the full judge level for a note press.
/// Follows beatoraja's logic:
/// - If unjudged: find matching window -> index 0..3 = PG/GR/GD/BD, index 4 = PR (skip to +1), else EmptyPoor
/// - If already judged: check MS window -> EmptyPoor(5), else beyond(6)
///
/// Returns (judge_index, is_empty_poor).
/// judge_index: 0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS(empty poor), 6=beyond
pub fn determine_note_judge(dmtime: i64, is_unjudged: bool, judge_table: &[[i64; 2]]) -> usize {
    if !is_unjudged {
        // Already judged note: check MS window (index 4)
        if judge_table.len() > 4 && dmtime >= judge_table[4][0] && dmtime <= judge_table[4][1] {
            5 // EmptyPoor (MS window hit)
        } else {
            6 // Beyond all windows
        }
    } else {
        // Find matching window
        let mut judge = judge_table.len();
        for (i, window) in judge_table.iter().enumerate() {
            if dmtime >= window[0] && dmtime <= window[1] {
                judge = i;
                break;
            }
        }
        // Index 4+ maps to index+1 (skipping PR to MS convention)
        if judge >= 4 { judge + 1 } else { judge }
    }
}

/// HCN gauge increase/decrease interval in microseconds.
pub const HCN_DURATION_US: i64 = 200_000;

/// Per-lane judge state tracking.
#[derive(Debug, Clone, Default)]
pub struct LaneJudgeState {
    /// Currently processing long note end (if in the middle of a LN/CN/HCN).
    pub processing: bool,
    /// LN start judge level (for LN end comparison).
    pub ln_start_judge: usize,
    /// LN start time difference.
    pub ln_start_duration: i64,
    /// HCN passing state (currently passing through an HCN).
    pub passing: bool,
    /// HCN accumulator for gauge tick.
    pub passing_count: i64,
    /// Whether HCN is being held (increasing gauge).
    pub increasing: bool,
    /// Release time for delayed LN end judgment.
    pub release_time: Option<i64>,
    /// LN end judge level for delayed judgment.
    pub ln_end_judge: Option<usize>,
}

/// Scoring state tracked by the judge manager.
#[derive(Debug, Clone, Default)]
pub struct JudgeScore {
    /// Judge counts [PG, GR, GD, BD, PR, MS] for early and late.
    pub early_counts: [u32; 6],
    pub late_counts: [u32; 6],
    /// Current combo.
    pub combo: u32,
    /// Maximum combo achieved.
    pub max_combo: u32,
    /// Number of notes that have been judged (passed).
    pub pass_notes: u32,
    /// Ghost data: judge level per note, in order.
    pub ghost: Vec<u8>,
    /// Recent judge timings (circular buffer, microseconds).
    pub recent_judges_us: Vec<i64>,
    /// Index into recent_judges_us circular buffer.
    pub recent_index: usize,
}

impl JudgeScore {
    pub fn new(total_notes: usize) -> Self {
        Self {
            ghost: vec![4; total_notes], // 4 = PR default
            recent_judges_us: vec![i64::MIN; 100],
            ..Default::default()
        }
    }

    /// Add a judge count.
    pub fn add_judge(&mut self, judge: usize, is_early: bool) {
        if judge < 6 {
            if is_early {
                self.early_counts[judge] += 1;
            } else {
                self.late_counts[judge] += 1;
            }
        }
    }

    /// Get total judge count for a level.
    pub fn judge_count(&self, judge: usize) -> u32 {
        if judge < 6 {
            self.early_counts[judge] + self.late_counts[judge]
        } else {
            0
        }
    }

    /// Record a judge timing in the recent buffer.
    pub fn record_recent_timing(&mut self, timing_us: i64) {
        self.recent_index = (self.recent_index + 1) % self.recent_judges_us.len();
        self.recent_judges_us[self.recent_index] = timing_us;
    }

    /// Update combo based on judge level and combo condition.
    pub fn update_combo(&mut self, judge: usize, combo_cond: &[bool]) {
        if judge < combo_cond.len() && combo_cond[judge] && judge < 5 {
            self.combo += 1;
            self.max_combo = self.max_combo.max(self.combo);
        }
        if judge < combo_cond.len() && !combo_cond[judge] {
            self.combo = 0;
        }
    }

    /// Record ghost data and increment pass_notes.
    pub fn record_ghost(&mut self, judge: usize) {
        let idx = self.pass_notes as usize;
        if idx < self.ghost.len() {
            self.ghost[idx] = judge as u8;
        }
        self.pass_notes += 1;
    }
}

/// Core judge update logic.
/// Processes a single note judgment and updates scoring state.
///
/// Corresponds to updateMicro() in beatoraja's JudgeManager.
pub fn update_judge(
    score: &mut JudgeScore,
    judge: usize,
    time_diff_us: i64,
    vanish: bool,
    miss_condition: MissCondition,
    combo_cond: &[bool],
    play_time: i64,
) {
    if vanish {
        score.record_ghost(judge);
    }

    // MissCondition::One: skip duplicate miss for notes that have been pressed
    if miss_condition == MissCondition::One && judge == 4 && play_time != 0 {
        return;
    }

    let is_early = time_diff_us >= 0;
    score.add_judge(judge, is_early);

    // Record timing for non-BD+ judges
    if judge < 4 {
        score.record_recent_timing(time_diff_us);
    }

    score.update_combo(judge, combo_cond);
}

/// Select the best note from candidates using a judge algorithm.
///
/// Given a list of candidate notes (time, is_unjudged, play_time), the press time,
/// judge table, algorithm, and miss condition, returns the index of the selected note
/// and its judge level, or None if no valid note.
///
/// Follows beatoraja's note selection loop in JudgeManager.update().
pub fn select_note(
    candidates: &[(i64, bool, i64)], // (note_time_us, is_unjudged, play_time_us)
    press_time: i64,
    judge_table: &[[i64; 2]],
    judge_start: i64, // minimum window start
    judge_end: i64,   // maximum window end
    algorithm: JudgeAlgorithm,
    miss_condition: MissCondition,
) -> Option<(usize, usize)> {
    // (index, judge_level)
    let mut best: Option<(usize, usize, i64)> = None; // (index, judge, note_time)

    for (idx, &(note_time, is_unjudged, play_time)) in candidates.iter().enumerate() {
        let dmtime = note_time - press_time;

        if dmtime >= judge_end {
            break;
        }
        if dmtime < judge_start {
            continue;
        }

        // Check if we should prefer this note over current best
        let should_select = match &best {
            None => true,
            Some((_, _, best_time)) => {
                let best_unjudged = candidates
                    .get(best.as_ref().unwrap().0)
                    .is_some_and(|c| c.1);
                !best_unjudged
                    || algorithm.compare(
                        *best_time,
                        note_time,
                        is_unjudged,
                        press_time,
                        judge_table,
                    )
            }
        };

        if !should_select {
            continue;
        }

        // MissCondition::One check
        if miss_condition == MissCondition::One
            && (!is_unjudged
                || (is_unjudged
                    && play_time != 0
                    && (dmtime > judge_table[2][1] || dmtime < judge_table[2][0])))
        {
            continue;
        }

        let judge = determine_note_judge(dmtime, is_unjudged, judge_table);

        if judge < 6 {
            // Valid judge: prefer closer note for BD+ (judge >= 4)
            let should_keep = judge < 4
                || best.is_none()
                || best.as_ref().is_none_or(|(bi, _, _)| {
                    let best_time = candidates[*bi].0;
                    (best_time - press_time).abs() > (note_time - press_time).abs()
                });
            if should_keep {
                best = Some((idx, judge, note_time));
            }
        } else {
            // Beyond all windows: clear best
            best = None;
        }
    }

    best.map(|(idx, judge, _)| (idx, judge))
}

/// Get judge tables for a given judge property.
/// Returns (note_table, ln_end_table, scratch_table, scratch_ln_end_table).
pub fn build_judge_tables(
    property: &JudgeProperty,
    judgerank: i32,
    key_rate: &[i32; 3],
    scratch_rate: &[i32; 3],
) -> JudgeTables {
    let note = property.get_judge(JudgeNoteType::Note, judgerank, key_rate);
    let ln_end = property.get_judge(JudgeNoteType::LongNoteEnd, judgerank, key_rate);
    let scratch = property.get_judge(JudgeNoteType::Scratch, judgerank, scratch_rate);
    let scratch_ln_end = property.get_judge(JudgeNoteType::LongScratchEnd, judgerank, scratch_rate);

    // Compute overall judge start/end from note and scratch tables
    let mut judge_start = 0i64;
    let mut judge_end = 0i64;
    for window in note.iter().chain(scratch.iter()) {
        judge_start = judge_start.min(window[0]);
        judge_end = judge_end.max(window[1]);
    }

    JudgeTables {
        note,
        ln_end,
        scratch,
        scratch_ln_end,
        note_release_margin: property.longnote_margin,
        scratch_release_margin: property.longscratch_margin,
        judge_start,
        judge_end,
    }
}

/// Pre-computed judge tables for a play session.
#[derive(Debug, Clone)]
pub struct JudgeTables {
    /// Normal note judge windows.
    pub note: Vec<[i64; 2]>,
    /// Long note end judge windows.
    pub ln_end: Vec<[i64; 2]>,
    /// Scratch note judge windows.
    pub scratch: Vec<[i64; 2]>,
    /// Scratch long note end judge windows.
    pub scratch_ln_end: Vec<[i64; 2]>,
    /// LN release margin in microseconds.
    pub note_release_margin: i64,
    /// Scratch LN release margin in microseconds.
    pub scratch_release_margin: i64,
    /// Overall minimum window start (for early-exit optimization).
    pub judge_start: i64,
    /// Overall maximum window end (for early-exit optimization).
    pub judge_end: i64,
}

impl JudgeTables {
    /// Get the appropriate judge table for a lane.
    pub fn table_for_lane(&self, is_scratch: bool) -> &[[i64; 2]] {
        if is_scratch {
            &self.scratch
        } else {
            &self.note
        }
    }

    /// Get the appropriate LN end judge table for a lane.
    pub fn ln_end_table_for_lane(&self, is_scratch: bool) -> &[[i64; 2]] {
        if is_scratch {
            &self.scratch_ln_end
        } else {
            &self.ln_end
        }
    }

    /// Get the release margin for a lane.
    pub fn release_margin_for_lane(&self, is_scratch: bool) -> i64 {
        if is_scratch {
            self.scratch_release_margin
        } else {
            self.note_release_margin
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // JudgeLevel tests
    // =========================================================================

    #[test]
    fn judge_level_from_index() {
        assert_eq!(JudgeLevel::from_index(0), Some(JudgeLevel::PerfectGreat));
        assert_eq!(JudgeLevel::from_index(3), Some(JudgeLevel::Bad));
        assert_eq!(JudgeLevel::from_index(5), Some(JudgeLevel::Miss));
        assert_eq!(JudgeLevel::from_index(6), Some(JudgeLevel::EmptyPoor));
        assert_eq!(JudgeLevel::from_index(7), None);
    }

    #[test]
    fn judge_level_continues_combo() {
        // SEVENKEYS combo: [true, true, true, false, false, true]
        let combo_cond = &[true, true, true, false, false, true];
        assert!(JudgeLevel::PerfectGreat.continues_combo(combo_cond));
        assert!(JudgeLevel::Great.continues_combo(combo_cond));
        assert!(JudgeLevel::Good.continues_combo(combo_cond));
        assert!(!JudgeLevel::Bad.continues_combo(combo_cond));
        assert!(!JudgeLevel::Poor.continues_combo(combo_cond));
        assert!(JudgeLevel::Miss.continues_combo(combo_cond));
    }

    // =========================================================================
    // determine_judge_index tests
    // =========================================================================

    const SEVENKEYS_TABLE: [[i64; 2]; 5] = [
        [-20000, 20000],
        [-60000, 60000],
        [-150000, 150000],
        [-280000, 220000],
        [-150000, 500000],
    ];

    #[test]
    fn determine_judge_index_pg() {
        assert_eq!(determine_judge_index(0, &SEVENKEYS_TABLE), Some(0)); // exact
        assert_eq!(determine_judge_index(20000, &SEVENKEYS_TABLE), Some(0)); // boundary
        assert_eq!(determine_judge_index(-20000, &SEVENKEYS_TABLE), Some(0)); // boundary
    }

    #[test]
    fn determine_judge_index_gr() {
        assert_eq!(determine_judge_index(30000, &SEVENKEYS_TABLE), Some(1));
        assert_eq!(determine_judge_index(-30000, &SEVENKEYS_TABLE), Some(1));
    }

    #[test]
    fn determine_judge_index_gd() {
        assert_eq!(determine_judge_index(100000, &SEVENKEYS_TABLE), Some(2));
    }

    #[test]
    fn determine_judge_index_bd() {
        assert_eq!(determine_judge_index(200000, &SEVENKEYS_TABLE), Some(3));
        assert_eq!(determine_judge_index(-200000, &SEVENKEYS_TABLE), Some(3));
    }

    #[test]
    fn determine_judge_index_ms() {
        // MS window: [-150000, 500000]
        // Note: 300000 is in BD (220000 boundary) so it's BD, but 400000 is outside BD early
        // and inside MS. Let's check: BD is [-280000, 220000], so 300000 > 220000 -> not BD.
        // MS: [-150000, 500000], 300000 is within -> MS.
        assert_eq!(determine_judge_index(300000, &SEVENKEYS_TABLE), Some(4));
    }

    #[test]
    fn determine_judge_index_outside() {
        assert_eq!(determine_judge_index(600000, &SEVENKEYS_TABLE), None);
        assert_eq!(determine_judge_index(-300000, &SEVENKEYS_TABLE), None);
    }

    // =========================================================================
    // determine_note_judge tests
    // =========================================================================

    #[test]
    fn note_judge_unjudged_pg() {
        assert_eq!(determine_note_judge(0, true, &SEVENKEYS_TABLE), 0);
    }

    #[test]
    fn note_judge_unjudged_gr() {
        assert_eq!(determine_note_judge(30000, true, &SEVENKEYS_TABLE), 1);
    }

    #[test]
    fn note_judge_unjudged_ms_window() {
        // Falls in MS window (index 4) -> returns 4+1 = 5
        assert_eq!(determine_note_judge(300000, true, &SEVENKEYS_TABLE), 5);
    }

    #[test]
    fn note_judge_unjudged_outside() {
        // Outside all windows -> returns len+1 = 6
        assert_eq!(determine_note_judge(600000, true, &SEVENKEYS_TABLE), 6);
    }

    #[test]
    fn note_judge_already_judged_in_ms_window() {
        // Already judged, in MS window -> 5 (empty poor)
        assert_eq!(determine_note_judge(300000, false, &SEVENKEYS_TABLE), 5);
    }

    #[test]
    fn note_judge_already_judged_outside() {
        assert_eq!(determine_note_judge(600000, false, &SEVENKEYS_TABLE), 6);
    }

    // =========================================================================
    // JudgeScore tests
    // =========================================================================

    #[test]
    fn judge_score_new() {
        let score = JudgeScore::new(100);
        assert_eq!(score.ghost.len(), 100);
        assert!(score.ghost.iter().all(|&g| g == 4));
        assert_eq!(score.recent_judges_us.len(), 100);
    }

    #[test]
    fn judge_score_add_and_count() {
        let mut score = JudgeScore::new(10);
        score.add_judge(0, true); // PG early
        score.add_judge(0, false); // PG late
        score.add_judge(1, true); // GR early
        assert_eq!(score.judge_count(0), 2);
        assert_eq!(score.judge_count(1), 1);
        assert_eq!(score.judge_count(2), 0);
        assert_eq!(score.early_counts[0], 1);
        assert_eq!(score.late_counts[0], 1);
    }

    #[test]
    fn judge_score_combo() {
        let combo_cond = &[true, true, true, false, false, true];
        let mut score = JudgeScore::new(10);

        score.update_combo(0, combo_cond); // PG
        assert_eq!(score.combo, 1);
        score.update_combo(1, combo_cond); // GR
        assert_eq!(score.combo, 2);
        score.update_combo(3, combo_cond); // BD -> break
        assert_eq!(score.combo, 0);
        assert_eq!(score.max_combo, 2);
        score.update_combo(0, combo_cond); // PG
        assert_eq!(score.combo, 1);
    }

    #[test]
    fn judge_score_ghost() {
        let mut score = JudgeScore::new(3);
        score.record_ghost(0); // PG
        score.record_ghost(1); // GR
        score.record_ghost(3); // BD
        assert_eq!(score.ghost, vec![0, 1, 3]);
        assert_eq!(score.pass_notes, 3);
    }

    // =========================================================================
    // update_judge tests
    // =========================================================================

    #[test]
    fn update_judge_basic() {
        let mut score = JudgeScore::new(5);
        let combo_cond = &[true, true, true, false, false, true];

        update_judge(
            &mut score,
            0,
            5000,
            true,
            MissCondition::Always,
            combo_cond,
            0,
        );
        assert_eq!(score.judge_count(0), 1);
        assert_eq!(score.early_counts[0], 1);
        assert_eq!(score.combo, 1);
        assert_eq!(score.pass_notes, 1);
    }

    #[test]
    fn update_judge_miss_condition_one_skips_duplicate() {
        let mut score = JudgeScore::new(5);
        let combo_cond = &[true, true, true, false, false, false];

        // First PR with play_time != 0 under MissCondition::One should be skipped
        update_judge(
            &mut score,
            4,
            -200000,
            true,
            MissCondition::One,
            combo_cond,
            12345,
        );
        assert_eq!(score.judge_count(4), 0); // skipped
        assert_eq!(score.pass_notes, 1); // ghost still recorded

        // First PR with play_time == 0 should count
        update_judge(
            &mut score,
            4,
            -200000,
            true,
            MissCondition::One,
            combo_cond,
            0,
        );
        assert_eq!(score.judge_count(4), 1);
    }

    #[test]
    fn update_judge_miss_condition_always_counts_all() {
        let mut score = JudgeScore::new(5);
        let combo_cond = &[true, true, true, false, false, false];

        update_judge(
            &mut score,
            4,
            -200000,
            true,
            MissCondition::Always,
            combo_cond,
            12345,
        );
        assert_eq!(score.judge_count(4), 1); // counted even with play_time != 0
    }

    // =========================================================================
    // select_note tests
    // =========================================================================

    #[test]
    fn select_note_single_unjudged() {
        let candidates = vec![(100000i64, true, 0i64)];
        let result = select_note(
            &candidates,
            100000,
            &SEVENKEYS_TABLE,
            -280000,
            500000,
            JudgeAlgorithm::Combo,
            MissCondition::Always,
        );
        assert_eq!(result, Some((0, 0))); // PG (dmtime = 0)
    }

    #[test]
    fn select_note_picks_closer_with_duration() {
        let candidates = vec![
            (50000i64, true, 0i64),  // 50ms behind press
            (110000i64, true, 0i64), // 10ms ahead of press
        ];
        let result = select_note(
            &candidates,
            100000,
            &SEVENKEYS_TABLE,
            -280000,
            500000,
            JudgeAlgorithm::Duration,
            MissCondition::Always,
        );
        assert_eq!(result, Some((1, 0))); // Note 2 is closer -> PG
    }

    #[test]
    fn select_note_lowest_keeps_first() {
        let candidates = vec![(80000i64, true, 0i64), (100000i64, true, 0i64)];
        let result = select_note(
            &candidates,
            100000,
            &SEVENKEYS_TABLE,
            -280000,
            500000,
            JudgeAlgorithm::Lowest,
            MissCondition::Always,
        );
        assert_eq!(result, Some((0, 0))); // Always first note
    }

    #[test]
    fn select_note_empty_candidates() {
        let candidates: Vec<(i64, bool, i64)> = vec![];
        let result = select_note(
            &candidates,
            100000,
            &SEVENKEYS_TABLE,
            -280000,
            500000,
            JudgeAlgorithm::Combo,
            MissCondition::Always,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn select_note_outside_window() {
        let candidates = vec![(900000i64, true, 0i64)]; // Very far from press
        let result = select_note(
            &candidates,
            100000,
            &SEVENKEYS_TABLE,
            -280000,
            500000,
            JudgeAlgorithm::Combo,
            MissCondition::Always,
        );
        // dmtime = 900000 - 100000 = 800000 >= judge_end (500000), so break
        assert_eq!(result, None);
    }

    // =========================================================================
    // build_judge_tables tests
    // =========================================================================

    #[test]
    fn build_judge_tables_sevenkeys() {
        let prop = &JudgeProperty::SEVENKEYS;
        let rate = [100, 100, 100];
        let tables = build_judge_tables(prop, 100, &rate, &rate);

        assert_eq!(tables.note.len(), 5);
        assert_eq!(tables.ln_end.len(), 4);
        assert_eq!(tables.scratch.len(), 5);
        assert_eq!(tables.scratch_ln_end.len(), 4);
        assert_eq!(tables.note_release_margin, 0);
        assert_eq!(tables.scratch_release_margin, 0);

        // Check judge_start/judge_end
        assert!(tables.judge_start <= -280000);
        assert!(tables.judge_end >= 500000);
    }

    #[test]
    fn build_judge_tables_pms() {
        let prop = &JudgeProperty::PMS;
        let rate = [100, 100, 100];
        let tables = build_judge_tables(prop, 100, &rate, &rate);

        assert_eq!(tables.note_release_margin, 200000);
        assert!(tables.scratch.is_empty());
    }

    #[test]
    fn judge_tables_lane_selection() {
        let prop = &JudgeProperty::SEVENKEYS;
        let rate = [100, 100, 100];
        let tables = build_judge_tables(prop, 100, &rate, &rate);

        // Normal lane
        assert_eq!(tables.table_for_lane(false), tables.note.as_slice());
        assert_eq!(
            tables.ln_end_table_for_lane(false),
            tables.ln_end.as_slice()
        );

        // Scratch lane
        assert_eq!(tables.table_for_lane(true), tables.scratch.as_slice());
        assert_eq!(
            tables.ln_end_table_for_lane(true),
            tables.scratch_ln_end.as_slice()
        );
    }

    // =========================================================================
    // HCN constant test
    // =========================================================================

    #[test]
    fn hcn_duration_matches_beatoraja() {
        assert_eq!(HCN_DURATION_US, 200_000); // 200ms
    }
}
