//! Score graph display for IIDX-style layout

use macroquad::prelude::*;

use super::font::{draw_text_jp, measure_text_jp};
use crate::skin::{GraphAreaLayout, Rect as SkinRect};

/// Score graph with grade lines and comparison
pub struct ScoreGraph {
    /// Maximum possible score (notes * 2)
    max_score: u32,
    /// Current score
    current_score: u32,
    /// Target score (e.g., personal best)
    target_score: Option<u32>,
}

impl ScoreGraph {
    pub fn new(total_notes: u32) -> Self {
        Self {
            max_score: total_notes * 2,
            current_score: 0,
            target_score: None,
        }
    }

    /// Set the target score (personal best)
    #[allow(dead_code)]
    pub fn set_target(&mut self, score: Option<u32>) {
        self.target_score = score;
    }

    /// Update current score
    pub fn update(&mut self, score: u32) {
        self.current_score = score;
    }

    /// Draw the score graph within the specified rect
    pub fn draw(&self, rect: &SkinRect, layout: &GraphAreaLayout) {
        let padding = layout.graph_padding;
        let header_height = layout.header_height;
        let graph_x = rect.x + padding;
        let graph_y = rect.y + header_height;
        let graph_width = rect.width - padding * 2.0;
        let graph_height = rect.height - header_height - padding;

        if header_height > 0.0 {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.width,
                header_height,
                Color::new(0.12, 0.12, 0.16, 1.0),
            );
        }

        // Header
        draw_text_jp(
            "GRAPH INFORMATION",
            rect.x + 10.0,
            rect.y + layout.header_text_y,
            16.0,
            Color::new(1.0, 0.6, 0.0, 1.0),
        );

        // Graph background
        draw_rectangle(
            graph_x,
            graph_y,
            graph_width,
            graph_height,
            Color::new(0.1, 0.1, 0.15, 0.9),
        );

        // Grade lines (MAX, AAA, AA, A)
        let grades = [
            ("MAX", 1.0, Color::new(1.0, 0.84, 0.0, 1.0)),
            ("AAA", 8.0 / 9.0, Color::new(0.9, 0.9, 0.9, 1.0)),
            ("AA", 7.0 / 9.0, Color::new(0.8, 0.8, 0.0, 1.0)),
            ("A", 6.0 / 9.0, Color::new(0.0, 0.8, 0.0, 1.0)),
        ];

        for (label, ratio, color) in grades {
            let line_y = graph_y + graph_height * (1.0 - ratio);
            draw_line(
                graph_x,
                line_y,
                graph_x + graph_width,
                line_y,
                1.0,
                Color::new(color.r, color.g, color.b, 0.5),
            );
            draw_text_jp(label, graph_x + 2.0, line_y - 2.0, 12.0, color);
        }

        // Current score bar
        if self.max_score > 0 {
            let score_ratio = self.current_score as f32 / self.max_score as f32;
            let bar_height = graph_height * score_ratio;
            let bar_width = graph_width * 0.3;
            let bar_x = graph_x + graph_width * 0.35;

            draw_rectangle(
                bar_x,
                graph_y + graph_height - bar_height,
                bar_width,
                bar_height,
                Color::new(0.0, 0.8, 1.0, 0.8),
            );

            // Target score bar (if available)
            if let Some(target) = self.target_score {
                let target_ratio = target as f32 / self.max_score as f32;
                let target_bar_height = graph_height * target_ratio;
                let target_bar_x = graph_x + graph_width * 0.65;

                draw_rectangle(
                    target_bar_x,
                    graph_y + graph_height - target_bar_height,
                    bar_width * 0.8,
                    target_bar_height,
                    Color::new(0.0, 1.0, 0.5, 0.6),
                );
            }
        }

        // Score display below graph
        let score_label_y = rect.y + layout.score_label_y;
        let score_value_y = rect.y + layout.score_value_y;
        let label_x = rect.x + layout.score_label_x;
        let value_right_x = rect.x + rect.width - layout.score_value_right_margin;

        // Current score
        let score_text = format!("{}", self.current_score);
        let score_dims = measure_text_jp(&score_text, layout.score_value_font_size);
        draw_text_jp(
            "YOU",
            label_x,
            score_label_y,
            layout.score_label_font_size,
            GRAY,
        );
        draw_text_jp(
            &score_text,
            value_right_x - score_dims.width,
            score_value_y,
            layout.score_value_font_size,
            WHITE,
        );

        // Target comparison
        if let Some(target) = self.target_score {
            let diff = self.current_score as i32 - target as i32;
            let (diff_str, diff_color) = if diff >= 0 {
                (format!("+{}", diff), Color::new(0.0, 1.0, 0.5, 1.0))
            } else {
                (format!("{}", diff), Color::new(1.0, 0.3, 0.3, 1.0))
            };
            let diff_dims = measure_text_jp(&diff_str, layout.score_value_font_size);
            draw_text_jp(
                "MYBEST",
                label_x,
                score_label_y + layout.score_line_gap,
                layout.score_label_font_size,
                GRAY,
            );
            draw_text_jp(
                &diff_str,
                value_right_x - diff_dims.width,
                score_value_y + layout.score_line_gap,
                layout.score_value_font_size,
                diff_color,
            );
        }
    }
}

impl Default for ScoreGraph {
    fn default() -> Self {
        Self::new(0)
    }
}
