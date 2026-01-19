use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use crate::audio::{AudioManager, AudioScheduler};
use crate::bms::{BmsLoader, Chart, LnType, MAX_LANE_COUNT, NoteType};
use crate::config::GameSettings;
use crate::render::font::draw_text_jp;
use crate::render::{
    BgaManager, BpmDisplay, EffectManager, Highway, HighwayConfig, JudgeStats, LaneCover,
    ProgressBar, ScoreGraph, Turntable, VIRTUAL_HEIGHT, VIRTUAL_WIDTH,
};
use crate::skin::{
    GraphAreaLayout, IidxLayout, InfoAreaLayout, LayoutConfig, PlayAreaLayout, Rect, SkinLoader,
};

use super::{
    ClearLamp, GamePlayState, GaugeManager, GaugeSystem, GaugeType, InputHandler, JudgeRank,
    JudgeResult, JudgeSystem, JudgeSystemType, PlayResult, RandomOption, ScoreManager, TimingStats,
    apply_battle, apply_legacy_note, apply_random_option, generate_seed,
};

const START_DELAY_MS: f64 = 1500.0;
const END_DELAY_MS: f64 = 3000.0;

/// Active long note state tracking
#[derive(Clone, Copy)]
struct ActiveLongNote {
    /// Index of LongStart note
    start_idx: usize,
    /// End time in ms
    end_time_ms: f64,
    /// Judgment result from start press
    start_judgment: JudgeResult,
    /// Long note type (LN/CN/HCN)
    ln_type: LnType,
    /// Whether key is currently being held (for HCN)
    is_holding: bool,
}

pub struct GameState {
    chart: Option<Chart>,
    lane_index: [Vec<usize>; MAX_LANE_COUNT],
    audio: Option<AudioManager>,
    scheduler: AudioScheduler,
    highway: Highway,
    input: InputHandler,
    judge: JudgeSystem,
    score: ScoreManager,
    gauge: Option<GaugeManager>,
    effects: EffectManager,
    play_state: Option<GamePlayState>,
    timing_stats: TimingStats,
    scroll_speed: f32,
    current_time_ms: f64,
    last_time_ms: f64,
    start_delay_ms: f64,
    end_delay_ms: f64,
    playing: bool,
    last_judgment: Option<JudgeResult>,
    last_timing_diff_ms: Option<f64>,
    active_long_notes: [Option<ActiveLongNote>; MAX_LANE_COUNT],
    /// Damage timers for HCN (Hell Charge Note) in ms
    hcn_damage_timers: [f64; MAX_LANE_COUNT],
    random_option: RandomOption,
    auto_scratch: bool,
    legacy_note: bool,
    expand_judge: bool,
    battle: bool,
    // BGA
    bga: BgaManager,
    // Layout config for UI positioning
    #[allow(dead_code)]
    layout: LayoutConfig,
    // IIDX-style layout components
    iidx_layout: IidxLayout,
    play_area_layout: PlayAreaLayout,
    info_area_layout: InfoAreaLayout,
    graph_area_layout: GraphAreaLayout,
    turntable: Turntable,
    judge_stats: JudgeStats,
    bpm_display: BpmDisplay,
    score_graph: ScoreGraph,
    /// Cached measure start times for drawing measure lines
    measure_times: Vec<f64>,
    /// Progress bar showing song progress
    progress_bar: ProgressBar,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            chart: None,
            lane_index: Default::default(),
            audio: None,
            scheduler: AudioScheduler::new(),
            highway: Highway::new(),
            input: InputHandler::new(),
            judge: JudgeSystem::default(),
            score: ScoreManager::new(),
            gauge: None,
            effects: EffectManager::default(),
            play_state: None,
            timing_stats: TimingStats::default(),
            scroll_speed: 1.0,
            current_time_ms: 0.0,
            last_time_ms: 0.0,
            start_delay_ms: START_DELAY_MS,
            end_delay_ms: END_DELAY_MS,
            playing: false,
            last_judgment: None,
            last_timing_diff_ms: None,
            active_long_notes: [const { None }; MAX_LANE_COUNT],
            hcn_damage_timers: [0.0; MAX_LANE_COUNT],
            random_option: RandomOption::Off,
            auto_scratch: false,
            legacy_note: false,
            expand_judge: false,
            battle: false,
            bga: BgaManager::new(),
            layout: LayoutConfig::default(),
            // IIDX-style layout components
            iidx_layout: IidxLayout::default(),
            play_area_layout: PlayAreaLayout::default(),
            info_area_layout: InfoAreaLayout::default(),
            graph_area_layout: GraphAreaLayout::default(),
            turntable: Turntable::new(),
            judge_stats: JudgeStats::new(),
            bpm_display: BpmDisplay::default(),
            score_graph: ScoreGraph::default(),
            measure_times: Vec::new(),
            progress_bar: ProgressBar::default(),
        }
    }

    pub fn load_chart(&mut self, path: &str) -> Result<()> {
        let path = Path::new(path);
        let result = BmsLoader::load_full(path)?;
        let mut chart = result.chart;
        let wav_files = result.wav_files;
        let bmp_files = result.bmp_files;

        println!(
            "Loaded: {} - {}",
            chart.metadata.title, chart.metadata.artist
        );
        println!("BPM: {}", chart.metadata.bpm);
        println!("Notes: {}", chart.note_count());
        println!("BGM events: {}", chart.bgm_events.len());
        println!("BGA events: {}", chart.bga_events.len());
        println!("BMP files: {}", bmp_files.len());

        // Load settings
        let settings = GameSettings::load();
        let skin = if settings.skin_name.is_empty() {
            SkinLoader::load_default(None)
        } else {
            match SkinLoader::load_by_name(&settings.skin_name, None) {
                Ok(skin) => skin,
                Err(err) => {
                    eprintln!(
                        "Failed to load skin '{}': {}. Falling back to default.",
                        settings.skin_name, err
                    );
                    SkinLoader::load_default(None)
                }
            }
        };

        self.layout = skin.layout.clone();
        self.iidx_layout = skin.layout.iidx.screen;
        self.play_area_layout = skin.layout.iidx.play;
        self.info_area_layout = skin.layout.iidx.info;
        self.graph_area_layout = skin.layout.iidx.graph;
        self.highway = Highway::with_config(HighwayConfig::for_mode_with_skin(
            chart.metadata.play_mode,
            skin.theme.clone(),
        ));
        self.effects = EffectManager::with_config(
            VIRTUAL_WIDTH / 2.0,
            VIRTUAL_HEIGHT / 2.0 + 50.0,
            skin.effects.clone(),
        );

        let mut audio = AudioManager::new()?;
        // Apply volume settings
        audio.set_master_volume(settings.volume.master as f64 / 100.0);
        audio.set_keysound_volume(settings.volume.keysound as f64 / 100.0);
        audio.set_bgm_volume(settings.volume.bgm as f64 / 100.0);

        if let Some(parent) = path.parent() {
            let load_result = audio.load_keysounds(parent, &wav_files);
            println!(
                "Loaded {}/{} keysounds",
                load_result.loaded,
                load_result.total()
            );

            // Load BGA media (images and videos)
            if !bmp_files.is_empty() {
                let (images, videos) = self.bga.load_media(parent, &bmp_files);
                if images > 0 || videos > 0 {
                    println!("Loaded {} BGA images and {} videos", images, videos);
                }
            }
        }

        // Apply random option before building lane index
        self.random_option = settings.random_option;
        if settings.random_option != RandomOption::Off {
            let seed = generate_seed();
            apply_random_option(&mut chart, settings.random_option, seed);
            println!("Applied random option: {:?}", settings.random_option);
        }

        // Apply auto scratch option
        self.auto_scratch = settings.auto_scratch;
        if self.auto_scratch {
            println!("Auto scratch enabled");
        }

        // Apply legacy note option (convert LN to normal notes)
        self.legacy_note = settings.legacy_note;
        if settings.legacy_note {
            apply_legacy_note(&mut chart);
            println!("Legacy note enabled (LN converted to normal notes)");
        }

        // Apply battle option (flip to 2P side layout)
        self.battle = settings.battle;
        if settings.battle {
            apply_battle(&mut chart);
            println!("Battle enabled (flipped to 2P side layout)");
        }

        let note_count = chart.note_count();
        self.lane_index = chart.build_lane_index_for_mode();
        self.play_state = Some(GamePlayState::new(chart.notes.len()));

        // Initialize judge system based on chart rank and settings
        let judge_rank = JudgeRank::from_bms_rank(chart.metadata.rank);
        let gauge_system = match settings.judge_system {
            JudgeSystemType::Beatoraja => GaugeSystem::Beatoraja,
            JudgeSystemType::Lr2 => GaugeSystem::Lr2,
        };
        let mut judge = JudgeSystem::for_system(settings.judge_system, judge_rank);
        self.expand_judge = settings.expand_judge;
        if settings.expand_judge {
            judge = judge.with_expand();
            println!("Expand judge enabled (1.5x judgment windows)");
        }
        self.judge = judge;

        // Initialize gauge with GAS enabled (all gauges tracked)
        let total_value = chart.metadata.total;
        self.gauge = Some(GaugeManager::new_with_gas(
            settings.gauge_type,
            gauge_system,
            note_count,
            total_value,
            true,
        ));

        // Initialize IIDX layout components (before chart is moved)
        self.score_graph = ScoreGraph::new(note_count as u32);
        let bpm = chart.metadata.bpm as u32;
        // TODO: Calculate actual min/max BPM from bpm_events
        self.bpm_display.update(bpm, bpm, bpm);

        // Cache measure start times for drawing measure lines
        self.measure_times = chart.measure_start_times();

        // Initialize progress bar with total duration
        self.progress_bar = ProgressBar::new(chart.total_duration_ms());

        audio.stop_clock();
        self.scheduler.reset();
        self.current_time_ms = -self.start_delay_ms;
        self.last_time_ms = self.current_time_ms;
        self.scheduler.update(
            &chart,
            &mut audio,
            self.current_time_ms,
            self.start_delay_ms,
        );
        audio.start_clock();
        self.chart = Some(chart);
        self.audio = Some(audio);
        self.score.reset();

        // Apply lane cover and scroll speed settings
        self.scroll_speed = settings.scroll_speed;
        self.highway.set_lane_cover(LaneCover::new(
            settings.sudden,
            settings.hidden,
            settings.lift,
        ));

        // Apply key bindings
        self.input = InputHandler::with_bindings(settings.key_bindings.to_keycodes());

        // Reset BGA state
        self.bga.reset();

        // Start playing automatically after chart is loaded
        self.playing = true;

        Ok(())
    }

    pub fn update(&mut self) {
        // Update gamepad input state
        self.input.update();

        if is_key_pressed(KeyCode::Space) {
            self.playing = !self.playing;
            if let Some(audio) = &mut self.audio {
                if self.playing {
                    audio.start_clock();
                } else {
                    audio.pause_clock();
                }
                let now_ms = audio.current_time_ms() - self.start_delay_ms;
                self.current_time_ms = now_ms;
                self.last_time_ms = now_ms;
            }
        }

        if is_key_pressed(KeyCode::R) {
            self.current_time_ms = -self.start_delay_ms;
            self.last_time_ms = self.current_time_ms;
            self.scheduler.reset();
            self.score.reset();
            if let Some(play_state) = &mut self.play_state {
                play_state.reset();
            }
            // Reset gauge and judge
            if let Some(chart) = &self.chart {
                let judge_rank = JudgeRank::from_bms_rank(chart.metadata.rank);
                self.judge = JudgeSystem::for_system(JudgeSystemType::Beatoraja, judge_rank);
                self.gauge = Some(GaugeManager::new_with_gas(
                    GaugeType::Normal,
                    GaugeSystem::Beatoraja,
                    chart.note_count(),
                    chart.metadata.total,
                    true,
                ));
            }
            self.playing = false;
            self.last_judgment = None;
            self.active_long_notes = [const { None }; MAX_LANE_COUNT];
            self.hcn_damage_timers = [0.0; MAX_LANE_COUNT];
            self.bga.reset();
            if let Some(audio) = &mut self.audio {
                audio.stop_clock();
            }
        }

        if is_key_pressed(KeyCode::Up) {
            self.scroll_speed += 0.1;
        }
        if is_key_pressed(KeyCode::Down) {
            self.scroll_speed = (self.scroll_speed - 0.1).max(0.1);
        }

        // Lane cover controls
        if is_key_pressed(KeyCode::Q) {
            self.highway.lane_cover_mut().adjust_sudden(50);
        }
        if is_key_pressed(KeyCode::W) {
            self.highway.lane_cover_mut().adjust_sudden(-50);
        }
        if is_key_pressed(KeyCode::A) {
            self.highway.lane_cover_mut().adjust_lift(50);
        }
        if is_key_pressed(KeyCode::E) {
            self.highway.lane_cover_mut().adjust_lift(-50);
        }
        if is_key_pressed(KeyCode::Key1) {
            self.highway.lane_cover_mut().adjust_hidden(50);
        }
        if is_key_pressed(KeyCode::Key2) {
            self.highway.lane_cover_mut().adjust_hidden(-50);
        }

        if self.playing {
            let now_ms = self
                .audio
                .as_ref()
                .map(|audio| audio.current_time_ms() - self.start_delay_ms)
                .unwrap_or(self.current_time_ms);
            let delta_ms = (now_ms - self.last_time_ms).max(0.0);
            self.current_time_ms = now_ms;
            self.last_time_ms = now_ms;

            if let (Some(chart), Some(audio)) = (&self.chart, &mut self.audio) {
                self.scheduler
                    .update(chart, audio, self.current_time_ms, self.start_delay_ms);
            }

            // Update BGA
            if let Some(chart) = &self.chart {
                let is_poor = self.last_judgment == Some(JudgeResult::Poor);
                self.bga
                    .update(self.current_time_ms, &chart.bga_events, is_poor);
            }

            self.process_input();
            if self.auto_scratch {
                self.process_auto_scratch();
            }
            self.check_missed_notes();
            self.process_hcn_damage(delta_ms);
        }

        // Update key beam states
        self.update_key_beams();

        // Update effects animation
        self.effects.update(get_frame_time());

        // Update IIDX layout components
        let scratch_active = self.input.get_held_lanes().contains(&0);
        self.turntable.update(scratch_active, get_frame_time());

        self.judge_stats.update(crate::render::JudgeData {
            pgreat: self.score.pgreat_count,
            great: self.score.great_count,
            good: self.score.good_count,
            bad: self.score.bad_count,
            poor: self.score.poor_count,
            fast: self.timing_stats.fast_count,
            slow: self.timing_stats.slow_count,
        });

        self.score_graph.update(self.score.ex_score());
    }

    fn process_input(&mut self) {
        let (chart, audio, play_state) = match (&self.chart, &mut self.audio, &mut self.play_state)
        {
            (Some(c), Some(a), Some(p)) => (c, a, p),
            _ => return,
        };

        let ln_type = chart.metadata.ln_type;
        let pressed_lanes = self.input.get_pressed_lanes();
        let released_lanes = self.input.get_released_lanes();

        // Handle key presses
        for lane in pressed_lanes {
            let lane_idx = lane.lane_index();

            // Trigger lane flash effect
            self.effects.trigger_lane_flash(lane_idx);

            // Check if this is an HCN re-press
            if let Some(ref mut active_ln) = self.active_long_notes[lane_idx] {
                if active_ln.ln_type == LnType::Hcn && !active_ln.is_holding {
                    // Resume holding HCN
                    active_ln.is_holding = true;
                    self.hcn_damage_timers[lane_idx] = 0.0;
                    continue;
                }
            }

            for &i in &self.lane_index[lane_idx] {
                let note = &chart.notes[i];

                if !play_state.get_state(i).is_some_and(|s| s.is_pending()) {
                    continue;
                }

                if !matches!(note.note_type, NoteType::Normal | NoteType::LongStart) {
                    continue;
                }

                let time_diff = note.time_ms - self.current_time_ms;

                if let Some(result) = self.judge.judge(time_diff) {
                    play_state.set_judged(i, result);
                    self.score.add_judgment(result);
                    if let Some(gauge) = &mut self.gauge {
                        gauge.apply_judgment(result);
                    }
                    self.timing_stats.record(result, time_diff);
                    self.last_judgment = Some(result);
                    self.last_timing_diff_ms = Some(time_diff);
                    audio.play(note.keysound_id);

                    // Trigger visual effects
                    let effect_x = VIRTUAL_WIDTH / 2.0;
                    let effect_y = VIRTUAL_HEIGHT / 2.0;
                    self.effects.trigger_judge(result, effect_x, effect_y);
                    self.effects.trigger_bomb(lane_idx);
                    self.effects.update_combo(self.score.combo);

                    if note.note_type == NoteType::LongStart {
                        let end_time_ms = note.long_end_time_ms.unwrap_or(note.time_ms);
                        self.active_long_notes[lane_idx] = Some(ActiveLongNote {
                            start_idx: i,
                            end_time_ms,
                            start_judgment: result,
                            ln_type,
                            is_holding: true,
                        });
                        // Reset HCN damage timer
                        self.hcn_damage_timers[lane_idx] = 0.0;
                    }
                    break;
                }
            }
        }

        // Handle key releases for long notes
        for lane in released_lanes {
            let lane_idx = lane.lane_index();

            // Check if this is an HCN that should continue tracking
            if let Some(ref mut active_ln) = self.active_long_notes[lane_idx] {
                if active_ln.ln_type == LnType::Hcn {
                    let time_diff = active_ln.end_time_ms - self.current_time_ms;
                    if time_diff > 0.0 {
                        // HCN released before end - mark as not holding but keep tracking
                        active_ln.is_holding = false;
                        continue;
                    }
                }
            }

            if let Some(active_ln) = self.active_long_notes[lane_idx].take() {
                // Calculate time difference from end time
                let time_diff = active_ln.end_time_ms - self.current_time_ms;
                let effect_x = VIRTUAL_WIDTH / 2.0;
                let effect_y = VIRTUAL_HEIGHT / 2.0;

                // Find the corresponding LongEnd note
                let mut long_end_idx = None;
                for &i in &self.lane_index[lane_idx] {
                    let note = &chart.notes[i];
                    if note.note_type == NoteType::LongEnd
                        && note.time_ms > chart.notes[active_ln.start_idx].time_ms
                        && play_state.get_state(i).is_some_and(|s| s.is_pending())
                    {
                        long_end_idx = Some(i);
                        break;
                    }
                }

                match active_ln.ln_type {
                    LnType::Ln => {
                        // LN: No release judgment, just check if held until end
                        if let Some(end_idx) = long_end_idx {
                            if time_diff <= 0.0 {
                                // Released at or after end time - success
                                play_state.set_judged(end_idx, active_ln.start_judgment);
                                self.score.add_judgment(active_ln.start_judgment);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(active_ln.start_judgment);
                                }
                                if let Some(audio) = &mut self.audio {
                                    audio.play(chart.notes[end_idx].keysound_id);
                                }
                                self.effects.trigger_judge(
                                    active_ln.start_judgment,
                                    effect_x,
                                    effect_y,
                                );
                                self.effects.update_combo(self.score.combo);
                                self.last_judgment = Some(active_ln.start_judgment);
                                self.last_timing_diff_ms = None;
                            } else {
                                // Released too early - POOR
                                play_state.set_missed(end_idx);
                                self.score.add_judgment(JudgeResult::Poor);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(JudgeResult::Poor);
                                }
                                self.effects
                                    .trigger_judge(JudgeResult::Poor, effect_x, effect_y);
                                self.effects.update_combo(self.score.combo);
                                self.last_judgment = Some(JudgeResult::Poor);
                                self.last_timing_diff_ms = None;
                            }
                        }
                    }
                    LnType::Cn => {
                        // CN: Judge the release timing
                        if let Some(end_idx) = long_end_idx {
                            if self.judge.is_early_release(time_diff) {
                                // Released too early - POOR
                                play_state.set_missed(end_idx);
                                self.score.add_judgment(JudgeResult::Poor);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(JudgeResult::Poor);
                                }
                                self.effects
                                    .trigger_judge(JudgeResult::Poor, effect_x, effect_y);
                                self.effects.update_combo(self.score.combo);
                                self.last_judgment = Some(JudgeResult::Poor);
                                self.last_timing_diff_ms = Some(time_diff);
                            } else if let Some(release_result) = self.judge.judge_release(time_diff)
                            {
                                // Release judgment
                                play_state.set_judged(end_idx, release_result);
                                self.score.add_judgment(release_result);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(release_result);
                                }
                                self.last_judgment = Some(release_result);
                                if let Some(audio) = &mut self.audio {
                                    audio.play(chart.notes[end_idx].keysound_id);
                                }
                                self.effects
                                    .trigger_judge(release_result, effect_x, effect_y);
                                self.effects.update_combo(self.score.combo);
                                self.last_timing_diff_ms = Some(time_diff);
                            } else {
                                // Released too late - BAD
                                play_state.set_judged(end_idx, JudgeResult::Bad);
                                self.score.add_judgment(JudgeResult::Bad);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(JudgeResult::Bad);
                                }
                                self.effects
                                    .trigger_judge(JudgeResult::Bad, effect_x, effect_y);
                                self.effects.update_combo(self.score.combo);
                                self.last_judgment = Some(JudgeResult::Bad);
                                self.last_timing_diff_ms = Some(time_diff);
                            }
                        }
                    }
                    LnType::Hcn => {
                        // HCN: Released at or after end time - success
                        if let Some(end_idx) = long_end_idx {
                            play_state.set_judged(end_idx, active_ln.start_judgment);
                            self.score.add_judgment(active_ln.start_judgment);
                            if let Some(gauge) = &mut self.gauge {
                                gauge.apply_judgment(active_ln.start_judgment);
                            }
                            if let Some(audio) = &mut self.audio {
                                audio.play(chart.notes[end_idx].keysound_id);
                            }
                            self.effects.trigger_judge(
                                active_ln.start_judgment,
                                effect_x,
                                effect_y,
                            );
                            self.effects.update_combo(self.score.combo);
                            self.last_judgment = Some(active_ln.start_judgment);
                            self.last_timing_diff_ms = None;
                        }
                    }
                }
            }
        }
    }

    fn check_missed_notes(&mut self) {
        let (chart, play_state) = match (&self.chart, &mut self.play_state) {
            (Some(c), Some(p)) => (c, p),
            _ => return,
        };

        for (i, note) in chart.notes.iter().enumerate() {
            if !play_state.get_state(i).is_some_and(|s| s.is_pending()) {
                continue;
            }

            // Skip LongEnd if it's tracked by active_long_notes
            if note.note_type == NoteType::LongEnd {
                continue;
            }

            let time_diff = note.time_ms - self.current_time_ms;

            if self.judge.is_missed(time_diff) {
                play_state.set_missed(i);
                self.score.add_judgment(JudgeResult::Poor);
                if let Some(gauge) = &mut self.gauge {
                    gauge.apply_judgment(JudgeResult::Poor);
                }
                self.last_judgment = Some(JudgeResult::Poor);

                // If LongStart is missed, also miss the held long note
                if note.note_type == NoteType::LongStart {
                    let lane_idx = note.channel.lane_index();
                    self.active_long_notes[lane_idx] = None;

                    // Also mark the corresponding LongEnd as missed
                    for &end_idx in &self.lane_index[lane_idx] {
                        let end_note = &chart.notes[end_idx];
                        if end_note.note_type == NoteType::LongEnd
                            && end_note.time_ms > note.time_ms
                            && play_state
                                .get_state(end_idx)
                                .is_some_and(|s| s.is_pending())
                        {
                            play_state.set_missed(end_idx);
                            break;
                        }
                    }
                }
            }
        }

        // Check for missed long note ends (held too long without release)
        for lane_idx in 0..MAX_LANE_COUNT {
            if let Some(active_ln) = &self.active_long_notes[lane_idx] {
                let start_note = &chart.notes[active_ln.start_idx];

                // Find the corresponding LongEnd
                for &i in &self.lane_index[lane_idx] {
                    let note = &chart.notes[i];

                    if note.note_type != NoteType::LongEnd {
                        continue;
                    }

                    if note.time_ms <= start_note.time_ms {
                        continue;
                    }

                    if !play_state.get_state(i).is_some_and(|s| s.is_pending()) {
                        continue;
                    }

                    let time_diff = note.time_ms - self.current_time_ms;

                    // For CN, use release window; for LN/HCN, use normal window
                    let missed = match active_ln.ln_type {
                        LnType::Cn => time_diff < -self.judge.release_bad_window(),
                        _ => self.judge.is_missed(time_diff),
                    };

                    if missed {
                        play_state.set_missed(i);
                        self.score.add_judgment(JudgeResult::Poor);
                        if let Some(gauge) = &mut self.gauge {
                            gauge.apply_judgment(JudgeResult::Poor);
                        }
                        self.last_judgment = Some(JudgeResult::Poor);
                        self.active_long_notes[lane_idx] = None;
                    }
                    break;
                }
            }
        }
    }

    /// Process HCN (Hell Charge Note) damage for released keys
    fn process_hcn_damage(&mut self, delta_ms: f64) {
        const HCN_DAMAGE_INTERVAL_MS: f64 = 100.0;

        for lane_idx in 0..MAX_LANE_COUNT {
            let Some(ref active_ln) = self.active_long_notes[lane_idx] else {
                continue;
            };

            // Only process HCN that is not being held
            if active_ln.ln_type != LnType::Hcn || active_ln.is_holding {
                continue;
            }

            self.hcn_damage_timers[lane_idx] += delta_ms;

            // Apply damage every 100ms
            while self.hcn_damage_timers[lane_idx] >= HCN_DAMAGE_INTERVAL_MS {
                self.hcn_damage_timers[lane_idx] -= HCN_DAMAGE_INTERVAL_MS;

                // Apply POOR judgment damage to gauge
                if let Some(gauge) = &mut self.gauge {
                    gauge.apply_judgment(JudgeResult::Poor);
                }
            }
        }
    }

    /// Update key beam states based on held keys
    fn update_key_beams(&mut self) {
        let held_lanes = self.input.get_held_lanes();
        for lane in 0..MAX_LANE_COUNT {
            self.effects.set_key_held(lane, held_lanes.contains(&lane));
        }
    }

    /// Process auto scratch notes (automatically hit scratch notes at the perfect timing)
    fn process_auto_scratch(&mut self) {
        use crate::bms::NoteChannel;

        let (chart, audio, play_state) = match (&self.chart, &mut self.audio, &mut self.play_state)
        {
            (Some(c), Some(a), Some(p)) => (c, a, p),
            _ => return,
        };

        let scratch_lane = NoteChannel::Scratch.lane_index();

        for &i in &self.lane_index[scratch_lane] {
            let note = &chart.notes[i];

            if !play_state.get_state(i).is_some_and(|s| s.is_pending()) {
                continue;
            }

            if !matches!(note.note_type, NoteType::Normal | NoteType::LongStart) {
                continue;
            }

            let time_diff = note.time_ms - self.current_time_ms;

            // Auto-hit scratch at PGREAT timing (within ~20ms)
            if time_diff.abs() <= 20.0 {
                let result = JudgeResult::PGreat;
                play_state.set_judged(i, result);
                self.score.add_judgment(result);
                if let Some(gauge) = &mut self.gauge {
                    gauge.apply_judgment(result);
                }
                self.timing_stats.record(result, time_diff);
                audio.play(note.keysound_id);

                // Trigger visual effects
                self.effects.trigger_lane_flash(scratch_lane);
                let effect_x = VIRTUAL_WIDTH / 2.0;
                let effect_y = VIRTUAL_HEIGHT / 2.0;
                self.effects.trigger_judge(result, effect_x, effect_y);
                self.effects.trigger_bomb(scratch_lane);
                self.effects.update_combo(self.score.combo);
                self.last_judgment = Some(result);
                self.last_timing_diff_ms = Some(time_diff);

                // Handle long notes
                if note.note_type == NoteType::LongStart {
                    let ln_type = chart.metadata.ln_type;
                    let end_time_ms = note.long_end_time_ms.unwrap_or(note.time_ms);
                    self.active_long_notes[scratch_lane] = Some(ActiveLongNote {
                        start_idx: i,
                        end_time_ms,
                        start_judgment: result,
                        ln_type,
                        is_holding: true,
                    });
                    self.hcn_damage_timers[scratch_lane] = 0.0;
                }
            }
        }

        // Auto-release scratch long notes at the end
        if let Some(active_ln) = &self.active_long_notes[scratch_lane] {
            let time_diff = active_ln.end_time_ms - self.current_time_ms;

            // Release at or just after end time
            if (-100.0..=20.0).contains(&time_diff) {
                let active_ln = self.active_long_notes[scratch_lane].take().unwrap();

                // Find the corresponding LongEnd note
                for &i in &self.lane_index[scratch_lane] {
                    let note = &chart.notes[i];
                    if note.note_type == NoteType::LongEnd
                        && note.time_ms > chart.notes[active_ln.start_idx].time_ms
                        && play_state.get_state(i).is_some_and(|s| s.is_pending())
                    {
                        let result = active_ln.start_judgment;
                        play_state.set_judged(i, result);
                        self.score.add_judgment(result);
                        if let Some(gauge) = &mut self.gauge {
                            gauge.apply_judgment(result);
                        }
                        audio.play(note.keysound_id);
                        let effect_x = VIRTUAL_WIDTH / 2.0;
                        let effect_y = VIRTUAL_HEIGHT / 2.0;
                        self.effects.trigger_judge(result, effect_x, effect_y);
                        self.effects.update_combo(self.score.combo);
                        self.last_judgment = Some(result);
                        self.last_timing_diff_ms = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        // Calculate IIDX-style 3-column layout
        let areas = self
            .iidx_layout
            .calculate_areas(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);

        // Draw each area
        self.draw_play_area(&areas.play);
        self.draw_graph_area(&areas.graph);
        self.draw_info_area(&areas.info);
    }

    fn draw_play_area(&self, area: &Rect) {
        // Calculate sub-areas
        let highway_rect = self.play_area_layout.highway_rect(area);
        let turntable_rect = self.play_area_layout.turntable_rect(area);
        let gauge_rect = self.play_area_layout.gauge_rect(area);
        let score_rect = self.play_area_layout.score_rect(area);

        // Draw progress bar on the left side of the play area
        if self.play_area_layout.progress_bar_width > 0.0 {
            let progress_bar_rect = Rect::new(
                area.x + self.play_area_layout.progress_bar_offset_x,
                area.y,
                self.play_area_layout.progress_bar_width,
                highway_rect.height,
            );
            self.progress_bar
                .draw(&progress_bar_rect, self.current_time_ms);
        }

        // Draw highway
        if let (Some(chart), Some(play_state)) = (&self.chart, &self.play_state) {
            self.highway.draw_in_rect(
                &highway_rect,
                chart,
                play_state,
                self.current_time_ms,
                self.scroll_speed,
                &self.measure_times,
            );
        }

        // Draw key beams
        let judge_y = self.highway.judge_y_in_rect(&highway_rect);
        let lane_widths = self.highway.get_lane_widths();
        let lane_colors = self.highway.get_lane_colors();
        let scale = highway_rect.width / self.highway.total_width();
        self.effects.draw_key_beams_in_rect(
            &highway_rect,
            &lane_widths,
            &lane_colors,
            scale,
            judge_y,
        );
        self.effects
            .draw_bombs_in_rect(&highway_rect, &lane_widths, scale, judge_y);

        // Draw judge text and combo (centered on highway, 120px above judge line)
        let effect_center_x = highway_rect.x + highway_rect.width / 2.0;
        let effect_y = judge_y - 120.0;
        self.effects
            .draw_judge_and_combo_at(effect_center_x, effect_y, self.last_timing_diff_ms);

        // Draw turntable
        self.turntable.draw(&turntable_rect);

        // Draw keyboard visualization (simplified)
        self.draw_keyboard_area(area);

        // Draw gauge bar (horizontal)
        self.draw_gauge_bar(&gauge_rect);

        // Draw score and hi-speed
        self.draw_score_area(&score_rect);
    }

    fn draw_keyboard_area(&self, play_area: &Rect) {
        let tt_width = play_area.width * self.play_area_layout.turntable_ratio;
        let kb_x = play_area.x + tt_width;
        let kb_y = play_area.y + play_area.height * self.play_area_layout.highway_height_ratio;
        let kb_width = play_area.width - tt_width;
        let kb_height = play_area.height * (1.0 - self.play_area_layout.highway_height_ratio)
            - self.play_area_layout.gauge_height
            - self.play_area_layout.score_area_height;

        // Background
        draw_rectangle(
            kb_x,
            kb_y,
            kb_width,
            kb_height,
            Color::new(0.1, 0.1, 0.12, 1.0),
        );

        // Draw 7 key indicators
        let key_width = kb_width / 7.0;
        let held_lanes = self.input.get_held_lanes();
        let key_padding_x = self.play_area_layout.key_padding_x;
        let key_padding_y = self.play_area_layout.key_padding_y;

        for i in 0..7 {
            let lane_idx = i + 1; // Skip scratch (lane 0)
            let x = kb_x + i as f32 * key_width;
            let is_held = held_lanes.contains(&lane_idx);

            let color = if is_held {
                if i % 2 == 0 {
                    Color::new(1.0, 1.0, 1.0, 0.9) // White key pressed
                } else {
                    Color::new(0.3, 0.5, 0.9, 0.9) // Blue key pressed
                }
            } else if i % 2 == 0 {
                Color::new(0.3, 0.3, 0.3, 1.0) // White key
            } else {
                Color::new(0.15, 0.2, 0.35, 1.0) // Blue key
            };

            let key_draw_width = (key_width - key_padding_x * 2.0).max(0.0);
            let key_draw_height = (kb_height - key_padding_y * 2.0).max(0.0);
            draw_rectangle(
                x + key_padding_x,
                kb_y + key_padding_y,
                key_draw_width,
                key_draw_height,
                color,
            );
        }
    }

    fn draw_gauge_bar(&self, rect: &Rect) {
        if let Some(gauge) = &self.gauge {
            let hp = gauge.hp();
            let gauge_color = if gauge.active_gauge().is_survival() {
                if hp > 30.0 {
                    Color::new(1.0, 0.2, 0.2, 1.0)
                } else {
                    Color::new(1.0, 0.5, 0.0, 1.0)
                }
            } else if hp >= 80.0 {
                Color::new(0.0, 1.0, 0.5, 1.0)
            } else {
                Color::new(0.2, 0.6, 1.0, 1.0)
            };

            // Background
            draw_rectangle(rect.x, rect.y, rect.width, rect.height, DARKGRAY);
            // Fill
            draw_rectangle(
                rect.x,
                rect.y,
                rect.width * (hp / 100.0),
                rect.height,
                gauge_color,
            );
            // Border
            draw_rectangle_lines(rect.x, rect.y, rect.width, rect.height, 1.0, GRAY);

            // GROOVE GAUGE label
            draw_text_jp("GROOVE GAUGE", rect.x + 5.0, rect.y - 2.0, 12.0, GRAY);

            // Percentage
            draw_text_jp(
                &format!("{:.0}", hp),
                rect.x + rect.width - 30.0,
                rect.y + rect.height - 4.0,
                14.0,
                WHITE,
            );
        }
    }

    fn draw_score_area(&self, rect: &Rect) {
        // Background
        draw_rectangle(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            Color::new(0.05, 0.05, 0.08, 1.0),
        );

        // SCORE label and value
        draw_text_jp("SCORE", rect.x + 10.0, rect.y + 20.0, 14.0, GRAY);
        draw_text_jp(
            &format!("{}", self.score.ex_score()),
            rect.x + 10.0,
            rect.y + 40.0,
            24.0,
            YELLOW,
        );

        // HI-SPEED
        draw_text_jp(
            "HI-SPEED",
            rect.x + rect.width / 2.0,
            rect.y + 20.0,
            14.0,
            GRAY,
        );
        draw_text_jp(
            &format!("{:.2}", self.scroll_speed),
            rect.x + rect.width / 2.0,
            rect.y + 40.0,
            24.0,
            WHITE,
        );
    }

    fn draw_graph_area(&self, area: &Rect) {
        // Background
        draw_rectangle(
            area.x,
            area.y,
            area.width,
            area.height,
            Color::new(0.08, 0.08, 0.1, 1.0),
        );

        // Score graph
        let graph_rect = self.graph_area_layout.score_graph_rect(area);
        self.score_graph
            .draw(&graph_rect, &self.graph_area_layout);

        // Option display
        let option_pos = self
            .graph_area_layout
            .resolve_position(area, self.graph_area_layout.option_position);
        if self.random_option != RandomOption::Off {
            draw_text_jp(
                &format!("{:?}", self.random_option),
                option_pos.x,
                option_pos.y,
                self.graph_area_layout.option_font_size,
                Color::new(1.0, 0.3, 0.3, 1.0),
            );
        }

        // Green number
        let green_number = self.calculate_green_number();
        let green_pos = self
            .graph_area_layout
            .resolve_position(area, self.graph_area_layout.green_number_position);
        draw_text_jp(
            &format!("GREEN: {:.0}", green_number),
            green_pos.x,
            green_pos.y,
            self.graph_area_layout.green_number_font_size,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );
    }

    fn draw_info_area(&self, area: &Rect) {
        // Header with song info
        let header_rect = self.info_area_layout.header_rect(area);
        self.draw_song_header(&header_rect);

        // Bottom panel background
        let bottom_panel_rect = Rect::new(
            area.x,
            area.y + area.height - self.info_area_layout.bottom_panel_height,
            area.width,
            self.info_area_layout.bottom_panel_height,
        );
        draw_rectangle(
            bottom_panel_rect.x,
            bottom_panel_rect.y,
            bottom_panel_rect.width,
            bottom_panel_rect.height,
            Color::new(0.05, 0.05, 0.08, 1.0),
        );
        draw_rectangle_lines(
            bottom_panel_rect.x,
            bottom_panel_rect.y,
            bottom_panel_rect.width,
            bottom_panel_rect.height,
            1.0,
            Color::new(0.2, 0.2, 0.25, 1.0),
        );

        // BGA
        let bga_rect = self.info_area_layout.bga_rect(area);
        if self.bga.has_textures() {
            self.bga
                .draw(bga_rect.x, bga_rect.y, bga_rect.width, bga_rect.height);
        } else {
            // Black background when no BGA
            draw_rectangle(
                bga_rect.x,
                bga_rect.y,
                bga_rect.width,
                bga_rect.height,
                BLACK,
            );
        }

        // Judge stats
        let stats_rect = self.info_area_layout.judge_stats_rect(area);
        self.judge_stats
            .draw(&stats_rect, &self.info_area_layout.judge_stats);

        // BPM display
        let bpm_rect = self.info_area_layout.bpm_rect(area);
        self.bpm_display
            .draw(&bpm_rect, &self.info_area_layout.bpm);
    }

    fn draw_song_header(&self, rect: &Rect) {
        // Background
        draw_rectangle(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            Color::new(0.05, 0.05, 0.08, 1.0),
        );

        if let Some(chart) = &self.chart {
            // Difficulty badge (simplified)
            draw_rectangle(
                rect.x + 10.0,
                rect.y + 15.0,
                80.0,
                25.0,
                Color::new(0.8, 0.2, 0.2, 1.0),
            );
            draw_text_jp(
                &format!("Lv.{}", chart.metadata.play_level),
                rect.x + 15.0,
                rect.y + 35.0,
                16.0,
                WHITE,
            );

            // Title
            draw_text_jp(
                &chart.metadata.title,
                rect.x + 100.0,
                rect.y + 25.0,
                18.0,
                WHITE,
            );

            // Artist
            draw_text_jp(
                &chart.metadata.artist,
                rect.x + 100.0,
                rect.y + 50.0,
                14.0,
                GRAY,
            );
        }
    }

    #[allow(dead_code)]
    fn draw_ui(&self) {
        // Help text (currently not used - IIDX layout handles all UI)
        draw_text_jp(
            "[Space] Play/Pause | [R] Reset | [Up/Down] Speed",
            10.0,
            VIRTUAL_HEIGHT - 20.0,
            12.0,
            GRAY,
        );
    }

    /// Calculate Green Number (IIDX specification)
    /// Unit: 0.1 frames at 60fps
    /// Formula: G = 174.75 × (1000 - W) / B / HS
    /// where W = white number (SUDDEN+ + LIFT), B = BPM, HS = HI-SPEED
    fn calculate_green_number(&self) -> f32 {
        let base_bpm = self
            .chart
            .as_ref()
            .map(|c| c.metadata.bpm as f32)
            .unwrap_or(150.0);

        // IIDX formula: G = 174.75 × (1000 - W) / B / HS
        // W = SUDDEN+ + LIFT (HIDDEN is not included in white number)
        let lane_cover = self.highway.lane_cover();
        let white_number = (lane_cover.sudden + lane_cover.lift) as f32;
        let visible_lane = 1000.0 - white_number;

        174.75 * visible_lane / base_bpm / self.scroll_speed
    }

    pub fn is_finished(&self) -> bool {
        let Some(chart) = &self.chart else {
            return false;
        };
        let Some(play_state) = &self.play_state else {
            return false;
        };

        if !self.playing {
            return false;
        }

        let end_time_ms = chart.total_duration_ms() + self.end_delay_ms;
        if self.current_time_ms < end_time_ms {
            return false;
        }

        play_state.all_notes_processed(chart.notes.len())
    }

    pub fn get_result(&self, chart_path: &str) -> PlayResult {
        use crate::ir::PlayOptionFlags;

        let (title, artist) = self
            .chart
            .as_ref()
            .map(|c| (c.metadata.title.clone(), c.metadata.artist.clone()))
            .unwrap_or_default();

        let total_notes = self
            .chart
            .as_ref()
            .map(|c| c.note_count() as u32)
            .unwrap_or(0);

        // Determine clear lamp from gauge
        let best_clear = self.gauge.as_ref().and_then(|g| g.best_clear());
        let is_full_combo = self.score.bad_count == 0 && self.score.poor_count == 0;
        let clear_lamp = ClearLamp::from_gauge(best_clear, is_full_combo);

        // Get active gauge type for play options
        let gauge_type = self
            .gauge
            .as_ref()
            .map(|g| g.active_gauge())
            .unwrap_or(GaugeType::Normal);

        PlayResult {
            chart_path: chart_path.to_string(),
            title,
            artist,
            ex_score: self.score.ex_score(),
            max_combo: self.score.max_combo,
            pgreat_count: self.score.pgreat_count,
            great_count: self.score.great_count,
            good_count: self.score.good_count,
            bad_count: self.score.bad_count,
            poor_count: self.score.poor_count,
            total_notes,
            clear_lamp,
            random_option: self.random_option,
            fast_count: self.timing_stats.fast_count,
            slow_count: self.timing_stats.slow_count,
            play_options: PlayOptionFlags {
                random_option: self.random_option,
                gauge_type,
                auto_scratch: self.auto_scratch,
                legacy_note: self.legacy_note,
                expand_judge: self.expand_judge,
                battle: self.battle,
            },
        }
    }

    /// Check if the player has failed (all gauges depleted)
    pub fn is_failed(&self) -> bool {
        self.gauge.as_ref().is_some_and(|g| g.is_failed())
    }

    /// Get current gameplay settings for persistence
    pub fn get_gameplay_settings(&self) -> (f32, u16, u16, u16) {
        let lane_cover = self.highway.lane_cover();
        (
            self.scroll_speed,
            lane_cover.sudden,
            lane_cover.hidden,
            lane_cover.lift,
        )
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
