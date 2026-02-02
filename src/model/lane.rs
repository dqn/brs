use crate::model::note::Lane;
use macroquad::prelude::Color;

/// Layout properties for a single lane.
#[derive(Debug, Clone)]
pub struct LaneLayout {
    pub x: f32,
    pub width: f32,
    pub color: Color,
    pub is_scratch: bool,
}

impl LaneLayout {
    /// Create a new lane layout.
    pub fn new(x: f32, width: f32, color: Color, is_scratch: bool) -> Self {
        Self {
            x,
            width,
            color,
            is_scratch,
        }
    }
}

/// Configuration for all lanes in the play area.
#[derive(Debug, Clone)]
pub struct LaneConfig {
    pub judge_line_y: f32,
    pub lane_top_y: f32,
    pub lane_height: f32,
    pub layouts: Vec<LaneLayout>,
    pub total_width: f32,
    pub offset_x: f32,
}

impl LaneConfig {
    /// Create a default 7-key configuration.
    pub fn default_7k() -> Self {
        let scratch_width = 60.0;
        let white_key_width = 50.0;
        let blue_key_width = 40.0;

        let white_color = Color::new(0.9, 0.9, 0.9, 1.0);
        let blue_color = Color::new(0.3, 0.5, 0.9, 1.0);
        let scratch_color = Color::new(0.8, 0.2, 0.2, 1.0);

        let mut layouts = Vec::new();
        let mut x = 0.0;

        layouts.push(LaneLayout::new(x, scratch_width, scratch_color, true));
        x += scratch_width;

        let key_colors = [
            white_color, // Key1
            blue_color,  // Key2
            white_color, // Key3
            blue_color,  // Key4
            white_color, // Key5
            blue_color,  // Key6
            white_color, // Key7
        ];

        let key_widths = [
            white_key_width, // Key1
            blue_key_width,  // Key2
            white_key_width, // Key3
            blue_key_width,  // Key4
            white_key_width, // Key5
            blue_key_width,  // Key6
            white_key_width, // Key7
        ];

        for (color, width) in key_colors.iter().zip(key_widths.iter()) {
            layouts.push(LaneLayout::new(x, *width, *color, false));
            x += width;
        }

        let total_width = x;
        let offset_x = 100.0;

        Self {
            judge_line_y: 900.0,
            lane_top_y: 100.0,
            lane_height: 800.0,
            layouts,
            total_width,
            offset_x,
        }
    }

    /// Get the layout for a specific lane.
    pub fn get_layout(&self, lane: Lane) -> Option<&LaneLayout> {
        self.layouts.get(lane.index())
    }

    /// Get the X position for a lane (with offset).
    pub fn lane_x(&self, lane: Lane) -> f32 {
        self.layouts
            .get(lane.index())
            .map(|l| self.offset_x + l.x)
            .unwrap_or(0.0)
    }

    /// Get the width for a lane.
    pub fn lane_width(&self, lane: Lane) -> f32 {
        self.layouts
            .get(lane.index())
            .map(|l| l.width)
            .unwrap_or(0.0)
    }

    /// Convert time in milliseconds to Y position.
    pub fn time_to_y(&self, time_ms: f64, current_time_ms: f64, hi_speed: f32) -> f32 {
        let delta_ms = (time_ms - current_time_ms) as f32;
        let pixels_per_ms = hi_speed * 0.5;
        self.judge_line_y - delta_ms * pixels_per_ms
    }

    /// Check if a Y position is visible on screen.
    pub fn is_visible(&self, y: f32) -> bool {
        y >= self.lane_top_y - 50.0 && y <= self.judge_line_y + 50.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_7k_config() {
        let config = LaneConfig::default_7k();

        assert_eq!(config.layouts.len(), 8);
        assert!(config.layouts[0].is_scratch);

        for layout in &config.layouts[1..] {
            assert!(!layout.is_scratch);
        }
    }

    #[test]
    fn test_time_to_y() {
        let config = LaneConfig::default_7k();

        let y = config.time_to_y(0.0, 0.0, 1.0);
        assert!((y - config.judge_line_y).abs() < 0.001);

        let y_future = config.time_to_y(1000.0, 0.0, 1.0);
        assert!(y_future < config.judge_line_y);
    }
}
