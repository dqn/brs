use std::path::PathBuf;

use macroquad::prelude::*;

use super::{ResultScene, Scene, SceneTransition};
use crate::game::GameState;

/// Error types that can occur during gameplay
#[derive(Debug)]
pub enum GameplayError {
    /// Chart file not found
    ChartNotFound(PathBuf),
    /// Failed to parse chart
    ChartParseError(String),
    /// Failed to load audio
    AudioLoadError(String),
    /// Other errors
    Other(String),
}

impl std::fmt::Display for GameplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChartNotFound(path) => write!(f, "Chart not found: {}", path.display()),
            Self::ChartParseError(msg) => write!(f, "Failed to parse chart: {}", msg),
            Self::AudioLoadError(msg) => write!(f, "Failed to load audio: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

pub struct GameplayScene {
    state: GameState,
    chart_path: String,
    loaded: bool,
    finished: bool,
    /// Error state for displaying error UI
    error: Option<GameplayError>,
}

impl GameplayScene {
    pub fn new(chart_path: String) -> Self {
        Self {
            state: GameState::new(),
            chart_path,
            loaded: false,
            finished: false,
            error: None,
        }
    }

    /// Draw error screen
    fn draw_error(&self, error: &GameplayError) {
        clear_background(Color::from_rgba(20, 20, 30, 255));

        let screen_w = screen_width();
        let screen_h = screen_height();

        // Error title
        let title = "Error";
        let title_size = 48.0;
        let title_dims = measure_text(title, None, title_size as u16, 1.0);
        draw_text(
            title,
            (screen_w - title_dims.width) / 2.0,
            screen_h / 3.0,
            title_size,
            RED,
        );

        // Error message
        let message = error.to_string();
        let message_size = 24.0;
        let message_dims = measure_text(&message, None, message_size as u16, 1.0);

        // Word wrap if message is too long
        let max_width = screen_w * 0.8;
        if message_dims.width > max_width {
            // Simple word wrap
            let words: Vec<&str> = message.split_whitespace().collect();
            let mut lines = Vec::new();
            let mut current_line = String::new();

            for word in words {
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                let test_dims = measure_text(&test_line, None, message_size as u16, 1.0);
                if test_dims.width > max_width && !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = word.to_string();
                } else {
                    current_line = test_line;
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }

            let line_height = message_size * 1.5;
            let start_y = screen_h / 2.0;
            for (i, line) in lines.iter().enumerate() {
                let line_dims = measure_text(line, None, message_size as u16, 1.0);
                draw_text(
                    line,
                    (screen_w - line_dims.width) / 2.0,
                    start_y + (i as f32) * line_height,
                    message_size,
                    WHITE,
                );
            }
        } else {
            draw_text(
                &message,
                (screen_w - message_dims.width) / 2.0,
                screen_h / 2.0,
                message_size,
                WHITE,
            );
        }

        // Instructions
        let instruction = "Press ESC or Enter to return";
        let instruction_size = 20.0;
        let instruction_dims = measure_text(instruction, None, instruction_size as u16, 1.0);
        draw_text(
            instruction,
            (screen_w - instruction_dims.width) / 2.0,
            screen_h * 2.0 / 3.0,
            instruction_size,
            GRAY,
        );
    }

    /// Convert anyhow::Error to GameplayError
    fn categorize_error(e: anyhow::Error, chart_path: &str) -> GameplayError {
        let error_str = e.to_string().to_lowercase();

        if error_str.contains("not found") || error_str.contains("no such file") {
            GameplayError::ChartNotFound(PathBuf::from(chart_path))
        } else if error_str.contains("parse")
            || error_str.contains("bms")
            || error_str.contains("format")
        {
            GameplayError::ChartParseError(e.to_string())
        } else if error_str.contains("audio") || error_str.contains("sound") {
            GameplayError::AudioLoadError(e.to_string())
        } else {
            GameplayError::Other(e.to_string())
        }
    }
}

impl Scene for GameplayScene {
    fn update(&mut self) -> SceneTransition {
        // Handle error state
        if self.error.is_some() {
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                return SceneTransition::Pop;
            }
            return SceneTransition::None;
        }

        if !self.loaded {
            if let Err(e) = self.state.load_chart(&self.chart_path) {
                let error = Self::categorize_error(e, &self.chart_path);
                eprintln!("Error loading chart: {}", error);
                self.error = Some(error);
                return SceneTransition::None;
            }
            self.loaded = true;
        }

        if is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        self.state.update();

        // Check for game completion or failure
        let should_finish = self.state.is_finished() || self.state.is_failed();
        if should_finish && !self.finished {
            self.finished = true;
            let result = self.state.get_result(&self.chart_path);
            return SceneTransition::Replace(Box::new(ResultScene::new(result)));
        }

        SceneTransition::None
    }

    fn draw(&self) {
        if let Some(ref error) = self.error {
            self.draw_error(error);
        } else {
            self.state.draw();
        }
    }
}
