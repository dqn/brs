use crate::model::{LaneConfig, Timelines};
use macroquad::prelude::*;

/// Renderer for lane backgrounds, borders, judge line, and measure lines.
pub struct LaneRenderer<'a> {
    config: &'a LaneConfig,
}

impl<'a> LaneRenderer<'a> {
    /// Create a new lane renderer.
    pub fn new(config: &'a LaneConfig) -> Self {
        Self { config }
    }

    /// Draw all lane-related elements.
    pub fn draw(&self, timelines: &Timelines, current_time_ms: f64, hi_speed: f32) {
        self.draw_lane_backgrounds();
        self.draw_lane_borders();
        self.draw_measure_lines(timelines, current_time_ms, hi_speed);
        self.draw_judge_line();
    }

    /// Draw lane backgrounds.
    fn draw_lane_backgrounds(&self) {
        let bg_alpha = 0.15;

        for layout in &self.config.layouts {
            let x = self.config.offset_x + layout.x;
            let bg_color = Color::new(layout.color.r, layout.color.g, layout.color.b, bg_alpha);

            draw_rectangle(
                x,
                self.config.lane_top_y,
                layout.width,
                self.config.lane_height,
                bg_color,
            );
        }
    }

    /// Draw lane borders.
    fn draw_lane_borders(&self) {
        let border_color = Color::new(0.3, 0.3, 0.3, 1.0);
        let border_width = 1.0;

        let left_x = self.config.offset_x;
        draw_line(
            left_x,
            self.config.lane_top_y,
            left_x,
            self.config.judge_line_y,
            border_width,
            border_color,
        );

        for layout in &self.config.layouts {
            let x = self.config.offset_x + layout.x + layout.width;
            draw_line(
                x,
                self.config.lane_top_y,
                x,
                self.config.judge_line_y,
                border_width,
                border_color,
            );
        }
    }

    /// Draw the judge line.
    fn draw_judge_line(&self) {
        let judge_color = Color::new(1.0, 0.2, 0.2, 1.0);
        let line_height = 3.0;

        draw_rectangle(
            self.config.offset_x,
            self.config.judge_line_y - line_height / 2.0,
            self.config.total_width,
            line_height,
            judge_color,
        );
    }

    /// Draw measure lines.
    fn draw_measure_lines(&self, timelines: &Timelines, current_time_ms: f64, hi_speed: f32) {
        let measure_line_color = Color::new(0.5, 0.5, 0.5, 0.8);
        let line_height = 1.0;

        for timeline in timelines.measure_lines() {
            let y = self
                .config
                .time_to_y(timeline.time_ms, current_time_ms, hi_speed);

            if !self.config.is_visible(y) {
                continue;
            }

            draw_rectangle(
                self.config.offset_x,
                y - line_height / 2.0,
                self.config.total_width,
                line_height,
                measure_line_color,
            );
        }
    }
}
