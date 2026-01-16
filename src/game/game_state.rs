use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use macroquad::prelude::*;

use crate::audio::{AudioManager, AudioScheduler};
use crate::bms::{BmsLoader, Chart, LnType, MAX_LANE_COUNT, NoteType};
use crate::config::GameSettings;
use crate::render::{BgaManager, EffectManager, Highway, LaneCover};
use crate::skin::LayoutConfig;

use super::{
    ClearLamp, GamePlayState, GaugeManager, GaugeSystem, GaugeType, InputHandler, JudgeRank,
    JudgeResult, JudgeSystem, JudgeSystemType, PlayResult, RandomOption, ScoreManager, TimingStats,
    apply_battle, apply_legacy_note, apply_random_option, generate_seed,
};

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
    pending_bga_load: Option<(PathBuf, HashMap<u32, String>)>,
    // Layout config for UI positioning
    layout: LayoutConfig,
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
            pending_bga_load: None,
            layout: LayoutConfig::default(),
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

        let mut audio = AudioManager::new()?;
        if let Some(parent) = path.parent() {
            let load_result = audio.load_keysounds(parent, &wav_files);
            println!(
                "Loaded {}/{} keysounds",
                load_result.loaded,
                load_result.total()
            );

            // Store BMP files for async loading
            if !bmp_files.is_empty() {
                self.pending_bga_load = Some((parent.to_path_buf(), bmp_files));
            }
        }

        // Load settings
        let settings = GameSettings::load();

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
        self.chart = Some(chart);
        self.audio = Some(audio);
        self.scheduler.reset();
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

        Ok(())
    }

    /// Load BGA textures asynchronously (call after load_chart)
    #[allow(dead_code)] // BGA loading will be integrated when BGA rendering is complete
    pub async fn load_bga(&mut self) {
        if let Some((base_path, bmp_files)) = self.pending_bga_load.take() {
            let loaded = self.bga.load_textures(&base_path, &bmp_files).await;
            if loaded > 0 {
                println!("Loaded {} BGA textures", loaded);
            }
        }
    }

    pub fn update(&mut self) {
        // Update gamepad input state
        self.input.update();

        if is_key_pressed(KeyCode::Space) {
            self.playing = !self.playing;
        }

        if is_key_pressed(KeyCode::R) {
            self.current_time_ms = 0.0;
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
            self.current_time_ms += get_frame_time() as f64 * 1000.0;

            if let (Some(chart), Some(audio)) = (&self.chart, &mut self.audio) {
                self.scheduler.update(chart, audio, self.current_time_ms);
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
            self.process_hcn_damage(get_frame_time() as f64 * 1000.0);
        }

        // Update key beam states
        self.update_key_beams();

        // Update effects animation
        self.effects.update(get_frame_time());
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
                    let effect_x = screen_width() / 2.0;
                    let effect_y = screen_height() / 2.0;
                    self.effects.trigger_judge(result, effect_x, effect_y);
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
                            } else {
                                // Released too early - POOR
                                play_state.set_missed(end_idx);
                                self.score.add_judgment(JudgeResult::Poor);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(JudgeResult::Poor);
                                }
                                self.last_judgment = Some(JudgeResult::Poor);
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
                                self.last_judgment = Some(JudgeResult::Poor);
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
                            } else {
                                // Released too late - BAD
                                play_state.set_judged(end_idx, JudgeResult::Bad);
                                self.score.add_judgment(JudgeResult::Bad);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(JudgeResult::Bad);
                                }
                                self.last_judgment = Some(JudgeResult::Bad);
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
                let effect_x = screen_width() / 2.0;
                let effect_y = screen_height() / 2.0;
                self.effects.trigger_judge(result, effect_x, effect_y);
                self.effects.update_combo(self.score.combo);

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
                        break;
                    }
                }
            }
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        // Draw BGA in the background (position from layout config)
        if self.bga.has_textures() {
            let bga_rect = self.layout.bga_rect();
            self.bga
                .draw(bga_rect.x, bga_rect.y, bga_rect.width, bga_rect.height);
        }

        // Draw lane flash effects (behind notes)
        let highway_x = (screen_width() - 50.0 * 8.0) / 2.0;
        self.effects
            .draw_lane_flashes(highway_x, 50.0, screen_height());

        if let (Some(chart), Some(play_state)) = (&self.chart, &self.play_state) {
            self.highway.draw_with_state(
                chart,
                play_state,
                self.current_time_ms,
                self.scroll_speed,
            );

            // Draw key beams (after notes, before UI)
            let highway_x = self.highway.highway_x();
            let lane_width = self.highway.lane_width();
            let judge_y = self.highway.judge_line_y();
            let lane_colors = self.highway.get_lane_colors();
            self.effects
                .draw_key_beams(highway_x, lane_width, judge_y, &lane_colors);
        } else {
            draw_text(
                "No chart loaded. Pass BMS file path as argument.",
                20.0,
                screen_height() / 2.0,
                30.0,
                WHITE,
            );
        }

        // Draw judgment and combo effects
        self.effects.draw_judge();
        self.effects.draw_combo();

        self.draw_ui();
    }

    fn draw_ui(&self) {
        draw_text(
            &format!("Time: {:.2}s", self.current_time_ms / 1000.0),
            10.0,
            30.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Speed: {:.1}x", self.scroll_speed),
            10.0,
            50.0,
            20.0,
            WHITE,
        );

        // Green Number: visible time in milliseconds
        let green_number = self.calculate_green_number();
        draw_text(
            &format!("GREEN: {:.0}", green_number),
            10.0,
            70.0,
            20.0,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );

        let status = if self.playing { "Playing" } else { "Paused" };
        draw_text(&format!("Status: {}", status), 10.0, 90.0, 20.0, WHITE);

        draw_text(
            &format!("EX Score: {}", self.score.ex_score()),
            screen_width() - 200.0,
            30.0,
            20.0,
            YELLOW,
        );
        draw_text(
            &format!("Combo: {}", self.score.combo),
            screen_width() - 200.0,
            50.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!("Max Combo: {}", self.score.max_combo),
            screen_width() - 200.0,
            70.0,
            20.0,
            GRAY,
        );

        // Draw gauge
        if let Some(gauge) = &self.gauge {
            let gauge_type_str = match gauge.active_gauge() {
                GaugeType::AssistEasy => "ASSIST",
                GaugeType::Easy => "EASY",
                GaugeType::Normal => "NORMAL",
                GaugeType::Hard => "HARD",
                GaugeType::ExHard => "EX-HARD",
                GaugeType::Hazard => "HAZARD",
            };
            let hp = gauge.hp();
            let gauge_color = if gauge.active_gauge().is_survival() {
                if hp > 30.0 {
                    Color::new(1.0, 0.2, 0.2, 1.0) // Red for survival gauges
                } else {
                    Color::new(1.0, 0.5, 0.0, 1.0) // Orange when low
                }
            } else if hp >= 80.0 {
                Color::new(0.0, 1.0, 0.5, 1.0) // Green when cleared
            } else {
                Color::new(0.2, 0.6, 1.0, 1.0) // Blue for groove gauges
            };

            draw_text(
                &format!("{}: {:.1}%", gauge_type_str, hp),
                screen_width() - 200.0,
                95.0,
                20.0,
                gauge_color,
            );

            // Draw gauge bar
            let bar_x = screen_width() - 200.0;
            let bar_y = 105.0;
            let bar_width = 150.0;
            let bar_height = 12.0;

            // Background
            draw_rectangle(bar_x, bar_y, bar_width, bar_height, DARKGRAY);
            // Fill
            draw_rectangle(
                bar_x,
                bar_y,
                bar_width * (hp / 100.0),
                bar_height,
                gauge_color,
            );
        }

        // Display FAST/SLOW with milliseconds for non-PGREAT judgments
        if let Some(result) = self.last_judgment {
            if result != JudgeResult::PGreat {
                if let Some(timing_diff) = self.last_timing_diff_ms {
                    use super::TimingDirection;
                    let direction = TimingDirection::from_timing_diff(timing_diff);
                    let (timing_label, timing_color) = match direction {
                        TimingDirection::Fast => ("FAST", Color::new(0.0, 0.8, 1.0, 1.0)),
                        TimingDirection::Slow => ("SLOW", Color::new(1.0, 0.5, 0.0, 1.0)),
                        TimingDirection::Exact => ("", WHITE),
                    };
                    if !timing_label.is_empty() {
                        // Format: "FAST -15ms" or "SLOW +23ms"
                        let timing_text = format!("{} {:+.0}ms", timing_label, -timing_diff);
                        let x = screen_width() / 2.0 - 50.0;
                        draw_text(
                            &timing_text,
                            x,
                            screen_height() / 2.0 + 40.0,
                            24.0,
                            timing_color,
                        );
                    }
                }
            }
        }

        // Draw FAST/SLOW statistics
        draw_text(
            &format!(
                "FAST:{} / SLOW:{}",
                self.timing_stats.fast_count, self.timing_stats.slow_count
            ),
            screen_width() - 200.0,
            130.0,
            16.0,
            GRAY,
        );

        draw_text(
            "[Space] Play/Pause | [R] Reset | [Up/Down] Speed | [Q/W] SUD+ | [1/2] HID+ | [A/E] LIFT",
            10.0,
            screen_height() - 20.0,
            16.0,
            GRAY,
        );

        draw_text(
            "Keys: Shift=SC, Z/S/X/D/C/F/V = 1-7",
            10.0,
            screen_height() - 40.0,
            16.0,
            GRAY,
        );
    }

    /// Calculate Green Number (visible time in milliseconds)
    /// Green Number represents how long a note is visible before reaching the judge line
    fn calculate_green_number(&self) -> f32 {
        // Base scroll time at 150 BPM, 1.0x speed for full lane visibility
        // This gives roughly 1600ms at 150 BPM with 1.0x speed
        let base_bpm = self
            .chart
            .as_ref()
            .map(|c| c.metadata.bpm as f32)
            .unwrap_or(150.0);

        // Base time for notes to travel the visible lane at 150 BPM
        let base_time_ms = 60000.0 / 150.0 * 4.0; // 4 beats visible

        // Adjust for actual BPM and speed
        let scroll_time = base_time_ms * 150.0 / base_bpm / self.scroll_speed;

        // Apply lane cover visibility ratio
        let visible_ratio = self.highway.lane_cover().visible_ratio();

        scroll_time * visible_ratio
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
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
