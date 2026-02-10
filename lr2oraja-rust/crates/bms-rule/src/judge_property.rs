use serde::{Deserialize, Serialize};

/// A judge window pair: [late_limit, early_limit] in microseconds.
/// late_limit is typically negative, early_limit is positive.
pub type JudgeWindow = [i64; 2];

/// Judge window table: 5 elements for PG, GR, GD, BD, MS in order.
pub type JudgeWindowTable = Vec<JudgeWindow>;

/// Condition for MISS generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissCondition {
    /// MISS only fires for the first note in the window
    One,
    /// MISS fires for all notes that pass the window
    Always,
}

/// Note type categories for judge window lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgeNoteType {
    Note,
    LongNoteEnd,
    Scratch,
    LongScratchEnd,
}

/// Judge window scaling rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgeWindowRule {
    Normal,
    Pms,
    Lr2,
}

/// LR2 scaling reference table for 2D interpolation.
/// Rows: [0] = zero baseline, [1] = PGREAT, [2] = GREAT, [3] = GOOD.
/// Columns: interpolation points at judgerank 0, 25, 50, 75, 100.
const LR2_SCALING: [[i64; 5]; 4] = [
    [0, 0, 0, 0, 0],
    [0, 8000, 15000, 18000, 21000],
    [0, 24000, 30000, 40000, 60000],
    [0, 40000, 60000, 100000, 120000],
];

/// Judgerank scaling factors per difficulty for each JudgeWindowRule.
/// Order: VERYHARD, HARD, NORMAL, EASY, VERYEASY
const NORMAL_JUDGERANK: [i32; 5] = [25, 50, 75, 100, 125];
const PMS_JUDGERANK: [i32; 5] = [33, 50, 70, 100, 133];
const LR2_JUDGERANK: [i32; 5] = [25, 50, 75, 100, 75];

/// Whether each judge level (PG, GR, GD, BD, MS) is fixed regardless of judgerank.
const NORMAL_FIXJUDGE: [bool; 5] = [false, false, false, false, true];
const PMS_FIXJUDGE: [bool; 5] = [true, false, false, true, true];
const LR2_FIXJUDGE: [bool; 5] = [false, false, false, true, true];

/// LR2 judge scaling algorithm using pure integer arithmetic.
///
/// Applies LR2-specific non-linear scaling to a base judge window value.
/// For judgerank >= 100, this is simple linear scaling.
/// For judgerank < 100, uses 2D interpolation over the LR2_SCALING table.
fn lr2_judge_scaling(base: i64, judgerank: i32) -> i64 {
    let sign: i64;
    let abs_base: i64;
    if base < 0 {
        abs_base = -base;
        sign = -1;
    } else {
        abs_base = base;
        sign = 1;
    }

    if judgerank >= 100 {
        return sign * abs_base * judgerank as i64 / 100;
    }

    let last = LR2_SCALING[0].len() - 1; // 4
    let judgeindex = (judgerank / 25) as usize;

    // Find the interpolation row bracket
    let mut s: usize = 0;
    while s < LR2_SCALING.len() && abs_base >= LR2_SCALING[s][last] {
        s += 1;
    }

    let (n, d, x1, x2): (i64, i64, i64, i64);

    if s < LR2_SCALING.len() {
        // Interpolate between rows s-1 and s
        n = abs_base - LR2_SCALING[s - 1][last];
        d = LR2_SCALING[s][last] - LR2_SCALING[s - 1][last];
        x1 = d * LR2_SCALING[s - 1][judgeindex]
            + n * (LR2_SCALING[s][judgeindex] - LR2_SCALING[s - 1][judgeindex]);
        x2 = d * LR2_SCALING[s - 1][judgeindex + 1]
            + n * (LR2_SCALING[s][judgeindex + 1] - LR2_SCALING[s - 1][judgeindex + 1]);
    } else {
        // Beyond the table: extrapolate from last row
        n = abs_base;
        d = LR2_SCALING[s - 1][last];
        x1 = n * LR2_SCALING[s - 1][judgeindex];
        x2 = n * LR2_SCALING[s - 1][judgeindex + 1];
    }

    sign * (x1 + (judgerank as i64 - judgeindex as i64 * 25) * (x2 - x1) / 25) / d
}

/// Create judge windows using NORMAL or PMS rules.
///
/// Scales base windows by judgerank, clamps against fixed windows,
/// then applies judgeWindowRate correction for PG/GR/GD.
fn create_normal(
    org: &[JudgeWindow],
    judgerank: i32,
    judge_window_rate: &[i32; 3],
    fixjudge: &[bool; 5],
) -> JudgeWindowTable {
    let mut judge: JudgeWindowTable = org
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if fixjudge[i] {
                *w
            } else {
                [w[0] * judgerank as i64 / 100, w[1] * judgerank as i64 / 100]
            }
        })
        .collect();

    // Clamp non-fixed windows against surrounding fixed windows
    let len = org.len().min(4);
    let mut fixmin: Option<usize> = None;
    for i in 0..len {
        if fixjudge[i] {
            fixmin = Some(i);
            continue;
        }
        // Find next fixed window above
        let mut fixmax: Option<usize> = None;
        #[allow(clippy::needless_range_loop)]
        for j in (i + 1)..4 {
            if fixjudge[j] {
                fixmax = Some(j);
                break;
            }
        }

        #[allow(clippy::needless_range_loop)]
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

    // Apply judgeWindowRate correction for PG, GR, GD
    let rate_len = org.len().min(3);
    for i in 0..rate_len {
        for j in 0..2 {
            judge[i][j] = judge[i][j] * judge_window_rate[i] as i64 / 100;
            // Clamp: must not exceed BD window
            if judge.len() > 3 && judge[i][j].abs() > judge[3][j].abs() {
                judge[i][j] = judge[3][j];
            }
            // Clamp: must not be smaller than previous window
            if i > 0 && judge[i][j].abs() < judge[i - 1][j].abs() {
                judge[i][j] = judge[i - 1][j];
            }
        }
    }

    judge
}

/// Create judge windows using LR2-specific scaling.
///
/// Applies lr2_judge_scaling to PG/GR/GD, clamps inner <= outer,
/// then applies judgeWindowRate correction.
fn create_lr2(
    org: &[JudgeWindow],
    judgerank: i32,
    judge_window_rate: &[i32; 3],
) -> JudgeWindowTable {
    let mut judge: JudgeWindowTable = org.to_vec();

    // Apply LR2 scaling to PG, GR, GD only (indices 0..3)
    let fix_max = 3.min(judge.len());
    for i in 0..fix_max {
        for j in 0..2 {
            judge[i][j] = lr2_judge_scaling(org[i][j], judgerank);
        }
    }

    // Clamp: inner windows must not exceed outer windows
    for i in (0..fix_max).rev() {
        if i + 1 < judge.len() {
            #[allow(clippy::needless_range_loop)]
            for j in 0..2 {
                if judge[i][j].abs() > judge[i + 1][j].abs() {
                    judge[i][j] = judge[i + 1][j];
                }
            }
        }
    }

    // Apply judgeWindowRate correction for PG, GR, GD
    let rate_len = org.len().min(3);
    for i in 0..rate_len {
        for j in 0..2 {
            judge[i][j] = judge[i][j] * judge_window_rate[i] as i64 / 100;
            // Clamp: must not exceed BD window
            if judge.len() > 3 && judge[i][j].abs() > judge[3][j].abs() {
                judge[i][j] = judge[3][j];
            }
            // Clamp: must not be smaller than previous window
            if i > 0 && judge[i][j].abs() < judge[i - 1][j].abs() {
                judge[i][j] = judge[i - 1][j];
            }
        }
    }

    judge
}

impl JudgeWindowRule {
    /// Convert raw judge rank value to effective judgerank for window scaling.
    ///
    /// Matches Java's `BMSPlayerRule.validate()` conversion logic:
    /// - `BmsRank`: index into the rule's judgerank table `[VERYHARD, HARD, NORMAL, EASY, VERYEASY]`
    /// - `BmsDefExRank`: `raw * judgerank[2] / 100` (percentage of rule's default)
    /// - `BmsonJudgeRank`: direct value (100 = standard), or 100 if <= 0
    pub fn resolve_judge_rank(self, raw: i32, rank_type: bms_model::JudgeRankType) -> i32 {
        match rank_type {
            bms_model::JudgeRankType::BmsRank => {
                let factors = self.judgerank_factors();
                if raw >= 0 && (raw as usize) < factors.len() {
                    factors[raw as usize]
                } else {
                    factors[2] // default: NORMAL
                }
            }
            bms_model::JudgeRankType::BmsDefExRank => {
                let base = self.judgerank_factors()[2];
                if raw > 0 { raw * base / 100 } else { base }
            }
            bms_model::JudgeRankType::BmsonJudgeRank => {
                if raw > 0 {
                    raw
                } else {
                    100
                }
            }
        }
    }

    /// Judgerank scaling factors [VERYHARD, HARD, NORMAL, EASY, VERYEASY]
    pub fn judgerank_factors(self) -> &'static [i32; 5] {
        match self {
            Self::Normal => &NORMAL_JUDGERANK,
            Self::Pms => &PMS_JUDGERANK,
            Self::Lr2 => &LR2_JUDGERANK,
        }
    }

    /// Whether each judge level is fixed regardless of judgerank
    pub fn fixjudge(self) -> &'static [bool; 5] {
        match self {
            Self::Normal => &NORMAL_FIXJUDGE,
            Self::Pms => &PMS_FIXJUDGE,
            Self::Lr2 => &LR2_FIXJUDGE,
        }
    }

    /// Create scaled judge windows from base windows.
    pub fn create(
        self,
        org: &[JudgeWindow],
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> JudgeWindowTable {
        match self {
            Self::Normal | Self::Pms => {
                create_normal(org, judgerank, judge_window_rate, self.fixjudge())
            }
            Self::Lr2 => create_lr2(org, judgerank, judge_window_rate),
        }
    }
}

/// Judge property set for a specific play mode.
///
/// Contains base judge windows for each note type and configuration
/// for combo continuation, miss condition, and judge vanishing.
#[derive(Debug, Clone)]
pub struct JudgeProperty {
    /// Base judge windows for normal notes [PG, GR, GD, BD, MS]
    pub note: Vec<JudgeWindow>,
    /// Base judge windows for scratch notes [PG, GR, GD, BD, MS]
    pub scratch: Vec<JudgeWindow>,
    /// Base judge windows for long note end [PG, GR, GD, BD]
    pub longnote: Vec<JudgeWindow>,
    /// Margin time for long note end in microseconds
    pub longnote_margin: i64,
    /// Base judge windows for long scratch end [PG, GR, GD, BD]
    pub longscratch: Vec<JudgeWindow>,
    /// Margin time for long scratch end in microseconds
    pub longscratch_margin: i64,
    /// Whether each judge level continues combo [PG, GR, GD, BD, PR, MS]
    pub combo: [bool; 6],
    /// MISS generation condition
    pub miss: MissCondition,
    /// Whether each judge level causes note vanishing [PG, GR, GD, BD, PR, MS]
    pub judge_vanish: [bool; 6],
    /// Window scaling rule
    pub window_rule: JudgeWindowRule,
}

impl JudgeProperty {
    /// FIVEKEYS judge property (5-key BMS mode)
    pub fn fivekeys() -> Self {
        Self {
            note: vec![
                [-20000, 20000],
                [-50000, 50000],
                [-100000, 100000],
                [-150000, 150000],
                [-150000, 500000],
            ],
            scratch: vec![
                [-30000, 30000],
                [-60000, 60000],
                [-110000, 110000],
                [-160000, 160000],
                [-160000, 500000],
            ],
            longnote: vec![
                [-120000, 120000],
                [-150000, 150000],
                [-200000, 200000],
                [-250000, 250000],
            ],
            longnote_margin: 0,
            longscratch: vec![
                [-130000, 130000],
                [-160000, 160000],
                [-110000, 110000],
                [-260000, 260000],
            ],
            longscratch_margin: 0,
            combo: [true, true, true, false, false, false],
            miss: MissCondition::Always,
            judge_vanish: [true, true, true, true, true, false],
            window_rule: JudgeWindowRule::Normal,
        }
    }

    /// SEVENKEYS judge property (7-key IIDX-style mode)
    pub fn sevenkeys() -> Self {
        Self {
            note: vec![
                [-20000, 20000],
                [-60000, 60000],
                [-150000, 150000],
                [-280000, 220000],
                [-150000, 500000],
            ],
            scratch: vec![
                [-30000, 30000],
                [-70000, 70000],
                [-160000, 160000],
                [-290000, 230000],
                [-160000, 500000],
            ],
            longnote: vec![
                [-120000, 120000],
                [-160000, 160000],
                [-200000, 200000],
                [-280000, 220000],
            ],
            longnote_margin: 0,
            longscratch: vec![
                [-130000, 130000],
                [-170000, 170000],
                [-210000, 210000],
                [-290000, 230000],
            ],
            longscratch_margin: 0,
            combo: [true, true, true, false, false, true],
            miss: MissCondition::Always,
            judge_vanish: [true, true, true, true, true, false],
            window_rule: JudgeWindowRule::Normal,
        }
    }

    /// PMS judge property (Pop'n Music style)
    pub fn pms() -> Self {
        Self {
            note: vec![
                [-20000, 20000],
                [-50000, 50000],
                [-117000, 117000],
                [-183000, 183000],
                [-175000, 500000],
            ],
            scratch: vec![],
            longnote: vec![
                [-120000, 120000],
                [-150000, 150000],
                [-217000, 217000],
                [-283000, 283000],
            ],
            longnote_margin: 200000,
            longscratch: vec![],
            longscratch_margin: 0,
            combo: [true, true, true, false, false, false],
            miss: MissCondition::One,
            judge_vanish: [true, true, true, false, true, false],
            window_rule: JudgeWindowRule::Pms,
        }
    }

    /// KEYBOARD judge property (24-key keyboard mode)
    pub fn keyboard() -> Self {
        Self {
            note: vec![
                [-30000, 30000],
                [-90000, 90000],
                [-200000, 200000],
                [-320000, 240000],
                [-200000, 650000],
            ],
            scratch: vec![],
            longnote: vec![
                [-160000, 25000],
                [-200000, 75000],
                [-260000, 140000],
                [-320000, 240000],
            ],
            longnote_margin: 0,
            longscratch: vec![],
            longscratch_margin: 0,
            combo: [true, true, true, false, false, true],
            miss: MissCondition::Always,
            judge_vanish: [true, true, true, true, true, false],
            window_rule: JudgeWindowRule::Normal,
        }
    }

    /// LR2 judge property (LR2-compatible mode)
    pub fn lr2() -> Self {
        Self {
            note: vec![
                [-21000, 21000],
                [-60000, 60000],
                [-120000, 120000],
                [-200000, 200000],
                [0, 1000000],
            ],
            scratch: vec![
                [-21000, 21000],
                [-60000, 60000],
                [-120000, 120000],
                [-200000, 200000],
                [0, 1000000],
            ],
            longnote: vec![
                [-120000, 120000],
                [-120000, 120000],
                [-120000, 120000],
                [-200000, 200000],
            ],
            longnote_margin: 0,
            longscratch: vec![
                [-120000, 120000],
                [-120000, 120000],
                [-120000, 120000],
                [-200000, 200000],
            ],
            longscratch_margin: 0,
            combo: [true, true, true, false, false, true],
            miss: MissCondition::Always,
            judge_vanish: [true, true, true, true, true, false],
            window_rule: JudgeWindowRule::Lr2,
        }
    }

    /// Get scaled judge windows for a specific note type.
    ///
    /// # Arguments
    /// * `note_type` - The type of note to get judge windows for
    /// * `judgerank` - The judgerank value (higher = easier)
    /// * `judge_window_rate` - Rate multiplier for [PG, GR, GD] windows (100 = normal)
    pub fn judge_windows(
        &self,
        note_type: JudgeNoteType,
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> JudgeWindowTable {
        let base = match note_type {
            JudgeNoteType::Note => &self.note,
            JudgeNoteType::LongNoteEnd => &self.longnote,
            JudgeNoteType::Scratch => &self.scratch,
            JudgeNoteType::LongScratchEnd => &self.longscratch,
        };
        self.window_rule.create(base, judgerank, judge_window_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- LR2 Judge Scaling Tests ---

    #[test]
    fn lr2_scaling_judgerank_100_is_identity() {
        assert_eq!(lr2_judge_scaling(21000, 100), 21000);
        assert_eq!(lr2_judge_scaling(-21000, 100), -21000);
        assert_eq!(lr2_judge_scaling(60000, 100), 60000);
        assert_eq!(lr2_judge_scaling(120000, 100), 120000);
    }

    #[test]
    fn lr2_scaling_judgerank_above_100_linear() {
        assert_eq!(lr2_judge_scaling(21000, 200), 42000);
        assert_eq!(lr2_judge_scaling(60000, 150), 90000);
        assert_eq!(lr2_judge_scaling(-120000, 125), -150000);
    }

    #[test]
    fn lr2_scaling_judgerank_0_is_zero() {
        assert_eq!(lr2_judge_scaling(21000, 0), 0);
        assert_eq!(lr2_judge_scaling(60000, 0), 0);
        assert_eq!(lr2_judge_scaling(120000, 0), 0);
    }

    #[test]
    fn lr2_scaling_negative_base() {
        let pos = lr2_judge_scaling(21000, 50);
        let neg = lr2_judge_scaling(-21000, 50);
        assert_eq!(neg, -pos);
    }

    #[test]
    fn lr2_scaling_judgerank_50() {
        // base=21000 falls in row 1 bracket (21000 == LR2_SCALING[1][4])
        // s=2, n=0, d=39000, judgeindex=2
        // x1 = 39000*15000 = 585_000_000
        // x2 = 39000*18000 = 702_000_000
        // result = 585_000_000 / 39000 = 15000
        assert_eq!(lr2_judge_scaling(21000, 50), 15000);
    }

    #[test]
    fn lr2_scaling_judgerank_75() {
        // base=21000, s=2, n=0, d=39000, judgeindex=3
        // x1 = 39000*18000 = 702_000_000
        // x2 = 39000*21000 = 819_000_000
        // result = 702_000_000 / 39000 = 18000
        assert_eq!(lr2_judge_scaling(21000, 75), 18000);
    }

    #[test]
    fn lr2_scaling_judgerank_25() {
        // base=60000, s=3 (60000 >= LR2_SCALING[2][4]=60000)
        // n=0, d=60000, judgeindex=1
        // x1 = 60000*24000 = 1_440_000_000
        // x2 = 60000*30000 = 1_800_000_000
        // result = 1_440_000_000 / 60000 = 24000
        assert_eq!(lr2_judge_scaling(60000, 25), 24000);
    }

    #[test]
    fn lr2_scaling_large_base_extrapolation() {
        // base=200000, judgerank=50, judgeindex=2
        // s=4 (beyond table)
        // n=200000, d=120000
        // x1 = 200000 * 60000 = 12_000_000_000
        // x2 = 200000 * 100000 = 20_000_000_000
        // result = 12_000_000_000 / 120000 = 100000
        assert_eq!(lr2_judge_scaling(200000, 50), 100000);
    }

    #[test]
    fn lr2_scaling_base_zero() {
        assert_eq!(lr2_judge_scaling(0, 50), 0);
        assert_eq!(lr2_judge_scaling(0, 100), 0);
        assert_eq!(lr2_judge_scaling(0, 0), 0);
    }

    #[test]
    fn lr2_scaling_monotonic_pgreat() {
        let base = 21000;
        let r25 = lr2_judge_scaling(base, 25);
        let r50 = lr2_judge_scaling(base, 50);
        let r75 = lr2_judge_scaling(base, 75);
        let r100 = lr2_judge_scaling(base, 100);

        assert!(r25 <= r50);
        assert!(r50 <= r75);
        assert!(r75 <= r100);
        assert_eq!(r100, 21000);
    }

    #[test]
    fn lr2_scaling_monotonic_great() {
        let base = 60000;
        let r25 = lr2_judge_scaling(base, 25);
        let r50 = lr2_judge_scaling(base, 50);
        let r75 = lr2_judge_scaling(base, 75);
        let r100 = lr2_judge_scaling(base, 100);

        assert!(r25 <= r50);
        assert!(r50 <= r75);
        assert!(r75 <= r100);
        assert_eq!(r100, 60000);
    }

    #[test]
    fn lr2_scaling_monotonic_good() {
        let base = 120000;
        let r25 = lr2_judge_scaling(base, 25);
        let r50 = lr2_judge_scaling(base, 50);
        let r75 = lr2_judge_scaling(base, 75);
        let r100 = lr2_judge_scaling(base, 100);

        assert!(r25 <= r50);
        assert!(r50 <= r75);
        assert!(r75 <= r100);
        assert_eq!(r100, 120000);
    }

    // --- Normal Window Creation Tests ---

    #[test]
    fn sevenkeys_note_judgerank_100() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        assert_eq!(windows.len(), 5);
        assert_eq!(windows[0], [-20000, 20000]);
        assert_eq!(windows[1], [-60000, 60000]);
        assert_eq!(windows[2], [-150000, 150000]);
        assert_eq!(windows[3], [-280000, 220000]);
        assert_eq!(windows[4], [-150000, 500000]);
    }

    #[test]
    fn sevenkeys_note_judgerank_50() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 50, &rate);

        assert_eq!(windows[0], [-10000, 10000]);
        assert_eq!(windows[1], [-30000, 30000]);
        assert_eq!(windows[2], [-75000, 75000]);
        assert_eq!(windows[3], [-140000, 110000]);
        assert_eq!(windows[4], [-150000, 500000]);
    }

    #[test]
    fn sevenkeys_note_judgerank_125() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 125, &rate);

        assert_eq!(windows[0], [-25000, 25000]);
        assert_eq!(windows[1], [-75000, 75000]);
        assert_eq!(windows[2], [-187500, 187500]);
        assert_eq!(windows[3], [-350000, 275000]);
        assert_eq!(windows[4], [-150000, 500000]);
    }

    #[test]
    fn sevenkeys_scratch_judgerank_100() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Scratch, 100, &rate);

        assert_eq!(windows[0], [-30000, 30000]);
        assert_eq!(windows[1], [-70000, 70000]);
        assert_eq!(windows[2], [-160000, 160000]);
        assert_eq!(windows[3], [-290000, 230000]);
        assert_eq!(windows[4], [-160000, 500000]);
    }

    #[test]
    fn sevenkeys_longnote_end_judgerank_100() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::LongNoteEnd, 100, &rate);

        assert_eq!(windows[0], [-120000, 120000]);
        assert_eq!(windows[1], [-160000, 160000]);
        assert_eq!(windows[2], [-200000, 200000]);
        assert_eq!(windows[3], [-280000, 220000]);
    }

    // --- PMS Window Tests ---

    #[test]
    fn pms_note_judgerank_100() {
        let prop = JudgeProperty::pms();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        assert_eq!(windows[0], [-20000, 20000]);
        assert_eq!(windows[1], [-50000, 50000]);
        assert_eq!(windows[2], [-117000, 117000]);
        assert_eq!(windows[3], [-183000, 183000]);
        assert_eq!(windows[4], [-175000, 500000]);
    }

    #[test]
    fn pms_note_judgerank_50() {
        let prop = JudgeProperty::pms();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 50, &rate);

        // PG fixed, GR = 50000*50/100 = 25000, GD = 117000*50/100 = 58500, BD/MS fixed
        assert_eq!(windows[0], [-20000, 20000]);
        assert_eq!(windows[1], [-25000, 25000]);
        assert_eq!(windows[2], [-58500, 58500]);
        assert_eq!(windows[3], [-183000, 183000]);
        assert_eq!(windows[4], [-175000, 500000]);
    }

    // --- LR2 Window Tests ---

    #[test]
    fn lr2_note_judgerank_100() {
        let prop = JudgeProperty::lr2();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        assert_eq!(windows[0], [-21000, 21000]);
        assert_eq!(windows[1], [-60000, 60000]);
        assert_eq!(windows[2], [-120000, 120000]);
        assert_eq!(windows[3], [-200000, 200000]);
        assert_eq!(windows[4], [0, 1000000]);
    }

    #[test]
    fn lr2_note_judgerank_50() {
        let prop = JudgeProperty::lr2();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 50, &rate);

        assert_eq!(windows[0], [-15000, 15000]);
        assert_eq!(windows[1], [-30000, 30000]);
        assert_eq!(windows[2], [-60000, 60000]);
        assert_eq!(windows[3], [-200000, 200000]);
        assert_eq!(windows[4], [0, 1000000]);
    }

    // --- Judge Window Rate Tests ---

    #[test]
    fn judge_window_rate_scales_pg_gr_gd() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [50, 200, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        // PG: [-20000, 20000] * 50/100 = [-10000, 10000]
        // GR: [-60000, 60000] * 200/100 = [-120000, 120000]
        // GD: [-150000, 150000] * 100/100 = [-150000, 150000]
        assert_eq!(windows[0], [-10000, 10000]);
        assert_eq!(windows[1], [-120000, 120000]);
        assert_eq!(windows[2], [-150000, 150000]);
        assert_eq!(windows[3], [-280000, 220000]);
        assert_eq!(windows[4], [-150000, 500000]);
    }

    #[test]
    fn judge_window_rate_clamped_to_bd() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 100, 500];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        // GD: [-150000*500/100, 150000*500/100] = [-750000, 750000]
        // Clamped to BD: [-280000, 220000]
        assert_eq!(windows[2], [-280000, 220000]);
    }

    #[test]
    fn judge_window_rate_clamped_to_previous() {
        let prop = JudgeProperty::sevenkeys();
        let rate = [100, 10, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        // GR: [-60000*10/100, 60000*10/100] = [-6000, 6000]
        // Clamped to PG: [-20000, 20000]
        assert_eq!(windows[1], [-20000, 20000]);
    }

    // --- Empty scratch table tests ---

    #[test]
    fn pms_scratch_returns_empty() {
        let prop = JudgeProperty::pms();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Scratch, 100, &rate);
        assert!(windows.is_empty());
    }

    #[test]
    fn keyboard_scratch_returns_empty() {
        let prop = JudgeProperty::keyboard();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Scratch, 100, &rate);
        assert!(windows.is_empty());
    }

    // --- Fivekeys basic test ---

    #[test]
    fn fivekeys_note_judgerank_100() {
        let prop = JudgeProperty::fivekeys();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        assert_eq!(windows[0], [-20000, 20000]);
        assert_eq!(windows[1], [-50000, 50000]);
        assert_eq!(windows[2], [-100000, 100000]);
        assert_eq!(windows[3], [-150000, 150000]);
        assert_eq!(windows[4], [-150000, 500000]);
    }

    // --- Keyboard tests ---

    #[test]
    fn keyboard_note_judgerank_100() {
        let prop = JudgeProperty::keyboard();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 100, &rate);

        assert_eq!(windows[0], [-30000, 30000]);
        assert_eq!(windows[1], [-90000, 90000]);
        assert_eq!(windows[2], [-200000, 200000]);
        assert_eq!(windows[3], [-320000, 240000]);
        assert_eq!(windows[4], [-200000, 650000]);
    }

    #[test]
    fn keyboard_longnote_end_judgerank_100() {
        let prop = JudgeProperty::keyboard();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::LongNoteEnd, 100, &rate);

        assert_eq!(windows[0], [-160000, 25000]);
        assert_eq!(windows[1], [-200000, 75000]);
        assert_eq!(windows[2], [-260000, 140000]);
        assert_eq!(windows[3], [-320000, 240000]);
    }

    // --- LR2 inner window clamping ---

    #[test]
    fn lr2_windows_inner_clamped_to_outer() {
        let prop = JudgeProperty::lr2();
        let rate = [100, 100, 100];
        let windows = prop.judge_windows(JudgeNoteType::Note, 25, &rate);

        for j in 0..2 {
            assert!(
                windows[0][j].abs() <= windows[1][j].abs(),
                "PG[{j}] should <= GR[{j}]"
            );
            assert!(
                windows[1][j].abs() <= windows[2][j].abs(),
                "GR[{j}] should <= GD[{j}]"
            );
            assert!(
                windows[2][j].abs() <= windows[3][j].abs(),
                "GD[{j}] should <= BD[{j}]"
            );
        }
    }

    // --- Margin tests ---

    #[test]
    fn pms_has_longnote_margin() {
        let prop = JudgeProperty::pms();
        assert_eq!(prop.longnote_margin, 200000);
    }

    #[test]
    fn sevenkeys_has_no_longnote_margin() {
        let prop = JudgeProperty::sevenkeys();
        assert_eq!(prop.longnote_margin, 0);
    }

    // --- JudgeWindowRule accessor tests ---

    #[test]
    fn window_rule_judgerank_factors() {
        assert_eq!(
            JudgeWindowRule::Normal.judgerank_factors(),
            &[25, 50, 75, 100, 125]
        );
        assert_eq!(
            JudgeWindowRule::Pms.judgerank_factors(),
            &[33, 50, 70, 100, 133]
        );
        assert_eq!(
            JudgeWindowRule::Lr2.judgerank_factors(),
            &[25, 50, 75, 100, 75]
        );
    }

    #[test]
    fn window_rule_fixjudge() {
        assert_eq!(
            JudgeWindowRule::Normal.fixjudge(),
            &[false, false, false, false, true]
        );
        assert_eq!(
            JudgeWindowRule::Pms.fixjudge(),
            &[true, false, false, true, true]
        );
        assert_eq!(
            JudgeWindowRule::Lr2.fixjudge(),
            &[false, false, false, true, true]
        );
    }
}
