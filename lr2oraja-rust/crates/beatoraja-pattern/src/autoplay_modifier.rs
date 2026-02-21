use bms_model::bms_model::BMSModel;

use crate::pattern_modifier::{
    AssistLevel, PatternModifier, PatternModifierBase, move_to_background,
};

pub struct AutoplayModifier {
    pub base: PatternModifierBase,
    lanes: Vec<i32>,
    margin: i32,
}

impl AutoplayModifier {
    pub fn new(lanes: Vec<i32>) -> Self {
        Self::with_margin(lanes, 0)
    }

    pub fn with_margin(lanes: Vec<i32>, margin: i32) -> Self {
        AutoplayModifier {
            base: PatternModifierBase::new(),
            lanes,
            margin,
        }
    }
}

impl PatternModifier for AutoplayModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        let mut assist = AssistLevel::None;
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0) as usize;

        let timelines = model.get_all_time_lines_mut();
        let tl_len = timelines.len();
        let mut pos = 0usize;
        let mut lns = vec![false; mode_key];

        for i in 0..tl_len {
            let mut remove = false;

            if self.margin > 0 {
                while timelines[pos].get_time() < timelines[i].get_time() - self.margin {
                    for lane in 0..lns.len() {
                        if let Some(note) = timelines[pos].get_note(lane as i32)
                            && note.is_long()
                        {
                            lns[lane] = !note.is_end();
                        }
                    }
                    pos += 1;
                }
                let mut endtime = timelines[i].get_time() + self.margin;
                for &lane in &self.lanes {
                    if let Some(note) = timelines[i].get_note(lane)
                        && note.is_long()
                        && !note.is_end()
                    {
                        endtime = endtime.max(note.get_time() + self.margin);
                    }
                }

                for j in pos..tl_len {
                    if timelines[j].get_time() >= endtime {
                        break;
                    }
                    for lane in 0..mode_key {
                        let mut b = true;
                        for &rlane in &self.lanes {
                            if lane as i32 == rlane {
                                b = false;
                                break;
                            }
                        }
                        if b && (timelines[j].get_note(lane as i32).is_some() || lns[lane]) {
                            remove = true;
                            break;
                        }
                    }
                    if remove {
                        break;
                    }
                }
            } else {
                remove = true;
            }

            if remove {
                for &lane in &self.lanes {
                    if timelines[i].exist_note_at(lane) {
                        assist = AssistLevel::Assist;
                    }
                    move_to_background(timelines, i, lane);
                }
            }
        }
        self.base.assist = assist;
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
