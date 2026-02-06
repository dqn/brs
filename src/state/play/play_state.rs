use anyhow::Result;

use crate::model::bms_model::{BmsModel, PlayerRule};
use crate::model::note::{NoteType, PlayMode};
use crate::play::clear_type::ClearType;
use crate::play::gauge::gauge_property::GaugePropertyType;
use crate::play::gauge::groove_gauge::GrooveGauge;
use crate::play::judge::judge_manager::{
    JudgeScore, JudgeTables, build_judge_tables, select_note, update_judge,
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
    pub gauge_type: usize,
    pub autoplay_mode: AutoplayMode,
}

/// Per-lane note state for tracking judge progress.
#[derive(Debug, Clone)]
struct LaneState {
    /// Notes for this lane, sorted by time.
    notes: Vec<LaneNote>,
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
    /// WAV ID for keysound playback.
    wav_id: u32,
    /// Damage for mine notes.
    damage: f64,
    /// Press time (for judge tracking). 0 if not pressed.
    play_time: i64,
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
            .map(|_| LaneState { notes: Vec::new() })
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
            .lane_to_player(event.key.min(self.mode.lane_count() - 1));
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

        // Copy judge table data to avoid borrow conflict
        let judge_table: Vec<[i64; 2]> = self.judge_tables.table_for_lane(is_scratch).to_vec();
        let judge_start = self.judge_tables.judge_start;
        let judge_end = self.judge_tables.judge_end;

        // Build candidates for note selection
        let candidates: Vec<(i64, bool, i64)> = self.lanes[lane]
            .notes
            .iter()
            .filter(|n| !matches!(n.note_type, NoteType::Mine | NoteType::Invisible))
            .map(|n| (n.time_us, !n.judged, n.play_time))
            .collect();

        let result = select_note(
            &candidates,
            press_time,
            &judge_table,
            judge_start,
            judge_end,
            crate::play::judge::judge_algorithm::JudgeAlgorithm::Combo,
            self.miss_condition,
        );

        if let Some((candidate_idx, judge)) = result {
            // Find the actual note index to update
            let playable_notes: Vec<usize> = self.lanes[lane]
                .notes
                .iter()
                .enumerate()
                .filter(|(_, n)| !matches!(n.note_type, NoteType::Mine | NoteType::Invisible))
                .map(|(i, _)| i)
                .collect();

            // Map candidate index back to lane note index
            if candidate_idx < playable_notes.len() {
                let note_idx = playable_notes[candidate_idx];
                let note = &self.lanes[lane].notes[note_idx];
                let time_diff = note.time_us - press_time;
                let vanish = judge < 5;
                let note_play_time = note.play_time;

                // Determine if this is a bomb-triggering judge (Great or better)
                let player = self.mode.lane_to_player(lane);
                let skin_offset = self.mode.lane_to_skin_offset(lane);

                // Set judge timer
                self.timer.set_judge(skin_offset, player, press_time);

                // Set bomb timer on Great or better
                if judge <= 1 {
                    self.timer.set_bomb(skin_offset, player, press_time);
                }

                // Update judge score
                update_judge(
                    &mut self.judge_score,
                    judge,
                    time_diff,
                    vanish,
                    self.miss_condition,
                    self.combo_cond,
                    note_play_time,
                );

                // Update gauge
                self.gauge.update_judge(judge);

                // Mark note as judged
                self.lanes[lane].notes[note_idx].judged = true;
                self.lanes[lane].notes[note_idx].play_time = press_time;
            }
        }
    }

    /// Process a key release on a lane (for LN/CN end judgment).
    fn process_key_release(&mut self, _lane: usize, _release_time: i64) {
        // LN end judgment is complex and will be refined later.
        // For basic play, normal notes don't need release processing.
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

    /// Process notes that have passed the miss window.
    fn process_miss_notes(&mut self, now_us: i64) {
        for lane_idx in 0..self.lanes.len() {
            let is_scratch = self.is_scratch_lane.get(lane_idx).copied().unwrap_or(false);
            let judge_table = self.judge_tables.table_for_lane(is_scratch);

            // Check unjudged notes that are past the late miss boundary
            let miss_boundary = if !judge_table.is_empty() {
                judge_table[judge_table.len() - 1][0] // Furthest late boundary
            } else {
                continue;
            };

            for note in &mut self.lanes[lane_idx].notes {
                if note.judged {
                    continue;
                }
                if matches!(note.note_type, NoteType::Mine | NoteType::Invisible) {
                    continue;
                }

                let dmtime = note.time_us - now_us;
                // Note is past the late boundary (dmtime < miss_boundary means it's too late)
                if dmtime < miss_boundary {
                    note.judged = true;
                    let judge = 5; // Miss
                    let time_diff = dmtime;
                    update_judge(
                        &mut self.judge_score,
                        judge,
                        time_diff,
                        true,
                        self.miss_condition,
                        self.combo_cond,
                        note.play_time,
                    );
                    self.gauge.update_judge(judge);
                }
            }
        }
    }

    /// Build the final play result.
    pub fn build_result(&self) -> PlayResult {
        let mut score = ScoreData::new(self.total_notes);
        score.early_counts = self.judge_score.early_counts;
        score.late_counts = self.judge_score.late_counts;
        score.max_combo = self.judge_score.max_combo;
        score.pass_notes = self.judge_score.pass_notes;
        score.min_bp = self.judge_score.judge_count(3)
            + self.judge_score.judge_count(4)
            + self.judge_score.judge_count(5);

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

        assert_eq!(state.judge_score().judge_count(0), 1); // PG
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

        assert_eq!(state.judge_score().judge_count(1), 1); // GR
    }

    #[test]
    fn process_miss_by_timeout() {
        let config = make_config(vec![make_normal_note(0, 2_000_000)], AutoplayMode::Off);
        let mut state = PlayState::new(config);
        state.create().unwrap();
        state.update(READY_DURATION_US + 1).unwrap();

        // Let time pass beyond the miss window
        state.update(3_000_000).unwrap();
        assert_eq!(state.judge_score().judge_count(5), 1); // Miss
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
        assert_eq!(state.judge_score().judge_count(0), 5);
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
        assert_eq!(result.score.judge_count(0), 1);
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
        assert_eq!(state.judge_score().judge_count(0), 0);
    }
}
