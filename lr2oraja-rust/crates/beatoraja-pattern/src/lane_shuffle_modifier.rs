use std::collections::HashSet;

use bms_model::bms_model::BMSModel;
use bms_model::mode::Mode;
use bms_model::note::Note;
use rand::prelude::*;
use rand::rngs::StdRng;

use crate::pattern_modifier::{AssistLevel, PatternModifier, PatternModifierBase};
use crate::stubs::RandomTrainer;

fn lane_shuffle_modify(
    base: &mut PatternModifierBase,
    model: &mut BMSModel,
    is_scratch_lane_modify: bool,
    _show_shuffle_pattern: bool,
    make_random: impl FnOnce(&[i32], &BMSModel, i64) -> Vec<i32>,
) -> Vec<i32> {
    let mode = match model.get_mode() {
        Some(m) => m.clone(),
        None => return Vec::new(),
    };
    let keys = PatternModifierBase::get_keys_static(&mode, base.player, is_scratch_lane_modify);
    if keys.is_empty() {
        return Vec::new();
    }
    let random = make_random(&keys, model, base.seed);

    // Random Trainer History
    if random.len() == 8 {
        let mut random_sb = String::new();
        for i in 0..random.len() - 1 {
            random_sb.push_str(&(random[i] + 1).to_string());
        }
        let rt = RandomTrainer::new();
        RandomTrainer::add_random_history(
            rt.new_random_history_entry(model.get_title(), &random_sb),
        );
    }

    let lanes = mode.key() as usize;
    let timelines = model.get_all_time_lines_mut();
    for index in 0..timelines.len() {
        let tl = &timelines[index];
        if tl.exist_note() || tl.exist_hidden_note() {
            let mut notes: Vec<Option<Note>> = Vec::with_capacity(lanes);
            let mut hnotes: Vec<Option<Note>> = Vec::with_capacity(lanes);
            let mut cloned: Vec<bool> = vec![false; lanes];
            for i in 0..lanes {
                notes.push(timelines[index].get_note(i as i32).cloned());
                hnotes.push(timelines[index].get_hidden_note(i as i32).cloned());
            }
            for i in 0..lanes {
                let m = if i < random.len() {
                    random[i] as usize
                } else {
                    i
                };
                if cloned[m] {
                    if let Some(ref note) = notes[m] {
                        if note.is_long() && note.is_end() {
                            // Find pair in previous timelines
                            let note_section = note.get_section();
                            for j in (0..index).rev() {
                                if let Some(pair_note) = &notes[m]
                                    && pair_note.is_long()
                                    && let Some(pair_idx) = pair_note.get_pair()
                                {
                                    let _ = pair_idx; // unused in this context
                                }
                                let prev_section = timelines[j].get_section();
                                if (prev_section - note_section).abs() < f64::EPSILON {
                                    // Get the LN from previous timeline at lane i
                                    if let Some(ln_prev) = timelines[j].get_note(i as i32).cloned()
                                        && ln_prev.is_long()
                                    {
                                        // Create the end note clone
                                        timelines[index]
                                            .set_note(i as i32, Some(notes[m].clone().unwrap()));
                                        break;
                                    }
                                    break;
                                }
                            }
                        } else {
                            timelines[index].set_note(i as i32, Some(note.clone()));
                        }
                    } else {
                        timelines[index].set_note(i as i32, None);
                    }
                    if let Some(ref hn) = hnotes[m] {
                        timelines[index].set_hidden_note(i as i32, Some(hn.clone()));
                    } else {
                        timelines[index].set_hidden_note(i as i32, None);
                    }
                } else {
                    timelines[index].set_note(i as i32, notes[m].take());
                    timelines[index].set_hidden_note(i as i32, hnotes[m].take());
                    cloned[m] = true;
                }
            }
        }
    }

    random
}

impl PatternModifierBase {
    pub fn get_keys_static(mode: &Mode, player: i32, contains_scratch: bool) -> Vec<i32> {
        if player >= mode.player() {
            return Vec::new();
        }
        let startkey = mode.key() * player / mode.player();
        (startkey..startkey + mode.key() / mode.player())
            .filter(|&i| contains_scratch || !mode.is_scratch_key(i))
            .collect()
    }
}

// ---- LaneMirrorShuffleModifier ----

pub struct LaneMirrorShuffleModifier {
    pub base: PatternModifierBase,
    pub is_scratch_lane_modify: bool,
    random: Vec<i32>,
}

impl LaneMirrorShuffleModifier {
    pub fn new(player: i32, is_scratch_lane_modify: bool) -> Self {
        let mut base = PatternModifierBase::with_player(player);
        base.assist = if is_scratch_lane_modify {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        };
        LaneMirrorShuffleModifier {
            base,
            is_scratch_lane_modify,
            random: Vec::new(),
        }
    }

    fn make_random(keys: &[i32], model: &BMSModel, _seed: i64) -> Vec<i32> {
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);
        let mut result: Vec<i32> = (0..mode_key).collect();
        for lane in 0..keys.len() {
            result[keys[lane] as usize] = keys[keys.len() - 1 - lane];
        }
        result
    }
}

impl PatternModifier for LaneMirrorShuffleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.random = lane_shuffle_modify(
            &mut self.base,
            model,
            self.is_scratch_lane_modify,
            false,
            Self::make_random,
        );
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

// ---- LaneRotateShuffleModifier ----

pub struct LaneRotateShuffleModifier {
    pub base: PatternModifierBase,
    pub is_scratch_lane_modify: bool,
    random: Vec<i32>,
}

impl LaneRotateShuffleModifier {
    pub fn new(player: i32, is_scratch_lane_modify: bool) -> Self {
        let mut base = PatternModifierBase::with_player(player);
        base.assist = if is_scratch_lane_modify {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        };
        LaneRotateShuffleModifier {
            base,
            is_scratch_lane_modify,
            random: Vec::new(),
        }
    }

    fn make_random(keys: &[i32], model: &BMSModel, seed: i64) -> Vec<i32> {
        let mut rand = StdRng::seed_from_u64(seed as u64);
        let inc = rand.gen_range(0..2) == 1;
        let start = rand.gen_range(0..(keys.len() - 1)) + if inc { 1 } else { 0 };
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);
        let mut result: Vec<i32> = (0..mode_key).collect();
        let mut rlane = start;
        for lane in 0..keys.len() {
            result[keys[lane] as usize] = keys[rlane];
            if inc {
                rlane = (rlane + 1) % keys.len();
            } else {
                rlane = (rlane + keys.len() - 1) % keys.len();
            }
        }
        result
    }
}

impl PatternModifier for LaneRotateShuffleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.random = lane_shuffle_modify(
            &mut self.base,
            model,
            self.is_scratch_lane_modify,
            true,
            Self::make_random,
        );
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

// ---- LaneRandomShuffleModifier ----

pub struct LaneRandomShuffleModifier {
    pub base: PatternModifierBase,
    pub is_scratch_lane_modify: bool,
    random: Vec<i32>,
}

impl LaneRandomShuffleModifier {
    pub fn new(player: i32, is_scratch_lane_modify: bool) -> Self {
        let mut base = PatternModifierBase::with_player(player);
        base.assist = if is_scratch_lane_modify {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        };
        LaneRandomShuffleModifier {
            base,
            is_scratch_lane_modify,
            random: Vec::new(),
        }
    }

    fn make_random(keys: &[i32], model: &BMSModel, seed: i64) -> Vec<i32> {
        let mut rand = StdRng::seed_from_u64(seed as u64);
        let mut l: Vec<i32> = keys.to_vec();
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);
        let mut result: Vec<i32> = (0..mode_key).collect();
        for lane in 0..keys.len() {
            let r = rand.gen_range(0..l.len());
            result[keys[lane] as usize] = l[r];
            l.remove(r);
        }
        result
    }
}

impl PatternModifier for LaneRandomShuffleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.random = lane_shuffle_modify(
            &mut self.base,
            model,
            self.is_scratch_lane_modify,
            true,
            Self::make_random,
        );
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

// ---- PlayerFlipModifier ----

pub struct PlayerFlipModifier {
    pub base: PatternModifierBase,
    random: Vec<i32>,
}

impl Default for PlayerFlipModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerFlipModifier {
    pub fn new() -> Self {
        let mut base = PatternModifierBase::with_player(0);
        base.assist = AssistLevel::None;
        PlayerFlipModifier {
            base,
            random: Vec::new(),
        }
    }

    fn make_random(_keys: &[i32], model: &BMSModel, _seed: i64) -> Vec<i32> {
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0) as usize;
        let mut result: Vec<i32> = (0..mode_key as i32).collect();
        if model.get_mode().map(|m| m.player()).unwrap_or(0) == 2 {
            for i in 0..result.len() {
                result[i] = ((i + result.len() / 2) % result.len()) as i32;
            }
        }
        result
    }
}

impl PatternModifier for PlayerFlipModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.random = lane_shuffle_modify(&mut self.base, model, true, false, Self::make_random);
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

// ---- PlayerBattleModifier ----

pub struct PlayerBattleModifier {
    pub base: PatternModifierBase,
    random: Vec<i32>,
}

impl Default for PlayerBattleModifier {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerBattleModifier {
    pub fn new() -> Self {
        let mut base = PatternModifierBase::with_player(0);
        base.assist = AssistLevel::Assist;
        PlayerBattleModifier {
            base,
            random: Vec::new(),
        }
    }

    fn make_random(keys: &[i32], model: &BMSModel, _seed: i64) -> (Vec<i32>, AssistLevel) {
        if model.get_mode().map(|m| m.player()).unwrap_or(0) == 1 {
            (Vec::new(), AssistLevel::Assist)
        } else {
            let mut result = vec![0i32; keys.len() * 2];
            result[..keys.len()].copy_from_slice(keys);
            result[keys.len()..keys.len() * 2].copy_from_slice(keys);
            (result, AssistLevel::Assist)
        }
    }
}

impl PatternModifier for PlayerBattleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        let mode = match model.get_mode() {
            Some(m) => m.clone(),
            None => return,
        };
        let keys = PatternModifierBase::get_keys_static(&mode, self.base.player, true);
        if keys.is_empty() {
            return;
        }
        let (random, assist) = Self::make_random(&keys, model, self.base.seed);
        self.base.assist = assist;
        if random.is_empty() {
            return;
        }

        let lanes = mode.key() as usize;
        let timelines = model.get_all_time_lines_mut();
        for tl in timelines.iter_mut() {
            if tl.exist_note() || tl.exist_hidden_note() {
                let mut notes: Vec<Option<Note>> = Vec::with_capacity(lanes);
                let mut hnotes: Vec<Option<Note>> = Vec::with_capacity(lanes);
                let mut cloned: Vec<bool> = vec![false; lanes];
                for i in 0..lanes {
                    notes.push(tl.get_note(i as i32).cloned());
                    hnotes.push(tl.get_hidden_note(i as i32).cloned());
                }
                for i in 0..lanes {
                    let m = if i < random.len() {
                        random[i] as usize
                    } else {
                        i
                    };
                    if cloned[m] {
                        if let Some(ref note) = notes[m] {
                            tl.set_note(i as i32, Some(note.clone()));
                        } else {
                            tl.set_note(i as i32, None);
                        }
                        if let Some(ref hn) = hnotes[m] {
                            tl.set_hidden_note(i as i32, Some(hn.clone()));
                        } else {
                            tl.set_hidden_note(i as i32, None);
                        }
                    } else {
                        tl.set_note(i as i32, notes[m].take());
                        tl.set_hidden_note(i as i32, hnotes[m].take());
                        cloned[m] = true;
                    }
                }
            }
        }
        self.random = random;
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

// ---- LaneCrossShuffleModifier ----

pub struct LaneCrossShuffleModifier {
    pub base: PatternModifierBase,
    pub is_scratch_lane_modify: bool,
    random: Vec<i32>,
}

impl LaneCrossShuffleModifier {
    pub fn new(player: i32, is_scratch_lane_modify: bool) -> Self {
        let mut base = PatternModifierBase::with_player(player);
        base.assist = AssistLevel::LightAssist;
        LaneCrossShuffleModifier {
            base,
            is_scratch_lane_modify,
            random: Vec::new(),
        }
    }

    fn make_random(keys: &[i32], model: &BMSModel, _seed: i64) -> Vec<i32> {
        let mode_key = model.get_mode().map(|m| m.key()).unwrap_or(0);
        let mut result: Vec<i32> = (0..mode_key).collect();
        let mut i = 0;
        while i < keys.len() / 2 - 1 {
            result[keys[i] as usize] = keys[i + 1];
            result[keys[i + 1] as usize] = keys[i];
            result[keys[keys.len() - i - 1] as usize] = keys[keys.len() - i - 2];
            result[keys[keys.len() - i - 2] as usize] = keys[keys.len() - i - 1];
            i += 2;
        }
        result
    }
}

impl PatternModifier for LaneCrossShuffleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.random = lane_shuffle_modify(
            &mut self.base,
            model,
            self.is_scratch_lane_modify,
            true,
            Self::make_random,
        );
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

// ---- LanePlayableRandomShuffleModifier ----

pub struct LanePlayableRandomShuffleModifier {
    pub base: PatternModifierBase,
    pub is_scratch_lane_modify: bool,
    random: Vec<i32>,
}

impl LanePlayableRandomShuffleModifier {
    pub fn new(player: i32, is_scratch_lane_modify: bool) -> Self {
        let mut base = PatternModifierBase::with_player(player);
        base.assist = AssistLevel::LightAssist;
        LanePlayableRandomShuffleModifier {
            base,
            is_scratch_lane_modify,
            random: Vec::new(),
        }
    }

    fn make_random(keys: &[i32], model: &BMSModel, _seed: i64) -> Vec<i32> {
        let mode = match model.get_mode() {
            Some(m) => m.clone(),
            None => return Vec::new(),
        };
        let lanes = mode.key() as usize;
        let mut ln = vec![-1i32; lanes];
        let mut end_ln_note_time = vec![-1i32; lanes];
        let mut max = 0;
        for key in keys {
            max = max.max(*key);
        }
        let mut is_impossible = false;
        let mut original_pattern_list: HashSet<i32> = HashSet::new();

        // Build list of 3+ simultaneous press patterns
        for tl in model.get_all_time_lines() {
            if tl.exist_note() {
                // LN
                for i in 0..lanes {
                    if let Some(n) = tl.get_note(i as i32)
                        && n.is_long()
                    {
                        if n.is_end() && tl.get_time() == end_ln_note_time[i] {
                            ln[i] = -1;
                            end_ln_note_time[i] = -1;
                        } else {
                            ln[i] = i as i32;
                            if !n.is_end() {
                                // Get pair time
                                end_ln_note_time[i] = n.get_time();
                            }
                        }
                    }
                }
                // Normal notes
                let mut note_lane: Vec<i32> = Vec::new();
                for i in 0..lanes {
                    if let Some(n) = tl.get_note(i as i32) {
                        if n.is_normal() || ln[i] != -1 {
                            note_lane.push(i as i32);
                        }
                    } else if ln[i] != -1 {
                        note_lane.push(i as i32);
                    }
                }
                if note_lane.len() >= 7 {
                    is_impossible = true;
                    break;
                } else if note_lane.len() >= 3 {
                    let mut pattern = 0i32;
                    for &i in &note_lane {
                        pattern += (2f64).powi(i) as i32;
                    }
                    original_pattern_list.insert(pattern);
                }
            }
        }

        let mut kouho_pattern_list: Vec<Vec<i32>> = Vec::new();
        if !is_impossible {
            kouho_pattern_list =
                search_for_no_murioshi_lane_combinations(&original_pattern_list, keys);
        }

        log::info!("No-murioshi pattern count: {}", kouho_pattern_list.len());

        let mut result = vec![0i32; 9];
        if !kouho_pattern_list.is_empty() {
            let r = (rand::random::<f64>() * kouho_pattern_list.len() as f64) as usize;
            for i in 0..9 {
                result[kouho_pattern_list[r][i] as usize] = i as i32;
            }
        } else {
            let mirror = (rand::random::<f64>() * 2.0) as i32;
            for i in 0..9 {
                result[i] = if mirror == 0 { i as i32 } else { 8 - i as i32 };
            }
        }
        result
    }
}

fn search_for_no_murioshi_lane_combinations(
    original_pattern_list: &HashSet<i32>,
    _keys: &[i32],
) -> Vec<Vec<i32>> {
    let mut no_murioshi_lane_combinations: Vec<Vec<i32>> = Vec::new();
    let mut indexes = [0usize; 9];
    let mut lane_numbers = [0i32; 9];
    for i in 0..9 {
        lane_numbers[i] = i as i32;
        indexes[i] = 0;
    }

    let murioshi_chords: Vec<Vec<i32>> = vec![
        vec![1, 4, 7],
        vec![1, 4, 8],
        vec![1, 4, 9],
        vec![1, 5, 8],
        vec![1, 5, 9],
        vec![1, 6, 9],
        vec![2, 5, 8],
        vec![2, 5, 9],
        vec![2, 6, 9],
        vec![3, 6, 9],
    ];

    let mut i = 0usize;
    while i < 9 {
        if indexes[i] < i {
            let swap_idx = if i.is_multiple_of(2) { 0 } else { indexes[i] };
            lane_numbers.swap(swap_idx, i);

            let mut murioshi_flag = false;
            for pattern in original_pattern_list {
                let mut temp_pattern: Vec<i32> = Vec::new();
                for j in 0..9 {
                    if ((pattern / (2f64).powi(j as i32) as i32) % 2) == 1 {
                        temp_pattern.push(lane_numbers[j] + 1);
                    }
                }

                murioshi_flag = murioshi_chords
                    .iter()
                    .any(|chord| chord.iter().all(|c| temp_pattern.contains(c)));
                if murioshi_flag {
                    break;
                }
            }
            if !murioshi_flag {
                let random_combination: Vec<i32> = lane_numbers.to_vec();
                no_murioshi_lane_combinations.push(random_combination);
            }

            indexes[i] += 1;
            i = 0;
        } else {
            indexes[i] = 0;
            i += 1;
        }
    }

    let mirror_pattern: Vec<i32> = vec![8, 7, 6, 5, 4, 3, 2, 1, 0];
    no_murioshi_lane_combinations.retain(|p| *p != mirror_pattern);
    no_murioshi_lane_combinations
}

impl PatternModifier for LanePlayableRandomShuffleModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        self.random = lane_shuffle_modify(
            &mut self.base,
            model,
            self.is_scratch_lane_modify,
            true,
            Self::make_random,
        );
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
