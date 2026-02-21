use crate::bms_player_rule::BMSPlayerRule;
use crate::judge_algorithm::JudgeAlgorithm;
use crate::judge_property::{MissCondition, NoteType};
use beatoraja_core::score_data::ScoreData;
use bms_model::bms_model::BMSModel;
use bms_model::mode::Mode;

/// HCN gauge change interval (microseconds)
const HCN_MDURATION: i64 = 200000;

/// Note judge manager
pub struct JudgeManager {
    lntype: i32,
    score: ScoreData,
    combo: i32,
    coursecombo: i32,
    coursemaxcombo: i32,
    /// Judge laser color per player per lane
    judge: Vec<Vec<i32>>,
    /// Current judge display
    judgenow: Vec<i32>,
    judgecombo: Vec<i32>,
    /// Ghost record
    ghost: Vec<i32>,
    /// Judge timing difference (ms, + is early)
    judgefast: Vec<i64>,
    mjudgefast: Vec<i64>,
    keyassign: Vec<i32>,
    sckey: Vec<i32>,
    /// Note judge table
    nmjudge: Vec<[i64; 2]>,
    mjudgestart: i64,
    mjudgeend: i64,
    /// CN end judge table
    cnendmjudge: Vec<[i64; 2]>,
    nreleasemargin: i64,
    /// Scratch judge table
    smjudge: Vec<[i64; 2]>,
    scnendmjudge: Vec<[i64; 2]>,
    sreleasemargin: i64,
    /// PMS combo condition
    combocond: Vec<bool>,
    miss: MissCondition,
    /// Judge vanish flags
    judge_vanish: Vec<bool>,
    prevmtime: i64,
    autoplay: bool,
    auto_presstime: Vec<i64>,
    auto_minduration: i64,
    algorithm: JudgeAlgorithm,
    /// Recent 100 note judge timings
    recent_judges: Vec<i64>,
    micro_recent_judges: Vec<i64>,
    recent_judges_index: usize,
    presses_since_last_autoadjust: i32,
}

impl Default for JudgeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JudgeManager {
    pub fn new() -> Self {
        JudgeManager {
            lntype: 0,
            score: ScoreData::default(),
            combo: 0,
            coursecombo: 0,
            coursemaxcombo: 0,
            judge: Vec::new(),
            judgenow: Vec::new(),
            judgecombo: Vec::new(),
            ghost: Vec::new(),
            judgefast: Vec::new(),
            mjudgefast: Vec::new(),
            keyassign: Vec::new(),
            sckey: Vec::new(),
            nmjudge: Vec::new(),
            mjudgestart: 0,
            mjudgeend: 0,
            cnendmjudge: Vec::new(),
            nreleasemargin: 0,
            smjudge: Vec::new(),
            scnendmjudge: Vec::new(),
            sreleasemargin: 0,
            combocond: Vec::new(),
            miss: MissCondition::One,
            judge_vanish: Vec::new(),
            prevmtime: 0,
            autoplay: false,
            auto_presstime: Vec::new(),
            auto_minduration: 80,
            algorithm: JudgeAlgorithm::Combo,
            recent_judges: vec![i64::MIN; 100],
            micro_recent_judges: vec![i64::MIN; 100],
            recent_judges_index: 0,
            presses_since_last_autoadjust: 0,
        }
    }

    pub fn init(&mut self, model: &BMSModel, judgeregion: i32) {
        self.prevmtime = 0;
        self.judgenow = vec![0; judgeregion as usize];
        self.judgecombo = vec![0; judgeregion as usize];
        self.judgefast = vec![0; judgeregion as usize];
        self.mjudgefast = vec![0; judgeregion as usize];

        let orgmode = model.get_mode().cloned().unwrap_or(Mode::BEAT_7K);
        self.score = ScoreData::default();
        self.score.notes = model.get_total_notes();

        self.ghost = vec![4; model.get_total_notes() as usize];
        self.lntype = model.get_lntype();

        let rule = BMSPlayerRule::get_bms_player_rule(&orgmode);
        let judgerank = model.get_judgerank();
        let key_judge_window_rate = [100, 100, 100];
        let scratch_judge_window_rate = [100, 100, 100];

        self.combocond = rule.judge.combo.clone();
        self.miss = rule.judge.miss;
        self.judge_vanish = rule.judge.judge_vanish.clone();

        self.nmjudge = rule
            .judge
            .get_judge(NoteType::Note, judgerank, &key_judge_window_rate);
        self.cnendmjudge =
            rule.judge
                .get_judge(NoteType::LongnoteEnd, judgerank, &key_judge_window_rate);
        self.nreleasemargin = rule.judge.longnote_margin;
        self.smjudge =
            rule.judge
                .get_judge(NoteType::Scratch, judgerank, &scratch_judge_window_rate);
        self.scnendmjudge = rule.judge.get_judge(
            NoteType::LongscratchEnd,
            judgerank,
            &scratch_judge_window_rate,
        );
        self.sreleasemargin = rule.judge.longscratch_margin;

        self.mjudgestart = 0;
        self.mjudgeend = 0;
        for l in &self.nmjudge {
            self.mjudgestart = self.mjudgestart.min(l[0]);
            self.mjudgeend = self.mjudgeend.max(l[1]);
        }
        for l in &self.smjudge {
            self.mjudgestart = self.mjudgestart.min(l[0]);
            self.mjudgeend = self.mjudgeend.max(l[1]);
        }

        let player_count = orgmode.player();
        let keys_per_player = orgmode.key() / player_count;
        self.judge = vec![vec![0; keys_per_player as usize + 1]; player_count as usize];

        self.recent_judges = vec![i64::MIN; 100];
        self.micro_recent_judges = vec![i64::MIN; 100];
        self.recent_judges_index = 0;
        self.presses_since_last_autoadjust = 0;
    }

    pub fn update(&mut self, _mtime: i64) {
        // TODO: Phase 7+ dependency - requires BMSPlayer, BMSPlayerInputProcessor, AudioDriver
        // This is the main judge update loop (400+ lines in Java)
        // It handles:
        // - Pass-through notes (HCN, mine notes)
        // - Autoplay processing
        // - HCN gauge changes
        // - Key press/release processing
        // - LN start/end processing
        // - BSS processing
        // - Empty POOR processing
        // - Miss POOR (late) processing
        // - LN timer processing
    }

    pub fn get_recent_judges(&self) -> &[i64] {
        &self.recent_judges
    }

    pub fn get_micro_recent_judges(&self) -> &[i64] {
        &self.micro_recent_judges
    }

    pub fn get_recent_judges_index(&self) -> usize {
        self.recent_judges_index
    }

    pub fn get_recent_judge_timing(&self, player: usize) -> i64 {
        if player < self.judgefast.len() {
            self.judgefast[player]
        } else {
            0
        }
    }

    pub fn get_recent_judge_micro_timing(&self, player: usize) -> i64 {
        if player < self.mjudgefast.len() {
            self.mjudgefast[player]
        } else {
            0
        }
    }

    pub fn get_auto_presstime(&self) -> &[i64] {
        &self.auto_presstime
    }

    pub fn get_combo(&self) -> i32 {
        self.combo
    }

    pub fn get_course_combo(&self) -> i32 {
        self.coursecombo
    }

    pub fn set_course_combo(&mut self, combo: i32) {
        self.coursecombo = combo;
    }

    pub fn get_course_maxcombo(&self) -> i32 {
        self.coursemaxcombo
    }

    pub fn set_course_maxcombo(&mut self, combo: i32) {
        self.coursemaxcombo = combo;
    }

    pub fn get_judge_time_region(&self, _lane: usize) -> &[[i64; 2]] {
        // TODO: check if lane has scratch key
        &self.nmjudge
    }

    pub fn get_score_data(&self) -> &ScoreData {
        &self.score
    }

    pub fn get_judge_count(&self, judge: i32) -> i32 {
        self.score.get_judge_count_total(judge)
    }

    pub fn get_judge_count_fast(&self, judge: i32, fast: bool) -> i32 {
        self.score.get_judge_count(judge, fast)
    }

    pub fn get_now_judge(&self, player: usize) -> i32 {
        if player < self.judgenow.len() {
            self.judgenow[player]
        } else {
            0
        }
    }

    pub fn get_now_combo(&self, player: usize) -> i32 {
        if player < self.judgecombo.len() {
            self.judgecombo[player]
        } else {
            0
        }
    }

    pub fn get_judge_table(&self, sc: bool) -> &[[i64; 2]] {
        if sc { &self.smjudge } else { &self.nmjudge }
    }

    pub fn get_past_notes(&self) -> i32 {
        self.score.passnotes
    }

    pub fn get_ghost(&self) -> &[i32] {
        &self.ghost
    }
}
