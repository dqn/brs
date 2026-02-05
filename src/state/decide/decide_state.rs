use std::time::Instant;

use anyhow::Result;
use macroquad::prelude::*;

use crate::database::SongData;
use crate::input::InputManager;
use crate::state::decide::loading_task::{LoadingStage, LoadingTask, PreparedPlayData};

/// Phase of the decide screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecidePhase {
    /// Resources are being loaded in background.
    Loading,
    /// Loading complete, showing animation/ready state.
    Showing,
    /// Transitioning to play screen.
    Transition,
}

/// Transition from the decide screen.
#[derive(Debug, Default)]
pub enum DecideTransition {
    /// No transition, stay in decide screen.
    #[default]
    None,
    /// Resources ready, transition to play.
    Ready(Box<PreparedPlayData>),
    /// User cancelled (Escape pressed).
    Cancel,
    /// Loading error occurred.
    Error(String),
}

/// Decide screen state - shows after song selection while loading resources.
pub struct DecideState {
    song_data: SongData,
    input_manager: InputManager,
    phase: DecidePhase,
    transition: DecideTransition,
    loading_task: Option<LoadingTask>,
    prepared_data: Option<PreparedPlayData>,
    show_start_time: Option<Instant>,
}

impl DecideState {
    /// Minimum time to show the decide screen after loading (ms).
    const MINIMUM_SHOW_MS: u128 = 500;

    /// Auto-transition time after showing ready state (ms).
    const AUTO_TRANSITION_MS: u128 = 2000;

    /// Create a new DecideState and start loading resources.
    pub fn new(song_data: SongData, input_manager: InputManager) -> Result<Self> {
        let loading_task = LoadingTask::start(song_data.clone());

        Ok(Self {
            song_data,
            input_manager,
            phase: DecidePhase::Loading,
            transition: DecideTransition::None,
            loading_task: Some(loading_task),
            prepared_data: None,
            show_start_time: None,
        })
    }

    /// Get the current phase.
    pub fn phase(&self) -> DecidePhase {
        self.phase
    }

    /// Take the transition request, resetting it to None.
    pub fn take_transition(&mut self) -> DecideTransition {
        std::mem::take(&mut self.transition)
    }

    /// Take the input manager from this state.
    pub fn take_input_manager(&mut self) -> InputManager {
        let key_config = self.input_manager.key_config().clone();
        let dummy = InputManager::new(key_config).unwrap();
        std::mem::replace(&mut self.input_manager, dummy)
    }

    /// Get loading progress (0.0 to 1.0).
    pub fn loading_progress(&self) -> f32 {
        if let Some(ref task) = self.loading_task {
            task.progress().overall_progress()
        } else {
            1.0
        }
    }

    /// Update the decide state. Call once per frame.
    pub fn update(&mut self) -> Result<()> {
        self.input_manager.update();

        // Check for cancel
        if is_key_pressed(KeyCode::Escape) {
            self.transition = DecideTransition::Cancel;
            return Ok(());
        }

        match self.phase {
            DecidePhase::Loading => {
                if let Some(ref task) = self.loading_task {
                    let progress = task.progress();

                    if task.is_complete() {
                        match task.take_result() {
                            Some(Ok(data)) => {
                                self.prepared_data = Some(data);
                                self.phase = DecidePhase::Showing;
                                self.show_start_time = Some(Instant::now());
                            }
                            Some(Err(e)) => {
                                self.transition =
                                    DecideTransition::Error(format!("Loading failed: {}", e));
                            }
                            None => {}
                        }
                    } else if progress.stage == LoadingStage::Failed {
                        let error_msg = progress
                            .error
                            .clone()
                            .unwrap_or_else(|| "Unknown error".to_string());
                        self.transition = DecideTransition::Error(error_msg);
                    }
                }
            }
            DecidePhase::Showing => {
                let show_elapsed = self
                    .show_start_time
                    .map(|t| t.elapsed().as_millis())
                    .unwrap_or(0);

                // Check for manual transition (Enter key) after minimum show time
                if show_elapsed >= Self::MINIMUM_SHOW_MS
                    && (is_key_pressed(KeyCode::Enter) || self.input_manager.is_start_pressed())
                {
                    self.phase = DecidePhase::Transition;
                }

                // Auto-transition after timeout
                if show_elapsed >= Self::AUTO_TRANSITION_MS {
                    self.phase = DecidePhase::Transition;
                }
            }
            DecidePhase::Transition => {
                if let Some(data) = self.prepared_data.take() {
                    self.transition = DecideTransition::Ready(Box::new(data));
                }
            }
        }

        Ok(())
    }

    /// Draw the decide screen.
    pub fn draw(&self) {
        // For now, always use fallback UI since we don't have decide skin support yet
        self.draw_fallback_ui();
    }

    fn draw_fallback_ui(&self) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Background
        draw_rectangle(
            0.0,
            0.0,
            screen_w,
            screen_h,
            Color::new(0.05, 0.05, 0.1, 1.0),
        );

        // Title
        draw_text("MUSIC DECIDE", 100.0, 80.0, 48.0, WHITE);

        // Song info box
        draw_rectangle(80.0, 120.0, 600.0, 200.0, Color::new(0.1, 0.1, 0.15, 1.0));
        draw_rectangle_lines(
            80.0,
            120.0,
            600.0,
            200.0,
            2.0,
            Color::new(0.3, 0.3, 0.4, 1.0),
        );

        // Song title
        let title = &self.song_data.title;
        let display_title = if title.len() > 50 {
            format!("{}...", &title[..47])
        } else {
            title.clone()
        };
        draw_text(&display_title, 100.0, 170.0, 32.0, WHITE);

        // Subtitle
        if !self.song_data.subtitle.is_empty() {
            draw_text(&self.song_data.subtitle, 100.0, 200.0, 20.0, GRAY);
        }

        // Artist
        draw_text(
            &format!("Artist: {}", self.song_data.artist),
            100.0,
            240.0,
            22.0,
            LIGHTGRAY,
        );

        // Genre and Level
        draw_text(
            &format!("Genre: {}", self.song_data.genre),
            100.0,
            275.0,
            18.0,
            GRAY,
        );
        draw_text(
            &format!("Level: {}", self.song_data.level),
            400.0,
            275.0,
            18.0,
            YELLOW,
        );

        // BPM
        let bpm_text = if self.song_data.min_bpm == self.song_data.max_bpm {
            format!("BPM: {}", self.song_data.max_bpm)
        } else {
            format!(
                "BPM: {} - {}",
                self.song_data.min_bpm, self.song_data.max_bpm
            )
        };
        draw_text(&bpm_text, 500.0, 275.0, 18.0, GRAY);

        // Loading status
        let progress = self.loading_progress();
        let status_y = 380.0;

        let (status_text, status_color) = match self.phase {
            DecidePhase::Loading => {
                let progress_info = if let Some(ref task) = self.loading_task {
                    let p = task.progress();
                    match p.stage {
                        LoadingStage::NotStarted => "Initializing...".to_string(),
                        LoadingStage::ParsingBms => "Parsing BMS file...".to_string(),
                        LoadingStage::LoadingAudio => {
                            format!("Loading sounds... ({}/{})", p.audio_loaded, p.audio_total)
                        }
                        LoadingStage::Complete => "Complete!".to_string(),
                        LoadingStage::Failed => "Failed!".to_string(),
                    }
                } else {
                    "Loading...".to_string()
                };
                (
                    format!("Loading... {:.0}%\n{}", progress * 100.0, progress_info),
                    YELLOW,
                )
            }
            DecidePhase::Showing => ("Ready! Press Enter to start".to_string(), GREEN),
            DecidePhase::Transition => ("Starting...".to_string(), WHITE),
        };
        draw_text(&status_text, 100.0, status_y, 24.0, status_color);

        // Progress bar
        let bar_x = 100.0;
        let bar_y = status_y + 40.0;
        let bar_width = 500.0;
        let bar_height = 24.0;

        // Background
        draw_rectangle(bar_x, bar_y, bar_width, bar_height, DARKGRAY);

        // Progress fill
        let fill_color = match self.phase {
            DecidePhase::Loading => Color::new(0.2, 0.6, 1.0, 1.0),
            DecidePhase::Showing | DecidePhase::Transition => GREEN,
        };
        draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, fill_color);

        // Border
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, WHITE);

        // Controls hint
        let hint_y = screen_h - 60.0;
        draw_text("Controls:", 100.0, hint_y, 18.0, GRAY);
        draw_text("Enter: Start game", 200.0, hint_y, 16.0, GRAY);
        draw_text("Escape: Cancel", 380.0, hint_y, 16.0, GRAY);
    }
}
