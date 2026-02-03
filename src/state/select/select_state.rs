use anyhow::Result;
use macroquad::prelude::*;
use std::path::Path;
use tracing::warn;

use crate::audio::PreviewPlayer;
use crate::database::{ClearType, Database, SongData};
use crate::input::InputManager;
use crate::model::load_chart;
use crate::model::note::Lane;
use crate::state::select::bar::Bar;
use crate::state::select::bar_manager::BarManager;

/// Phase of the select screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectPhase {
    /// Loading song list from database.
    Loading,
    /// Actively selecting a song.
    Selecting,
    /// Song has been decided, transitioning to play.
    Decided,
}

/// Transition from the select screen.
#[derive(Debug, Clone)]
pub enum SelectTransition {
    /// No transition, stay in select screen.
    None,
    /// Transition to decide screen with the selected song.
    Decide(Box<SongData>),
    /// Exit the application.
    Exit,
}

/// Main select state for the song selection screen.
pub struct SelectState {
    bar_manager: BarManager,
    input_manager: InputManager,
    phase: SelectPhase,
    song_db: Database,
    score_db: Database,
    transition: SelectTransition,
    preview_player: PreviewPlayer,
    current_preview_sha256: Option<String>,
}

impl SelectState {
    /// Number of visible bars in the song list.
    const VISIBLE_BARS: usize = 15;

    /// Create a new SelectState.
    pub fn new(input_manager: InputManager, song_db: Database, score_db: Database) -> Result<Self> {
        Ok(Self {
            bar_manager: BarManager::new(),
            input_manager,
            phase: SelectPhase::Loading,
            song_db,
            score_db,
            transition: SelectTransition::None,
            preview_player: PreviewPlayer::new()?,
            current_preview_sha256: None,
        })
    }

    /// Get the current phase.
    pub fn phase(&self) -> SelectPhase {
        self.phase
    }

    /// Get the transition request.
    pub fn transition(&self) -> &SelectTransition {
        &self.transition
    }

    /// Take the transition request, resetting it to None.
    pub fn take_transition(&mut self) -> SelectTransition {
        std::mem::replace(&mut self.transition, SelectTransition::None)
    }

    /// Get a reference to the input manager.
    pub fn input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    /// Take the input manager from this state.
    pub fn take_input_manager(&mut self) -> InputManager {
        // Create a dummy input manager to swap
        let key_config = self.input_manager.key_config().clone();
        let dummy = InputManager::new(key_config).unwrap();
        std::mem::replace(&mut self.input_manager, dummy)
    }

    /// Set the input manager.
    pub fn set_input_manager(&mut self, input_manager: InputManager) {
        self.input_manager = input_manager;
    }

    /// Update the select state. Call once per frame.
    pub fn update(&mut self) -> Result<()> {
        self.input_manager.update();

        match self.phase {
            SelectPhase::Loading => {
                self.bar_manager.load_songs(&self.song_db, &self.score_db)?;
                self.update_preview();
                self.phase = SelectPhase::Selecting;
            }
            SelectPhase::Selecting => {
                self.process_input();
            }
            SelectPhase::Decided => {
                // Wait for transition to be taken
            }
        }

        Ok(())
    }

    fn process_input(&mut self) {
        let prev_cursor = self.bar_manager.cursor();

        // Up: Key2 (S key / index 2)
        if self.input_manager.just_pressed(Lane::Key2) || is_key_pressed(KeyCode::Up) {
            self.bar_manager.move_up();
        }

        // Down: Key4 (D key / index 4)
        if self.input_manager.just_pressed(Lane::Key4) || is_key_pressed(KeyCode::Down) {
            self.bar_manager.move_down();
        }

        if self.bar_manager.cursor() != prev_cursor {
            self.update_preview();
        }

        // Enter: Start or Enter key
        if self.input_manager.is_start_pressed() || is_key_pressed(KeyCode::Enter) {
            if let Some(bar) = self.bar_manager.current_bar() {
                if let Some(song_bar) = bar.as_song() {
                    self.preview_player.stop();
                    self.transition = SelectTransition::Decide(Box::new(song_bar.song.clone()));
                    self.phase = SelectPhase::Decided;
                }
            }
        }

        // Escape: Exit
        if is_key_pressed(KeyCode::Escape) {
            self.transition = SelectTransition::Exit;
        }
    }

    fn update_preview(&mut self) {
        let Some(Bar::Song(song_bar)) = self.bar_manager.current_bar() else {
            self.preview_player.stop();
            self.current_preview_sha256 = None;
            return;
        };

        if self
            .current_preview_sha256
            .as_deref()
            .is_some_and(|sha| sha == song_bar.song.sha256)
        {
            return;
        }

        self.current_preview_sha256 = Some(song_bar.song.sha256.clone());

        let loaded = match load_chart(&song_bar.song.path) {
            Ok(loaded) => loaded,
            Err(e) => {
                warn!("Failed to load chart for preview: {}", e);
                self.preview_player.stop();
                return;
            }
        };

        let preview = loaded
            .bms
            .music_info
            .preview_music
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        if let Some(preview) = preview {
            let base_dir = song_bar.song.path.parent().unwrap_or(Path::new("."));
            let preview_path = base_dir.join(preview);
            if let Err(e) = self.preview_player.play(&preview_path) {
                warn!("Failed to play preview: {}", e);
            }
        } else {
            self.preview_player.stop();
        }
    }

    /// Draw the select screen.
    pub fn draw(&self) {
        self.draw_header();
        self.draw_song_list();
        self.draw_song_info();
        self.draw_footer();
    }

    fn draw_header(&self) {
        // Title
        draw_text("MUSIC SELECT", 50.0, 50.0, 40.0, WHITE);

        // Position indicator
        if !self.bar_manager.is_empty() {
            let position_text = format!(
                "{} / {}",
                self.bar_manager.cursor() + 1,
                self.bar_manager.len()
            );
            draw_text(&position_text, 400.0, 50.0, 24.0, YELLOW);
        }
    }

    fn draw_song_list(&self) {
        let start_x = 50.0;
        let start_y = 100.0;
        let row_height = 40.0;

        if self.bar_manager.is_empty() {
            draw_text("No songs found.", start_x, start_y + 50.0, 24.0, GRAY);
            draw_text(
                "Please add BMS files to the database.",
                start_x,
                start_y + 80.0,
                18.0,
                GRAY,
            );
            return;
        }

        let (start_index, visible_bars) = self.bar_manager.visible_bars(Self::VISIBLE_BARS);
        let cursor = self.bar_manager.cursor();

        for (i, bar) in visible_bars.iter().enumerate() {
            let global_index = start_index + i;
            let y = start_y + (i as f32) * row_height;
            let is_selected = global_index == cursor;

            // Background for selected bar
            if is_selected {
                draw_rectangle(start_x - 10.0, y - 25.0, 600.0, row_height - 5.0, DARKBLUE);
            }

            // Clear lamp
            let lamp_color = self.get_clear_lamp_color(bar);
            draw_rectangle(start_x - 5.0, y - 20.0, 5.0, 25.0, lamp_color);

            // Title
            let title_color = if is_selected { WHITE } else { LIGHTGRAY };
            let title = bar.title();
            let display_title = if title.len() > 40 {
                format!("{}...", &title[..37])
            } else {
                title.to_string()
            };
            draw_text(&display_title, start_x + 10.0, y, 20.0, title_color);

            // Level (for songs)
            if let Bar::Song(song_bar) = bar {
                let level_text = format!("Lv.{}", song_bar.song.level);
                let level_color = self.get_level_color(song_bar.song.level);
                draw_text(&level_text, start_x + 500.0, y, 18.0, level_color);
            }
        }
    }

    fn draw_song_info(&self) {
        let x = 700.0;
        let y = 100.0;

        if let Some(Bar::Song(song_bar)) = self.bar_manager.current_bar() {
            let song = &song_bar.song;

            // Title
            draw_text("SONG INFO", x, y, 24.0, YELLOW);
            draw_text(&song.title, x, y + 40.0, 22.0, WHITE);

            if !song.subtitle.is_empty() {
                draw_text(&song.subtitle, x, y + 65.0, 16.0, GRAY);
            }

            // Artist
            draw_text(
                &format!("Artist: {}", song.artist),
                x,
                y + 100.0,
                18.0,
                LIGHTGRAY,
            );

            if !song.subartist.is_empty() {
                draw_text(&song.subartist, x, y + 120.0, 14.0, GRAY);
            }

            // Genre
            draw_text(&format!("Genre: {}", song.genre), x, y + 150.0, 16.0, GRAY);

            // BPM
            let bpm_text = if song.min_bpm == song.max_bpm {
                format!("BPM: {}", song.max_bpm)
            } else {
                format!("BPM: {} - {}", song.min_bpm, song.max_bpm)
            };
            draw_text(&bpm_text, x, y + 180.0, 18.0, LIGHTGRAY);

            // Notes
            draw_text(
                &format!("Notes: {}", song.notes),
                x,
                y + 210.0,
                18.0,
                LIGHTGRAY,
            );

            // Score info
            if let Some(ref score) = song_bar.score {
                draw_text("BEST SCORE", x, y + 260.0, 20.0, YELLOW);

                let clear_text = Self::clear_type_name(score.clear);
                let clear_color =
                    self.get_clear_lamp_color(&Bar::Song(Box::new((**song_bar).clone())));
                draw_text(
                    &format!("Clear: {}", clear_text),
                    x,
                    y + 290.0,
                    18.0,
                    clear_color,
                );

                draw_text(
                    &format!("EX-SCORE: {}", score.ex_score),
                    x,
                    y + 315.0,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("MAX COMBO: {}", score.max_combo),
                    x,
                    y + 340.0,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("MIN BP: {}", score.min_bp),
                    x,
                    y + 365.0,
                    18.0,
                    WHITE,
                );
                draw_text(
                    &format!("Play Count: {}", score.play_count),
                    x,
                    y + 395.0,
                    16.0,
                    GRAY,
                );
            } else {
                draw_text("NO PLAY", x, y + 260.0, 20.0, GRAY);
            }
        }
    }

    fn draw_footer(&self) {
        let y = screen_height() - 50.0;

        draw_text("Controls:", 50.0, y, 18.0, GRAY);
        draw_text("Up/Down or S/D: Move cursor", 150.0, y, 16.0, GRAY);
        draw_text("Enter/Start: Select song", 400.0, y, 16.0, GRAY);
        draw_text("Escape: Exit", 600.0, y, 16.0, GRAY);
    }

    fn get_clear_lamp_color(&self, bar: &Bar) -> Color {
        if let Bar::Song(song_bar) = bar {
            if let Some(ref score) = song_bar.score {
                return match score.clear {
                    ClearType::NoPlay => DARKGRAY,
                    ClearType::Failed => Color::new(0.5, 0.0, 0.0, 1.0),
                    ClearType::AssistEasy | ClearType::LightAssistEasy => {
                        Color::new(0.5, 0.0, 0.5, 1.0)
                    }
                    ClearType::Easy => Color::new(0.0, 0.5, 0.0, 1.0),
                    ClearType::Normal => Color::new(0.0, 0.0, 1.0, 1.0),
                    ClearType::Hard => Color::new(1.0, 1.0, 1.0, 1.0),
                    ClearType::ExHard => Color::new(1.0, 1.0, 0.0, 1.0),
                    ClearType::FullCombo => Color::new(0.0, 1.0, 1.0, 1.0),
                    ClearType::Perfect => Color::new(1.0, 0.5, 0.0, 1.0),
                    ClearType::Max => Color::new(1.0, 0.84, 0.0, 1.0),
                };
            }
        }
        DARKGRAY
    }

    fn get_level_color(&self, level: i32) -> Color {
        match level {
            0..=3 => Color::new(0.5, 1.0, 0.5, 1.0),
            4..=6 => Color::new(1.0, 1.0, 0.5, 1.0),
            7..=9 => Color::new(1.0, 0.7, 0.5, 1.0),
            10..=11 => Color::new(1.0, 0.5, 0.5, 1.0),
            _ => Color::new(1.0, 0.3, 0.3, 1.0),
        }
    }

    fn clear_type_name(clear: ClearType) -> &'static str {
        match clear {
            ClearType::NoPlay => "NO PLAY",
            ClearType::Failed => "FAILED",
            ClearType::AssistEasy => "ASSIST EASY",
            ClearType::LightAssistEasy => "L-ASSIST EASY",
            ClearType::Easy => "EASY",
            ClearType::Normal => "NORMAL",
            ClearType::Hard => "HARD",
            ClearType::ExHard => "EX-HARD",
            ClearType::FullCombo => "FULL COMBO",
            ClearType::Perfect => "PERFECT",
            ClearType::Max => "MAX",
        }
    }
}
