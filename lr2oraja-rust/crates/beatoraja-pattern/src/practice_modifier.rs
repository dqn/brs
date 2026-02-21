use bms_model::bms_model::BMSModel;

use crate::pattern_modifier::{
    AssistLevel, PatternModifier, PatternModifierBase, move_to_background,
};

pub struct PracticeModifier {
    pub base: PatternModifierBase,
    start: i64,
    end: i64,
}

impl PracticeModifier {
    pub fn new(start: i64, end: i64) -> Self {
        PracticeModifier {
            base: PatternModifierBase::with_assist(AssistLevel::Assist),
            start,
            end,
        }
    }
}

impl PatternModifier for PracticeModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        let totalnotes = model.get_total_notes();
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);

        let timelines = model.get_all_time_lines_mut();
        let tl_len = timelines.len();
        for tl_idx in 0..tl_len {
            let time = timelines[tl_idx].get_time();
            for i in 0..mode_key {
                if (time as i64) < self.start || (time as i64) >= self.end {
                    move_to_background(timelines, tl_idx, i);
                }
            }
        }

        let new_total_notes = model.get_total_notes();
        if totalnotes > 0 {
            let total = model.get_total();
            model.set_total(total * new_total_notes as f64 / totalnotes as f64);
        }
    }

    fn get_assist_level(&self) -> AssistLevel {
        self.base.assist
    }

    fn set_assist_level(&mut self, assist: AssistLevel) {
        self.base.assist = assist;
    }

    fn get_seed(&self) -> i64 {
        self.base.seed
    }

    fn set_seed(&mut self, seed: i64) {
        if seed >= 0 {
            self.base.seed = seed;
        }
    }

    fn get_player(&self) -> i32 {
        self.base.player
    }
}
