use crate::gauge_property::GaugeProperty;
use crate::judge_property::{JudgeProperty, JudgePropertyType};
use bms_model::bms_model::{BMSModel, JudgeRankType, TotalType};
use bms_model::mode::Mode;

/// Player rule
#[derive(Clone, Debug)]
pub struct BMSPlayerRule {
    /// Gauge specification
    pub gauge: GaugeProperty,
    /// Judge specification
    pub judge: JudgeProperty,
    /// Target modes. Empty means all modes
    pub mode: Vec<Mode>,
}

impl BMSPlayerRule {
    fn new(gauge: GaugeProperty, judge_type: JudgePropertyType, modes: Vec<Mode>) -> Self {
        BMSPlayerRule {
            gauge,
            judge: judge_type.get(),
            mode: modes,
        }
    }

    pub fn get_bms_player_rule(mode: &Mode) -> BMSPlayerRule {
        let ruleset = bms_player_rule_set_lr2();
        for rule in &ruleset {
            if rule.mode.is_empty() {
                return rule.clone();
            }
            for m in &rule.mode {
                if m == mode {
                    return rule.clone();
                }
            }
        }
        // fallback: LR2
        BMSPlayerRule::new(GaugeProperty::Lr2, JudgePropertyType::Lr2, vec![])
    }

    pub fn validate(model: &mut BMSModel) {
        let mode = model.get_mode().cloned().unwrap_or(Mode::BEAT_7K);
        let rule = Self::get_bms_player_rule(&mode);
        let judgerank = model.get_judgerank();
        match model.get_judgerank_type() {
            JudgeRankType::BmsRank => {
                let new_rank = if (0..5).contains(&judgerank) {
                    rule.judge.windowrule.judgerank[judgerank as usize]
                } else {
                    rule.judge.windowrule.judgerank[2]
                };
                model.set_judgerank(new_rank);
            }
            JudgeRankType::BmsDefexrank => {
                let new_rank = if judgerank > 0 {
                    judgerank * rule.judge.windowrule.judgerank[2] / 100
                } else {
                    rule.judge.windowrule.judgerank[2]
                };
                model.set_judgerank(new_rank);
            }
            JudgeRankType::BmsonJudgerank => {
                let new_rank = if judgerank > 0 { judgerank } else { 100 };
                model.set_judgerank(new_rank);
            }
        }
        model.set_judgerank_type(JudgeRankType::BmsonJudgerank);

        let totalnotes = model.get_total_notes();
        match model.get_total_type() {
            TotalType::Bms => {
                // TOTAL undefined case
                if model.get_total() <= 0.0 {
                    model.set_total(calculate_default_total(&mode, totalnotes));
                }
            }
            TotalType::Bmson => {
                let total = calculate_default_total(&mode, totalnotes);
                let new_total = if model.get_total() > 0.0 {
                    model.get_total() / 100.0 * total
                } else {
                    total
                };
                model.set_total(new_total);
            }
        }
        model.set_total_type(TotalType::Bms);
    }
}

fn calculate_default_total(_mode: &Mode, totalnotes: i32) -> f64 {
    160.0 + (totalnotes as f64 + (totalnotes as f64 - 400.0).max(0.0).min(200.0)) * 0.16
}

/// BMSPlayerRuleSet::LR2
fn bms_player_rule_set_lr2() -> Vec<BMSPlayerRule> {
    vec![BMSPlayerRule::new(
        GaugeProperty::Lr2,
        JudgePropertyType::Lr2,
        vec![],
    )]
}

/// BMSPlayerRuleSet::Beatoraja
#[allow(dead_code)]
fn bms_player_rule_set_beatoraja() -> Vec<BMSPlayerRule> {
    vec![
        BMSPlayerRule::new(
            GaugeProperty::FiveKeys,
            JudgePropertyType::FiveKeys,
            vec![Mode::BEAT_5K, Mode::BEAT_10K],
        ),
        BMSPlayerRule::new(
            GaugeProperty::SevenKeys,
            JudgePropertyType::SevenKeys,
            vec![Mode::BEAT_7K, Mode::BEAT_14K],
        ),
        BMSPlayerRule::new(
            GaugeProperty::Pms,
            JudgePropertyType::Pms,
            vec![Mode::POPN_5K, Mode::POPN_9K],
        ),
        BMSPlayerRule::new(
            GaugeProperty::Keyboard,
            JudgePropertyType::Keyboard,
            vec![Mode::KEYBOARD_24K, Mode::KEYBOARD_24K_DOUBLE],
        ),
        BMSPlayerRule::new(
            GaugeProperty::SevenKeys,
            JudgePropertyType::SevenKeys,
            vec![],
        ),
    ]
}
