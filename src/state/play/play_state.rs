use anyhow::Result;

use crate::model::bms_model::{BmsModel, PlayerRule};
use crate::model::note::{NoteType, PlayMode};
use crate::play::clear_type::ClearType;
use crate::play::gauge::gauge_property::{GaugePropertyType, GaugeType};
use crate::play::gauge::groove_gauge::GrooveGauge;
use crate::play::judge::judge_manager::{
    JudgeLevel, JudgeScore, JudgeTables, LaneJudgeState, build_judge_tables, determine_judge_index,
    select_note, update_judge,
};
use crate::play::judge::judge_property::JudgeProperty;
use crate::play::play_result::PlayResult;
use crate::play::score::ScoreData;
use crate::replay::replay_recorder::ReplayRecorder;
use crate::state::game_state::StateTransition;
use crate::state::play::autoplay::{AutoplayMode, ScriptedInput, generate_autoplay_events};
use crate::state::play::timer_manager::TimerManager;
use crate::traits::input::KeyEvent;

use super::timer_manager::{TIMER_FAILED, TIMER_PLAY, TIMER_READY};

/// Play phase state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayPhase {
    /// Loading resources before play.
    Preload,
    /// Ready countdown.
    Ready,
    /// Active play.
    Play,
    /// Song finished successfully.
    Finished,
    /// Gauge depleted (hard gauge death).
    Failed,
}

/// Time margin before the first note (microseconds).
const READY_DURATION_US: i64 = 1_000_000;

/// Time after last note before finishing (microseconds).
const FINISH_MARGIN_US: i64 = 3_000_000;

/// Configuration for creating a PlayState.
pub struct PlayConfig {
    pub model: BmsModel,
    pub mode: PlayMode,
    pub gauge_type: GaugeType,
    pub autoplay_mode: AutoplayMode,
}

/// Per-lane note state for tracking judge progress.
#[derive(Debug, Clone)]
struct LaneState {
    /// Notes for this lane, sorted by time.
    notes: Vec<LaneNote>,
    /// Index of the next note to check for miss processing.
    next_miss_index: usize,
    /// Judge state for LN processing.
    judge_state: LaneJudgeState,
    /// Index of the LN note currently being processed (start note index).
    processing_note_idx: Option<usize>,
}

/// A note within a lane for judge tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct LaneNote {
    /// Time of the note in microseconds.
    time_us: i64,
    /// End time for LN/CN/HCN (0 for normal).
    end_time_us: i64,
    /// Note type.
    note_type: NoteType,
    /// Whether the note has been judged.
    judged: bool,
    /// Whether the LN end has been judged.
    end_judged: bool,
    /// WAV ID for keysound playback.
    wav_id: u32,
    /// Damage for mine notes.
    damage: f64,
    /// Press time (for judge tracking). 0 if not pressed.
    play_time: i64,
}

impl LaneNote {
    /// Whether this note is a long note type.
    fn is_long(&self) -> bool {
        matches!(
            self.note_type,
            NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote
        )
    }
}

/// Core play state managing the game loop.
#[allow(dead_code)]
pub struct PlayState {
    /// Current play phase.
    phase: PlayPhase,
    /// Play mode.
    mode: PlayMode,
    /// Per-lane note states.
    lanes: Vec<LaneState>,
    /// Judge score tracking.
    judge_score: JudgeScore,
    /// Pre-computed judge tables.
    judge_tables: JudgeTables,
    /// Judge property reference.
    judge_property: &'static JudgeProperty,
    /// Groove gauge.
    gauge: GrooveGauge,
    /// Timer manager.
    pub timer: TimerManager,
    /// Current play time in microseconds.
    current_time_us: i64,
    /// Last note time in microseconds.
    last_note_time_us: i64,
    /// Total playable notes.
    total_notes: u32,
    /// BMS model SHA-256.
    sha256: String,
    /// Replay recorder.
    recorder: ReplayRecorder,
    /// Autoplay scripted input (if autoplay is enabled).
    autoplay_input: Option<ScriptedInput>,
    /// Whether scratch lanes exist.
    is_scratch_lane: Vec<bool>,
    /// Judge combo condition.
    combo_cond: &'static [bool],
    /// Judge miss condition.
    miss_condition: crate::play::judge::judge_property::MissCondition,

    // Reusable buffers to avoid per-press allocations.
    candidates_buf: Vec<(i64, bool, i64)>,
    playable_indices_buf: Vec<usize>,
}

impl PlayState {
    /// Create a new PlayState from configuration.
    pub fn new(config: PlayConfig) -> Self {
        let mode = config.mode;
        let lane_count = mode.lane_count();
        let rule = PlayerRule::from_mode(mode);
        let judge_property = JudgeProperty::from_rule_type(rule.rule_type);
        let rate = [100, 100, 100];
        let judge_tables =
            build_judge_tables(judge_property, config.model.judge_rank, &rate, &rate);

        let total_notes = config.model.total_notes() as u32;
        let gauge = GrooveGauge::new(
            config.gauge_type,
            GaugePropertyType::from_mode(mode),
            config.model.total,
            total_notes as usize,
        );

        // Build per-lane note arrays
        let mut lanes: Vec<LaneState> = (0..lane_count)
            .map(|_| LaneState {
                notes: Vec::new(),
                next_miss_index: 0,
                judge_state: LaneJudgeState::default(),
                processing_note_idx: None,
            })
            .collect();

        let mut last_note_time_us = 0i64;
        for note in &config.model.notes {
            if note.lane < lane_count {
                let end_time = if note.is_long_note() {
                    note.end_time_us
                } else {
                    0
                };
                last_note_time_us = last_note_time_us.max(note.time_us).max(end_time);
                lanes[note.lane].notes.push(LaneNote {
                    time_us: note.time_us,
                    end_time_us: note.end_time_us,
                    note_type: note.note_type,
                    judged: false,
                    end_judged: false,
                    wav_id: note.wav_id,
                    damage: note.damage,
                    play_time: 0,
                });
            }
        }

        // Sort each lane's notes by time
        for lane in &mut lanes {
            lane.notes.sort_by_key(|n| n.time_us);
        }

        let is_scratch_lane: Vec<bool> = (0..lane_count).map(|l| mode.is_scratch(l)).collect();

        // Generate autoplay events if needed
        let autoplay_input = if config.autoplay_mode != AutoplayMode::Off {
            let events = generate_autoplay_events(&config.model.notes, mode, config.autoplay_mode);
            Some(ScriptedInput::new(events, mode.key_count()))
        } else {
            None
        };

        let sha256 = config.model.sha256.clone();

        Self {
            phase: PlayPhase::Preload,
            mode,
            lanes,
            judge_score: JudgeScore::new(total_notes as usize),
            judge_tables,
            judge_property,
            gauge,
            timer: TimerManager::new(),
            current_time_us: 0,
            last_note_time_us,
            total_notes,
            sha256,
            recorder: ReplayRecorder::new(),
            autoplay_input,
            is_scratch_lane,
            combo_cond: judge_property.combo,
            miss_condition: judge_property.miss,
            candidates_buf: Vec::new(),
            playable_indices_buf: Vec::new(),
        }
    }

    /// Get the current play phase.
    pub fn phase(&self) -> PlayPhase {
        self.phase
    }

    /// Get the current play time in microseconds.
    pub fn current_time_us(&self) -> i64 {
        self.current_time_us
    }

    /// Get a reference to the judge score.
    pub fn judge_score(&self) -> &JudgeScore {
        &self.judge_score
    }

    /// Get a reference to the groove gauge.
    pub fn gauge(&self) -> &GrooveGauge {
        &self.gauge
    }

    /// Get the timer manager.
    pub fn timer(&self) -> &TimerManager {
        &self.timer
    }

    /// Get a reference to the replay recorder.
    pub fn recorder(&self) -> &ReplayRecorder {
        &self.recorder
    }

    /// Initialize the play state.
    pub fn create(&mut self) -> Result<()> {
        self.phase = PlayPhase::Ready;
        self.timer.set(TIMER_READY, 0);
        Ok(())
    }

    /// Update the play state with the given time.
    /// `now_us` is the absolute time from start.
    /// Returns state transition.
    pub fn update(&mut self, now_us: i64) -> Result<StateTransition> {
        self.current_time_us = now_us;

        // Process autoplay events
        if let Some(ref mut autoplay) = self.autoplay_input {
            let events = autoplay.poll_up_to(now_us);
            for event in events {
                self.process_key_event(event);
            }
        }

        match self.phase {
            PlayPhase::Preload => {
                // Transition to Ready immediately after create()
                self.phase = PlayPhase::Ready;
                self.timer.set(TIMER_READY, now_us);
                Ok(StateTransition::None)
            }
            PlayPhase::Ready => {
                if let Some(ready_start) = self.timer.get(TIMER_READY)
                    && now_us - ready_start >= READY_DURATION_US
                {
                    self.phase = PlayPhase::Play;
                    self.timer.set(TIMER_PLAY, now_us);
                }
                Ok(StateTransition::None)
            }
            PlayPhase::Play => {
                // Process miss notes (notes that have passed the judge window)
                self.process_miss_notes(now_us);

                // Check gauge death (hard/ex-hard gauges)
                if self.gauge.value() <= 0.0 {
                    self.phase = PlayPhase::Failed;
                    self.timer.set(TIMER_FAILED, now_us);
                    return Ok(StateTransition::None);
                }

                // Check if song is finished
                if now_us > self.last_note_time_us + FINISH_MARGIN_US {
                    self.phase = PlayPhase::Finished;
                    return Ok(StateTransition::Next);
                }

                Ok(StateTransition::None)
            }
            PlayPhase::Finished => Ok(StateTransition::Next),
            PlayPhase::Failed => Ok(StateTransition::Next),
        }
    }

    /// Process an input key event.
    pub fn process_key_event(&mut self, event: KeyEvent) {
        // Record the event for replay
        self.recorder.record(event);

        // Update key-on/key-off timers
        let player = self
            .mode
            .lane_to_player(event.key.min(self.mode.lane_count().saturating_sub(1)));
        let skin_offset = if event.key < self.mode.lane_count() {
            self.mode.lane_to_skin_offset(event.key)
        } else {
            return;
        };

        if event.pressed {
            self.timer.set_keyon(skin_offset, player, event.time_us);
        } else {
            self.timer.set_keyoff(skin_offset, player, event.time_us);
        }

        if self.phase != PlayPhase::Play {
            return;
        }

        let lane = event.key;
        if lane >= self.lanes.len() {
            return;
        }

        if event.pressed {
            self.process_key_press(lane, event.time_us);
        } else {
            self.process_key_release(lane, event.time_us);
        }
    }

    /// Process a key press on a lane.
    fn process_key_press(&mut self, lane: usize, press_time: i64) {
        let is_scratch = self.is_scratch_lane.get(lane).copied().unwrap_or(false);

        // Check for mine notes first
        if self.try_mine_damage(lane, press_time) {
            return;
        }

        let judge_start = self.judge_tables.judge_start;
        let judge_end = self.judge_tables.judge_end;

        // Build candidates and playable index map using reusable buffers.
        self.candidates_buf.clear();
        self.playable_indices_buf.clear();
        for (i, n) in self.lanes[lane].notes.iter().enumerate() {
            if !matches!(n.note_type, NoteType::Mine | NoteType::Invisible) {
                self.candidates_buf
                    .push((n.time_us, !n.judged, n.play_time));
                self.playable_indices_buf.push(i);
            }
        }

        let result = select_note(
            &self.candidates_buf,
            press_time,
            self.judge_tables.table_for_lane(is_scratch),
            judge_start,
            judge_end,
            crate::play::judge::judge_algorithm::JudgeAlgorithm::Combo,
            self.miss_condition,
        );

        if let Some((candidate_idx, judge)) = result {
            // Map candidate index back to lane note index
            if candidate_idx < self.playable_indices_buf.len() {
                let note_idx = self.playable_indices_buf[candidate_idx];
                let note = &self.lanes[lane].notes[note_idx];
                let time_diff = note.time_us - press_time;
                let vanish = judge < 5;
                let note_play_time = note.play_time;
                let note_type = note.note_type;
                let is_ln = note.is_long();

                let player = self.mode.lane_to_player(lane);
                let skin_offset = self.mode.lane_to_skin_offset(lane);

                // Set judge timer
                self.timer.set_judge(skin_offset, player, press_time);

                // Set bomb timer on Great or better
                if judge <= 1 {
                    self.timer.set_bomb(skin_offset, player, press_time);
                }

                if is_ln && vanish {
                    // LN/CN/HCN start: set up processing state
                    match note_type {
                        NoteType::LongNote => {
                            // LN: defer start judgment to release
                            self.lanes[lane].judge_state.ln_start_judge = judge;
                            self.lanes[lane].judge_state.ln_start_duration = time_diff;
                            self.lanes[lane].judge_state.processing = true;
                            self.lanes[lane].judge_state.release_time = None;
                            self.lanes[lane].judge_state.ln_end_judge = None;
                            self.lanes[lane].processing_note_idx = Some(note_idx);
                        }
                        NoteType::ChargeNote | NoteType::HellChargeNote => {
                            // CN/HCN: judge start immediately, set up processing for end
                            update_judge(
                                &mut self.judge_score,
                                judge,
                                time_diff,
                                vanish,
                                self.miss_condition,
                                self.combo_cond,
                                note_play_time,
                            );
                            self.gauge.update_judge(judge);
                            self.lanes[lane].judge_state.processing = true;
                            self.lanes[lane].judge_state.release_time = None;
                            self.lanes[lane].judge_state.ln_end_judge = None;
                            self.lanes[lane].processing_note_idx = Some(note_idx);
                        }
                        _ => {}
                    }
                } else {
                    // Normal note or non-vanishing LN: judge immediately
                    update_judge(
                        &mut self.judge_score,
                        judge,
                        time_diff,
                        vanish,
                        self.miss_condition,
                        self.combo_cond,
                        note_play_time,
                    );
                    self.gauge.update_judge(judge);
                }

                // Mark note as judged
                self.lanes[lane].notes[note_idx].judged = true;
                self.lanes[lane].notes[note_idx].play_time = press_time;
            }
        }
    }

    /// Process a key release on a lane (for LN/CN end judgment).
    fn process_key_release(&mut self, lane: usize, release_time: i64) {
        if !self.lanes[lane].judge_state.processing {
            return;
        }

        let note_idx = match self.lanes[lane].processing_note_idx {
            Some(idx) => idx,
            None => return,
        };

        let note = &self.lanes[lane].notes[note_idx];
        let end_time = note.end_time_us;
        let note_type = note.note_type;
        let play_time = note.play_time;
        let is_scratch = self.is_scratch_lane.get(lane).copied().unwrap_or(false);
        let ln_end_table: Vec<[i64; 2]> =
            self.judge_tables.ln_end_table_for_lane(is_scratch).to_vec();

        // dmtime = end_time - release_time (positive = released early, negative = released late)
        let dmtime = end_time - release_time;
        let judge = determine_judge_index(dmtime, &ln_end_table).unwrap_or(ln_end_table.len());

        match note_type {
            NoteType::ChargeNote | NoteType::HellChargeNote => {
                // CN/HCN release: if judge is BD or worse (>=3) and early (dmtime > 0),
                // defer the judgment.
                if judge >= 3 && dmtime > 0 {
                    self.lanes[lane].judge_state.release_time = Some(release_time);
                    self.lanes[lane].judge_state.ln_end_judge = Some(judge);
                } else {
                    // Finalize CN/HCN end judgment immediately
                    let final_judge = if judge >= ln_end_table.len() {
                        4
                    } else {
                        judge
                    };
                    update_judge(
                        &mut self.judge_score,
                        final_judge,
                        dmtime,
                        true,
                        self.miss_condition,
                        self.combo_cond,
                        play_time,
                    );
                    self.gauge.update_judge(final_judge);
                    self.lanes[lane].notes[note_idx].end_judged = true;
                    self.lanes[lane].judge_state.processing = false;
                    self.lanes[lane].processing_note_idx = None;
                    self.lanes[lane].judge_state.release_time = None;
                    self.lanes[lane].judge_state.ln_end_judge = None;
                }
            }
            NoteType::LongNote => {
                // LN release: take the worse of start and end judge
                let start_judge = self.lanes[lane].judge_state.ln_start_judge;
                let start_duration = self.lanes[lane].judge_state.ln_start_duration;
                let mut final_judge = judge.max(start_judge);
                let final_dmtime = if start_duration.abs() > dmtime.abs() {
                    start_duration
                } else {
                    dmtime
                };

                if final_judge >= 3 && dmtime > 0 {
                    // Defer: released early with BD or worse
                    // For LN, deferred judge is capped at BD (3)
                    self.lanes[lane].judge_state.release_time = Some(release_time);
                    self.lanes[lane].judge_state.ln_end_judge = Some(3);
                } else {
                    // Finalize LN end judgment
                    if final_judge >= ln_end_table.len() {
                        final_judge = 4; // Poor
                    }
                    update_judge(
                        &mut self.judge_score,
                        final_judge,
                        final_dmtime,
                        true,
                        self.miss_condition,
                        self.combo_cond,
                        play_time,
                    );
                    self.gauge.update_judge(final_judge);
                    self.lanes[lane].notes[note_idx].end_judged = true;
                    self.lanes[lane].judge_state.processing = false;
                    self.lanes[lane].processing_note_idx = None;
                    self.lanes[lane].judge_state.release_time = None;
                    self.lanes[lane].judge_state.ln_end_judge = None;
                }
            }
            _ => {}
        }
    }

    /// Check for mine notes at the press time and apply damage.
    /// Returns true if a mine was hit.
    fn try_mine_damage(&mut self, lane: usize, press_time: i64) -> bool {
        // Find a mine note near the press time
        let mine_window = 100_000; // 100ms window for mine detection
        for note in &mut self.lanes[lane].notes {
            if note.note_type == NoteType::Mine
                && !note.judged
                && (note.time_us - press_time).abs() < mine_window
            {
                note.judged = true;
                // Apply mine damage as negative gauge value
                let damage = -(note.damage as f32);
                self.gauge.add_value(damage);
                return true;
            }
        }
        false
    }

    /// Process deferred LN release judgments and LN end auto-completion.
    fn process_ln_timers(&mut self, now_us: i64) {
        for lane_idx in 0..self.lanes.len() {
            if !self.lanes[lane_idx].judge_state.processing {
                continue;
            }

            let note_idx = match self.lanes[lane_idx].processing_note_idx {
                Some(idx) => idx,
                None => continue,
            };

            let note = &self.lanes[lane_idx].notes[note_idx];
            let note_type = note.note_type;
            let end_time = note.end_time_us;
            let play_time = note.play_time;
            let is_scratch = self.is_scratch_lane.get(lane_idx).copied().unwrap_or(false);
            let release_margin = self.judge_tables.release_margin_for_lane(is_scratch);

            // Check deferred release (release_time + margin has elapsed)
            if let Some(rel_time) = self.lanes[lane_idx].judge_state.release_time
                && rel_time + release_margin <= now_us
            {
                let ln_end_judge = self.lanes[lane_idx].judge_state.ln_end_judge.unwrap_or(4);
                let dmtime = end_time - rel_time;

                update_judge(
                    &mut self.judge_score,
                    ln_end_judge,
                    dmtime,
                    true,
                    self.miss_condition,
                    self.combo_cond,
                    play_time,
                );
                self.gauge.update_judge(ln_end_judge);
                self.lanes[lane_idx].notes[note_idx].end_judged = true;
                self.lanes[lane_idx].judge_state.processing = false;
                self.lanes[lane_idx].processing_note_idx = None;
                self.lanes[lane_idx].judge_state.release_time = None;
                self.lanes[lane_idx].judge_state.ln_end_judge = None;
                continue;
            }

            // LN auto-completion: if held past end time, use start judge
            if note_type == NoteType::LongNote
                && self.lanes[lane_idx].judge_state.release_time.is_none()
                && end_time < now_us
            {
                let start_judge = self.lanes[lane_idx].judge_state.ln_start_judge;
                let start_duration = self.lanes[lane_idx].judge_state.ln_start_duration;

                update_judge(
                    &mut self.judge_score,
                    start_judge,
                    start_duration,
                    true,
                    self.miss_condition,
                    self.combo_cond,
                    play_time,
                );
                self.gauge.update_judge(start_judge);
                self.lanes[lane_idx].notes[note_idx].end_judged = true;
                self.lanes[lane_idx].judge_state.processing = false;
                self.lanes[lane_idx].processing_note_idx = None;
                self.lanes[lane_idx].judge_state.release_time = None;
                self.lanes[lane_idx].judge_state.ln_end_judge = None;
            }

            // CN/HCN end miss: if end time has passed the miss window without release
            if matches!(note_type, NoteType::ChargeNote | NoteType::HellChargeNote)
                && self.lanes[lane_idx].judge_state.release_time.is_none()
            {
                let judge_table = self.judge_tables.table_for_lane(is_scratch);
                let miss_boundary = if !judge_table.is_empty() {
                    judge_table[judge_table.len() - 1][0]
                } else {
                    continue;
                };
                let dmtime = end_time - now_us;
                if dmtime < miss_boundary {
                    // CN/HCN end missed
                    update_judge(
                        &mut self.judge_score,
                        4, // Poor
                        dmtime,
                        true,
                        self.miss_condition,
                        self.combo_cond,
                        play_time,
                    );
                    self.gauge.update_judge(4);
                    self.lanes[lane_idx].notes[note_idx].end_judged = true;
                    self.lanes[lane_idx].judge_state.processing = false;
                    self.lanes[lane_idx].processing_note_idx = None;
                    self.lanes[lane_idx].judge_state.release_time = None;
                    self.lanes[lane_idx].judge_state.ln_end_judge = None;
                }
            }
        }
    }

    /// Process notes that have passed the miss window.
    fn process_miss_notes(&mut self, now_us: i64) {
        // Process deferred LN releases and auto-completions first
        self.process_ln_timers(now_us);

        for lane_idx in 0..self.lanes.len() {
            let is_scratch = self.is_scratch_lane.get(lane_idx).copied().unwrap_or(false);
            let judge_table = self.judge_tables.table_for_lane(is_scratch);

            // Furthest late boundary (negative value).
            let miss_boundary = if !judge_table.is_empty() {
                judge_table[judge_table.len() - 1][0]
            } else {
                continue;
            };

            let start = self.lanes[lane_idx].next_miss_index;
            let notes = &mut self.lanes[lane_idx].notes;

            let mut new_start = start;
            for (i, note) in notes.iter_mut().enumerate().skip(start) {
                let dmtime = note.time_us - now_us;

                // Notes are sorted by time; if this note is still within or
                // ahead of the judge window, all subsequent notes will be too.
                if dmtime >= miss_boundary {
                    break;
                }

                if note.judged {
                    // For LN notes, also check if end was judged before advancing
                    if note.is_long() && !note.end_judged {
                        continue;
                    }
                    // Advance past already-judged notes so we skip them next frame.
                    if i == new_start {
                        new_start = i + 1;
                    }
                    continue;
                }
                if matches!(note.note_type, NoteType::Mine | NoteType::Invisible) {
                    if i == new_start {
                        new_start = i + 1;
                    }
                    continue;
                }

                // Note is past the late boundary -> miss
                note.judged = true;
                let time_diff = dmtime;
                let play_time = note.play_time;
                let is_ln = note.is_long();
                update_judge(
                    &mut self.judge_score,
                    5, // Miss
                    time_diff,
                    true,
                    self.miss_condition,
                    self.combo_cond,
                    play_time,
                );
                self.gauge.update_judge(5);

                // For CN/HCN, also miss the end note
                if is_ln
                    && matches!(
                        note.note_type,
                        NoteType::ChargeNote | NoteType::HellChargeNote
                    )
                {
                    note.end_judged = true;
                    update_judge(
                        &mut self.judge_score,
                        4, // Poor for end
                        time_diff,
                        true,
                        self.miss_condition,
                        self.combo_cond,
                        play_time,
                    );
                    self.gauge.update_judge(4);
                }

                if i == new_start {
                    new_start = i + 1;
                }
            }

            self.lanes[lane_idx].next_miss_index = new_start;
        }
    }

    /// Build the final play result.
    pub fn build_result(&self) -> PlayResult {
        let mut score = ScoreData::new(self.total_notes);
        score.early_counts = self.judge_score.early_counts;
        score.late_counts = self.judge_score.late_counts;
        score.max_combo = self.judge_score.max_combo;
        score.pass_notes = self.judge_score.pass_notes;
        score.min_bp = self.judge_score.judge_count(JudgeLevel::Bad)
            + self.judge_score.judge_count(JudgeLevel::Poor)
            + self.judge_score.judge_count(JudgeLevel::Miss);

        PlayResult::new(&self.gauge, score)
    }

    /// Get the clear type based on current state.
    pub fn clear_type(&self) -> ClearType {
        self.build_result().clear_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::note::{Note, NoteType};
    use crate::play::gauge::gauge_property::*;
    use crate::state::play::timer_manager::{
        TIMER_BOMB_1P_BASE, TIMER_JUDGE_1P_BASE, TIMER_KEYON_1P_BASE,
    };

    fn make_model(notes: Vec<Note>) -> BmsModel {
        let mut model = BmsModel {
            mode: PlayMode::Beat7K,
            judge_rank: 100,
            total: 300.0,
            notes,
            ..Default::default()
        };
        model.validate();
        model
    }

    fn make_config(notes: Vec<Note>, autoplay: AutoplayMode) -> PlayConfig {
        let model = make_model(notes);
        PlayConfig {
            mode: model.mode,
            gauge_type: GAUGE_NORMAL,
            autoplay_mode: autoplay,
            model,
        }
    }

    fn make_normal_note(lane: usize, time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::Normal,
            time_us,
            end_time_us: 0,
            wav_id: 0,
            damage: 0.0,
        }
    }

    fn make_mine_note(lane: usize, time_us: i64, damage: f64) -> Note {
        Note {
            lane,
            note_type: NoteType::Mine,
            time_us,
            end_time_us: 0,
            wav_id: 0,
            damage,
        }
    }

    fn make_ln_note(lane: usize, time_us: i64, end_time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::LongNote,
            time_us,
            end_time_us,
            wav_id: 0,
            damage: 0.0,
        }
    }

    fn make_cn_note(lane: usize, time_us: i64, end_time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::ChargeNote,
            time_us,
            end_time_us,
            wav_id: 0,
            damage: 0.0,
        }
    }

    fn make_hcn_note(lane: usize, time_us: i64, end_time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::HellChargeNote,
            time_us,
            end_time_us,
            wav_id: 0,
            damage: 0.0,
        }
    }

    // =========================================================================
    // Basic lifecycle tests
    // =========================================================================

    #[test]
    fn play_state_create_transitions_to_ready() {
        let config = make_config(vec![make_normal_note(0, 2_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        assert_eq!(state.phase(), PlayPhase::Preload);

        state.create().unwrap();
        assert_eq!(state.phase(), PlayPhase::Ready);
    }

    #[test]
    fn play_state_ready_to_play_transition() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();

        // Before ready duration expires
        state.update(500_000).unwrap();
        assert_eq!(state.phase(), PlayPhase::Ready);

        // After ready duration expires
        state.update(READY_DURATION_US + 1).unwrap();
        assert_eq!(state.phase(), PlayPhase::Play);
    }

    #[test]
    fn play_state_finishes_after_last_note() {
        let config = make_config(vec![make_normal_note(0, 2_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();

        // Move to Play
        state.update(READY_DURATION_US + 1).unwrap();
        assert_eq!(state.phase(), PlayPhase::Play);

        // After last note + margin
        let result = state.update(2_000_000 + FINISH_MARGIN_US + 1).unwrap();
        assert_eq!(state.phase(), PlayPhase::Finished);
        assert_eq!(result, StateTransition::Next);
    }

    // =========================================================================
    // Judge tests
    // =========================================================================

    #[test]
    fn process_perfect_great_press() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact note time -> PG
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 1); // PG
        assert_eq!(state.judge_score().combo, 1);
    }

    #[test]
    fn process_great_press() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press 40ms early (within GR window [-60000, 60000])
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 4_960_000,
        });

        assert_eq!(state.judge_score().judge_count(JudgeLevel::Great), 1); // GR
    }

    #[test]
    fn process_miss_by_timeout() {
        let config = make_config(vec![make_normal_note(0, 2_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Let time pass beyond the miss window
        state.update(3_000_000).unwrap();
        assert_eq!(state.judge_score().judge_count(JudgeLevel::Miss), 1); // Miss
        assert_eq!(state.judge_score().combo, 0);
    }

    // =========================================================================
    // Gauge tests
    // =========================================================================

    #[test]
    fn gauge_increases_on_good_judge() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        let initial_gauge = state.gauge().value();
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });
        assert!(state.gauge().value() > initial_gauge);
    }

    #[test]
    fn gauge_decreases_on_miss() {
        let notes: Vec<Note> = (0..10)
            .map(|i| make_normal_note(0, 2_000_000 + i * 500_000))
            .collect();
        let config = make_config(notes, AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Set gauge to a known value
        state.gauge = GrooveGauge::new(
            GAUGE_NORMAL,
            GaugePropertyType::from_mode(PlayMode::Beat7K),
            300.0,
            10,
        );
        state.gauge.set_value(50.0);

        let initial = state.gauge().value();
        // Let first note miss
        state.update(3_000_000).unwrap();
        assert!(state.gauge().value() < initial);
    }

    // =========================================================================
    // Mine note tests
    // =========================================================================

    #[test]
    fn mine_note_damage_on_press() {
        let config = make_config(vec![make_mine_note(0, 5_000_000, 10.0)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        let initial = state.gauge().value();
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });
        assert!(state.gauge().value() < initial);
    }

    #[test]
    fn mine_note_no_damage_if_not_pressed() {
        let config = make_config(vec![make_mine_note(0, 5_000_000, 10.0)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        let initial = state.gauge().value();
        // Time passes but no key press
        state.update(6_000_000).unwrap();
        assert!((state.gauge().value() - initial).abs() < 0.001);
    }

    // =========================================================================
    // Autoplay tests
    // =========================================================================

    #[test]
    fn autoplay_full_produces_perfect() {
        let notes: Vec<Note> = (0..5)
            .map(|i| make_normal_note(i % 7, 3_000_000 + i as i64 * 1_000_000))
            .collect();
        let config = make_config(notes, AutoplayMode::Full);
        let mut state = PlayState::new(config);
        state.create().unwrap();

        // Simulate progression
        let mut time = 0i64;
        while state.phase() != PlayPhase::Finished && time < 20_000_000 {
            time += 16_667; // ~60fps
            state.update(time).unwrap();
        }

        // All notes should be PG
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 5);
        assert_eq!(state.judge_score().max_combo, 5);
    }

    // =========================================================================
    // Timer tests
    // =========================================================================

    #[test]
    fn timer_ready_set_on_create() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        assert!(state.timer().is_active(TIMER_READY));
    }

    #[test]
    fn timer_play_set_on_play_start() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        assert!(!state.timer().is_active(TIMER_PLAY));

        state.update(READY_DURATION_US + 1).unwrap();
        assert!(state.timer().is_active(TIMER_PLAY));
    }

    #[test]
    fn timer_judge_set_on_note_press() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Lane 0 for Beat7K has skin offset 1
        let skin_offset = PlayMode::Beat7K.lane_to_skin_offset(0);
        assert!(!state.timer().is_active(TIMER_JUDGE_1P_BASE + skin_offset));

        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });
        assert!(state.timer().is_active(TIMER_JUDGE_1P_BASE + skin_offset));
    }

    #[test]
    fn timer_bomb_set_on_great_or_better() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        let skin_offset = PlayMode::Beat7K.lane_to_skin_offset(0);

        // PG press
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });
        assert!(state.timer().is_active(TIMER_BOMB_1P_BASE + skin_offset));
    }

    #[test]
    fn timer_keyon_set_on_press() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();

        let skin_offset = PlayMode::Beat7K.lane_to_skin_offset(0);
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 1_000_000,
        });
        assert!(state.timer().is_active(TIMER_KEYON_1P_BASE + skin_offset));
    }

    // =========================================================================
    // Replay recorder tests
    // =========================================================================

    #[test]
    fn recorder_captures_events() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });
        assert_eq!(state.recorder().events().len(), 1);
    }

    // =========================================================================
    // Build result tests
    // =========================================================================

    #[test]
    fn build_result_returns_correct_data() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // PG press
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        let result = state.build_result();
        assert_eq!(result.score.judge_count(JudgeLevel::PerfectGreat), 1);
        assert_eq!(result.score.exscore(), 2);
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn empty_chart_finishes_immediately() {
        let config = make_config(vec![], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // No notes, so should finish after margin
        let result = state
            .update(FINISH_MARGIN_US + READY_DURATION_US + 2)
            .unwrap();
        assert_eq!(state.phase(), PlayPhase::Finished);
        assert_eq!(result, StateTransition::Next);
    }

    #[test]
    fn key_event_out_of_range_lane_ignored() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Lane 99 is out of range for Beat7K (8 lanes)
        state.process_key_event(KeyEvent {
            key: 99,
            pressed: true,
            time_us: 5_000_000,
        });
        // Should not panic or change state
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 0);
    }

    // =========================================================================
    // LN release judgment tests
    // =========================================================================

    #[test]
    fn ln_press_sets_processing_state() {
        // LN start at 5s, end at 6s
        let config = make_config(
            vec![make_ln_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // LN start should not produce a judge count yet (deferred to release)
        assert_eq!(state.judge_score().pass_notes, 0);
        assert!(state.lanes[0].judge_state.processing);
        assert_eq!(state.lanes[0].processing_note_idx, Some(0));
    }

    #[test]
    fn ln_release_at_end_produces_pg() {
        let config = make_config(
            vec![make_ln_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time (PG start)
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // Release at exact end time (PG end)
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 6_000_000,
        });

        // LN end should be judged as PG (max of PG start and PG end)
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 1);
        assert_eq!(state.judge_score().pass_notes, 1);
        assert!(!state.lanes[0].judge_state.processing);
    }

    #[test]
    fn ln_held_past_end_auto_completes() {
        let config = make_config(
            vec![make_ln_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time (PG start)
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // Hold past end time without releasing -> auto-complete with start judge
        state.update(6_100_000).unwrap();

        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 1);
        assert!(!state.lanes[0].judge_state.processing);
    }

    #[test]
    fn ln_release_takes_worse_of_start_and_end() {
        let config = make_config(
            vec![make_ln_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press 40ms early -> GR start
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 4_960_000,
        });

        // Release at exact end time -> PG end, but result = max(GR, PG) = GR
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 6_000_000,
        });

        assert_eq!(state.judge_score().judge_count(JudgeLevel::Great), 1);
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 0);
    }

    #[test]
    fn cn_press_judges_start_immediately() {
        let config = make_config(
            vec![make_cn_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // CN start is judged immediately (unlike LN)
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 1);
        assert_eq!(state.judge_score().pass_notes, 1);
        assert!(state.lanes[0].judge_state.processing);
    }

    #[test]
    fn cn_release_at_end_produces_pg() {
        let config = make_config(
            vec![make_cn_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time (PG start)
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // Release at exact end time (PG end)
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 6_000_000,
        });

        // CN start PG + CN end PG = 2 PG judgments
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 2);
        assert_eq!(state.judge_score().pass_notes, 2);
        assert!(!state.lanes[0].judge_state.processing);
    }

    #[test]
    fn cn_end_missed_produces_poor() {
        let config = make_config(
            vec![make_cn_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // Don't release; let time pass far beyond end time
        state.update(7_000_000).unwrap();

        // Start was PG, end should be Poor (missed)
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 1);
        assert_eq!(state.judge_score().judge_count(JudgeLevel::Poor), 1);
        assert!(!state.lanes[0].judge_state.processing);
    }

    #[test]
    fn hcn_release_at_end_produces_pg() {
        let config = make_config(
            vec![make_hcn_note(0, 5_000_000, 6_000_000)],
            AutoplayMode::Off,
        );
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Press at exact start time
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 5_000_000,
        });

        // Release at exact end time
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 6_000_000,
        });

        // HCN: start PG + end PG = 2 PG
        assert_eq!(state.judge_score().judge_count(JudgeLevel::PerfectGreat), 2);
        assert!(!state.lanes[0].judge_state.processing);
    }

    #[test]
    fn release_on_non_processing_lane_is_noop() {
        let config = make_config(vec![make_normal_note(0, 5_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Release without any LN processing should be a no-op
        state.process_key_event(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 5_000_000,
        });

        assert_eq!(state.judge_score().pass_notes, 0);
    }
}
