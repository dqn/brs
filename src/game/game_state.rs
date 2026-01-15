use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use crate::audio::{AudioManager, AudioScheduler};
use crate::bms::{BmsLoader, Chart, LANE_COUNT, NoteType};
use crate::render::Highway;

use super::{GamePlayState, InputHandler, JudgeResult, JudgeSystem, PlayResult, ScoreManager};

pub struct GameState {
    chart: Option<Chart>,
    lane_index: [Vec<usize>; LANE_COUNT],
    audio: Option<AudioManager>,
    scheduler: AudioScheduler,
    highway: Highway,
    input: InputHandler,
    judge: JudgeSystem,
    score: ScoreManager,
    play_state: Option<GamePlayState>,
    scroll_speed: f32,
    current_time_ms: f64,
    playing: bool,
    last_judgment: Option<JudgeResult>,
    active_long_notes: [Option<usize>; LANE_COUNT],
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
            play_state: None,
            scroll_speed: 1.0,
            current_time_ms: 0.0,
            playing: false,
            last_judgment: None,
            active_long_notes: [None; LANE_COUNT],
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

        let note_count = chart.notes.len();
        self.lane_index = chart.build_lane_index();
        self.play_state = Some(GamePlayState::new(note_count));
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
            self.playing = false;
            self.last_judgment = None;
            self.active_long_notes = [None; LANE_COUNT];
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
                    self.last_judgment = Some(result);
                    audio.play(note.keysound_id);

                    if note.note_type == NoteType::LongStart {
                        self.active_long_notes[lane_idx] = Some(i);
                    }
                    break;
                }
            }
        }

        // Handle key releases for long notes
        for lane in released_lanes {
            let lane_idx = lane.lane_index();
            if let Some(_start_idx) = self.active_long_notes[lane_idx].take() {
                // Find the corresponding LongEnd
                for &i in &self.lane_index[lane_idx] {
                    let note = &chart.notes[i];

                    if note.note_type != NoteType::LongEnd {
                        continue;
                    }

                    if !play_state.get_state(i).is_some_and(|s| s.is_pending()) {
                        continue;
                    }

                    let time_diff = note.time_ms - self.current_time_ms;

                    if let Some(result) = self.judge.judge(time_diff) {
                        play_state.set_judged(i, result);
                        self.score.add_judgment(result);
                        self.last_judgment = Some(result);
                        audio.play(note.keysound_id);
                        break;
                    } else if time_diff < 0.0 {
                        // Released too early
                        play_state.set_missed(i);
                        self.score.add_judgment(JudgeResult::Poor);
                        self.last_judgment = Some(JudgeResult::Poor);
                        break;
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
                self.last_judgment = Some(JudgeResult::Poor);

                // If LongStart is missed, also miss the held long note
                if note.note_type == NoteType::LongStart {
                    let lane_idx = note.channel.lane_index();
                    self.active_long_notes[lane_idx] = None;
                }
            }
        }

        // Check for missed long note ends (released too late)
        for lane_idx in 0..LANE_COUNT {
            if let Some(start_idx) = self.active_long_notes[lane_idx] {
                let start_note = &chart.notes[start_idx];

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

                    if self.judge.is_missed(time_diff) {
                        play_state.set_missed(i);
                        self.score.add_judgment(JudgeResult::Poor);
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
        }

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

    pub fn get_result(&self) -> PlayResult {
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

        PlayResult {
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
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
