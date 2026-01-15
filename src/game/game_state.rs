use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use crate::audio::{AudioManager, AudioScheduler};
use crate::bms::{BmsLoader, Chart, LANE_COUNT, LnType, NoteType};
use crate::render::Highway;

use super::{
    ClearLamp, GamePlayState, GaugeManager, GaugeSystem, GaugeType, InputHandler, JudgeRank,
    JudgeResult, JudgeSystem, JudgeSystemType, PlayResult, ScoreManager, TimingStats,
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
}

pub struct GameState {
    chart: Option<Chart>,
    lane_index: [Vec<usize>; LANE_COUNT],
    audio: Option<AudioManager>,
    scheduler: AudioScheduler,
    highway: Highway,
    input: InputHandler,
    judge: JudgeSystem,
    score: ScoreManager,
    gauge: Option<GaugeManager>,
    play_state: Option<GamePlayState>,
    timing_stats: TimingStats,
    scroll_speed: f32,
    current_time_ms: f64,
    playing: bool,
    last_judgment: Option<JudgeResult>,
    last_timing_diff_ms: Option<f64>,
    active_long_notes: [Option<ActiveLongNote>; LANE_COUNT],
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
            play_state: None,
            timing_stats: TimingStats::default(),
            scroll_speed: 1.0,
            current_time_ms: 0.0,
            playing: false,
            last_judgment: None,
            last_timing_diff_ms: None,
            active_long_notes: [const { None }; LANE_COUNT],
        }
    }

    pub fn load_chart(&mut self, path: &str) -> Result<()> {
        let path = Path::new(path);
        let (chart, wav_files) = BmsLoader::load(path)?;

        println!(
            "Loaded: {} - {}",
            chart.metadata.title, chart.metadata.artist
        );
        println!("BPM: {}", chart.metadata.bpm);
        println!("Notes: {}", chart.note_count());
        println!("BGM events: {}", chart.bgm_events.len());

        let mut audio = AudioManager::new()?;
        if let Some(parent) = path.parent() {
            let loaded = audio.load_keysounds(parent, &wav_files)?;
            println!("Loaded {} keysounds", loaded);
        }

        let note_count = chart.note_count();
        self.lane_index = chart.build_lane_index();
        self.play_state = Some(GamePlayState::new(chart.notes.len()));

        // Initialize judge system based on chart rank
        // TODO: Make system type configurable via settings
        let judge_rank = JudgeRank::from_bms_rank(chart.metadata.rank);
        self.judge = JudgeSystem::for_system(JudgeSystemType::Beatoraja, judge_rank);

        // Initialize gauge with GAS enabled (all gauges tracked)
        self.gauge = Some(GaugeManager::new_with_gas(
            GaugeType::Normal,
            GaugeSystem::Beatoraja,
            note_count,
            true,
        ));
        self.chart = Some(chart);
        self.audio = Some(audio);
        self.scheduler.reset();
        self.score.reset();

        Ok(())
    }

    pub fn update(&mut self) {
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
                    true,
                ));
            }
            self.playing = false;
            self.last_judgment = None;
            self.active_long_notes = [const { None }; LANE_COUNT];
        }

        if is_key_pressed(KeyCode::Up) {
            self.scroll_speed += 0.1;
        }
        if is_key_pressed(KeyCode::Down) {
            self.scroll_speed = (self.scroll_speed - 0.1).max(0.1);
        }

        if self.playing {
            self.current_time_ms += get_frame_time() as f64 * 1000.0;

            if let (Some(chart), Some(audio)) = (&self.chart, &mut self.audio) {
                self.scheduler.update(chart, audio, self.current_time_ms);
            }

            self.process_input();
            self.check_missed_notes();
        }
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

                    if note.note_type == NoteType::LongStart {
                        let end_time_ms = note.long_end_time_ms.unwrap_or(note.time_ms);
                        self.active_long_notes[lane_idx] = Some(ActiveLongNote {
                            start_idx: i,
                            end_time_ms,
                            start_judgment: result,
                            ln_type,
                        });
                    }
                    break;
                }
            }
        }

        // Handle key releases for long notes
        for lane in released_lanes {
            let lane_idx = lane.lane_index();
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
                        // HCN: Similar to LN but with damage while released (handled elsewhere)
                        if let Some(end_idx) = long_end_idx {
                            if time_diff <= 0.0 {
                                play_state.set_judged(end_idx, active_ln.start_judgment);
                                self.score.add_judgment(active_ln.start_judgment);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(active_ln.start_judgment);
                                }
                                if let Some(audio) = &mut self.audio {
                                    audio.play(chart.notes[end_idx].keysound_id);
                                }
                            } else {
                                // Released during hold - continue tracking (re-press allowed)
                                // For now, treat early release as POOR
                                play_state.set_missed(end_idx);
                                self.score.add_judgment(JudgeResult::Poor);
                                if let Some(gauge) = &mut self.gauge {
                                    gauge.apply_judgment(JudgeResult::Poor);
                                }
                                self.last_judgment = Some(JudgeResult::Poor);
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
        for lane_idx in 0..LANE_COUNT {
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

    pub fn draw(&self) {
        clear_background(BLACK);

        if let (Some(chart), Some(play_state)) = (&self.chart, &self.play_state) {
            self.highway.draw_with_state(
                chart,
                play_state,
                self.current_time_ms,
                self.scroll_speed,
            );
        } else {
            draw_text(
                "No chart loaded. Pass BMS file path as argument.",
                20.0,
                screen_height() / 2.0,
                30.0,
                WHITE,
            );
        }

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

        let status = if self.playing { "Playing" } else { "Paused" };
        draw_text(&format!("Status: {}", status), 10.0, 70.0, 20.0, WHITE);

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

        if let Some(result) = self.last_judgment {
            let (text, color) = match result {
                JudgeResult::PGreat => ("PGREAT", Color::new(1.0, 1.0, 0.0, 1.0)),
                JudgeResult::Great => ("GREAT", Color::new(1.0, 0.8, 0.0, 1.0)),
                JudgeResult::Good => ("GOOD", Color::new(0.0, 1.0, 0.5, 1.0)),
                JudgeResult::Bad => ("BAD", Color::new(0.5, 0.5, 1.0, 1.0)),
                JudgeResult::Poor => ("POOR", Color::new(1.0, 0.3, 0.3, 1.0)),
            };
            let x = screen_width() / 2.0 - 60.0;
            draw_text(text, x, screen_height() / 2.0, 40.0, color);

            // Display FAST/SLOW for non-PGREAT judgments
            if result != JudgeResult::PGreat {
                if let Some(timing_diff) = self.last_timing_diff_ms {
                    use super::TimingDirection;
                    let direction = TimingDirection::from_timing_diff(timing_diff);
                    let (timing_text, timing_color) = match direction {
                        TimingDirection::Fast => ("FAST", Color::new(0.0, 0.8, 1.0, 1.0)),
                        TimingDirection::Slow => ("SLOW", Color::new(1.0, 0.5, 0.0, 1.0)),
                        TimingDirection::Exact => ("", WHITE),
                    };
                    if !timing_text.is_empty() {
                        draw_text(
                            timing_text,
                            x,
                            screen_height() / 2.0 + 30.0,
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
            "[Space] Play/Pause | [R] Reset | [Up/Down] Speed",
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
            fast_count: self.timing_stats.fast_count,
            slow_count: self.timing_stats.slow_count,
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
