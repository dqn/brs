use macroquad::prelude::*;

use crate::game::PlayResult;

use super::{Scene, SceneTransition};

pub struct ResultScene {
    result: PlayResult,
}

impl ResultScene {
    pub fn new(result: PlayResult) -> Self {
        Self { result }
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

        draw_text(&self.result.title, center_x - 150.0, 100.0, 28.0, YELLOW);
        draw_text(&self.result.artist, center_x - 150.0, 130.0, 20.0, GRAY);

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

        draw_text(
            "[Enter] Continue",
            center_x - 80.0,
            screen_height() - 30.0,
            20.0,
            GRAY,
        );
    }
}
