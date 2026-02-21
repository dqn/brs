use bms_model::bms_model::BMSModel;

use crate::pattern_modifier::{AssistLevel, PatternModifier, PatternModifierBase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Remove,
    Add,
}

impl Mode {
    pub fn values() -> &'static [Mode] {
        &[Mode::Remove, Mode::Add]
    }

    pub fn from_index(index: i32) -> Mode {
        let values = Self::values();
        if index >= 0 && (index as usize) < values.len() {
            values[index as usize]
        } else {
            Mode::Remove
        }
    }
}

pub struct ScrollSpeedModifier {
    pub base: PatternModifierBase,
    mode: Mode,
    section: i32,
    rate: f64,
}

impl Default for ScrollSpeedModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollSpeedModifier {
    pub fn new() -> Self {
        ScrollSpeedModifier {
            base: PatternModifierBase::new(),
            mode: Mode::Remove,
            section: 4,
            rate: 0.5,
        }
    }

    pub fn with_params(mode: i32, section: i32, scrollrate: f64) -> Self {
        ScrollSpeedModifier {
            base: PatternModifierBase::new(),
            mode: Mode::from_index(mode),
            section,
            rate: scrollrate,
        }
    }
}

impl PatternModifier for ScrollSpeedModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        if self.mode == Mode::Remove {
            let mut assist = AssistLevel::None;
            let timelines = model.get_all_time_lines_mut();

            let start_bpm = timelines[0].get_bpm();
            let start_scroll = timelines[0].get_scroll();

            for tl in timelines.iter_mut() {
                if tl.get_bpm() != start_bpm
                    || tl.get_scroll() != start_scroll
                    || tl.get_stop() != 0
                {
                    assist = AssistLevel::LightAssist;
                }
                tl.set_section(start_bpm * tl.get_micro_time() as f64 / 240000000.0);
                tl.set_stop(0);
                tl.set_bpm(start_bpm);
                tl.set_scroll(start_scroll);
            }
            self.base.assist = assist;
        } else {
            let timelines = model.get_all_time_lines_mut();
            let base = timelines[0].get_scroll();
            let mut current = base;
            let mut sectioncount = 0;
            for tl in timelines.iter_mut() {
                if tl.get_section_line() {
                    sectioncount += 1;
                    if self.section == sectioncount {
                        current =
                            base * (1.0 + rand::random::<f64>() * self.rate * 2.0 - self.rate);
                        sectioncount = 0;
                    }
                }
                tl.set_scroll(current);
            }
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
