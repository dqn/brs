use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use crate::audio::{AudioDriver, KeysoundProcessor};
use crate::config::AppConfig;
use crate::input::InputManager;
use crate::model::note::{LANE_COUNT, Lane, NoteType};
use crate::model::{BMSModel, LaneConfig, LaneCoverSettings, PlayMode};
use crate::render::{BgaProcessor, LaneRenderer, NoteRenderer};
use crate::replay::ReplayPlayer;
use crate::skin::{JudgeType, LastJudge, MainState, MainStateTimers, SkinRenderer};
use crate::state::play::{
    AutoplayMode, AutoplayProcessor, GaugeType, GrooveGauge, JudgeManager, JudgeRank, JudgeWindow,
    NoteWithIndex, PlayResult, Score,
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
    lane_cover: LaneCoverSettings,
    hi_speed: f32,
    playback_speed: f64,
    practice_range: Option<(f64, f64)>,
    input_time_offset_ms: f64,
    current_time_ms: f64,
    countdown_ms: f64,
    skin_time_us: i64,
    skin_timers: MainStateTimers,
    phase: PlayPhase,
    notes_by_lane: [Vec<NoteWithIndex>; LANE_COUNT],
    all_notes: Vec<NoteWithIndex>,
    last_judge_display: Option<(JudgeRank, f64)>,
    bga_processor: Option<BgaProcessor>,
    bga_enabled: bool,
    stagefile_texture: Option<Texture2D>,
    backbmp_texture: Option<Texture2D>,
    banner_texture: Option<Texture2D>,
    player_name: String,
    rival_name: String,
    /// Optional skin renderer for custom UI.
    skin_renderer: Option<SkinRenderer>,
    /// Optional replay player for playback mode.
    replay_player: Option<ReplayPlayer>,
    /// Optional autoplay processor for automatic play.
    autoplay_processor: Option<AutoplayProcessor>,
}

impl PlayState {
    /// Default countdown duration before play starts.
    const DEFAULT_COUNTDOWN_MS: f64 = 3000.0;
    /// Default practice loop length when only start is set.
    /// 練習開始のみ設定したときのデフォルトループ長。
    const DEFAULT_PRACTICE_LENGTH_MS: f64 = 10000.0;

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
        let play_mode = model.play_mode;
        let total = model.total;
        let long_note_mode = model.long_note_mode;
        let judge_window = JudgeWindow::from_model(&model);
        let player_name = AppConfig::load().unwrap_or_default().player_name;

        let (all_notes, notes_by_lane) = Self::organize_notes(&model);

        // Use beatoraja-style narrow lane layout for 7K
        let lane_config = if play_mode == PlayMode::Beat7K {
            LaneConfig::beatoraja_7k()
        } else {
            LaneConfig::for_mode(play_mode)
        };

        Self {
            model,
            audio_driver,
            keysound_processor,
            input_manager,
            judge_manager: JudgeManager::new(judge_window, long_note_mode),
            gauge: Self::create_gauge(gauge_type, total, total_notes),
            score: Score::new(total_notes as u32),
            lane_config,
            lane_cover: LaneCoverSettings::default(),
            hi_speed,
            playback_speed: 1.0,
            practice_range: None,
            input_time_offset_ms: 0.0,
            current_time_ms: -Self::DEFAULT_COUNTDOWN_MS,
            countdown_ms: Self::DEFAULT_COUNTDOWN_MS,
            skin_time_us: 0,
            skin_timers: {
                let mut timers = MainStateTimers::new();
                timers.ready = 0;
                timers
            },
            phase: PlayPhase::Countdown,
            notes_by_lane,
            all_notes,
            last_judge_display: None,
            bga_processor: None,
            bga_enabled: true,
            stagefile_texture: None,
            backbmp_texture: None,
            banner_texture: None,
            player_name,
            rival_name: String::new(),
            skin_renderer: None,
            replay_player: None,
            autoplay_processor: None,
        }
    }

    /// Create a new PlayState for replay playback.
    pub fn new_replay(
        model: BMSModel,
        audio_driver: AudioDriver,
        keysound_processor: KeysoundProcessor,
        input_manager: InputManager,
        gauge_type: GaugeType,
        hi_speed: f32,
        replay_player: ReplayPlayer,
    ) -> Self {
        let mut state = Self::new(
            model,
            audio_driver,
            keysound_processor,
            input_manager,
            gauge_type,
            hi_speed,
        );
        state.replay_player = Some(replay_player);
        state
    }

    /// Create a new PlayState for autoplay.
    pub fn new_autoplay(
        model: BMSModel,
        audio_driver: AudioDriver,
        keysound_processor: KeysoundProcessor,
        input_manager: InputManager,
        gauge_type: GaugeType,
        hi_speed: f32,
        autoplay_mode: AutoplayMode,
    ) -> Self {
        let autoplay_processor = if autoplay_mode != AutoplayMode::Off {
            Some(AutoplayProcessor::new(autoplay_mode, &model))
        } else {
            None
        };

        let mut state = Self::new(
            model,
            audio_driver,
            keysound_processor,
            input_manager,
            gauge_type,
            hi_speed,
        );
        state.autoplay_processor = autoplay_processor;
        state
    }

    /// Check if this is a replay playback.
    pub fn is_replay(&self) -> bool {
        self.replay_player.is_some()
    }

    /// Check if autoplay is active.
    pub fn is_autoplay(&self) -> bool {
        self.autoplay_processor.is_some()
    }

    /// Get the autoplay mode if active.
    pub fn autoplay_mode(&self) -> Option<AutoplayMode> {
        self.autoplay_processor.as_ref().map(|p| p.mode())
    }

    /// Set the skin renderer for custom UI rendering.
    pub fn set_skin_renderer(&mut self, renderer: SkinRenderer) {
        if renderer.header().playstart > 0 {
            self.set_countdown_ms(renderer.header().playstart as f64);
        }
        self.skin_renderer = Some(renderer);
    }

    /// Update countdown duration (ms) and reset timing if still in countdown.
    pub fn set_countdown_ms(&mut self, countdown_ms: f64) {
        use crate::skin::skin_property::TIMER_OFF_VALUE;

        self.countdown_ms = countdown_ms;
        if self.phase == PlayPhase::Countdown {
            self.current_time_ms = -countdown_ms;
            self.skin_time_us = 0;
            self.skin_timers.ready = 0;
            self.skin_timers.play = TIMER_OFF_VALUE;
        }
    }

    /// Load BGA images and events for this play state.
    pub async fn load_bga(&mut self, base_dir: &Path) -> usize {
        let mut loaded = 0;
        if self.model.bga_files.is_empty() && self.model.poor_bga_file.is_none() {
            self.bga_processor = None;
        } else {
            let mut processor = BgaProcessor::new();

            let mut bga_files: std::collections::HashMap<u16, String> = self
                .model
                .bga_files
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect();
            let mut events = self.model.bga_events.clone();

            if let Some(poor_file) = &self.model.poor_bga_file {
                let mut poor_id = u16::MAX;
                while bga_files.contains_key(&poor_id) {
                    poor_id = poor_id.saturating_sub(1);
                }
                bga_files.insert(poor_id, poor_file.clone());
                events.push(crate::model::BgaEvent {
                    time_ms: 0.0,
                    bga_id: poor_id,
                    layer: crate::model::BgaLayer::Poor,
                });
            }

            processor.set_events(events);
            loaded = processor.load_images(&bga_files, base_dir).await;
            self.bga_processor = Some(processor);
        }

        self.stagefile_texture =
            Self::load_optional_texture(base_dir, &self.model.stage_file).await;
        self.backbmp_texture = Self::load_optional_texture(base_dir, &self.model.back_bmp).await;
        self.banner_texture = Self::load_optional_texture(base_dir, &self.model.banner).await;

        loaded
    }

    async fn load_optional_texture(base_dir: &Path, path: &Option<String>) -> Option<Texture2D> {
        let path = path.as_ref()?;
        let full_path = base_dir.join(path);
        let ext = full_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let is_image_ext = matches!(ext.as_str(), "bmp" | "png" | "jpg" | "jpeg");

        if is_image_ext {
            if let Some(texture) = crate::render::bga::load_texture_from_path(&full_path) {
                return Some(texture);
            }
        }

        let stem = full_path.with_extension("");
        for ext in &["bmp", "png", "jpg", "jpeg"] {
            let alt_path = stem.with_extension(ext);
            if let Some(texture) = crate::render::bga::load_texture_from_path(&alt_path) {
                return Some(texture);
            }
        }

        None
    }

    fn create_gauge(gauge_type: GaugeType, total: f64, total_notes: usize) -> GrooveGauge {
        match gauge_type {
            GaugeType::AssistEasy => GrooveGauge::assist_easy(total, total_notes),
            GaugeType::LightAssistEasy => GrooveGauge::light_assist_easy(total, total_notes),
            GaugeType::Easy => GrooveGauge::easy(total, total_notes),
            GaugeType::Normal => GrooveGauge::normal(total, total_notes),
            GaugeType::Hard => GrooveGauge::hard(total, total_notes),
            GaugeType::ExHard => GrooveGauge::exhard(total, total_notes),
            GaugeType::Hazard => GrooveGauge::hazard(total, total_notes),
            GaugeType::Class => GrooveGauge::class(total, total_notes),
        }
    }

    fn organize_notes(model: &BMSModel) -> (Vec<NoteWithIndex>, [Vec<NoteWithIndex>; LANE_COUNT]) {
        let mut all_notes = Vec::new();
        let mut notes_by_lane: [Vec<NoteWithIndex>; LANE_COUNT] = Default::default();
        let mut index = 0;

        for timeline in model.timelines.entries() {
            for note in &timeline.notes {
                let nwi = NoteWithIndex {
                    index,
                    note: note.clone(),
                };
                all_notes.push(nwi.clone());
                if note.lane.index() < notes_by_lane.len() {
                    notes_by_lane[note.lane.index()].push(nwi);
                }
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
        self.skin_time_us += (delta_ms * 1000.0) as i64;
        self.input_manager.update();

        match self.phase {
            PlayPhase::Countdown => {
                self.current_time_ms += delta_ms;
                if self.current_time_ms >= 0.0 {
                    self.phase = PlayPhase::Playing;
                    use crate::skin::skin_property::TIMER_OFF_VALUE;
                    self.skin_timers.play = self.skin_time_us;
                    self.skin_timers.ready = TIMER_OFF_VALUE;
                    self.input_manager.reset_time();
                    self.sync_input_time_offset();
                    // Only enable logging for live play, not replay
                    if self.replay_player.is_none() && self.should_record_replay() {
                        self.input_manager.enable_logging();
                    }
                }
            }
            PlayPhase::Playing => {
                let scaled_delta_ms = delta_ms * self.playback_speed;
                self.current_time_ms += scaled_delta_ms;

                if self.apply_practice_loop() {
                    return Ok(());
                }

                // Update replay player if in replay mode
                if let Some(ref mut player) = self.replay_player {
                    let current_time_us = (self.current_time_ms * 1000.0).max(0.0) as u64;
                    player.update(current_time_us);
                }

                // Update autoplay processor if in autoplay mode
                if let Some(ref mut processor) = self.autoplay_processor {
                    processor.update(self.current_time_ms);
                }

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

        if let Some(ref mut bga) = self.bga_processor {
            bga.update(self.current_time_ms);
        }

        Ok(())
    }

    fn process_input(&mut self) -> Result<()> {
        let lanes: Vec<Lane> = self.lane_config.lanes().to_vec();
        for lane in lanes {
            let (just_pressed, just_released) = self.get_lane_input(lane);

            if just_pressed {
                self.process_press(lane)?;
            }
            if just_released {
                self.process_release(lane)?;
            }
        }
        Ok(())
    }

    /// Get input state for a lane, considering replay, autoplay, and manual input.
    fn get_lane_input(&self, lane: Lane) -> (bool, bool) {
        // Replay takes highest priority
        if let Some(ref player) = self.replay_player {
            return (player.just_pressed(lane), player.just_released(lane));
        }

        // Check if autoplay handles this lane
        if let Some(ref processor) = self.autoplay_processor {
            if processor.mode().handles_lane(lane) {
                return (processor.just_pressed(lane), processor.just_released(lane));
            }
        }

        // Fall back to manual input
        (
            self.input_manager.just_pressed(lane),
            self.input_manager.just_released(lane),
        )
    }

    fn scaled_input_time_ms(&self, time_us: u64) -> f64 {
        (time_us as f64 / 1000.0) * self.playback_speed
    }

    fn press_time_ms_for_judge(&self, lane: Lane) -> f64 {
        if let Some(ref player) = self.replay_player {
            return player.press_time_us(lane) as f64 / 1000.0;
        }
        if let Some(ref processor) = self.autoplay_processor {
            if processor.mode().handles_lane(lane) {
                return processor.press_time_us(lane) as f64 / 1000.0;
            }
        }

        let input_time_ms = self.scaled_input_time_ms(self.input_manager.press_time_us(lane));
        input_time_ms - self.input_time_offset_ms
    }

    fn release_time_ms_for_judge(&self, lane: Lane) -> f64 {
        if let Some(ref player) = self.replay_player {
            return player.release_time_us(lane) as f64 / 1000.0;
        }
        if let Some(ref processor) = self.autoplay_processor {
            if processor.mode().handles_lane(lane) {
                return processor.release_time_us(lane) as f64 / 1000.0;
            }
        }

        let input_time_ms = self.scaled_input_time_ms(self.input_manager.release_time_us(lane));
        input_time_ms - self.input_time_offset_ms
    }

    fn sync_input_time_offset(&mut self) {
        let input_time_ms = self.scaled_input_time_ms(self.input_manager.current_time_us());
        self.input_time_offset_ms = input_time_ms - self.current_time_ms;
    }

    fn register_judge(&mut self, rank: JudgeRank) {
        self.last_judge_display = Some((rank, self.current_time_ms));
        self.skin_timers.judge_1p = self.skin_time_us;
        if self.score.combo > 0 {
            self.skin_timers.combo_1p = self.skin_time_us;
        }
    }

    fn set_keyon_timer(&mut self, lane: Lane) {
        use crate::skin::skin_property::TIMER_OFF_VALUE;

        let lane_idx = lane.index();
        if lane_idx >= self.skin_timers.keyon_1p.len() {
            return;
        }
        self.skin_timers.keyon_1p[lane_idx] = self.skin_time_us;
        self.skin_timers.keyoff_1p[lane_idx] = TIMER_OFF_VALUE;
    }

    fn set_keyoff_timer(&mut self, lane: Lane) {
        use crate::skin::skin_property::TIMER_OFF_VALUE;

        let lane_idx = lane.index();
        if lane_idx >= self.skin_timers.keyoff_1p.len() {
            return;
        }
        self.skin_timers.keyoff_1p[lane_idx] = self.skin_time_us;
        self.skin_timers.keyon_1p[lane_idx] = TIMER_OFF_VALUE;
    }

    fn process_press(&mut self, lane: Lane) -> Result<()> {
        self.set_keyon_timer(lane);
        let press_time_ms = self.press_time_ms_for_judge(lane);

        let result =
            self.judge_manager
                .judge_press(lane, press_time_ms, &self.notes_by_lane[lane.index()]);

        if let Some(ref result) = result {
            self.score.update(result.rank);
            self.gauge.update(result.rank);
            self.register_judge(result.rank);
            if matches!(result.rank, JudgeRank::Poor | JudgeRank::Miss) {
                self.trigger_poor_bga();
            }

            // Play keysound
            if let Some(nwi) = self.all_notes.iter().find(|n| n.index == result.note_index) {
                let _ = self
                    .keysound_processor
                    .play_player_keysound(&mut self.audio_driver, nwi.note.wav_id);
            }
        }

        // Mine handling (applies even when a visible note was hit)
        if let Some((mine_index, damage)) = self
            .find_note_hit(lane, press_time_ms, NoteType::Mine)
            .map(|mine| (mine.index, mine.note.mine_damage.unwrap_or(2.0)))
        {
            self.judge_manager.mark_judged(mine_index);
            self.gauge.apply_mine_damage(damage);
            self.register_judge(JudgeRank::Poor);
            self.trigger_poor_bga();
        }

        if result.is_none() {
            if let Some((invisible_index, wav_id)) = self
                .find_note_hit(lane, press_time_ms, NoteType::Invisible)
                .map(|invisible| (invisible.index, invisible.note.wav_id))
            {
                self.judge_manager.mark_judged(invisible_index);
                let _ = self
                    .keysound_processor
                    .play_player_keysound(&mut self.audio_driver, wav_id);
                return Ok(());
            }

            // Empty press (Poor)
            self.score.update(JudgeRank::Poor);
            self.gauge.update(JudgeRank::Poor);
            self.register_judge(JudgeRank::Poor);
            self.trigger_poor_bga();

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
        self.set_keyoff_timer(lane);
        let release_time_ms = self.release_time_ms_for_judge(lane);

        if let Some(result) = self.judge_manager.judge_release(
            lane,
            release_time_ms,
            &self.notes_by_lane[lane.index()],
        ) {
            self.score.update(result.rank);
            self.gauge.update(result.rank);
            self.register_judge(result.rank);
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
            self.register_judge(result.rank);
            if matches!(result.rank, JudgeRank::Poor | JudgeRank::Miss) {
                self.trigger_poor_bga();
            }
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

    fn find_note_hit(
        &self,
        lane: Lane,
        press_time_ms: f64,
        note_type: NoteType,
    ) -> Option<&NoteWithIndex> {
        let window = self.judge_manager.window();
        self.notes_by_lane[lane.index()]
            .iter()
            .filter(|n| {
                !self.judge_manager.is_judged(n.index)
                    && n.note.note_type == note_type
                    && (n.note.start_time_ms - press_time_ms).abs() <= window.pr
            })
            .min_by(|a, b| {
                let diff_a = (a.note.start_time_ms - press_time_ms).abs();
                let diff_b = (b.note.start_time_ms - press_time_ms).abs();
                diff_a
                    .partial_cmp(&diff_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn trigger_poor_bga(&mut self) {
        if let Some(ref mut bga) = self.bga_processor {
            bga.trigger_poor(self.current_time_ms);
        }
    }

    fn apply_practice_seek(&mut self, time_ms: f64) {
        use crate::skin::skin_property::TIMER_OFF_VALUE;

        self.current_time_ms = time_ms.max(0.0);
        self.score.reset();
        self.gauge.reset();
        self.judge_manager.reset();
        self.last_judge_display = None;
        self.skin_timers.judge_1p = TIMER_OFF_VALUE;
        self.skin_timers.combo_1p = TIMER_OFF_VALUE;
        self.skin_timers.bomb_1p = [TIMER_OFF_VALUE; 8];
        self.skin_timers.hold_1p = [TIMER_OFF_VALUE; 8];
        self.skin_timers.keyon_1p = [TIMER_OFF_VALUE; 8];
        self.skin_timers.keyoff_1p = [TIMER_OFF_VALUE; 8];

        self.keysound_processor.seek(self.current_time_ms);
        self.audio_driver.stop_all();

        if let Some(ref mut player) = self.replay_player {
            let time_us = (self.current_time_ms * 1000.0).max(0.0) as u64;
            player.seek(time_us);
        }

        if let Some(ref mut processor) = self.autoplay_processor {
            processor.seek(self.current_time_ms);
        }

        if let Some(ref mut bga) = self.bga_processor {
            bga.reset();
            bga.update(self.current_time_ms);
        }

        self.sync_input_time_offset();
    }

    fn apply_practice_loop(&mut self) -> bool {
        let Some((start_ms, end_ms)) = self.practice_range else {
            return false;
        };

        if self.current_time_ms <= end_ms {
            return false;
        }

        self.apply_practice_seek(start_ms);
        true
    }

    fn is_song_finished(&self) -> bool {
        let last_time = self.model.timelines.last_time_ms();
        self.current_time_ms > last_time + 1000.0
    }

    /// Draw the play state.
    pub fn draw(&self) {
        // Use beatoraja-style UI if no skin is loaded
        if self.skin_renderer.is_none() {
            self.draw_beatoraja_ui();
            return;
        }

        // Skin-based rendering
        // Draw skin
        if let Some(ref skin) = self.skin_renderer {
            let main_state = self.create_main_state();
            let now_us = self.skin_time_us;
            skin.draw_with_bga(&main_state, now_us, |dst, obj_data| {
                if !self.bga_enabled {
                    return;
                }

                let fallback_texture = self
                    .backbmp_texture
                    .as_ref()
                    .or(self.stagefile_texture.as_ref());
                let stretch = obj_data.stretch;

                let mut drew_bga = false;
                if let Some(ref bga) = self.bga_processor {
                    if bga.has_active_image() {
                        bga.draw_with_alpha_and_stretch(dst.x, dst.y, dst.w, dst.h, dst.a, stretch);
                        drew_bga = true;
                    }
                }

                if !drew_bga {
                    let color = Color::new(1.0, 1.0, 1.0, (dst.a / 255.0).clamp(0.0, 1.0));
                    if let Some(texture) = fallback_texture {
                        crate::render::bga::draw_texture_with_stretch(
                            texture, dst.x, dst.y, dst.w, dst.h, color, stretch,
                        );
                    }
                }
            });
        }

        // Draw lane and notes
        let render_lane_config = self.render_lane_config();
        let lane_renderer = LaneRenderer::new(&render_lane_config);
        if self.skin_renderer.is_some() {
            lane_renderer.draw_for_skin(&self.model.timelines, self.current_time_ms, self.hi_speed);
        } else {
            lane_renderer.draw(&self.model.timelines, self.current_time_ms, self.hi_speed);
        }

        let note_renderer = NoteRenderer::new(&render_lane_config);
        note_renderer.draw_with_cover(
            &self.model.timelines,
            self.current_time_ms,
            self.hi_speed,
            &self.lane_cover,
            |index| !self.judge_manager.is_judged(index),
        );

        // Draw lane cover overlay
        note_renderer.draw_cover_overlay(&self.lane_cover);

        // Draw combo and judge only for built-in UI.
        // ビルトインUIの場合のみコンボと判定を描画する。
        if self.skin_renderer.is_none() {
            self.draw_combo();
            self.draw_judge();
        }
    }

    #[allow(dead_code)]
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
            GaugeType::Normal
            | GaugeType::Easy
            | GaugeType::AssistEasy
            | GaugeType::LightAssistEasy => {
                if self.gauge.is_clear() {
                    Color::new(0.2, 0.8, 0.2, 1.0)
                } else {
                    Color::new(0.2, 0.4, 0.8, 1.0)
                }
            }
            GaugeType::Hard | GaugeType::ExHard | GaugeType::Hazard => {
                Color::new(0.8, 0.2, 0.2, 1.0)
            }
            GaugeType::Class => Color::new(0.8, 0.5, 0.2, 1.0), // Orange for Class
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    /// Draw beatoraja-style UI layout.
    pub fn draw_beatoraja_ui(&self) {
        // Dark background
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

        // BGA area position (referenced by multiple panels)
        let bga_x = 560.0;
        let bga_y = 160.0;
        let bga_w = 640.0;
        let bga_h = 480.0;

        // Draw BGA or SOUND ONLY
        let bga_drawn = if self.bga_enabled {
            if let Some(ref bga) = self.bga_processor {
                if bga.has_images() {
                    bga.draw(bga_x, bga_y, bga_w, bga_h);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        // If no BGA was drawn, show SOUND ONLY
        if !bga_drawn {
            draw_rectangle(
                bga_x,
                bga_y,
                bga_w,
                bga_h,
                Color::new(0.15, 0.15, 0.18, 1.0),
            );
            draw_text(
                "SOUND ONLY",
                bga_x + bga_w / 2.0 - 80.0,
                bga_y + bga_h / 2.0 + 10.0,
                32.0,
                Color::new(0.5, 0.5, 0.5, 1.0),
            );
        }

        // Draw panels
        self.draw_left_panel();
        self.draw_center_panel();
        self.draw_right_panel();
        self.draw_bottom_panel();

        // Draw lanes and notes (on top of left panel background)
        let lane_renderer = LaneRenderer::new(&self.lane_config);
        lane_renderer.draw(&self.model.timelines, self.current_time_ms, self.hi_speed);

        let note_renderer = NoteRenderer::new(&self.lane_config);
        note_renderer.draw_with_cover(
            &self.model.timelines,
            self.current_time_ms,
            self.hi_speed,
            &self.lane_cover,
            |index| !self.judge_manager.is_judged(index),
        );
        note_renderer.draw_cover_overlay(&self.lane_cover);

        // Draw combo and judge on lanes
        self.draw_combo();
        self.draw_judge();
    }

    #[allow(dead_code)]
    fn draw_skin_bga(&self, skin: &SkinRenderer, state: &MainState, now_time_us: i64) {
        if !self.bga_enabled {
            return;
        }
        let fallback_texture = self
            .backbmp_texture
            .as_ref()
            .or(self.stagefile_texture.as_ref());

        for dst in skin.bga_destinations(state, now_time_us) {
            let mut drew_bga = false;
            if let Some(ref bga) = self.bga_processor {
                if bga.has_active_image() {
                    bga.draw_with_alpha(dst.x, dst.y, dst.w, dst.h, dst.a);
                    drew_bga = true;
                }
            }
            if !drew_bga {
                let color = Color::new(1.0, 1.0, 1.0, (dst.a / 255.0).clamp(0.0, 1.0));
                if let Some(texture) = fallback_texture {
                    draw_texture_ex(
                        texture,
                        dst.x,
                        dst.y,
                        color,
                        DrawTextureParams {
                            dest_size: Some(vec2(dst.w, dst.h)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// Draw left panel: lanes background, gauge, hi-speed info.
    fn draw_left_panel(&self) {
        let panel_x = 0.0;
        let panel_w = 280.0;
        let panel_h = screen_height();

        // Panel background
        draw_rectangle(
            panel_x,
            0.0,
            panel_w,
            panel_h,
            Color::new(0.08, 0.08, 0.12, 1.0),
        );

        // Groove Gauge at bottom
        let gauge_x = 20.0;
        let gauge_y = 920.0;
        let gauge_w = 240.0;
        let gauge_h = 20.0;

        // Gauge label
        draw_text("GROOVE GAUGE", gauge_x, gauge_y - 8.0, 14.0, GRAY);

        // Gauge background
        draw_rectangle(
            gauge_x,
            gauge_y,
            gauge_w,
            gauge_h,
            Color::new(0.2, 0.2, 0.2, 1.0),
        );

        // Gauge fill
        let fill_ratio = self.gauge.ratio() as f32;
        let fill_w = gauge_w * fill_ratio;
        let gauge_color = if self.gauge.is_clear() {
            Color::new(0.2, 0.9, 0.3, 1.0) // Green when cleared
        } else {
            Color::new(0.3, 0.5, 0.9, 1.0) // Blue otherwise
        };
        draw_rectangle(gauge_x, gauge_y, fill_w, gauge_h, gauge_color);

        // Border line
        if self.gauge.border() > 0.0 {
            let border_x = gauge_x + gauge_w * (self.gauge.border() / 100.0) as f32;
            draw_line(border_x, gauge_y, border_x, gauge_y + gauge_h, 2.0, WHITE);
        }

        // Gauge percentage
        draw_text(
            &format!("{:.1}%", self.gauge.value()),
            gauge_x + gauge_w + 8.0,
            gauge_y + 15.0,
            16.0,
            WHITE,
        );

        // Hi-Speed display
        let info_y = 960.0;
        draw_text(
            &format!("HI-SPEED: {:.2}", self.hi_speed),
            gauge_x,
            info_y,
            16.0,
            Color::new(0.7, 0.7, 0.7, 1.0),
        );

        // Subtitle info (removed level display as it's not available in BMSModel)
        if !self.model.subtitle.is_empty() {
            draw_text(
                &self.model.subtitle,
                gauge_x,
                info_y + 20.0,
                16.0,
                Color::new(0.7, 0.7, 0.7, 1.0),
            );
        }
    }

    /// Draw center panel: score graph, comparisons.
    fn draw_center_panel(&self) {
        let panel_x = 280.0;
        let panel_w = 320.0;

        // Panel background
        draw_rectangle(
            panel_x,
            0.0,
            panel_w,
            screen_height(),
            Color::new(0.06, 0.06, 0.1, 1.0),
        );

        // Score Graph header
        let header_y = 30.0;
        draw_text("SCORE GRAPH", panel_x + 20.0, header_y, 18.0, GRAY);

        // Current score
        let score_y = 70.0;
        draw_text("YOU", panel_x + 20.0, score_y, 14.0, GRAY);
        draw_text(
            &format!("{:05}", self.score.ex_score()),
            panel_x + 80.0,
            score_y,
            20.0,
            Color::new(0.3, 0.9, 1.0, 1.0),
        );

        // Max score (pacemaker)
        let max_ex = (self.model.total_notes * 2) as u32;
        draw_text("MAX", panel_x + 20.0, score_y + 30.0, 14.0, GRAY);
        draw_text(
            &format!("{:05}", max_ex),
            panel_x + 80.0,
            score_y + 30.0,
            20.0,
            Color::new(1.0, 0.8, 0.3, 1.0),
        );

        // Score graph area
        let graph_x = panel_x + 20.0;
        let graph_y = 140.0;
        let graph_w = 280.0;
        let graph_h = 400.0;

        // Graph background
        draw_rectangle(
            graph_x,
            graph_y,
            graph_w,
            graph_h,
            Color::new(0.03, 0.03, 0.05, 1.0),
        );

        // Grade lines (AAA, AA, A)
        let grades = [
            ("AAA", 0.889, Color::new(1.0, 0.8, 0.0, 0.5)),
            ("AA", 0.778, Color::new(0.8, 0.8, 0.8, 0.5)),
            ("A", 0.667, Color::new(0.6, 0.6, 0.6, 0.5)),
        ];
        for (label, ratio, color) in grades {
            let y = graph_y + graph_h * (1.0 - ratio as f32);
            draw_line(graph_x, y, graph_x + graph_w, y, 1.0, color);
            draw_text(label, graph_x + graph_w + 5.0, y + 4.0, 12.0, color);
        }

        // Progress line (current position in song)
        let total_time = self.model.timelines.last_time_ms().max(1.0);
        let progress = (self.current_time_ms / total_time).clamp(0.0, 1.0) as f32;
        let progress_x = graph_x + graph_w * progress;
        draw_line(
            progress_x,
            graph_y,
            progress_x,
            graph_y + graph_h,
            2.0,
            WHITE,
        );

        // Score progress (actual score vs theoretical max at this point)
        if progress > 0.0 {
            let current_max = (max_ex as f32 * progress) as u32;
            let score_ratio = if current_max > 0 {
                self.score.ex_score() as f32 / current_max as f32
            } else {
                0.0
            };
            let score_y_pos = graph_y + graph_h * (1.0 - score_ratio.min(1.0));
            draw_circle(progress_x, score_y_pos, 4.0, Color::new(0.3, 0.9, 1.0, 1.0));
        }

        // Time left
        let time_left_y = graph_y + graph_h + 30.0;
        let remaining_ms = (total_time - self.current_time_ms).max(0.0);
        let remaining_sec = (remaining_ms / 1000.0) as u32;
        let min = remaining_sec / 60;
        let sec = remaining_sec % 60;
        draw_text("TIME LEFT", panel_x + 20.0, time_left_y, 14.0, GRAY);
        draw_text(
            &format!("{}:{:02}", min, sec),
            panel_x + 100.0,
            time_left_y,
            18.0,
            WHITE,
        );

        // Score difference
        let diff_y = time_left_y + 40.0;
        let theoretical_score = (max_ex as f64 * progress as f64) as i32;
        let diff = self.score.ex_score() as i32 - theoretical_score;
        let diff_color = if diff >= 0 {
            Color::new(0.3, 0.9, 0.3, 1.0)
        } else {
            Color::new(0.9, 0.3, 0.3, 1.0)
        };
        draw_text(
            &format!("{:+}", diff),
            panel_x + 20.0,
            diff_y,
            24.0,
            diff_color,
        );
    }

    /// Draw right panel: song info, BGA area, BPM.
    fn draw_right_panel(&self) {
        let panel_x = 600.0;

        // FREE STAGE indicator
        draw_text(
            "FREE STAGE",
            panel_x + 20.0,
            30.0,
            16.0,
            Color::new(0.5, 0.8, 1.0, 1.0),
        );

        // Song title
        let title = if self.model.title.len() > 40 {
            format!("{}...", &self.model.title[..40])
        } else {
            self.model.title.clone()
        };
        draw_text(&title, panel_x + 20.0, 70.0, 24.0, WHITE);

        // Artist
        let artist = if self.model.artist.len() > 50 {
            format!("{}...", &self.model.artist[..50])
        } else {
            self.model.artist.clone()
        };
        draw_text(&artist, panel_x + 20.0, 100.0, 16.0, GRAY);

        // Subtitle
        if !self.model.subtitle.is_empty() {
            draw_text(
                &format!("[{}]", self.model.subtitle),
                panel_x + 20.0,
                130.0,
                14.0,
                Color::new(1.0, 0.6, 0.2, 1.0),
            );
        }

        // BGA area frame (if no BGA loaded, show "SOUND ONLY")
        if self.bga_processor.is_none() || !self.bga_enabled {
            let bga_x = panel_x;
            let bga_y = 180.0;
            let bga_w = 640.0;
            let bga_h = 480.0;
            draw_rectangle(bga_x, bga_y, bga_w, bga_h, Color::new(0.1, 0.1, 0.1, 1.0));
            draw_text(
                "SOUND ONLY",
                bga_x + bga_w / 2.0 - 60.0,
                bga_y + bga_h / 2.0,
                24.0,
                GRAY,
            );
        }

        // BPM display
        let bpm_y = 700.0;
        draw_text(
            &format!("{:.0} BPM", self.model.initial_bpm),
            panel_x + 20.0,
            bpm_y,
            28.0,
            WHITE,
        );
        if (self.model.max_bpm - self.model.min_bpm).abs() > 0.1 {
            draw_text(
                &format!(
                    "MIN {:.0}  MAX {:.0}",
                    self.model.min_bpm, self.model.max_bpm
                ),
                panel_x + 20.0,
                bpm_y + 30.0,
                14.0,
                GRAY,
            );
        }
    }

    /// Draw bottom panel: judge detail, notes count.
    fn draw_bottom_panel(&self) {
        let panel_y = 950.0;
        let panel_h = 130.0;

        // Panel background
        draw_rectangle(
            280.0,
            panel_y,
            screen_width() - 280.0,
            panel_h,
            Color::new(0.05, 0.05, 0.08, 1.0),
        );

        // Judge counts
        let x_start = 300.0;
        let y = panel_y + 30.0;
        let spacing = 100.0;

        let judges = [
            ("PG", self.score.pg_count, Color::new(0.0, 1.0, 1.0, 1.0)),
            ("GR", self.score.gr_count, Color::new(1.0, 1.0, 0.0, 1.0)),
            ("GD", self.score.gd_count, Color::new(0.0, 1.0, 0.0, 1.0)),
            ("BD", self.score.bd_count, Color::new(0.5, 0.5, 1.0, 1.0)),
            ("PR", self.score.pr_count, Color::new(0.5, 0.5, 0.5, 1.0)),
            ("MS", self.score.ms_count, Color::new(1.0, 0.3, 0.3, 1.0)),
        ];

        for (i, (label, count, color)) in judges.iter().enumerate() {
            let x = x_start + i as f32 * spacing;
            draw_text(label, x, y, 14.0, *color);
            draw_text(&format!("{}", count), x, y + 24.0, 20.0, *color);
        }

        // Combo break count
        let cb_x = x_start + 6.0 * spacing;
        draw_text("CB", cb_x, y, 14.0, Color::new(0.8, 0.4, 0.4, 1.0));
        draw_text(
            &format!(
                "{}",
                self.score.bd_count + self.score.pr_count + self.score.ms_count
            ),
            cb_x,
            y + 24.0,
            20.0,
            Color::new(0.8, 0.4, 0.4, 1.0),
        );

        // Notes count
        let notes_x = x_start + 7.5 * spacing;
        draw_text("NOTES", notes_x, y, 14.0, GRAY);
        draw_text(
            &format!("{}", self.model.total_notes),
            notes_x,
            y + 24.0,
            20.0,
            WHITE,
        );

        // Fast/Slow
        let fs_x = x_start + 9.0 * spacing;
        draw_text("FAST", fs_x, y, 14.0, Color::new(0.3, 0.6, 1.0, 1.0));
        draw_text(
            &format!("{}", self.judge_manager.fast_count()),
            fs_x,
            y + 24.0,
            20.0,
            Color::new(0.3, 0.6, 1.0, 1.0),
        );
        draw_text("SLOW", fs_x + 80.0, y, 14.0, Color::new(1.0, 0.6, 0.3, 1.0));
        draw_text(
            &format!("{}", self.judge_manager.slow_count()),
            fs_x + 80.0,
            y + 24.0,
            20.0,
            Color::new(1.0, 0.6, 0.3, 1.0),
        );

        // Max Combo
        let combo_x = x_start + 11.0 * spacing;
        draw_text("MAX COMBO", combo_x, y, 14.0, YELLOW);
        draw_text(
            &format!("{}", self.score.max_combo),
            combo_x,
            y + 24.0,
            20.0,
            YELLOW,
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
            self.model.play_mode,
            self.model.long_note_mode,
            self.model.judge_rank,
            self.model.judge_rank_type,
            self.model.total,
            self.model.source_format,
        )
    }

    /// Take the input manager from this state.
    pub fn take_input_manager(&mut self) -> InputManager {
        let key_config = self.input_manager.key_config().clone();
        let dummy = InputManager::new(key_config).unwrap();
        std::mem::replace(&mut self.input_manager, dummy)
    }

    /// Get the current hi-speed.
    pub fn hi_speed(&self) -> f32 {
        self.hi_speed
    }

    /// Set the hi-speed.
    pub fn set_hi_speed(&mut self, hi_speed: f32) {
        self.hi_speed = hi_speed.clamp(0.25, 5.0);
    }

    pub fn toggle_bga(&mut self) {
        self.bga_enabled = !self.bga_enabled;
    }

    pub fn is_bga_enabled(&self) -> bool {
        self.bga_enabled
    }

    /// Get the playback speed.
    /// 再生速度を取得する。
    pub fn playback_speed(&self) -> f64 {
        self.playback_speed
    }

    /// Set the playback speed.
    /// 再生速度を設定する。
    pub fn set_playback_speed(&mut self, speed: f64) {
        let clamped = speed.clamp(0.25, 4.0);
        if (clamped - self.playback_speed).abs() < f64::EPSILON {
            return;
        }
        self.playback_speed = clamped;
        self.sync_input_time_offset();
    }

    /// Set practice start to the current time.
    /// 現在時刻を練習開始として設定する。
    pub fn set_practice_start(&mut self) {
        let start_ms = self.current_time_ms.max(0.0);
        let mut end_ms = self
            .practice_range
            .map(|(_, end)| end)
            .unwrap_or(start_ms + Self::DEFAULT_PRACTICE_LENGTH_MS);
        if end_ms <= start_ms {
            end_ms = start_ms + 10.0;
        }
        self.practice_range = Some((start_ms, end_ms));

        if self.phase == PlayPhase::Playing {
            self.apply_practice_seek(start_ms);
        }
    }

    /// Set practice end to the current time.
    /// 現在時刻を練習終了として設定する。
    pub fn set_practice_end(&mut self) {
        let end_ms = self.current_time_ms.max(0.0);
        let start_ms = self
            .practice_range
            .map(|(start, _)| start)
            .unwrap_or((end_ms - Self::DEFAULT_PRACTICE_LENGTH_MS).max(0.0));
        let end_ms = end_ms.max(start_ms + 10.0);
        self.practice_range = Some((start_ms, end_ms));
    }

    /// Clear practice range.
    /// 練習範囲を解除する。
    pub fn clear_practice(&mut self) {
        self.practice_range = None;
    }

    /// Get the practice range.
    /// 練習範囲を取得する。
    pub fn practice_range(&self) -> Option<(f64, f64)> {
        self.practice_range
    }

    /// Check if practice mode is enabled.
    /// 練習モードが有効かどうか。
    pub fn is_practice(&self) -> bool {
        self.practice_range.is_some()
    }

    /// Check if results should be saved.
    /// リザルトを保存するかどうか。
    pub fn should_save_result(&self) -> bool {
        let speed_delta = (self.playback_speed - 1.0).abs();
        self.practice_range.is_none()
            && speed_delta < 0.0001
            && !self.is_replay()
            && !self.is_autoplay()
    }

    fn should_record_replay(&self) -> bool {
        self.should_save_result()
    }

    /// Get the lane cover settings.
    pub fn lane_cover(&self) -> &LaneCoverSettings {
        &self.lane_cover
    }

    /// Get mutable lane cover settings.
    pub fn lane_cover_mut(&mut self) -> &mut LaneCoverSettings {
        &mut self.lane_cover
    }

    /// Set the lane cover settings.
    pub fn set_lane_cover(&mut self, cover: LaneCoverSettings) {
        self.lane_cover = cover;
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
        state.total_rate = state.score_rate;
        state.miss_count = self.score.bp();
        state.combo_break = self.score.bp();
        let max_ex = (self.model.total_notes * 2) as f64;
        let progress = if self.model.timelines.last_time_ms() > 0.0 {
            (self.current_time_ms.max(0.0) / self.model.timelines.last_time_ms()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        state.score_diff_ex = self.score.ex_score() as i32 - (max_ex * progress) as i32;

        // BPM
        state.current_bpm = self.current_bpm();
        state.min_bpm = self.model.min_bpm;
        state.max_bpm = self.model.max_bpm;
        state.bpm_change =
            (self.model.max_bpm - self.model.min_bpm).abs() > 0.1 || self.model.has_stop;

        // Chart info
        state.play_level = self.model.play_level.unwrap_or(0) as i32;
        state.difficulty = self.model.difficulty.unwrap_or(0) as i32;
        state.has_long_note = self.model.has_long_note;
        state.has_bga = self
            .bga_processor
            .as_ref()
            .map(|bga| bga.has_images())
            .unwrap_or(false);
        state.has_stagefile = self.stagefile_texture.is_some();
        state.has_backbmp = self.backbmp_texture.is_some();
        state.has_banner = self.banner_texture.is_some();

        state.is_7key = self.model.play_mode == PlayMode::Beat7K;
        state.is_5key = matches!(self.model.play_mode, PlayMode::Beat5K | PlayMode::PopN5K);
        state.is_14key = self.model.play_mode == PlayMode::Beat14K;
        state.is_10key = self.model.play_mode == PlayMode::Beat10K;
        state.is_9key = self.model.play_mode == PlayMode::PopN9K;

        // Time
        state.current_time_ms = self.current_time_ms.max(0.0);
        state.total_time_ms = self.model.timelines.last_time_ms();

        // Notes
        state.total_notes = self.model.total_notes as u32;

        // Hi-speed
        state.hi_speed = self.hi_speed;
        state.lane_cover_sudden = self.lane_cover.sudden_plus;
        state.lane_cover_hidden = self.lane_cover.hidden_plus;
        state.lane_cover_lift = self.lane_cover.lift;

        // Play state flags
        state.is_ready = self.phase == PlayPhase::Countdown;
        state.is_playing = self.phase == PlayPhase::Playing;
        state.is_finished = self.phase == PlayPhase::Finished;
        state.is_clear = self.phase == PlayPhase::Finished && self.gauge.is_clear();
        state.is_failed = self.phase == PlayPhase::Finished && !self.gauge.is_clear();
        state.is_replay = self.is_replay();
        state.is_replay_recording = self.should_record_replay();
        state.bga_enabled = self.bga_enabled;
        state.autoplay_enabled = self.autoplay_processor.is_some();

        state.song_title = self.model.title.clone();
        state.song_subtitle = self.model.subtitle.clone();
        state.full_title = if self.model.subtitle.is_empty() {
            self.model.title.clone()
        } else {
            format!("{} {}", self.model.title, self.model.subtitle)
        };
        state.genre = self.model.genre.clone();
        state.artist = self.model.artist.clone();
        state.subartist = self.model.subartist.clone();
        state.full_artist = if self.model.subartist.is_empty() {
            self.model.artist.clone()
        } else {
            format!("{} {}", self.model.artist, self.model.subartist)
        };
        state.song_folder = self.model.folder.clone();
        state.player_name = self.player_name.clone();
        state.rival_name = self.rival_name.clone();

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

    fn current_bpm(&self) -> f64 {
        let time = self.current_time_ms.max(0.0);
        self.model
            .timelines
            .entries()
            .iter()
            .rev()
            .find(|tl| tl.time_ms <= time)
            .map(|tl| tl.bpm)
            .unwrap_or(self.model.initial_bpm)
    }

    fn render_lane_config(&self) -> LaneConfig {
        if let Some(renderer) = &self.skin_renderer {
            if self.model.play_mode == PlayMode::Beat7K && renderer.name().contains("EC:FN") {
                return LaneConfig::ecfn_7k_left();
            }
        }
        self.lane_config.clone()
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
            GaugeType::LightAssistEasy => 1,
            GaugeType::Easy => 1,
            GaugeType::Normal => 0,
            GaugeType::Hard => 2,
            GaugeType::ExHard => 3,
            GaugeType::Hazard => 3,
            GaugeType::Class => 4, // Distinct value for Class gauge
        }
    }

    /// Create timer values for skin rendering.
    fn create_timers(&self) -> MainStateTimers {
        self.skin_timers.clone()
    }
}
