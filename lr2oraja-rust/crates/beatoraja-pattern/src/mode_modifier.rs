use beatoraja_core::player_config::PlayerConfig;
use bms_model::bms_model::BMSModel;
use bms_model::mode::Mode;
use bms_model::note::Note;

use crate::pattern_modifier::{AssistLevel, PatternModifier, PatternModifierBase};

pub struct ModeModifier {
    pub base: PatternModifierBase,
    config: PlayerConfig,
    hran_threshold: i32,
    before_mode: Mode,
    after_mode: Mode,
}

impl ModeModifier {
    pub fn new(before_mode: Mode, after_mode: Mode, config: PlayerConfig) -> Self {
        ModeModifier {
            base: PatternModifierBase::with_assist(AssistLevel::LightAssist),
            config,
            hran_threshold: 125,
            before_mode,
            after_mode,
        }
    }
}

impl PatternModifier for ModeModifier {
    fn modify(&mut self, model: &mut BMSModel) {
        model.set_mode(self.after_mode.clone());
        let algorithm = Algorithm::get(&self.before_mode, &self.after_mode);
        let lanes = self.after_mode.key() as usize;
        let mut ln = vec![-1i32; lanes];
        let mut last_note_time = vec![-100i32; lanes];
        let mut end_ln_note_time = vec![-1i32; lanes];

        if self.config.hran_threshold_bpm <= 0 {
            self.hran_threshold = 0;
        } else {
            self.hran_threshold =
                (15000.0f32 / self.config.hran_threshold_bpm as f32).ceil() as i32;
        }

        let after_mode = self.after_mode.clone();
        let hran_threshold = self.hran_threshold;
        let seven_to_nine_pattern = self.config.seven_to_nine_pattern;
        let seven_to_nine_type = self.config.seven_to_nine_type;

        let timelines = model.get_all_time_lines_mut();
        // Pre-compute timeline index → time for LN end note pair lookup
        let tl_times: Vec<i32> = timelines.iter().map(|tl| tl.get_time()).collect();
        for tl in timelines.iter_mut() {
            if tl.exist_note() || tl.exist_hidden_note() {
                let mut notes: Vec<Option<Note>> = Vec::with_capacity(lanes);
                let mut hnotes: Vec<Option<Note>> = Vec::with_capacity(lanes);
                for i in 0..lanes {
                    notes.push(tl.get_note(i as i32).cloned());
                    hnotes.push(tl.get_hidden_note(i as i32).cloned());
                }

                let keys = PatternModifierBase::get_keys_static(&after_mode, 0, true);
                let random = if let Some(alg) = algorithm {
                    if !keys.is_empty() {
                        alg.modify(
                            &keys,
                            &ln,
                            &notes,
                            &last_note_time,
                            tl.get_time(),
                            hran_threshold,
                            seven_to_nine_pattern,
                            seven_to_nine_type,
                        )
                    } else {
                        keys
                    }
                } else {
                    keys
                };

                for i in 0..lanes {
                    let m = if i < random.len() {
                        random[i] as usize
                    } else {
                        i
                    };
                    let n = notes[m].take();
                    let hn = hnotes[m].take();
                    if let Some(ref note) = n {
                        let is_long = note.is_long();
                        let is_end = note.is_end();
                        let _note_time = note.get_time();
                        if is_long {
                            if is_end && tl.get_time() == end_ln_note_time[i] {
                                tl.set_note(i as i32, n);
                                ln[i] = -1;
                                end_ln_note_time[i] = -1;
                            } else {
                                ln[i] = m as i32;
                                if !is_end {
                                    // Java: endLnNoteTime[i] = ln2.getPair().getTime()
                                    // Store the END note's timeline time (not the start note's)
                                    end_ln_note_time[i] =
                                        note.get_pair().map(|idx| tl_times[idx]).unwrap_or(-1);
                                }
                                last_note_time[i] = tl.get_time();
                                tl.set_note(i as i32, n);
                            }
                        } else {
                            last_note_time[i] = tl.get_time();
                            tl.set_note(i as i32, n);
                        }
                    } else {
                        tl.set_note(i as i32, None);
                    }
                    tl.set_hidden_note(i as i32, hn);
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

// get_keys_static is defined in lane_shuffle_modifier.rs

#[derive(Clone, Copy)]
enum Algorithm {
    SevenToNine,
}

impl Algorithm {
    fn get(before_mode: &Mode, after_mode: &Mode) -> Option<Algorithm> {
        if *before_mode == Mode::BEAT_7K && *after_mode == Mode::POPN_9K {
            Some(Algorithm::SevenToNine)
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn modify(
        &self,
        _keys: &[i32],
        activeln: &[i32],
        _notes: &[Option<Note>],
        last_note_time: &[i32],
        now: i32,
        duration: i32,
        seven_to_nine_pattern: i32,
        seven_to_nine_type: i32,
    ) -> Vec<i32> {
        match self {
            Algorithm::SevenToNine => {
                #[allow(clippy::eq_op)]
                let (key_lane, sc_lane, rest_lane) = match seven_to_nine_pattern {
                    1 => (2 - 1, 1 - 1, 9 - 1),
                    2 => (3 - 1, 1 - 1, 2 - 1),
                    4 => (1 - 1, 8 - 1, 9 - 1),
                    5 => (1 - 1, 9 - 1, 8 - 1),
                    6 => (2 - 1, 9 - 1, 1 - 1),
                    3 => (3 - 1, 2 - 1, 1 - 1),
                    _ => (3 - 1, 2 - 1, 1 - 1),
                };

                let mut result = vec![0i32; 9];
                for i in 0..7 {
                    result[i + key_lane as usize] = i as i32;
                }

                if activeln[sc_lane as usize] != -1 || activeln[rest_lane as usize] != -1 {
                    if activeln[sc_lane as usize] == 7 {
                        result[sc_lane as usize] = 7;
                        result[rest_lane as usize] = 8;
                    } else {
                        result[sc_lane as usize] = 8;
                        result[rest_lane as usize] = 7;
                    }
                } else {
                    match seven_to_nine_type {
                        1 => {
                            if now - last_note_time[sc_lane as usize] > duration
                                || now - last_note_time[sc_lane as usize]
                                    >= now - last_note_time[rest_lane as usize]
                            {
                                result[sc_lane as usize] = 7;
                                result[rest_lane as usize] = 8;
                            } else {
                                result[sc_lane as usize] = 8;
                                result[rest_lane as usize] = 7;
                            }
                        }
                        2 => {
                            if now - last_note_time[sc_lane as usize]
                                >= now - last_note_time[rest_lane as usize]
                            {
                                result[sc_lane as usize] = 7;
                                result[rest_lane as usize] = 8;
                            } else {
                                result[sc_lane as usize] = 8;
                                result[rest_lane as usize] = 7;
                            }
                        }
                        0 => {
                            result[sc_lane as usize] = 7;
                            result[rest_lane as usize] = 8;
                        }
                        _ => {
                            result[sc_lane as usize] = 7;
                            result[rest_lane as usize] = 8;
                        }
                    }
                }

                result
            }
        }
    }
}
