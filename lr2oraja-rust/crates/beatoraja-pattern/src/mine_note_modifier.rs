use bms_model::bms_model::BMSModel;
use bms_model::note::Note;

use crate::pattern_modifier::{AssistLevel, PatternModifier, PatternModifierBase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Remove,
    AddRandom,
    AddNear,
    AddBlank,
}

impl Mode {
    pub fn values() -> &'static [Mode] {
        &[Mode::Remove, Mode::AddRandom, Mode::AddNear, Mode::AddBlank]
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

pub struct MineNoteModifier {
    pub base: PatternModifierBase,
    exists: bool,
    mode: Mode,
    damage: i32,
}

impl Default for MineNoteModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl MineNoteModifier {
    pub fn new() -> Self {
        MineNoteModifier {
            base: PatternModifierBase::new(),
            exists: false,
            mode: Mode::Remove,
            damage: 10,
        }
    }

    pub fn with_mode(mode: i32) -> Self {
        MineNoteModifier {
            base: PatternModifierBase::new(),
            exists: false,
            mode: Mode::from_index(mode),
            damage: 10,
        }
    }

    pub fn mine_note_exists(&self) -> bool {
        self.exists
    }
}

impl PatternModifier for MineNoteModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);

        if self.mode == Mode::Remove {
            let mut assist = AssistLevel::None;
            let timelines = model.get_all_time_lines_mut();
            for tl in timelines.iter_mut() {
                for lane in 0..mode_key {
                    if let Some(note) = tl.get_note(lane)
                        && note.is_mine()
                    {
                        assist = AssistLevel::LightAssist;
                        self.exists = true;
                        tl.set_note(lane, None);
                    }
                }
            }
            self.base.assist = assist;
        } else {
            let timelines = model.get_all_time_lines_mut();
            let tl_len = timelines.len();
            let mut ln = vec![false; mode_key as usize];
            let mut blank = vec![false; mode_key as usize];

            for i in 0..tl_len {
                for key in 0..mode_key as usize {
                    let note = timelines[i].get_note(key as i32);
                    if let Some(n) = note
                        && n.is_long()
                    {
                        ln[key] = !n.is_end();
                    }
                    blank[key] = !ln[key] && timelines[i].get_note(key as i32).is_none();
                }

                for key in 0..mode_key as usize {
                    if blank[key] {
                        match self.mode {
                            Mode::AddRandom => {
                                if rand::random::<f64>() > 0.9 {
                                    timelines[i].set_note(
                                        key as i32,
                                        Some(Note::new_mine(-1, self.damage as f64)),
                                    );
                                }
                            }
                            Mode::AddNear => {
                                if (key > 0 && !blank[key - 1])
                                    || (key < mode_key as usize - 1 && !blank[key + 1])
                                {
                                    timelines[i].set_note(
                                        key as i32,
                                        Some(Note::new_mine(-1, self.damage as f64)),
                                    );
                                }
                            }
                            Mode::AddBlank => {
                                timelines[i].set_note(
                                    key as i32,
                                    Some(Note::new_mine(-1, self.damage as f64)),
                                );
                            }
                            _ => {}
                        }
                    }
                }
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
