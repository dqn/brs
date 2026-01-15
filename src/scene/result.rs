use std::path::Path;

use macroquad::prelude::*;

use crate::database::{SavedScore, ScoreRepository, compute_file_hash};
use crate::game::{ClearLamp, PlayResult};

use super::{Scene, SceneTransition};

pub struct ResultScene {
    result: PlayResult,
    is_new_record: bool,
}

impl ResultScene {
    pub fn new(result: PlayResult) -> Self {
        let is_new_record = Self::save_score(&result);
        Self {
            result,
            is_new_record,
        }
    }

    fn save_score(result: &PlayResult) -> bool {
        // Compute hash from chart file
        let hash = match compute_file_hash(Path::new(&result.chart_path)) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to compute chart hash: {}", e);
                return false;
            }
        };

        // Load repository and save score
        let mut repo = match ScoreRepository::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load score repository: {}", e);
                return false;
            }
        };

        let new_score = SavedScore::from_play_result(hash.clone(), result);
        let is_new_record = repo.update(&hash, new_score);

        if let Err(e) = repo.save() {
            eprintln!("Failed to save scores: {}", e);
        }

        is_new_record
    }

    fn clear_lamp_color(&self) -> Color {
        match self.result.clear_lamp {
            ClearLamp::NoPlay => GRAY,
            ClearLamp::Failed => Color::new(0.5, 0.0, 0.0, 1.0),
            ClearLamp::AssistEasy => Color::new(0.6, 0.3, 0.8, 1.0),
            ClearLamp::Easy => Color::new(0.0, 0.8, 0.3, 1.0),
            ClearLamp::Normal => Color::new(0.2, 0.6, 1.0, 1.0),
            ClearLamp::Hard => Color::new(1.0, 0.5, 0.0, 1.0),
            ClearLamp::ExHard => Color::new(1.0, 0.8, 0.0, 1.0),
            ClearLamp::FullCombo => Color::new(1.0, 0.0, 0.5, 1.0),
        }
    }
}

impl Scene for ResultScene {
    fn update(&mut self) -> SceneTransition {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }
        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));

        let center_x = screen_width() / 2.0;

        draw_text("RESULT", center_x - 60.0, 50.0, 40.0, WHITE);

        // Clear lamp display
        let lamp_text = self.result.clear_lamp.display_name();
        let lamp_color = self.clear_lamp_color();
        draw_text(lamp_text, center_x - 100.0, 80.0, 24.0, lamp_color);

        // New record indicator
        if self.is_new_record {
            draw_text("NEW RECORD!", center_x + 50.0, 80.0, 20.0, GOLD);
        }

        draw_text(&self.result.title, center_x - 150.0, 120.0, 28.0, YELLOW);
        draw_text(&self.result.artist, center_x - 150.0, 150.0, 20.0, GRAY);

        let rank = self.result.rank();
        let rank_color = match rank {
            "MAX" => Color::new(1.0, 0.8, 0.0, 1.0),
            "AAA" => Color::new(1.0, 0.6, 0.0, 1.0),
            "AA" => Color::new(0.8, 0.8, 0.0, 1.0),
            "A" => Color::new(0.0, 1.0, 0.5, 1.0),
            "B" => Color::new(0.0, 0.8, 1.0, 1.0),
            _ => WHITE,
        };

        draw_text(rank, center_x - 30.0, 220.0, 80.0, rank_color);
        draw_text(
            &format!("{:.2}%", self.result.accuracy()),
            center_x - 60.0,
            270.0,
            32.0,
            WHITE,
        );

        let stats_x = center_x - 120.0;
        let stats_start_y = 320.0;
        let line_height = 30.0;

        draw_text(
            &format!("EX SCORE: {}", self.result.ex_score),
            stats_x,
            stats_start_y,
            24.0,
            YELLOW,
        );
        draw_text(
            &format!("MAX COMBO: {}", self.result.max_combo),
            stats_x,
            stats_start_y + line_height,
            24.0,
            WHITE,
        );

        draw_text(
            &format!("PGREAT: {}", self.result.pgreat_count),
            stats_x,
            stats_start_y + line_height * 3.0,
            20.0,
            Color::new(1.0, 1.0, 0.0, 1.0),
        );
        draw_text(
            &format!("GREAT: {}", self.result.great_count),
            stats_x,
            stats_start_y + line_height * 4.0,
            20.0,
            Color::new(1.0, 0.8, 0.0, 1.0),
        );
        draw_text(
            &format!("GOOD: {}", self.result.good_count),
            stats_x,
            stats_start_y + line_height * 5.0,
            20.0,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );
        draw_text(
            &format!("BAD: {}", self.result.bad_count),
            stats_x,
            stats_start_y + line_height * 6.0,
            20.0,
            Color::new(0.5, 0.5, 1.0, 1.0),
        );
        draw_text(
            &format!("POOR: {}", self.result.poor_count),
            stats_x,
            stats_start_y + line_height * 7.0,
            20.0,
            Color::new(1.0, 0.3, 0.3, 1.0),
        );

        // FAST/SLOW statistics
        draw_text(
            &format!(
                "FAST:{} / SLOW:{}",
                self.result.fast_count, self.result.slow_count
            ),
            stats_x,
            stats_start_y + line_height * 9.0,
            18.0,
            GRAY,
        );

        draw_text(
            "[Enter] Continue",
            center_x - 80.0,
            screen_height() - 30.0,
            20.0,
            GRAY,
        );
    }
}
