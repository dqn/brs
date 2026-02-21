use beatoraja_core::player_config::PlayerConfig;
use bms_model::bms_model::BMSModel;
use bms_model::mode::Mode;

use crate::pattern_modifier::{AssistLevel, PatternModifier, PatternModifierBase};
use crate::random::Random;
use crate::randomizer::Randomizer;

pub struct NoteShuffleModifier {
    pub base: PatternModifierBase,
    randomizer: Randomizer,
    is_scratch_lane_modify: bool,
}

impl NoteShuffleModifier {
    pub fn new(r: Random, player: i32, mode: &Mode, config: &PlayerConfig) -> Self {
        let randomizer = Randomizer::create_with_side(r, player, mode, config);
        NoteShuffleModifier {
            base: PatternModifierBase::with_player(player),
            randomizer,
            is_scratch_lane_modify: r.is_scratch_lane_modify(),
        }
    }
}

impl PatternModifier for NoteShuffleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.randomizer.set_random_seed(self.base.seed);
        let mode = match model.get_mode() {
            Some(m) => m.clone(),
            None => return,
        };
        let keys = self.get_keys(&mode, self.base.player, self.is_scratch_lane_modify);
        self.randomizer.set_modify_lanes(&keys);
        let timelines = model.get_all_time_lines_mut();
        for tl in timelines.iter_mut() {
            if tl.exist_note() || tl.exist_hidden_note() {
                self.randomizer.permutate(tl);
            }
        }
        self.base.assist = self.randomizer.get_assist_level();
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
