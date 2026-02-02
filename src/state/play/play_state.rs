use anyhow::Result;
use macroquad::prelude::*;

use crate::audio::{AudioDriver, KeysoundProcessor};
use crate::input::InputManager;
use crate::model::note::{Lane, NoteType};
use crate::model::{BMSModel, LaneConfig};
use crate::render::{LaneRenderer, NoteRenderer};
use crate::skin::{JudgeType, LastJudge, MainState, MainStateTimers, SkinRenderer};
use crate::state::play::{
    GaugeType, GrooveGauge, JudgeManager, JudgeRank, JudgeWindow, NoteWithIndex, PlayResult, Score,
};

/// State of the play session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayPhase {
    /// Counting down before play starts.
    Countdown,
    /// Actively playing.
    Playing,
    /// Play finished (clear or fail).
    Finished,
}

/// Main play state for a BMS gameplay session.
pub struct PlayState {
    model: BMSModel,
    audio_driver: AudioDriver,
    keysound_processor: KeysoundProcessor,
    input_manager: InputManager,
    judge_manager: JudgeManager,
    gauge: GrooveGauge,
    score: Score,
    lane_config: LaneConfig,
    hi_speed: f32,
    current_time_ms: f64,
    countdown_ms: f64,
    phase: PlayPhase,
    notes_by_lane: [Vec<NoteWithIndex>; 8],
    all_notes: Vec<NoteWithIndex>,
    last_judge_display: Option<(JudgeRank, f64)>,
    /// Optional skin renderer for custom UI.
    skin_renderer: Option<SkinRenderer>,
}

impl PlayState {
    /// Default countdown duration before play starts.
    const DEFAULT_COUNTDOWN_MS: f64 = 3000.0;

    /// Create a new PlayState.
    pub fn new(
        model: BMSModel,
        audio_driver: AudioDriver,
        keysound_processor: KeysoundProcessor,
        input_manager: InputManager,
        gauge_type: GaugeType,
        hi_speed: f32,
    ) -> Self {
        let total_notes = model.total_notes;
        // Default TOTAL value (affects gauge recovery rate)
        let total = 200.0;

        let (all_notes, notes_by_lane) = Self::organize_notes(&model);

        Self {
            model,
            audio_driver,
            keysound_processor,
            input_manager,
            judge_manager: JudgeManager::new(JudgeWindow::sevenkeys()),
            gauge: Self::create_gauge(gauge_type, total, total_notes),
            score: Score::new(total_notes as u32),
            lane_config: LaneConfig::default_7k(),
            hi_speed,
            current_time_ms: -Self::DEFAULT_COUNTDOWN_MS,
            countdown_ms: Self::DEFAULT_COUNTDOWN_MS,
            phase: PlayPhase::Countdown,
            notes_by_lane,
            all_notes,
            last_judge_display: None,
            skin_renderer: None,
        }
    }

    /// Set the skin renderer for custom UI rendering.
    pub fn set_skin_renderer(&mut self, renderer: SkinRenderer) {
        self.skin_renderer = Some(renderer);
    }

    fn create_gauge(gauge_type: GaugeType, total: f64, total_notes: usize) -> GrooveGauge {
        match gauge_type {
            GaugeType::Normal => GrooveGauge::normal(total, total_notes),
            GaugeType::Easy => GrooveGauge::new(
                crate::state::play::GaugeProperty::sevenkeys_easy(),
                total,
                total_notes,
            ),
            GaugeType::Hard => GrooveGauge::hard(total, total_notes),
            GaugeType::ExHard => GrooveGauge::exhard(total, total_notes),
            _ => GrooveGauge::normal(total, total_notes),
        }
    }

    fn organize_notes(model: &BMSModel) -> (Vec<NoteWithIndex>, [Vec<NoteWithIndex>; 8]) {
        let mut all_notes = Vec::new();
        let mut notes_by_lane: [Vec<NoteWithIndex>; 8] = Default::default();
        let mut index = 0;

        for timeline in model.timelines.entries() {
            for note in &timeline.notes {
                let nwi = NoteWithIndex {
                    index,
                    note: note.clone(),
                };
                all_notes.push(nwi.clone());
                notes_by_lane[note.lane.index()].push(nwi);
                index += 1;
            }
        }

        for lane_notes in &mut notes_by_lane {
            lane_notes.sort_by(|a, b| {
                a.note
                    .start_time_ms
                    .partial_cmp(&b.note.start_time_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        (all_notes, notes_by_lane)
    }

    /// Update the play state. Call once per frame.
    pub fn update(&mut self, delta_ms: f64) -> Result<()> {
        self.input_manager.update();

        match self.phase {
            PlayPhase::Countdown => {
                self.current_time_ms += delta_ms;
                if self.current_time_ms >= 0.0 {
                    self.phase = PlayPhase::Playing;
                    self.input_manager.reset_time();
                    self.input_manager.enable_logging();
                }
            }
            PlayPhase::Playing => {
                self.current_time_ms += delta_ms;

                // Process BGM
                self.keysound_processor
                    .update(&mut self.audio_driver, self.current_time_ms)?;

                // Process input
                self.process_input()?;

                // Check for misses
                self.check_misses();

                // Check for death (HARD/EXHARD)
                if self.gauge.is_dead() {
                    self.phase = PlayPhase::Finished;
                }

                // Check for song end
                if self.is_song_finished() {
                    self.phase = PlayPhase::Finished;
                }
            }
            PlayPhase::Finished => {}
        }

        Ok(())
    }

    fn process_input(&mut self) -> Result<()> {
        for lane in Lane::all_7k() {
            if self.input_manager.just_pressed(*lane) {
                self.process_press(*lane)?;
            }
            if self.input_manager.just_released(*lane) {
                self.process_release(*lane)?;
            }
        }
        Ok(())
    }

    fn process_press(&mut self, lane: Lane) -> Result<()> {
        let press_time_us = self.input_manager.press_time_us(lane);
        let press_time_ms = press_time_us as f64 / 1000.0 + self.countdown_ms;

        let result =
            self.judge_manager
                .judge_press(lane, press_time_ms, &self.notes_by_lane[lane.index()]);

        if let Some(ref result) = result {
            self.score.update(result.rank);
            self.gauge.update(result.rank);
            self.last_judge_display = Some((result.rank, self.current_time_ms));

            // Play keysound
            if let Some(nwi) = self.all_notes.iter().find(|n| n.index == result.note_index) {
                let _ = self
                    .keysound_processor
                    .play_player_keysound(&mut self.audio_driver, nwi.note.wav_id);
            }
        } else {
            // Empty press (Poor)
            self.score.update(JudgeRank::Poor);
            self.gauge.update(JudgeRank::Poor);
            self.last_judge_display = Some((JudgeRank::Poor, self.current_time_ms));

            // Play the closest upcoming note's keysound
            let wav_id = self
                .find_closest_note_for_empty_press(lane)
                .map(|nwi| nwi.note.wav_id);
            if let Some(wav_id) = wav_id {
                let _ = self
                    .keysound_processor
                    .play_player_keysound(&mut self.audio_driver, wav_id);
            }
        }

        Ok(())
    }

    fn process_release(&mut self, lane: Lane) -> Result<()> {
        let release_time_us = self.input_manager.release_time_us(lane);
        let release_time_ms = release_time_us as f64 / 1000.0 + self.countdown_ms;

        if let Some(result) = self.judge_manager.judge_release(
            lane,
            release_time_ms,
            &self.notes_by_lane[lane.index()],
        ) {
            self.score.update(result.rank);
            self.gauge.update(result.rank);
            self.last_judge_display = Some((result.rank, self.current_time_ms));
        }

        Ok(())
    }

    fn check_misses(&mut self) {
        let results = self
            .judge_manager
            .check_misses(self.current_time_ms, &self.all_notes);

        for result in results {
            self.score.update(result.rank);
            self.gauge.update(result.rank);
            self.last_judge_display = Some((result.rank, self.current_time_ms));
        }
    }

    fn find_closest_note_for_empty_press(&self, lane: Lane) -> Option<&NoteWithIndex> {
        self.notes_by_lane[lane.index()]
            .iter()
            .filter(|n| {
                !self.judge_manager.is_judged(n.index)
                    && matches!(n.note.note_type, NoteType::Normal | NoteType::LongStart)
            })
            .min_by(|a, b| {
                let diff_a = (a.note.start_time_ms - self.current_time_ms).abs();
                let diff_b = (b.note.start_time_ms - self.current_time_ms).abs();
                diff_a
                    .partial_cmp(&diff_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn is_song_finished(&self) -> bool {
        let last_time = self.model.timelines.last_time_ms();
        self.current_time_ms > last_time + 1000.0
    }

    /// Draw the play state.
    pub fn draw(&self) {
        // Draw skin if available
        if let Some(ref skin) = self.skin_renderer {
            let main_state = self.create_main_state();
            let now_us = (self.current_time_ms.max(0.0) * 1000.0) as i64;
            skin.draw(&main_state, now_us);
        }

        // Always draw lane and notes (skin typically doesn't handle these)
        let lane_renderer = LaneRenderer::new(&self.lane_config);
        lane_renderer.draw(&self.model.timelines, self.current_time_ms, self.hi_speed);

        let note_renderer = NoteRenderer::new(&self.lane_config);
        note_renderer.draw_with_filter(
            &self.model.timelines,
            self.current_time_ms,
            self.hi_speed,
            |index| !self.judge_manager.is_judged(index),
        );

        // Draw fallback UI if no skin
        if self.skin_renderer.is_none() {
            self.draw_gauge();
            self.draw_score();
            self.draw_combo();
            self.draw_judge();
        }

        // Always show debug info
        self.draw_info();
    }

    fn draw_gauge(&self) {
        let x = 550.0;
        let y = 50.0;
        let width = 300.0;
        let height = 20.0;

        // Background
        draw_rectangle(x, y, width, height, DARKGRAY);

        // Gauge fill
        let fill_width = width * self.gauge.ratio() as f32;
        let gauge_color = match self.gauge.gauge_type() {
            GaugeType::Normal | GaugeType::Easy | GaugeType::AssistEasy => {
                if self.gauge.is_clear() {
                    Color::new(0.2, 0.8, 0.2, 1.0)
                } else {
                    Color::new(0.2, 0.4, 0.8, 1.0)
                }
            }
            GaugeType::Hard | GaugeType::ExHard | GaugeType::Hazard => {
                Color::new(0.8, 0.2, 0.2, 1.0)
            }
        };
        draw_rectangle(x, y, fill_width, height, gauge_color);

        // Border for clear threshold
        if self.gauge.border() > 0.0 {
            let border_x = x + width * (self.gauge.border() / 100.0) as f32;
            draw_line(border_x, y, border_x, y + height, 2.0, WHITE);
        }

        // Percentage text
        draw_text(
            &format!("{:.1}%", self.gauge.value()),
            x + width + 10.0,
            y + 15.0,
            20.0,
            WHITE,
        );
    }

    fn draw_score(&self) {
        let x = 550.0;
        let y = 90.0;

        draw_text(
            &format!("EX-SCORE: {}", self.score.ex_score()),
            x,
            y,
            24.0,
            WHITE,
        );
        draw_text(
            &format!("MAX COMBO: {}", self.score.max_combo),
            x,
            y + 28.0,
            20.0,
            YELLOW,
        );

        // Judge counts
        let y = y + 70.0;
        draw_text(
            &format!("PG: {}", self.score.pg_count),
            x,
            y,
            16.0,
            Color::new(0.0, 1.0, 1.0, 1.0),
        );
        draw_text(
            &format!("GR: {}", self.score.gr_count),
            x + 80.0,
            y,
            16.0,
            Color::new(1.0, 1.0, 0.0, 1.0),
        );
        draw_text(
            &format!("GD: {}", self.score.gd_count),
            x + 160.0,
            y,
            16.0,
            Color::new(0.0, 1.0, 0.0, 1.0),
        );
        draw_text(
            &format!("BD: {}", self.score.bd_count),
            x,
            y + 20.0,
            16.0,
            Color::new(0.5, 0.5, 1.0, 1.0),
        );
        draw_text(
            &format!("PR: {}", self.score.pr_count),
            x + 80.0,
            y + 20.0,
            16.0,
            GRAY,
        );
        draw_text(
            &format!("MS: {}", self.score.ms_count),
            x + 160.0,
            y + 20.0,
            16.0,
            Color::new(1.0, 0.3, 0.3, 1.0),
        );
    }

    fn draw_combo(&self) {
        if self.score.combo > 0 {
            let combo_text = format!("{}", self.score.combo);
            let font_size = 48.0;
            let x = self.lane_config.offset_x + self.lane_config.total_width / 2.0 - 30.0;
            let y = self.lane_config.judge_line_y - 100.0;

            draw_text(&combo_text, x, y, font_size, WHITE);
            draw_text("COMBO", x - 10.0, y + 30.0, 16.0, YELLOW);
        }
    }

    fn draw_judge(&self) {
        if let Some((rank, time)) = self.last_judge_display {
            let elapsed = self.current_time_ms - time;
            if elapsed < 500.0 {
                let alpha = (1.0 - elapsed / 500.0) as f32;
                let (text, color) = match rank {
                    JudgeRank::PerfectGreat => ("PERFECT GREAT", Color::new(0.0, 1.0, 1.0, alpha)),
                    JudgeRank::Great => ("GREAT", Color::new(1.0, 1.0, 0.0, alpha)),
                    JudgeRank::Good => ("GOOD", Color::new(0.0, 1.0, 0.0, alpha)),
                    JudgeRank::Bad => ("BAD", Color::new(0.5, 0.5, 1.0, alpha)),
                    JudgeRank::Poor => ("POOR", Color::new(0.5, 0.5, 0.5, alpha)),
                    JudgeRank::Miss => ("MISS", Color::new(1.0, 0.3, 0.3, alpha)),
                };

                let x = self.lane_config.offset_x + self.lane_config.total_width / 2.0 - 80.0;
                let y = self.lane_config.judge_line_y - 50.0;
                draw_text(text, x, y, 32.0, color);
            }
        }
    }

    fn draw_info(&self) {
        let x = 550.0;
        let y = 250.0;

        draw_text(&format!("Title: {}", self.model.title), x, y, 18.0, WHITE);
        draw_text(
            &format!("Time: {:.1}ms", self.current_time_ms),
            x,
            y + 24.0,
            18.0,
            YELLOW,
        );
        draw_text(
            &format!("Hi-Speed: {:.2}x", self.hi_speed),
            x,
            y + 48.0,
            18.0,
            YELLOW,
        );

        let phase_text = match self.phase {
            PlayPhase::Countdown => "COUNTDOWN",
            PlayPhase::Playing => "PLAYING",
            PlayPhase::Finished => {
                if self.gauge.is_clear() {
                    "CLEAR!"
                } else {
                    "FAILED"
                }
            }
        };
        draw_text(phase_text, x, y + 72.0, 24.0, GREEN);

        // Fast/Slow
        draw_text(
            &format!(
                "FAST: {} / SLOW: {}",
                self.judge_manager.fast_count(),
                self.judge_manager.slow_count()
            ),
            x,
            y + 100.0,
            16.0,
            GRAY,
        );
    }

    /// Check if the play is finished.
    pub fn is_finished(&self) -> bool {
        self.phase == PlayPhase::Finished
    }

    /// Get the play result.
    pub fn take_result(&self) -> PlayResult {
        PlayResult::new(
            self.score.clone(),
            self.gauge.value(),
            self.gauge.gauge_type(),
            self.gauge.is_clear(),
            self.current_time_ms,
            self.judge_manager.fast_count(),
            self.judge_manager.slow_count(),
        )
    }

    /// Get the current hi-speed.
    pub fn hi_speed(&self) -> f32 {
        self.hi_speed
    }

    /// Set the hi-speed.
    pub fn set_hi_speed(&mut self, hi_speed: f32) {
        self.hi_speed = hi_speed.clamp(0.25, 5.0);
    }

    /// Get the current phase.
    pub fn phase(&self) -> PlayPhase {
        self.phase
    }

    /// Create a MainState snapshot for skin rendering.
    pub fn create_main_state(&self) -> MainState {
        let mut state = MainState::new();

        // Judge counts
        state.pg_count = self.score.pg_count;
        state.gr_count = self.score.gr_count;
        state.gd_count = self.score.gd_count;
        state.bd_count = self.score.bd_count;
        state.pr_count = self.score.pr_count;
        state.ms_count = self.score.ms_count;

        // Combo
        state.combo = self.score.combo;
        state.max_combo = self.score.max_combo;

        // Gauge
        state.gauge_value = self.gauge.value();
        state.gauge_type = Self::gauge_type_to_int(self.gauge.gauge_type());

        // Score
        state.ex_score = self.score.ex_score();
        state.score_rate = self.score.clear_rate();

        // BPM
        state.current_bpm = self.model.initial_bpm;
        state.min_bpm = self.model.initial_bpm;
        state.max_bpm = self.model.initial_bpm;

        // Time
        state.current_time_ms = self.current_time_ms.max(0.0);
        state.total_time_ms = self.model.timelines.last_time_ms();

        // Notes
        state.total_notes = self.model.total_notes as u32;

        // Hi-speed
        state.hi_speed = self.hi_speed;

        // Play state flags
        state.is_ready = self.phase == PlayPhase::Countdown;
        state.is_playing = self.phase == PlayPhase::Playing;
        state.is_finished = self.phase == PlayPhase::Finished;
        state.is_clear = self.phase == PlayPhase::Finished && self.gauge.is_clear();
        state.is_failed = self.phase == PlayPhase::Finished && !self.gauge.is_clear();

        // Last judge
        if let Some((rank, time)) = self.last_judge_display {
            let is_early = self.judge_manager.fast_count() > self.judge_manager.slow_count();
            state.last_judge = Some(LastJudge {
                rank: Self::judge_rank_to_type(rank),
                is_early,
                time_ms: time,
            });
        }

        // Timers
        state.timers = self.create_timers();

        state
    }

    /// Convert JudgeRank to skin JudgeType.
    fn judge_rank_to_type(rank: JudgeRank) -> JudgeType {
        match rank {
            JudgeRank::PerfectGreat => JudgeType::Perfect,
            JudgeRank::Great => JudgeType::Great,
            JudgeRank::Good => JudgeType::Good,
            JudgeRank::Bad => JudgeType::Bad,
            JudgeRank::Poor => JudgeType::Poor,
            JudgeRank::Miss => JudgeType::Miss,
        }
    }

    /// Convert GaugeType to skin integer.
    fn gauge_type_to_int(gauge_type: GaugeType) -> i32 {
        match gauge_type {
            GaugeType::AssistEasy => 1,
            GaugeType::Easy => 1,
            GaugeType::Normal => 0,
            GaugeType::Hard => 2,
            GaugeType::ExHard => 3,
            GaugeType::Hazard => 3,
        }
    }

    /// Create timer values for skin rendering.
    fn create_timers(&self) -> MainStateTimers {
        use crate::skin::skin_property::TIMER_OFF_VALUE;

        let mut timers = MainStateTimers::new();

        // Ready timer (starts at countdown start)
        if self.phase != PlayPhase::Countdown || self.current_time_ms >= -self.countdown_ms {
            timers.ready = 0; // Ready from the beginning
        }

        // Play timer (starts when playing begins)
        if self.phase == PlayPhase::Playing || self.phase == PlayPhase::Finished {
            timers.play = 0; // Play started
        }

        // Judge timer (set when judge happens)
        if let Some((_, time)) = self.last_judge_display {
            let elapsed = self.current_time_ms - time;
            if elapsed < 500.0 {
                timers.judge_1p = (time * 1000.0) as i64;
            }
        }

        // Combo timer (same as judge timer for simplicity)
        if self.score.combo > 0 {
            if let Some((_, time)) = self.last_judge_display {
                timers.combo_1p = (time * 1000.0) as i64;
            }
        }

        // Key on/off timers from input manager
        for lane in Lane::all_7k() {
            let lane_idx = lane.index();
            if self.input_manager.is_pressed(*lane) {
                let press_time = self.input_manager.press_time_us(*lane) as i64;
                timers.keyon_1p[lane_idx] = press_time;
                timers.keyoff_1p[lane_idx] = TIMER_OFF_VALUE;
            } else {
                let release_time = self.input_manager.release_time_us(*lane);
                if release_time > 0 {
                    timers.keyoff_1p[lane_idx] = release_time as i64;
                }
            }
        }

        timers
    }
}
