use serde::{Deserialize, Serialize};

use super::note::{JudgeRankType, LongNoteMode, Note, PlayMode, TotalType};

/// BPM change event.
#[derive(Debug, Clone, PartialEq)]
pub struct BpmChange {
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// New BPM value.
    pub bpm: f64,
}

/// Stop event (scroll halt).
#[derive(Debug, Clone, PartialEq)]
pub struct StopEvent {
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// Duration of stop in microseconds.
    pub duration_us: i64,
}

/// Scroll speed change event.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollEvent {
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// Scroll speed multiplier.
    pub rate: f64,
}

/// WAV definition from BMS header.
#[derive(Debug, Clone, PartialEq)]
pub struct WavDef {
    /// Object ID (1-indexed in BMS, stored as u32).
    pub id: u32,
    /// Relative file path to the audio file.
    pub path: String,
}

/// BMP definition from BMS header.
#[derive(Debug, Clone, PartialEq)]
pub struct BmpDef {
    /// Object ID.
    pub id: u32,
    /// Relative file path to the image/video file.
    pub path: String,
}

/// BGA (background animation) event.
#[derive(Debug, Clone, PartialEq)]
pub struct BgaEvent {
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// BMP object ID.
    pub bmp_id: u32,
    /// BGA layer (0=base, 1=layer, 2=poor).
    pub layer: u32,
}

/// BGM (background music) auto-play event.
#[derive(Debug, Clone, PartialEq)]
pub struct BgmEvent {
    /// Time in microseconds from song start.
    pub time_us: i64,
    /// WAV object ID.
    pub wav_id: u32,
}

/// Parsed BMS model containing all chart data.
/// Corresponds to bms.model.BMSModel in beatoraja.
#[derive(Debug, Clone)]
pub struct BmsModel {
    // -- Metadata --
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub subartist: String,
    pub genre: String,
    pub banner: String,
    pub stagefile: String,
    pub back_bmp: String,
    pub preview: String,

    // -- Play parameters --
    pub mode: PlayMode,
    pub judge_rank: i32,
    pub judge_rank_type: JudgeRankType,
    pub total: f64,
    pub total_type: TotalType,
    pub bpm: f64,
    pub ln_mode: LongNoteMode,
    pub difficulty: i32,
    pub playlevel: String,

    // -- Chart data --
    pub notes: Vec<Note>,
    pub bpm_changes: Vec<BpmChange>,
    pub stops: Vec<StopEvent>,
    pub scrolls: Vec<ScrollEvent>,
    pub bgm_events: Vec<BgmEvent>,
    pub bga_events: Vec<BgaEvent>,

    // -- Definitions --
    pub wav_defs: Vec<WavDef>,
    pub bmp_defs: Vec<BmpDef>,

    // -- Hash --
    pub md5: String,
    pub sha256: String,

    // -- File path --
    pub path: String,
}

impl BmsModel {
    /// Total number of playable notes (excluding mines and invisible).
    pub fn total_notes(&self) -> usize {
        self.notes
            .iter()
            .filter(|n| {
                matches!(
                    n.note_type,
                    super::note::NoteType::Normal
                        | super::note::NoteType::LongNote
                        | super::note::NoteType::ChargeNote
                        | super::note::NoteType::HellChargeNote
                )
            })
            .count()
    }

    /// Calculate default TOTAL value based on mode and note count.
    /// Corresponds to BMSPlayerRule.calculateDefaultTotal in beatoraja.
    pub fn calculate_default_total(mode: PlayMode, total_notes: usize) -> f64 {
        let n = total_notes as f64;
        match mode {
            PlayMode::Keyboard24K | PlayMode::Keyboard24KDouble => {
                f64::max(300.0, 7.605 * (n + 100.0) / (0.01 * n + 6.5))
            }
            _ => f64::max(260.0, 7.605 * n / (0.01 * n + 6.5)),
        }
    }

    /// Validate and normalize judge_rank and total, matching BMSPlayerRule.validate().
    pub fn validate(&mut self) {
        let rule = PlayerRule::from_mode(self.mode);

        // Normalize judge_rank
        match self.judge_rank_type {
            JudgeRankType::BmsRank => {
                let rank = self.judge_rank;
                self.judge_rank = if (0..5).contains(&rank) {
                    rule.judge_window_rule.judge_rank[rank as usize]
                } else {
                    rule.judge_window_rule.judge_rank[2] // default to NORMAL
                };
            }
            JudgeRankType::BmsDefExRank => {
                let rank = self.judge_rank;
                self.judge_rank = if rank > 0 {
                    rank * rule.judge_window_rule.judge_rank[2] / 100
                } else {
                    rule.judge_window_rule.judge_rank[2]
                };
            }
            JudgeRankType::BmsonJudgeRank => {
                if self.judge_rank <= 0 {
                    self.judge_rank = 100;
                }
            }
        }
        self.judge_rank_type = JudgeRankType::BmsonJudgeRank;

        // Normalize total
        let total_notes = self.total_notes();
        match self.total_type {
            TotalType::Bms => {
                if self.total <= 0.0 {
                    self.total = Self::calculate_default_total(self.mode, total_notes);
                }
            }
            TotalType::Bmson => {
                let default_total = Self::calculate_default_total(self.mode, total_notes);
                self.total = if self.total > 0.0 {
                    self.total / 100.0 * default_total
                } else {
                    default_total
                };
            }
        }
        self.total_type = TotalType::Bms;
    }
}

impl Default for BmsModel {
    fn default() -> Self {
        Self {
            title: String::new(),
            subtitle: String::new(),
            artist: String::new(),
            subartist: String::new(),
            genre: String::new(),
            banner: String::new(),
            stagefile: String::new(),
            back_bmp: String::new(),
            preview: String::new(),
            mode: PlayMode::Beat7K,
            judge_rank: 100,
            judge_rank_type: JudgeRankType::BmsonJudgeRank,
            total: 0.0,
            total_type: TotalType::Bms,
            bpm: 130.0,
            ln_mode: LongNoteMode::default(),
            difficulty: 0,
            playlevel: String::new(),
            notes: Vec::new(),
            bpm_changes: Vec::new(),
            stops: Vec::new(),
            scrolls: Vec::new(),
            bgm_events: Vec::new(),
            bga_events: Vec::new(),
            wav_defs: Vec::new(),
            bmp_defs: Vec::new(),
            md5: String::new(),
            sha256: String::new(),
            path: String::new(),
        }
    }
}

/// Judge window rule per mode.
/// Corresponds to JudgeWindowRule in beatoraja JudgeProperty.java.
#[derive(Debug, Clone)]
pub struct JudgeWindowRule {
    /// Judge rank values: [VERYHARD, HARD, NORMAL, EASY, VERYEASY].
    pub judge_rank: [i32; 5],
    /// Whether judge window is fixed (not scaled by judgerank).
    pub fix_judge: [bool; 5],
}

impl JudgeWindowRule {
    pub const NORMAL: Self = Self {
        judge_rank: [25, 50, 75, 100, 125],
        fix_judge: [false, false, false, false, true],
    };

    pub const PMS: Self = Self {
        judge_rank: [33, 50, 70, 100, 133],
        fix_judge: [true, false, false, true, true],
    };

    /// Create scaled judge windows from base windows, judge_rank value, and per-window rate.
    /// Corresponds to JudgeWindowRule.create() in beatoraja.
    ///
    /// `org`: base windows as &[[i64; 2]], indexed [PG, GR, GD, BD, MS].
    /// `judgerank`: scaled judge rank value (percentage-like).
    /// `judge_window_rate`: per-window rate [PG, GR, GD] (percentage, typically 100).
    ///
    /// Returns scaled windows with the same length as `org`.
    #[allow(clippy::needless_range_loop)]
    pub fn create(
        &self,
        org: &[[i64; 2]],
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> Vec<[i64; 2]> {
        let len = org.len();
        let mut judge: Vec<[i64; 2]> = Vec::with_capacity(len);

        // Step 1: Apply judgerank scaling (skip if fixjudge)
        for (i, window) in org.iter().enumerate() {
            let scale = |v: i64| -> i64 {
                if self.fix_judge[i] {
                    v
                } else {
                    v * judgerank as i64 / 100
                }
            };
            judge.push([scale(window[0]), scale(window[1])]);
        }

        // Step 2: Clamp between fixed bounds
        // Uses index-based access because we cross-reference judge[i], judge[fixmin], judge[fixmax].
        let clamp_len = len.min(4);
        let mut fixmin: Option<usize> = None;
        for i in 0..clamp_len {
            if self.fix_judge[i] {
                fixmin = Some(i);
                continue;
            }

            let fixmax = ((i + 1)..4).find(|&j| self.fix_judge[j]);

            for j in 0..2 {
                if let Some(fm) = fixmin
                    && judge[i][j].abs() < judge[fm][j].abs()
                {
                    judge[i][j] = judge[fm][j];
                }
                if let Some(fm) = fixmax
                    && judge[i][j].abs() > judge[fm][j].abs()
                {
                    judge[i][j] = judge[fm][j];
                }
            }
        }

        // Step 3: judgeWindowRate correction for first 3 windows
        // Uses index-based access because we reference judge[3] (BD) and judge[i-1].
        let rate_len = len.min(3);
        for i in 0..rate_len {
            for j in 0..2 {
                judge[i][j] = judge[i][j] * judge_window_rate[i] as i64 / 100;
                if judge[i][j].abs() > judge[3][j].abs() {
                    judge[i][j] = judge[3][j];
                }
                if i > 0 && judge[i][j].abs() < judge[i - 1][j].abs() {
                    judge[i][j] = judge[i - 1][j];
                }
            }
        }

        judge
    }
}

/// Player rule mapping mode to gauge/judge properties.
/// Corresponds to BMSPlayerRule in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerRuleType {
    FiveKeys,
    SevenKeys,
    Pms,
    Keyboard,
}

/// Player rule containing gauge and judge property references.
#[derive(Debug, Clone)]
pub struct PlayerRule {
    pub rule_type: PlayerRuleType,
    pub judge_window_rule: JudgeWindowRule,
}

impl PlayerRule {
    pub fn from_mode(mode: PlayMode) -> Self {
        match mode {
            PlayMode::Beat5K | PlayMode::Beat10K => Self {
                rule_type: PlayerRuleType::FiveKeys,
                judge_window_rule: JudgeWindowRule::NORMAL,
            },
            PlayMode::Beat7K | PlayMode::Beat14K => Self {
                rule_type: PlayerRuleType::SevenKeys,
                judge_window_rule: JudgeWindowRule::NORMAL,
            },
            PlayMode::PopN9K => Self {
                rule_type: PlayerRuleType::Pms,
                judge_window_rule: JudgeWindowRule::PMS,
            },
            PlayMode::Keyboard24K | PlayMode::Keyboard24KDouble => Self {
                rule_type: PlayerRuleType::Keyboard,
                judge_window_rule: JudgeWindowRule::NORMAL,
            },
        }
    }
}

/// Gauge property type.
/// Corresponds to GaugeProperty enum in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GaugePropertyType {
    FiveKeys,
    SevenKeys,
    Pms,
    Keyboard,
    Lr2,
}

impl GaugePropertyType {
    pub fn from_mode(mode: PlayMode) -> Self {
        match mode {
            PlayMode::Beat5K | PlayMode::Beat10K => Self::FiveKeys,
            PlayMode::Beat7K | PlayMode::Beat14K => Self::SevenKeys,
            PlayMode::PopN9K => Self::Pms,
            PlayMode::Keyboard24K | PlayMode::Keyboard24KDouble => Self::Keyboard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::note::{Note, NoteType};

    #[test]
    fn calculate_default_total_beat7k() {
        // Beat7K with 1000 notes: max(260, 7.605 * 1000 / (0.01 * 1000 + 6.5))
        // = max(260, 7605 / 16.5) = max(260, 460.909...) = 460.909...
        let total = BmsModel::calculate_default_total(PlayMode::Beat7K, 1000);
        assert!((total - 460.909).abs() < 0.1);
    }

    #[test]
    fn calculate_default_total_keyboard() {
        // Keyboard24K with 1000 notes: max(300, 7.605 * 1100 / 16.5) = max(300, 506.999...)
        let total = BmsModel::calculate_default_total(PlayMode::Keyboard24K, 1000);
        assert!((total - 507.0).abs() < 0.1);
    }

    #[test]
    fn calculate_default_total_small_notes() {
        // With very few notes, should return the minimum
        let total = BmsModel::calculate_default_total(PlayMode::Beat7K, 1);
        assert!((total - 260.0).abs() < 0.1);
    }

    #[test]
    fn total_notes_count() {
        let model = BmsModel {
            notes: vec![
                Note {
                    lane: 0,
                    note_type: NoteType::Normal,
                    time_us: 0,
                    end_time_us: 0,
                    wav_id: 0,
                    damage: 0.0,
                },
                Note {
                    lane: 1,
                    note_type: NoteType::LongNote,
                    time_us: 0,
                    end_time_us: 1000,
                    wav_id: 0,
                    damage: 0.0,
                },
                Note {
                    lane: 2,
                    note_type: NoteType::Mine,
                    time_us: 0,
                    end_time_us: 0,
                    wav_id: 0,
                    damage: 5.0,
                },
                Note {
                    lane: 3,
                    note_type: NoteType::Invisible,
                    time_us: 0,
                    end_time_us: 0,
                    wav_id: 0,
                    damage: 0.0,
                },
            ],
            ..Default::default()
        };
        assert_eq!(model.total_notes(), 2); // Normal + LongNote
    }

    #[test]
    fn validate_bms_rank() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            judge_rank: 2, // NORMAL
            judge_rank_type: JudgeRankType::BmsRank,
            ..Default::default()
        };
        model.validate();
        assert_eq!(model.judge_rank, 75); // NORMAL judge_rank[2]
        assert_eq!(model.judge_rank_type, JudgeRankType::BmsonJudgeRank);
    }

    #[test]
    fn validate_bms_rank_out_of_range() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            judge_rank: 10, // out of range
            judge_rank_type: JudgeRankType::BmsRank,
            ..Default::default()
        };
        model.validate();
        assert_eq!(model.judge_rank, 75); // defaults to NORMAL
    }

    #[test]
    fn validate_bms_defexrank() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            judge_rank: 100, // 100%
            judge_rank_type: JudgeRankType::BmsDefExRank,
            ..Default::default()
        };
        model.validate();
        assert_eq!(model.judge_rank, 75); // 100 * 75 / 100
    }

    #[test]
    fn validate_bmson_judgerank_default() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            judge_rank: 0,
            judge_rank_type: JudgeRankType::BmsonJudgeRank,
            ..Default::default()
        };
        model.validate();
        assert_eq!(model.judge_rank, 100);
    }

    #[test]
    fn validate_total_bms_undefined() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            total: 0.0,
            total_type: TotalType::Bms,
            notes: vec![Note {
                lane: 0,
                note_type: NoteType::Normal,
                time_us: 0,
                end_time_us: 0,
                wav_id: 0,
                damage: 0.0,
            }],
            ..Default::default()
        };
        model.validate();
        assert!(model.total > 0.0);
    }

    #[test]
    fn validate_total_bms_defined() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            total: 300.0,
            total_type: TotalType::Bms,
            ..Default::default()
        };
        model.validate();
        assert_eq!(model.total, 300.0); // unchanged
    }

    #[test]
    fn validate_total_bmson() {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            total: 100.0, // 100% of default
            total_type: TotalType::Bmson,
            notes: (0..100)
                .map(|i| Note {
                    lane: i % 7,
                    note_type: NoteType::Normal,
                    time_us: i as i64 * 1000,
                    end_time_us: 0,
                    wav_id: 0,
                    damage: 0.0,
                })
                .collect(),
            ..Default::default()
        };
        let default_total = BmsModel::calculate_default_total(PlayMode::Beat7K, 100);
        model.validate();
        assert!((model.total - default_total).abs() < 0.001);
        assert_eq!(model.total_type, TotalType::Bms);
    }

    #[test]
    fn player_rule_from_mode() {
        assert_eq!(
            PlayerRule::from_mode(PlayMode::Beat5K).rule_type,
            PlayerRuleType::FiveKeys
        );
        assert_eq!(
            PlayerRule::from_mode(PlayMode::Beat7K).rule_type,
            PlayerRuleType::SevenKeys
        );
        assert_eq!(
            PlayerRule::from_mode(PlayMode::PopN9K).rule_type,
            PlayerRuleType::Pms
        );
        assert_eq!(
            PlayerRule::from_mode(PlayMode::Keyboard24K).rule_type,
            PlayerRuleType::Keyboard
        );
    }

    #[test]
    fn pms_judge_window_rule() {
        let rule = JudgeWindowRule::PMS;
        assert_eq!(rule.judge_rank, [33, 50, 70, 100, 133]);
        assert_eq!(rule.fix_judge, [true, false, false, true, true]);
    }
}
