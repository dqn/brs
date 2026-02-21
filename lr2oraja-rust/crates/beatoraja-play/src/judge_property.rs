/// Judge property configuration
#[derive(Clone, Debug)]
pub struct JudgeProperty {
    /// Normal note judge windows: PG, GR, GD, BD, MS order, {LATE lower, EARLY upper} pairs
    note: Vec<[i64; 2]>,
    /// Scratch note judge windows
    scratch: Vec<[i64; 2]>,
    /// Long note end judge windows
    longnote: Vec<[i64; 2]>,
    pub longnote_margin: i64,
    /// Long scratch end judge windows
    longscratch: Vec<[i64; 2]>,
    pub longscratch_margin: i64,
    /// Combo continuation per judge
    pub combo: Vec<bool>,
    /// Miss condition
    pub miss: MissCondition,
    /// Whether each judge causes note to vanish
    pub judge_vanish: Vec<bool>,
    pub windowrule: JudgeWindowRule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissCondition {
    One,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteType {
    Note,
    LongnoteEnd,
    Scratch,
    LongscratchEnd,
}

#[derive(Clone, Debug)]
pub struct JudgeWindowRule {
    pub judgerank: Vec<i32>,
    pub fixjudge: Vec<bool>,
    pub rule_type: JudgeWindowRuleType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgeWindowRuleType {
    Normal,
    Pms,
    Lr2,
}

static LR2_SCALING: [[i64; 5]; 4] = [
    [0, 0, 0, 0, 0],
    [0, 8000, 15000, 18000, 21000],
    [0, 24000, 30000, 40000, 60000],
    [0, 40000, 60000, 100000, 120000],
];

fn lr2_judge_scaling(mut base: i64, judgerank: i32) -> i64 {
    let mut sign: i64 = 1;
    if base < 0 {
        base = -base;
        sign = -1;
    }
    if judgerank >= 100 {
        return sign * base * judgerank as i64 / 100;
    }
    let last = LR2_SCALING[0].len() - 1;
    let judgeindex = judgerank as usize / 25;
    let mut s: usize = 0;
    while s < LR2_SCALING.len() && base >= LR2_SCALING[s][last] {
        s += 1;
    }
    let (x1, x2, d): (i64, i64, i64);
    if s < LR2_SCALING.len() {
        let n = base - LR2_SCALING[s - 1][last];
        d = LR2_SCALING[s][last] - LR2_SCALING[s - 1][last];
        x1 = d * LR2_SCALING[s - 1][judgeindex]
            + n * (LR2_SCALING[s][judgeindex] - LR2_SCALING[s - 1][judgeindex]);
        x2 = d * LR2_SCALING[s - 1][judgeindex + 1]
            + n * (LR2_SCALING[s][judgeindex + 1] - LR2_SCALING[s - 1][judgeindex + 1]);
    } else {
        let n = base;
        d = LR2_SCALING[s - 1][last];
        x1 = n * LR2_SCALING[s - 1][judgeindex];
        x2 = n * LR2_SCALING[s - 1][judgeindex + 1];
    }
    sign * (x1 + (judgerank as i64 - judgeindex as i64 * 25) * (x2 - x1) / 25) / d
}

fn create_lr2(org: &[[i64; 2]], judgerank: i32, judge_window_rate: &[i32]) -> Vec<[i64; 2]> {
    let mut judge: Vec<[i64; 2]> = org.to_vec();

    // Only change pgreat, great, good
    let fixmax = 3;
    for i in 0..fixmax {
        for j in 0..2 {
            judge[i][j] = lr2_judge_scaling(org[i][j], judgerank);
        }
    }

    // Correction if we exceed the bad windows
    for i in (0..fixmax).rev() {
        for j in 0..2 {
            if judge[i][j].abs() > judge[i + 1][j].abs() {
                judge[i][j] = judge[i + 1][j];
            }
        }
    }

    // judgeWindowRate correction
    let limit = std::cmp::min(org.len(), 3);
    for i in 0..limit {
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

impl JudgeWindowRule {
    fn create_normal(
        &self,
        org: &[[i64; 2]],
        judgerank: i32,
        judge_window_rate: &[i32],
    ) -> Vec<[i64; 2]> {
        let mut judge: Vec<[i64; 2]> = vec![[0, 0]; org.len()];
        for i in 0..org.len() {
            for j in 0..2 {
                judge[i][j] = if self.fixjudge[i] {
                    org[i][j]
                } else {
                    org[i][j] * judgerank as i64 / 100
                };
            }
        }

        let mut fixmin: i32 = -1;
        let limit = std::cmp::min(org.len(), 4);
        for i in 0..limit {
            if self.fixjudge[i] {
                fixmin = i as i32;
                continue;
            }
            let mut fixmax: i32 = -1;
            for j2 in (i + 1)..4 {
                if self.fixjudge[j2] {
                    fixmax = j2 as i32;
                    break;
                }
            }

            for j in 0..2 {
                if fixmin != -1 && judge[i][j].abs() < judge[fixmin as usize][j].abs() {
                    judge[i][j] = judge[fixmin as usize][j];
                }
                if fixmax != -1 && judge[i][j].abs() > judge[fixmax as usize][j].abs() {
                    judge[i][j] = judge[fixmax as usize][j];
                }
            }
        }

        // judgeWindowRate correction
        let limit2 = std::cmp::min(org.len(), 3);
        for i in 0..limit2 {
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

    pub fn create(
        &self,
        org: &[[i64; 2]],
        judgerank: i32,
        judge_window_rate: &[i32],
    ) -> Vec<[i64; 2]> {
        match self.rule_type {
            JudgeWindowRuleType::Lr2 => create_lr2(org, judgerank, judge_window_rate),
            _ => self.create_normal(org, judgerank, judge_window_rate),
        }
    }
}

fn convert_milli(judge: &[[i64; 2]]) -> Vec<Vec<i32>> {
    let mut mjudge: Vec<Vec<i32>> = Vec::with_capacity(judge.len());
    for row in judge {
        let mut mrow = Vec::with_capacity(row.len());
        for &val in row {
            mrow.push((val / 1000) as i32);
        }
        mjudge.push(mrow);
    }
    mjudge
}

impl JudgeProperty {
    pub fn get_note_judge(&self, judgerank: i32, judge_window_rate: &[i32]) -> Vec<Vec<i32>> {
        convert_milli(
            &self
                .windowrule
                .create(&self.note, judgerank, judge_window_rate),
        )
    }

    pub fn get_long_note_end_judge(
        &self,
        judgerank: i32,
        judge_window_rate: &[i32],
    ) -> Vec<Vec<i32>> {
        convert_milli(
            &self
                .windowrule
                .create(&self.longnote, judgerank, judge_window_rate),
        )
    }

    pub fn get_scratch_judge(&self, judgerank: i32, judge_window_rate: &[i32]) -> Vec<Vec<i32>> {
        convert_milli(
            &self
                .windowrule
                .create(&self.scratch, judgerank, judge_window_rate),
        )
    }

    pub fn get_long_scratch_end_judge(
        &self,
        judgerank: i32,
        judge_window_rate: &[i32],
    ) -> Vec<Vec<i32>> {
        convert_milli(
            &self
                .windowrule
                .create(&self.longscratch, judgerank, judge_window_rate),
        )
    }

    pub fn get_judge(
        &self,
        notetype: NoteType,
        judgerank: i32,
        judge_window_rate: &[i32],
    ) -> Vec<[i64; 2]> {
        match notetype {
            NoteType::Note => self
                .windowrule
                .create(&self.note, judgerank, judge_window_rate),
            NoteType::LongnoteEnd => {
                self.windowrule
                    .create(&self.longnote, judgerank, judge_window_rate)
            }
            NoteType::Scratch => {
                self.windowrule
                    .create(&self.scratch, judgerank, judge_window_rate)
            }
            NoteType::LongscratchEnd => {
                self.windowrule
                    .create(&self.longscratch, judgerank, judge_window_rate)
            }
        }
    }
}

// Pre-defined JudgeWindowRules
fn rule_normal() -> JudgeWindowRule {
    JudgeWindowRule {
        judgerank: vec![25, 50, 75, 100, 125],
        fixjudge: vec![false, false, false, false, true],
        rule_type: JudgeWindowRuleType::Normal,
    }
}

fn rule_pms() -> JudgeWindowRule {
    JudgeWindowRule {
        judgerank: vec![33, 50, 70, 100, 133],
        fixjudge: vec![true, false, false, true, true],
        rule_type: JudgeWindowRuleType::Pms,
    }
}

fn rule_lr2() -> JudgeWindowRule {
    JudgeWindowRule {
        judgerank: vec![25, 50, 75, 100, 75],
        fixjudge: vec![false, false, false, true, true],
        rule_type: JudgeWindowRuleType::Lr2,
    }
}

// Pre-defined JudgeProperty variants
pub fn fivekeys() -> JudgeProperty {
    JudgeProperty {
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
        combo: vec![true, true, true, false, false, false],
        miss: MissCondition::Always,
        judge_vanish: vec![true, true, true, true, true, false],
        windowrule: rule_normal(),
    }
}

pub fn sevenkeys() -> JudgeProperty {
    JudgeProperty {
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
        combo: vec![true, true, true, false, false, true],
        miss: MissCondition::Always,
        judge_vanish: vec![true, true, true, true, true, false],
        windowrule: rule_normal(),
    }
}

pub fn pms() -> JudgeProperty {
    JudgeProperty {
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
        combo: vec![true, true, true, false, false, false],
        miss: MissCondition::One,
        judge_vanish: vec![true, true, true, false, true, false],
        windowrule: rule_pms(),
    }
}

pub fn keyboard() -> JudgeProperty {
    JudgeProperty {
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
        combo: vec![true, true, true, false, false, true],
        miss: MissCondition::Always,
        judge_vanish: vec![true, true, true, true, true, false],
        windowrule: rule_normal(),
    }
}

pub fn lr2() -> JudgeProperty {
    JudgeProperty {
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
        combo: vec![true, true, true, false, false, true],
        miss: MissCondition::Always,
        judge_vanish: vec![true, true, true, true, true, false],
        windowrule: rule_lr2(),
    }
}

/// Enum-like accessor for JudgeProperty variants
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgePropertyType {
    FiveKeys,
    SevenKeys,
    Pms,
    Keyboard,
    Lr2,
}

impl JudgePropertyType {
    pub fn get(&self) -> JudgeProperty {
        match self {
            JudgePropertyType::FiveKeys => fivekeys(),
            JudgePropertyType::SevenKeys => sevenkeys(),
            JudgePropertyType::Pms => pms(),
            JudgePropertyType::Keyboard => keyboard(),
            JudgePropertyType::Lr2 => lr2(),
        }
    }
}
