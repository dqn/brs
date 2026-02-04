use std::collections::{HashMap, HashSet};

use crate::model::note::{Lane, Note, NoteType};
use crate::model::{BMSModel, JudgeRankType, LongNoteMode};

/// Judge rank representing the accuracy of a note hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JudgeRank {
    PerfectGreat,
    Great,
    Good,
    Bad,
    Poor,
    Miss,
}

impl JudgeRank {
    /// Returns true if this rank continues combo.
    pub fn continues_combo(self) -> bool {
        matches!(self, Self::PerfectGreat | Self::Great | Self::Good)
    }

    /// Returns the index for this rank (for array indexing).
    pub fn index(self) -> usize {
        match self {
            Self::PerfectGreat => 0,
            Self::Great => 1,
            Self::Good => 2,
            Self::Bad => 3,
            Self::Poor => 4,
            Self::Miss => 5,
        }
    }
}

/// Fast/Slow indicator for timing feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastSlow {
    Fast,
    Just,
    Slow,
}

/// Result of a single judgment.
#[derive(Debug, Clone)]
pub struct JudgeResult {
    pub rank: JudgeRank,
    pub fast_slow: FastSlow,
    pub time_diff_ms: f64,
    pub lane: Lane,
    pub note_index: usize,
}

/// Judge timing windows in milliseconds.
#[derive(Debug, Clone)]
pub struct JudgeWindow {
    pub pg: f64,
    pub gr: f64,
    pub gd: f64,
    pub bd: f64,
    pub pr: f64,
}

impl JudgeWindow {
    /// Base judge window used as the normal reference.
    pub fn base() -> Self {
        Self {
            pg: 20.0,
            gr: 50.0,
            gd: 100.0,
            bd: 150.0,
            pr: 200.0,
        }
    }

    /// Returns the default judge window for SEVENKEYS mode.
    pub fn sevenkeys() -> Self {
        Self::base()
    }

    /// Create a judge window based on the model's rank settings.
    pub fn from_model(model: &BMSModel) -> Self {
        Self::from_rank(model.judge_rank, model.judge_rank_type)
    }

    /// Create a judge window based on rank value and source type.
    pub fn from_rank(rank: i32, rank_type: JudgeRankType) -> Self {
        let base = Self::base();
        let scale = match rank_type {
            JudgeRankType::BmsRank => match rank {
                0 => 0.7,
                1 => 0.85,
                2 => 1.0,
                3 => 1.2,
                _ => 1.0,
            },
            JudgeRankType::BmsDefExRank => (rank as f64 / 100.0).clamp(0.5, 2.0),
            JudgeRankType::BmsonJudgeRank => (rank as f64 / 18.0).clamp(0.5, 2.0),
        };
        base.scale(scale)
    }

    fn scale(&self, factor: f64) -> Self {
        Self {
            pg: self.pg * factor,
            gr: self.gr * factor,
            gd: self.gd * factor,
            bd: self.bd * factor,
            pr: self.pr * factor,
        }
    }

    /// Determine the judge rank based on timing difference.
    pub fn judge(&self, diff_abs: f64) -> Option<JudgeRank> {
        if diff_abs <= self.pg {
            Some(JudgeRank::PerfectGreat)
        } else if diff_abs <= self.gr {
            Some(JudgeRank::Great)
        } else if diff_abs <= self.gd {
            Some(JudgeRank::Good)
        } else if diff_abs <= self.bd {
            Some(JudgeRank::Bad)
        } else if diff_abs <= self.pr {
            Some(JudgeRank::Poor)
        } else {
            None
        }
    }
}

impl Default for JudgeWindow {
    fn default() -> Self {
        Self::sevenkeys()
    }
}

/// A note with its global index for tracking judgment state.
#[derive(Debug, Clone)]
pub struct NoteWithIndex {
    pub index: usize,
    pub note: Note,
}

/// Manages judgment logic for gameplay.
pub struct JudgeManager {
    window: JudgeWindow,
    long_note_mode: LongNoteMode,
    judged_notes: HashSet<usize>,
    ln_holding: HashMap<usize, LnHoldState>,
    fast_count: u32,
    slow_count: u32,
}

/// State for a long note currently being held.
#[derive(Debug, Clone)]
struct LnHoldState {
    end_note_index: usize,
}

impl JudgeManager {
    /// Create a new JudgeManager with the given timing window.
    pub fn new(window: JudgeWindow, long_note_mode: LongNoteMode) -> Self {
        Self {
            window,
            long_note_mode,
            judged_notes: HashSet::new(),
            ln_holding: HashMap::new(),
            fast_count: 0,
            slow_count: 0,
        }
    }

    /// Reset judgment state and counters.
    /// 判定状態とカウンタをリセットする。
    pub fn reset(&mut self) {
        self.judged_notes.clear();
        self.ln_holding.clear();
        self.fast_count = 0;
        self.slow_count = 0;
    }

    /// Check if a note has been judged.
    pub fn is_judged(&self, note_index: usize) -> bool {
        self.judged_notes.contains(&note_index)
    }

    /// Mark a note as judged without producing a result.
    pub fn mark_judged(&mut self, note_index: usize) {
        self.judged_notes.insert(note_index);
    }

    /// Get the fast count.
    pub fn fast_count(&self) -> u32 {
        self.fast_count
    }

    /// Get the slow count.
    pub fn slow_count(&self) -> u32 {
        self.slow_count
    }

    /// Get the judge window settings.
    pub fn window(&self) -> &JudgeWindow {
        &self.window
    }

    /// Judge a key press. Returns the judgment result if a note was hit.
    pub fn judge_press(
        &mut self,
        lane: Lane,
        press_time_ms: f64,
        notes: &[NoteWithIndex],
    ) -> Option<JudgeResult> {
        let mut best_match: Option<(usize, f64, &NoteWithIndex)> = None;

        for nwi in notes {
            if nwi.note.lane != lane {
                continue;
            }
            if self.judged_notes.contains(&nwi.index) {
                continue;
            }
            if !matches!(nwi.note.note_type, NoteType::Normal | NoteType::LongStart) {
                continue;
            }

            let diff = nwi.note.start_time_ms - press_time_ms;
            let diff_abs = diff.abs();

            if diff_abs > self.window.pr {
                continue;
            }

            if best_match.is_none() || diff_abs < best_match.as_ref().unwrap().1 {
                best_match = Some((nwi.index, diff_abs, nwi));
            }
        }

        if let Some((index, diff_abs, nwi)) = best_match {
            let time_diff = nwi.note.start_time_ms - press_time_ms;
            let rank = self.window.judge(diff_abs).unwrap_or(JudgeRank::Poor);
            let fast_slow = self.calc_fast_slow(time_diff, rank);

            if nwi.note.note_type == NoteType::LongStart && nwi.note.end_time_ms.is_some() {
                let end_note_index = self.find_ln_end_index(nwi, notes);

                self.ln_holding
                    .insert(index, LnHoldState { end_note_index });
                self.judged_notes.insert(index);

                if matches!(self.long_note_mode, LongNoteMode::Ln) {
                    return None;
                }
            } else {
                self.judged_notes.insert(index);
            }

            self.update_fast_slow(fast_slow);

            Some(JudgeResult {
                rank,
                fast_slow,
                time_diff_ms: time_diff,
                lane,
                note_index: index,
            })
        } else {
            None
        }
    }

    /// Judge a key release (for long note ends).
    pub fn judge_release(
        &mut self,
        lane: Lane,
        release_time_ms: f64,
        notes: &[NoteWithIndex],
    ) -> Option<JudgeResult> {
        let holding_start = self
            .ln_holding
            .iter()
            .find(|(_, state)| {
                notes
                    .iter()
                    .find(|n| n.index == state.end_note_index)
                    .map(|n| n.note.lane == lane)
                    .unwrap_or(false)
            })
            .map(|(start_idx, state)| (*start_idx, state.clone()));

        if let Some((start_index, hold_state)) = holding_start {
            if let Some(end_nwi) = notes.iter().find(|n| n.index == hold_state.end_note_index) {
                let end_time = end_nwi.note.start_time_ms;
                let diff = end_time - release_time_ms;
                let diff_abs = diff.abs();

                let end_rank = self.window.judge(diff_abs).unwrap_or(JudgeRank::Miss);

                let final_rank = match self.long_note_mode {
                    LongNoteMode::Ln => end_rank,
                    LongNoteMode::Cn | LongNoteMode::Hcn => end_rank,
                };

                let fast_slow = self.calc_fast_slow(diff, final_rank);

                self.judged_notes.insert(hold_state.end_note_index);
                self.ln_holding.remove(&start_index);

                if final_rank != JudgeRank::Miss {
                    self.update_fast_slow(fast_slow);
                }

                return Some(JudgeResult {
                    rank: final_rank,
                    fast_slow,
                    time_diff_ms: diff,
                    lane,
                    note_index: hold_state.end_note_index,
                });
            }
        }

        None
    }

    /// Check for missed notes and return their judgments.
    pub fn check_misses(
        &mut self,
        current_time_ms: f64,
        notes: &[NoteWithIndex],
    ) -> Vec<JudgeResult> {
        let mut results = Vec::new();
        let miss_threshold = self.window.pr;

        for nwi in notes {
            if self.judged_notes.contains(&nwi.index) {
                continue;
            }

            match nwi.note.note_type {
                NoteType::Normal | NoteType::LongStart => {
                    let diff = current_time_ms - nwi.note.start_time_ms;
                    if diff > miss_threshold {
                        self.judged_notes.insert(nwi.index);
                        results.push(JudgeResult {
                            rank: JudgeRank::Miss,
                            fast_slow: FastSlow::Slow,
                            time_diff_ms: diff,
                            lane: nwi.note.lane,
                            note_index: nwi.index,
                        });

                        if nwi.note.note_type == NoteType::LongStart {
                            if let Some(end_idx) = self.find_ln_end_index_opt(nwi, notes) {
                                self.judged_notes.insert(end_idx);
                            }
                        }
                    }
                }
                NoteType::LongEnd => {
                    let is_being_held = self
                        .ln_holding
                        .values()
                        .any(|s| s.end_note_index == nwi.index);
                    if is_being_held {
                        let diff = current_time_ms - nwi.note.start_time_ms;
                        if diff > miss_threshold {
                            let start_idx = self
                                .ln_holding
                                .iter()
                                .find(|(_, s)| s.end_note_index == nwi.index)
                                .map(|(idx, _)| *idx);

                            if let Some(start_idx) = start_idx {
                                self.ln_holding.remove(&start_idx);
                            }

                            self.judged_notes.insert(nwi.index);
                            results.push(JudgeResult {
                                rank: JudgeRank::Miss,
                                fast_slow: FastSlow::Slow,
                                time_diff_ms: diff,
                                lane: nwi.note.lane,
                                note_index: nwi.index,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        results
    }

    fn calc_fast_slow(&self, time_diff: f64, rank: JudgeRank) -> FastSlow {
        if rank == JudgeRank::PerfectGreat {
            FastSlow::Just
        } else if time_diff > 0.0 {
            FastSlow::Fast
        } else {
            FastSlow::Slow
        }
    }

    fn update_fast_slow(&mut self, fast_slow: FastSlow) {
        match fast_slow {
            FastSlow::Fast => self.fast_count += 1,
            FastSlow::Slow => self.slow_count += 1,
            FastSlow::Just => {}
        }
    }

    fn find_ln_end_index(&self, start_nwi: &NoteWithIndex, notes: &[NoteWithIndex]) -> usize {
        self.find_ln_end_index_opt(start_nwi, notes)
            .unwrap_or(start_nwi.index)
    }

    fn find_ln_end_index_opt(
        &self,
        start_nwi: &NoteWithIndex,
        notes: &[NoteWithIndex],
    ) -> Option<usize> {
        let end_time = start_nwi.note.end_time_ms?;
        notes
            .iter()
            .find(|n| {
                n.note.lane == start_nwi.note.lane
                    && n.note.note_type == NoteType::LongEnd
                    && (n.note.start_time_ms - end_time).abs() < 1.0
            })
            .map(|n| n.index)
    }
}

/// Return the worse of two judge ranks.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_judge_window_sevenkeys() {
        let window = JudgeWindow::sevenkeys();
        assert_eq!(window.pg, 20.0);
        assert_eq!(window.gr, 50.0);
        assert_eq!(window.gd, 100.0);
        assert_eq!(window.bd, 150.0);
        assert_eq!(window.pr, 200.0);
    }

    #[test]
    fn test_judge_rank_from_diff() {
        let window = JudgeWindow::sevenkeys();

        assert_eq!(window.judge(0.0), Some(JudgeRank::PerfectGreat));
        assert_eq!(window.judge(20.0), Some(JudgeRank::PerfectGreat));
        assert_eq!(window.judge(21.0), Some(JudgeRank::Great));
        assert_eq!(window.judge(50.0), Some(JudgeRank::Great));
        assert_eq!(window.judge(51.0), Some(JudgeRank::Good));
        assert_eq!(window.judge(100.0), Some(JudgeRank::Good));
        assert_eq!(window.judge(101.0), Some(JudgeRank::Bad));
        assert_eq!(window.judge(150.0), Some(JudgeRank::Bad));
        assert_eq!(window.judge(151.0), Some(JudgeRank::Poor));
        assert_eq!(window.judge(200.0), Some(JudgeRank::Poor));
        assert_eq!(window.judge(201.0), None);
    }

    #[test]
    fn test_judge_rank_continues_combo() {
        assert!(JudgeRank::PerfectGreat.continues_combo());
        assert!(JudgeRank::Great.continues_combo());
        assert!(JudgeRank::Good.continues_combo());
        assert!(!JudgeRank::Bad.continues_combo());
        assert!(!JudgeRank::Poor.continues_combo());
        assert!(!JudgeRank::Miss.continues_combo());
    }

    // Helper to create a simple LN chart with start and end notes
    fn create_ln_notes(lane: Lane, start_ms: f64, end_ms: f64) -> Vec<NoteWithIndex> {
        let ln_start = Note {
            lane,
            start_time_ms: start_ms,
            end_time_ms: Some(end_ms),
            wav_id: 1,
            note_type: NoteType::LongStart,
            mine_damage: None,
        };
        let ln_end = Note {
            lane,
            start_time_ms: end_ms,
            end_time_ms: None,
            wav_id: 1,
            note_type: NoteType::LongEnd,
            mine_damage: None,
        };
        vec![
            NoteWithIndex {
                index: 0,
                note: ln_start,
            },
            NoteWithIndex {
                index: 1,
                note: ln_end,
            },
        ]
    }

    #[test]
    fn test_ln_chart_has_long_end_note() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);

        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].note.note_type, NoteType::LongStart);
        assert_eq!(notes[0].note.end_time_ms, Some(2000.0));
        assert_eq!(notes[1].note.note_type, NoteType::LongEnd);
        assert_eq!(notes[1].note.start_time_ms, 2000.0);
    }

    #[test]
    fn test_ln_press_starts_holding() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        // Press at the start of LN
        let result = manager.judge_press(Lane::Key1, 1000.0, &notes);

        // CN mode should return a result for the press
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.rank, JudgeRank::PerfectGreat);
        assert_eq!(result.note_index, 0);

        // The LN start should be marked as judged
        assert!(manager.is_judged(0));
        // The LN end should NOT be judged yet
        assert!(!manager.is_judged(1));
    }

    #[test]
    fn test_ln_mode_press_returns_none() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Ln);

        // In LN mode, press should return None (judgment at release)
        let result = manager.judge_press(Lane::Key1, 1000.0, &notes);
        assert!(result.is_none());

        // But the note should still be tracked as holding
        assert!(manager.is_judged(0));
    }

    #[test]
    fn test_ln_release_judgment_perfect() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        // Press at LN start
        manager.judge_press(Lane::Key1, 1000.0, &notes);

        // Release exactly at LN end
        let result = manager.judge_release(Lane::Key1, 2000.0, &notes);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.rank, JudgeRank::PerfectGreat);
        assert_eq!(result.note_index, 1); // LN end index

        // Both notes should be judged
        assert!(manager.is_judged(0));
        assert!(manager.is_judged(1));
    }

    #[test]
    fn test_ln_early_release_great() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        manager.judge_press(Lane::Key1, 1000.0, &notes);

        // Release 30ms early (within Great window)
        let result = manager.judge_release(Lane::Key1, 1970.0, &notes);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.rank, JudgeRank::Great);
    }

    #[test]
    fn test_ln_very_early_release_miss() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        manager.judge_press(Lane::Key1, 1000.0, &notes);

        // Release way too early (more than 200ms)
        let result = manager.judge_release(Lane::Key1, 1700.0, &notes);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.rank, JudgeRank::Miss);
    }

    #[test]
    fn test_ln_late_release_good() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        manager.judge_press(Lane::Key1, 1000.0, &notes);

        // Release 60ms late (within Good window)
        let result = manager.judge_release(Lane::Key1, 2060.0, &notes);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.rank, JudgeRank::Good);
    }

    #[test]
    fn test_ln_miss_without_press() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        // Don't press, just check for misses after the start time
        let misses = manager.check_misses(1300.0, &notes);

        // LN start should be missed
        assert_eq!(misses.len(), 1);
        assert_eq!(misses[0].rank, JudgeRank::Miss);
        assert_eq!(misses[0].note_index, 0);

        // LN end should also be marked as judged (auto-failed with start)
        assert!(manager.is_judged(1));
    }

    #[test]
    fn test_ln_end_miss_when_held_too_long() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        // Press at LN start
        manager.judge_press(Lane::Key1, 1000.0, &notes);

        // Hold past the miss threshold without releasing
        let misses = manager.check_misses(2300.0, &notes);

        // LN end should be missed
        assert_eq!(misses.len(), 1);
        assert_eq!(misses[0].rank, JudgeRank::Miss);
        assert_eq!(misses[0].note_index, 1);
    }

    #[test]
    fn test_release_without_hold_returns_none() {
        let notes = create_ln_notes(Lane::Key1, 1000.0, 2000.0);
        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        // Try to release without pressing first
        let result = manager.judge_release(Lane::Key1, 2000.0, &notes);
        assert!(result.is_none());
    }

    #[test]
    fn test_judge_window_from_rank_very_hard() {
        let window = JudgeWindow::from_rank(0, JudgeRankType::BmsRank);
        // RANK 0 (VERY HARD) = 0.7x scale
        assert!((window.pg - 14.0).abs() < 0.01);
        assert!((window.gr - 35.0).abs() < 0.01);
    }

    #[test]
    fn test_judge_window_from_rank_easy() {
        let window = JudgeWindow::from_rank(3, JudgeRankType::BmsRank);
        // RANK 3 (EASY) = 1.2x scale
        assert!((window.pg - 24.0).abs() < 0.01);
        assert!((window.gr - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_judge_window_from_defexrank() {
        let window = JudgeWindow::from_rank(150, JudgeRankType::BmsDefExRank);
        // DEFEXRANK 150 = 1.5x scale
        assert!((window.pg - 30.0).abs() < 0.01);
        assert!((window.gr - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_fast_slow_counting() {
        let notes: Vec<NoteWithIndex> = (0..5)
            .map(|i| NoteWithIndex {
                index: i,
                note: Note {
                    lane: Lane::Key1,
                    start_time_ms: (i as f64) * 500.0,
                    end_time_ms: None,
                    wav_id: 1,
                    note_type: NoteType::Normal,
                    mine_damage: None,
                },
            })
            .collect();

        let mut manager = JudgeManager::new(JudgeWindow::sevenkeys(), LongNoteMode::Cn);

        // Press early (fast)
        manager.judge_press(Lane::Key1, -30.0, &notes); // 30ms early
        // Press late (slow)
        manager.judge_press(Lane::Key1, 530.0, &notes); // 30ms late
        // Press perfect
        manager.judge_press(Lane::Key1, 1000.0, &notes); // exact

        assert_eq!(manager.fast_count(), 1);
        assert_eq!(manager.slow_count(), 1);
    }
}
