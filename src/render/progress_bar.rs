use macroquad::prelude::*;

use crate::skin::Rect;

/// Progress bar for showing song progress
pub struct ProgressBar {
    /// Total song duration in milliseconds
    total_duration_ms: f64,
}

impl ProgressBar {
    pub fn new(total_duration_ms: f64) -> Self {
        Self { total_duration_ms }
    }

    /// Draw the progress bar
    /// - rect: The area to draw in (vertical bar)
    /// - current_time_ms: Current playback position
    pub fn draw(&self, rect: &Rect, current_time_ms: f64) {
        if self.total_duration_ms <= 0.0 {
            return;
        }

        let progress = (current_time_ms / self.total_duration_ms).clamp(0.0, 1.0) as f32;

        // Background
        draw_rectangle(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            Color::new(0.1, 0.1, 0.15, 0.9),
        );

        // Track line (thin vertical line)
        let track_x = rect.x + rect.width / 2.0 - 1.0;
        draw_rectangle(
            track_x,
            rect.y,
            2.0,
            rect.height,
            Color::new(0.3, 0.3, 0.35, 0.9),
        );

        // Square indicator (moves from bottom to top)
        let indicator_height = 4.0;
        let indicator_y = rect.y + rect.height * (1.0 - progress) - indicator_height / 2.0;
        draw_rectangle(
            rect.x,
            indicator_y,
            rect.width,
            indicator_height,
            Color::new(0.2, 0.7, 1.0, 1.0),
        );

        // Border
        draw_rectangle_lines(rect.x, rect.y, rect.width, rect.height, 1.0, GRAY);

        // Time display (m:ss / m:ss) at the top
        let current_time_str = format_time(current_time_ms);
        let total_time_str = format_time(self.total_duration_ms);
        draw_text(&current_time_str, rect.x + 2.0, rect.y + 16.0, 14.0, WHITE);
        draw_text(
            &total_time_str,
            rect.x + 2.0,
            rect.y + 32.0,
            14.0,
            Color::new(0.6, 0.6, 0.6, 1.0),
        );
    }
}

/// Format milliseconds as m:ss
fn format_time(ms: f64) -> String {
    let seconds = (ms / 1000.0).max(0.0) as u32;
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{}:{:02}", minutes, secs)
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(0.0)
    }
}
