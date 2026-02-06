use serde::{Deserialize, Serialize};

use crate::model::bms_model::JudgeWindowRule;

/// Note type for judge window lookup.
/// Corresponds to JudgeProperty.NoteType in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgeNoteType {
    Note,
    LongNoteEnd,
    Scratch,
    LongScratchEnd,
}

/// Miss condition for judge.
/// Corresponds to JudgeProperty.MissCondition in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissCondition {
    /// Miss is counted once per note.
    One,
    /// Miss is always counted.
    Always,
}

/// Judge property defining judge windows per play mode.
/// Corresponds to JudgeProperty enum in beatoraja.
///
/// Each window is [late_min, early_max] in microseconds.
/// Windows are indexed as [PG, GR, GD, BD, MS] for notes/scratch,
/// and [PG, GR, GD, BD] for long note ends.
#[derive(Debug, Clone)]
pub struct JudgeProperty {
    /// Normal note judge windows.
    note: &'static [[i64; 2]],
    /// Scratch note judge windows.
    scratch: &'static [[i64; 2]],
    /// Long note end judge windows.
    longnote: &'static [[i64; 2]],
    /// Long note margin in microseconds.
    pub longnote_margin: i64,
    /// Long scratch end judge windows.
    longscratch: &'static [[i64; 2]],
    /// Long scratch margin in microseconds.
    pub longscratch_margin: i64,
    /// Combo continuation per judge level [PG, GR, GD, BD, PR, MS].
    pub combo: &'static [bool],
    /// Miss condition.
    pub miss: MissCondition,
    /// Whether each judge level causes note to vanish [PG, GR, GD, BD, PR, MS].
    pub judge_vanish: &'static [bool],
    /// Judge window rule.
    pub window_rule: &'static JudgeWindowRule,
}

impl JudgeProperty {
    pub const FIVEKEYS: Self = Self {
        note: &[
            [-20000, 20000],
            [-50000, 50000],
            [-100000, 100000],
            [-150000, 150000],
            [-150000, 500000],
        ],
        scratch: &[
            [-30000, 30000],
            [-60000, 60000],
            [-110000, 110000],
            [-160000, 160000],
            [-160000, 500000],
        ],
        longnote: &[
            [-120000, 120000],
            [-150000, 150000],
            [-200000, 200000],
            [-250000, 250000],
        ],
        longnote_margin: 0,
        longscratch: &[
            [-130000, 130000],
            [-160000, 160000],
            [-110000, 110000],
            [-260000, 260000],
        ],
        longscratch_margin: 0,
        combo: &[true, true, true, false, false, false],
        miss: MissCondition::Always,
        judge_vanish: &[true, true, true, true, true, false],
        window_rule: &JudgeWindowRule::NORMAL,
    };

    pub const SEVENKEYS: Self = Self {
        note: &[
            [-20000, 20000],
            [-60000, 60000],
            [-150000, 150000],
            [-280000, 220000],
            [-150000, 500000],
        ],
        scratch: &[
            [-30000, 30000],
            [-70000, 70000],
            [-160000, 160000],
            [-290000, 230000],
            [-160000, 500000],
        ],
        longnote: &[
            [-120000, 120000],
            [-160000, 160000],
            [-200000, 200000],
            [-280000, 220000],
        ],
        longnote_margin: 0,
        longscratch: &[
            [-130000, 130000],
            [-170000, 170000],
            [-210000, 210000],
            [-290000, 230000],
        ],
        longscratch_margin: 0,
        combo: &[true, true, true, false, false, true],
        miss: MissCondition::Always,
        judge_vanish: &[true, true, true, true, true, false],
        window_rule: &JudgeWindowRule::NORMAL,
    };

    pub const PMS: Self = Self {
        note: &[
            [-20000, 20000],
            [-50000, 50000],
            [-117000, 117000],
            [-183000, 183000],
            [-175000, 500000],
        ],
        scratch: &[],
        longnote: &[
            [-120000, 120000],
            [-150000, 150000],
            [-217000, 217000],
            [-283000, 283000],
        ],
        longnote_margin: 200000,
        longscratch: &[],
        longscratch_margin: 0,
        combo: &[true, true, true, false, false, false],
        miss: MissCondition::One,
        judge_vanish: &[true, true, true, false, true, false],
        window_rule: &JudgeWindowRule::PMS,
    };

    pub const KEYBOARD: Self = Self {
        note: &[
            [-30000, 30000],
            [-90000, 90000],
            [-200000, 200000],
            [-320000, 240000],
            [-200000, 650000],
        ],
        scratch: &[],
        longnote: &[
            [-160000, 25000],
            [-200000, 75000],
            [-260000, 140000],
            [-320000, 240000],
        ],
        longnote_margin: 0,
        longscratch: &[],
        longscratch_margin: 0,
        combo: &[true, true, true, false, false, true],
        miss: MissCondition::Always,
        judge_vanish: &[true, true, true, true, true, false],
        window_rule: &JudgeWindowRule::NORMAL,
    };

    /// Get the base windows for a given note type.
    fn base_windows(&self, note_type: JudgeNoteType) -> &[[i64; 2]] {
        match note_type {
            JudgeNoteType::Note => self.note,
            JudgeNoteType::LongNoteEnd => self.longnote,
            JudgeNoteType::Scratch => self.scratch,
            JudgeNoteType::LongScratchEnd => self.longscratch,
        }
    }

    /// Get scaled judge windows for a note type in microseconds.
    /// Corresponds to getJudge() in beatoraja.
    pub fn get_judge(
        &self,
        note_type: JudgeNoteType,
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> Vec<[i64; 2]> {
        let base = self.base_windows(note_type);
        self.window_rule.create(base, judgerank, judge_window_rate)
    }

    /// Get scaled judge windows for notes in milliseconds.
    /// Corresponds to getNoteJudge() in beatoraja.
    pub fn get_note_judge_ms(&self, judgerank: i32, judge_window_rate: &[i32; 3]) -> Vec<[i32; 2]> {
        Self::convert_to_ms(&self.get_judge(JudgeNoteType::Note, judgerank, judge_window_rate))
    }

    /// Get scaled judge windows for long note ends in milliseconds.
    /// Corresponds to getLongNoteEndJudge() in beatoraja.
    pub fn get_long_note_end_judge_ms(
        &self,
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> Vec<[i32; 2]> {
        Self::convert_to_ms(&self.get_judge(
            JudgeNoteType::LongNoteEnd,
            judgerank,
            judge_window_rate,
        ))
    }

    /// Get scaled judge windows for scratch in milliseconds.
    /// Corresponds to getScratchJudge() in beatoraja.
    pub fn get_scratch_judge_ms(
        &self,
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> Vec<[i32; 2]> {
        Self::convert_to_ms(&self.get_judge(JudgeNoteType::Scratch, judgerank, judge_window_rate))
    }

    /// Get scaled judge windows for long scratch ends in milliseconds.
    /// Corresponds to getLongScratchEndJudge() in beatoraja.
    pub fn get_long_scratch_end_judge_ms(
        &self,
        judgerank: i32,
        judge_window_rate: &[i32; 3],
    ) -> Vec<[i32; 2]> {
        Self::convert_to_ms(&self.get_judge(
            JudgeNoteType::LongScratchEnd,
            judgerank,
            judge_window_rate,
        ))
    }

    /// Convert microsecond windows to milliseconds.
    fn convert_to_ms(judge: &[[i64; 2]]) -> Vec<[i32; 2]> {
        judge
            .iter()
            .map(|pair| [(pair[0] / 1000) as i32, (pair[1] / 1000) as i32])
            .collect()
    }

    /// Get JudgeProperty from PlayerRuleType.
    pub fn from_rule_type(rule_type: crate::model::bms_model::PlayerRuleType) -> &'static Self {
        match rule_type {
            crate::model::bms_model::PlayerRuleType::FiveKeys => &Self::FIVEKEYS,
            crate::model::bms_model::PlayerRuleType::SevenKeys => &Self::SEVENKEYS,
            crate::model::bms_model::PlayerRuleType::Pms => &Self::PMS,
            crate::model::bms_model::PlayerRuleType::Keyboard => &Self::KEYBOARD,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default judge_window_rate: all 100%.
    const DEFAULT_RATE: [i32; 3] = [100, 100, 100];

    // =========================================================================
    // JudgeWindowRule::create tests
    // =========================================================================

    #[test]
    fn normal_rule_create_with_default_rank() {
        // SEVENKEYS note windows at judgerank=100 (EASY in NORMAL rule)
        let base = JudgeProperty::SEVENKEYS.note;
        let result = JudgeWindowRule::NORMAL.create(base, 100, &DEFAULT_RATE);

        // At 100%, no fixjudge except MS (index 4), so all scale by 100/100 = unchanged
        assert_eq!(result[0], [-20000, 20000]); // PG
        assert_eq!(result[1], [-60000, 60000]); // GR
        assert_eq!(result[2], [-150000, 150000]); // GD
        assert_eq!(result[3], [-280000, 220000]); // BD
        assert_eq!(result[4], [-150000, 500000]); // MS (fixjudge=true, unchanged)
    }

    #[test]
    fn normal_rule_create_with_half_rank() {
        // SEVENKEYS note windows at judgerank=50 (HARD in NORMAL rule)
        let base = JudgeProperty::SEVENKEYS.note;
        let result = JudgeWindowRule::NORMAL.create(base, 50, &DEFAULT_RATE);

        // Scaled by 50/100 = half, except MS which is fixed
        assert_eq!(result[0], [-10000, 10000]); // PG: -20000*50/100
        assert_eq!(result[1], [-30000, 30000]); // GR: -60000*50/100
        assert_eq!(result[2], [-75000, 75000]); // GD: -150000*50/100
        assert_eq!(result[3], [-140000, 110000]); // BD: -280000*50/100, 220000*50/100
        assert_eq!(result[4], [-150000, 500000]); // MS: fixed
    }

    #[test]
    fn pms_rule_create_with_default_rank() {
        // PMS note windows at judgerank=100
        // PMS fixjudge = [true, false, false, true, true]
        let base = JudgeProperty::PMS.note;
        let result = JudgeWindowRule::PMS.create(base, 100, &DEFAULT_RATE);

        // PG is fixed, GR/GD scale by 100/100=1, BD is fixed, MS is fixed
        assert_eq!(result[0], [-20000, 20000]); // PG: fixed
        assert_eq!(result[1], [-50000, 50000]); // GR
        assert_eq!(result[2], [-117000, 117000]); // GD
        assert_eq!(result[3], [-183000, 183000]); // BD: fixed
        assert_eq!(result[4], [-175000, 500000]); // MS: fixed
    }

    #[test]
    fn pms_rule_create_with_veryhard_rank() {
        // PMS note windows at judgerank=33 (VERYHARD in PMS rule)
        // fixjudge = [true, false, false, true, true]
        let base = JudgeProperty::PMS.note;
        let result = JudgeWindowRule::PMS.create(base, 33, &DEFAULT_RATE);

        // PG: fixed -> [-20000, 20000]
        // GR: -50000*33/100 = -16500, but clamped to >= PG abs (20000), so stays -20000/20000
        //   Wait, let me trace through the code more carefully.

        // Step 1: scale
        // PG: fixed -> [-20000, 20000]
        // GR: -50000*33/100 = -16500, 50000*33/100 = 16500
        // GD: -117000*33/100 = -38610, 117000*33/100 = 38610
        // BD: fixed -> [-183000, 183000]
        // MS: fixed -> [-175000, 500000]

        // Step 2: clamp (fix_judge = [true, false, false, true, true])
        // i=0: PG is fixed -> fixmin=Some(0), continue
        // i=1: GR is not fixed
        //   fixmin=Some(0), fixmax=Some(3) (BD is fixed)
        //   j=0: |GR[0]|=16500 < |PG[0]|=20000 -> GR[0] = -20000
        //   j=1: |GR[1]|=16500 < |PG[1]|=20000 -> GR[1] = 20000
        //   GR clamp against fixmax: |20000| < |183000| -> no change
        // i=2: GD is not fixed
        //   fixmin=Some(0), fixmax=Some(3)
        //   j=0: |GD[0]|=38610 > |PG[0]|=20000 -> no change from fixmin
        //   j=1: |GD[1]|=38610 > |PG[1]|=20000 -> no change from fixmin
        //   GD clamp against fixmax: |38610| < |183000| -> no change
        // i=3: BD is fixed -> fixmin=Some(3)

        // Step 3: judgeWindowRate correction (all 100, so effectively no change, just clamp)
        // i=0 PG: -20000*100/100=-20000, |20000| < |BD[0]|=183000 -> ok
        // i=1 GR: -20000*100/100=-20000, |20000| < |183000| -> ok, |20000| >= |PG[-20000]| -> ok
        // i=2 GD: -38610*100/100=-38610, |38610| < |183000| -> ok, |38610| >= |GR[-20000]| -> ok

        assert_eq!(result[0], [-20000, 20000]); // PG: fixed
        assert_eq!(result[1], [-20000, 20000]); // GR: clamped to PG
        assert_eq!(result[2], [-38610, 38610]); // GD
        assert_eq!(result[3], [-183000, 183000]); // BD: fixed
        assert_eq!(result[4], [-175000, 500000]); // MS: fixed
    }

    #[test]
    fn normal_rule_create_with_judgewindowrate() {
        // Test judgeWindowRate correction: PG rate=50, GR rate=100, GD rate=100
        let base = JudgeProperty::SEVENKEYS.note;
        let rate = [50, 100, 100];
        let result = JudgeWindowRule::NORMAL.create(base, 100, &rate);

        // Step 1: scale (100/100, all unchanged)
        // PG: [-20000, 20000], GR: [-60000, 60000], GD: [-150000, 150000], BD: [-280000, 220000], MS: fixed

        // Step 2: clamp (no fixjudge except MS, no effect in first 4)
        // (fixmin stays None through i=0..3 since none are fixed except index 4 which is outside range)

        // Step 3: judgeWindowRate
        // PG: -20000*50/100=-10000, |10000| < |BD[-280000]|=280000 -> ok
        // GR: -60000*100/100=-60000, |60000| < |280000| -> ok, |60000| >= |PG[-10000]|=10000 -> ok
        // GD: -150000*100/100=-150000, |150000| < |280000| -> ok, |150000| >= |GR[-60000]|=60000 -> ok

        assert_eq!(result[0], [-10000, 10000]); // PG: halved
        assert_eq!(result[1], [-60000, 60000]); // GR: unchanged
        assert_eq!(result[2], [-150000, 150000]); // GD: unchanged
        assert_eq!(result[3], [-280000, 220000]); // BD: unchanged
        assert_eq!(result[4], [-150000, 500000]); // MS: fixed
    }

    // =========================================================================
    // JudgeProperty base window value tests
    // =========================================================================

    #[test]
    fn fivekeys_note_windows() {
        let prop = &JudgeProperty::FIVEKEYS;
        assert_eq!(prop.note[0], [-20000, 20000]); // PG
        assert_eq!(prop.note[1], [-50000, 50000]); // GR
        assert_eq!(prop.note[2], [-100000, 100000]); // GD
        assert_eq!(prop.note[3], [-150000, 150000]); // BD
        assert_eq!(prop.note[4], [-150000, 500000]); // MS
    }

    #[test]
    fn fivekeys_scratch_windows() {
        let prop = &JudgeProperty::FIVEKEYS;
        assert_eq!(prop.scratch[0], [-30000, 30000]);
        assert_eq!(prop.scratch[1], [-60000, 60000]);
        assert_eq!(prop.scratch[2], [-110000, 110000]);
        assert_eq!(prop.scratch[3], [-160000, 160000]);
        assert_eq!(prop.scratch[4], [-160000, 500000]);
    }

    #[test]
    fn fivekeys_longnote_windows() {
        let prop = &JudgeProperty::FIVEKEYS;
        assert_eq!(prop.longnote[0], [-120000, 120000]);
        assert_eq!(prop.longnote[1], [-150000, 150000]);
        assert_eq!(prop.longnote[2], [-200000, 200000]);
        assert_eq!(prop.longnote[3], [-250000, 250000]);
        assert_eq!(prop.longnote_margin, 0);
    }

    #[test]
    fn fivekeys_longscratch_windows() {
        let prop = &JudgeProperty::FIVEKEYS;
        assert_eq!(prop.longscratch[0], [-130000, 130000]);
        assert_eq!(prop.longscratch[1], [-160000, 160000]);
        assert_eq!(prop.longscratch[2], [-110000, 110000]);
        assert_eq!(prop.longscratch[3], [-260000, 260000]);
        assert_eq!(prop.longscratch_margin, 0);
    }

    #[test]
    fn sevenkeys_note_windows() {
        let prop = &JudgeProperty::SEVENKEYS;
        assert_eq!(prop.note[0], [-20000, 20000]); // PG
        assert_eq!(prop.note[1], [-60000, 60000]); // GR
        assert_eq!(prop.note[2], [-150000, 150000]); // GD
        assert_eq!(prop.note[3], [-280000, 220000]); // BD (asymmetric)
        assert_eq!(prop.note[4], [-150000, 500000]); // MS
    }

    #[test]
    fn sevenkeys_scratch_windows() {
        let prop = &JudgeProperty::SEVENKEYS;
        assert_eq!(prop.scratch[0], [-30000, 30000]);
        assert_eq!(prop.scratch[1], [-70000, 70000]);
        assert_eq!(prop.scratch[2], [-160000, 160000]);
        assert_eq!(prop.scratch[3], [-290000, 230000]);
        assert_eq!(prop.scratch[4], [-160000, 500000]);
    }

    #[test]
    fn sevenkeys_longnote_windows() {
        let prop = &JudgeProperty::SEVENKEYS;
        assert_eq!(prop.longnote[0], [-120000, 120000]);
        assert_eq!(prop.longnote[1], [-160000, 160000]);
        assert_eq!(prop.longnote[2], [-200000, 200000]);
        assert_eq!(prop.longnote[3], [-280000, 220000]);
    }

    #[test]
    fn sevenkeys_longscratch_windows() {
        let prop = &JudgeProperty::SEVENKEYS;
        assert_eq!(prop.longscratch[0], [-130000, 130000]);
        assert_eq!(prop.longscratch[1], [-170000, 170000]);
        assert_eq!(prop.longscratch[2], [-210000, 210000]);
        assert_eq!(prop.longscratch[3], [-290000, 230000]);
    }

    #[test]
    fn pms_note_windows() {
        let prop = &JudgeProperty::PMS;
        assert_eq!(prop.note[0], [-20000, 20000]);
        assert_eq!(prop.note[1], [-50000, 50000]);
        assert_eq!(prop.note[2], [-117000, 117000]);
        assert_eq!(prop.note[3], [-183000, 183000]);
        assert_eq!(prop.note[4], [-175000, 500000]);
    }

    #[test]
    fn pms_longnote_windows() {
        let prop = &JudgeProperty::PMS;
        assert_eq!(prop.longnote[0], [-120000, 120000]);
        assert_eq!(prop.longnote[1], [-150000, 150000]);
        assert_eq!(prop.longnote[2], [-217000, 217000]);
        assert_eq!(prop.longnote[3], [-283000, 283000]);
        assert_eq!(prop.longnote_margin, 200000);
    }

    #[test]
    fn pms_has_no_scratch() {
        assert!(JudgeProperty::PMS.scratch.is_empty());
        assert!(JudgeProperty::PMS.longscratch.is_empty());
    }

    #[test]
    fn keyboard_note_windows() {
        let prop = &JudgeProperty::KEYBOARD;
        assert_eq!(prop.note[0], [-30000, 30000]);
        assert_eq!(prop.note[1], [-90000, 90000]);
        assert_eq!(prop.note[2], [-200000, 200000]);
        assert_eq!(prop.note[3], [-320000, 240000]);
        assert_eq!(prop.note[4], [-200000, 650000]);
    }

    #[test]
    fn keyboard_longnote_windows() {
        let prop = &JudgeProperty::KEYBOARD;
        assert_eq!(prop.longnote[0], [-160000, 25000]); // Asymmetric!
        assert_eq!(prop.longnote[1], [-200000, 75000]);
        assert_eq!(prop.longnote[2], [-260000, 140000]);
        assert_eq!(prop.longnote[3], [-320000, 240000]);
    }

    #[test]
    fn keyboard_has_no_scratch() {
        assert!(JudgeProperty::KEYBOARD.scratch.is_empty());
        assert!(JudgeProperty::KEYBOARD.longscratch.is_empty());
    }

    // =========================================================================
    // JudgeProperty combo/miss/vanish tests
    // =========================================================================

    #[test]
    fn fivekeys_combo_miss_vanish() {
        let prop = &JudgeProperty::FIVEKEYS;
        assert_eq!(prop.combo, &[true, true, true, false, false, false]);
        assert_eq!(prop.miss, MissCondition::Always);
        assert_eq!(prop.judge_vanish, &[true, true, true, true, true, false]);
    }

    #[test]
    fn sevenkeys_combo_miss_vanish() {
        let prop = &JudgeProperty::SEVENKEYS;
        assert_eq!(prop.combo, &[true, true, true, false, false, true]);
        assert_eq!(prop.miss, MissCondition::Always);
        assert_eq!(prop.judge_vanish, &[true, true, true, true, true, false]);
    }

    #[test]
    fn pms_combo_miss_vanish() {
        let prop = &JudgeProperty::PMS;
        assert_eq!(prop.combo, &[true, true, true, false, false, false]);
        assert_eq!(prop.miss, MissCondition::One);
        assert_eq!(prop.judge_vanish, &[true, true, true, false, true, false]);
    }

    #[test]
    fn keyboard_combo_miss_vanish() {
        let prop = &JudgeProperty::KEYBOARD;
        assert_eq!(prop.combo, &[true, true, true, false, false, true]);
        assert_eq!(prop.miss, MissCondition::Always);
        assert_eq!(prop.judge_vanish, &[true, true, true, true, true, false]);
    }

    // =========================================================================
    // JudgeProperty window rule association tests
    // =========================================================================

    #[test]
    fn fivekeys_uses_normal_rule() {
        assert_eq!(
            JudgeProperty::FIVEKEYS.window_rule.judge_rank,
            JudgeWindowRule::NORMAL.judge_rank
        );
        assert_eq!(
            JudgeProperty::FIVEKEYS.window_rule.fix_judge,
            JudgeWindowRule::NORMAL.fix_judge
        );
    }

    #[test]
    fn sevenkeys_uses_normal_rule() {
        assert_eq!(
            JudgeProperty::SEVENKEYS.window_rule.judge_rank,
            JudgeWindowRule::NORMAL.judge_rank
        );
    }

    #[test]
    fn pms_uses_pms_rule() {
        assert_eq!(
            JudgeProperty::PMS.window_rule.judge_rank,
            JudgeWindowRule::PMS.judge_rank
        );
        assert_eq!(
            JudgeProperty::PMS.window_rule.fix_judge,
            JudgeWindowRule::PMS.fix_judge
        );
    }

    #[test]
    fn keyboard_uses_normal_rule() {
        assert_eq!(
            JudgeProperty::KEYBOARD.window_rule.judge_rank,
            JudgeWindowRule::NORMAL.judge_rank
        );
    }

    // =========================================================================
    // get_judge / get_*_judge_ms integration tests
    // =========================================================================

    #[test]
    fn sevenkeys_get_note_judge_default() {
        let prop = &JudgeProperty::SEVENKEYS;
        let judge = prop.get_judge(JudgeNoteType::Note, 100, &DEFAULT_RATE);
        assert_eq!(judge[0], [-20000, 20000]);
        assert_eq!(judge[3], [-280000, 220000]);
    }

    #[test]
    fn sevenkeys_get_note_judge_ms_default() {
        let prop = &JudgeProperty::SEVENKEYS;
        let judge_ms = prop.get_note_judge_ms(100, &DEFAULT_RATE);
        assert_eq!(judge_ms[0], [-20, 20]); // PG in ms
        assert_eq!(judge_ms[1], [-60, 60]); // GR
        assert_eq!(judge_ms[3], [-280, 220]); // BD
    }

    #[test]
    fn sevenkeys_get_scratch_judge_default() {
        let prop = &JudgeProperty::SEVENKEYS;
        let judge = prop.get_judge(JudgeNoteType::Scratch, 100, &DEFAULT_RATE);
        assert_eq!(judge[0], [-30000, 30000]);
        assert_eq!(judge[3], [-290000, 230000]);
    }

    #[test]
    fn sevenkeys_get_longnote_end_judge_default() {
        let prop = &JudgeProperty::SEVENKEYS;
        let judge = prop.get_judge(JudgeNoteType::LongNoteEnd, 100, &DEFAULT_RATE);
        assert_eq!(judge[0], [-120000, 120000]);
        assert_eq!(judge[3], [-280000, 220000]);
    }

    #[test]
    fn sevenkeys_get_longscratch_end_judge_default() {
        let prop = &JudgeProperty::SEVENKEYS;
        let judge = prop.get_judge(JudgeNoteType::LongScratchEnd, 100, &DEFAULT_RATE);
        assert_eq!(judge[0], [-130000, 130000]);
        assert_eq!(judge[3], [-290000, 230000]);
    }

    #[test]
    fn from_rule_type() {
        let prop =
            JudgeProperty::from_rule_type(crate::model::bms_model::PlayerRuleType::SevenKeys);
        assert_eq!(prop.note[0], [-20000, 20000]);

        let prop = JudgeProperty::from_rule_type(crate::model::bms_model::PlayerRuleType::Pms);
        assert_eq!(prop.note[2], [-117000, 117000]);
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn very_small_judgerank_clamps_correctly() {
        // With judgerank=1, windows should be very small but still valid
        let prop = &JudgeProperty::SEVENKEYS;
        let judge = prop.get_judge(JudgeNoteType::Note, 1, &DEFAULT_RATE);

        // PG: -20000*1/100 = -200
        assert_eq!(judge[0], [-200, 200]);
        // GR: -60000*1/100 = -600, should be >= PG after clamp
        assert!(judge[1][0].abs() >= judge[0][0].abs());
    }

    #[test]
    fn judge_window_rate_clamps_to_bd() {
        // If rate is very high, PG/GR/GD should not exceed BD
        let prop = &JudgeProperty::SEVENKEYS;
        let rate = [1000, 1000, 1000]; // 10x
        let judge = prop.get_judge(JudgeNoteType::Note, 100, &rate);

        // PG: -20000*1000/100 = -200000, but |200000| < |BD[-280000]|, so ok
        // GR: -60000*1000/100 = -600000, but |600000| > |BD[-280000]|, so clamp to BD
        assert_eq!(judge[1][0], judge[3][0]); // GR clamped to BD
        assert_eq!(judge[2][0], judge[3][0]); // GD clamped to BD
    }
}
