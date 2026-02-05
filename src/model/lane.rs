use crate::model::PlayMode;
use crate::model::note::{LANE_COUNT, Lane};
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

#[derive(Debug, Clone, Copy)]
struct LaneSpec {
    lane: Lane,
    width: f32,
    color: Color,
    is_scratch: bool,
}

impl LaneSpec {
    fn new(lane: Lane, width: f32, color: Color, is_scratch: bool) -> Self {
        Self {
            lane,
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
    pub layouts: [Option<LaneLayout>; LANE_COUNT],
    pub lane_order: Vec<Lane>,
    pub total_width: f32,
    pub offset_x: f32,
}

impl LaneConfig {
    /// Create a default 7-key configuration.
    pub fn default_7k() -> Self {
        Self::for_mode(PlayMode::Beat7K)
    }

    /// Create a default 14-key (DP) configuration.
    pub fn default_14k() -> Self {
        Self::for_mode(PlayMode::Beat14K)
    }

    /// Create a beatoraja-style 7-key configuration with narrow lanes at the left.
    pub fn beatoraja_7k() -> Self {
        let white = Color::new(0.9, 0.9, 0.9, 1.0);
        let blue = Color::new(0.3, 0.5, 0.9, 1.0);
        let scratch = Color::new(0.8, 0.2, 0.2, 1.0);

        let specs = vec![
            LaneSpec::new(Lane::Scratch, 40.0, scratch, true),
            LaneSpec::new(Lane::Key1, 30.0, white, false),
            LaneSpec::new(Lane::Key2, 26.0, blue, false),
            LaneSpec::new(Lane::Key3, 30.0, white, false),
            LaneSpec::new(Lane::Key4, 26.0, blue, false),
            LaneSpec::new(Lane::Key5, 30.0, white, false),
            LaneSpec::new(Lane::Key6, 26.0, blue, false),
            LaneSpec::new(Lane::Key7, 30.0, white, false),
        ];

        let mut layouts: [Option<LaneLayout>; LANE_COUNT] = [(); LANE_COUNT].map(|_| None);
        let mut lane_order = Vec::with_capacity(specs.len());
        let mut x = 0.0;

        for spec in &specs {
            layouts[spec.lane.index()] =
                Some(LaneLayout::new(x, spec.width, spec.color, spec.is_scratch));
            lane_order.push(spec.lane);
            x += spec.width;
        }

        Self {
            judge_line_y: 850.0,
            lane_top_y: 50.0,
            lane_height: 800.0,
            layouts,
            lane_order,
            total_width: x, // ~238px
            offset_x: 20.0,
        }
    }

    /// Create a lane configuration for the given play mode.
    pub fn for_mode(play_mode: PlayMode) -> Self {
        let white = Color::new(0.9, 0.9, 0.9, 1.0);
        let blue = Color::new(0.3, 0.5, 0.9, 1.0);
        let scratch = Color::new(0.8, 0.2, 0.2, 1.0);

        let popn_colors = [
            Color::new(0.95, 0.3, 0.3, 1.0),
            Color::new(0.95, 0.7, 0.3, 1.0),
            Color::new(0.95, 0.95, 0.3, 1.0),
            Color::new(0.4, 0.85, 0.4, 1.0),
            Color::new(0.4, 0.75, 0.95, 1.0),
            Color::new(0.35, 0.4, 0.95, 1.0),
            Color::new(0.7, 0.4, 0.9, 1.0),
            Color::new(0.9, 0.4, 0.75, 1.0),
            Color::new(0.9, 0.6, 0.2, 1.0),
        ];

        match play_mode {
            PlayMode::Beat5K => {
                let specs = vec![
                    LaneSpec::new(Lane::Scratch, 60.0, scratch, true),
                    LaneSpec::new(Lane::Key1, 50.0, white, false),
                    LaneSpec::new(Lane::Key2, 40.0, blue, false),
                    LaneSpec::new(Lane::Key3, 50.0, white, false),
                    LaneSpec::new(Lane::Key4, 40.0, blue, false),
                    LaneSpec::new(Lane::Key5, 50.0, white, false),
                ];
                Self::build_from_specs(&specs, None, 0.0, 120.0)
            }
            PlayMode::Beat7K => {
                let specs = vec![
                    LaneSpec::new(Lane::Scratch, 60.0, scratch, true),
                    LaneSpec::new(Lane::Key1, 50.0, white, false),
                    LaneSpec::new(Lane::Key2, 40.0, blue, false),
                    LaneSpec::new(Lane::Key3, 50.0, white, false),
                    LaneSpec::new(Lane::Key4, 40.0, blue, false),
                    LaneSpec::new(Lane::Key5, 50.0, white, false),
                    LaneSpec::new(Lane::Key6, 40.0, blue, false),
                    LaneSpec::new(Lane::Key7, 50.0, white, false),
                ];
                Self::build_from_specs(&specs, None, 0.0, 100.0)
            }
            PlayMode::Beat10K => {
                let specs = vec![
                    LaneSpec::new(Lane::Scratch, 50.0, scratch, true),
                    LaneSpec::new(Lane::Key1, 40.0, white, false),
                    LaneSpec::new(Lane::Key2, 32.0, blue, false),
                    LaneSpec::new(Lane::Key3, 40.0, white, false),
                    LaneSpec::new(Lane::Key4, 32.0, blue, false),
                    LaneSpec::new(Lane::Key5, 40.0, white, false),
                    LaneSpec::new(Lane::Scratch2, 50.0, scratch, true),
                    LaneSpec::new(Lane::Key8, 40.0, white, false),
                    LaneSpec::new(Lane::Key9, 32.0, blue, false),
                    LaneSpec::new(Lane::Key10, 40.0, white, false),
                    LaneSpec::new(Lane::Key11, 32.0, blue, false),
                    LaneSpec::new(Lane::Key12, 40.0, white, false),
                ];
                Self::build_from_specs(&specs, Some(6), 20.0, 60.0)
            }
            PlayMode::Beat14K => {
                let specs = vec![
                    LaneSpec::new(Lane::Scratch, 50.0, scratch, true),
                    LaneSpec::new(Lane::Key1, 40.0, white, false),
                    LaneSpec::new(Lane::Key2, 32.0, blue, false),
                    LaneSpec::new(Lane::Key3, 40.0, white, false),
                    LaneSpec::new(Lane::Key4, 32.0, blue, false),
                    LaneSpec::new(Lane::Key5, 40.0, white, false),
                    LaneSpec::new(Lane::Key6, 32.0, blue, false),
                    LaneSpec::new(Lane::Key7, 40.0, white, false),
                    LaneSpec::new(Lane::Scratch2, 50.0, scratch, true),
                    LaneSpec::new(Lane::Key8, 40.0, white, false),
                    LaneSpec::new(Lane::Key9, 32.0, blue, false),
                    LaneSpec::new(Lane::Key10, 40.0, white, false),
                    LaneSpec::new(Lane::Key11, 32.0, blue, false),
                    LaneSpec::new(Lane::Key12, 40.0, white, false),
                    LaneSpec::new(Lane::Key13, 32.0, blue, false),
                    LaneSpec::new(Lane::Key14, 40.0, white, false),
                ];
                Self::build_from_specs(&specs, Some(8), 20.0, 50.0)
            }
            PlayMode::PopN5K => {
                let specs = vec![
                    LaneSpec::new(Lane::Key1, 55.0, popn_colors[0], false),
                    LaneSpec::new(Lane::Key2, 55.0, popn_colors[1], false),
                    LaneSpec::new(Lane::Key3, 55.0, popn_colors[2], false),
                    LaneSpec::new(Lane::Key4, 55.0, popn_colors[3], false),
                    LaneSpec::new(Lane::Key5, 55.0, popn_colors[4], false),
                ];
                Self::build_from_specs(&specs, None, 0.0, 200.0)
            }
            PlayMode::PopN9K => {
                let specs = vec![
                    LaneSpec::new(Lane::Key1, 55.0, popn_colors[0], false),
                    LaneSpec::new(Lane::Key2, 55.0, popn_colors[1], false),
                    LaneSpec::new(Lane::Key3, 55.0, popn_colors[2], false),
                    LaneSpec::new(Lane::Key4, 55.0, popn_colors[3], false),
                    LaneSpec::new(Lane::Key5, 55.0, popn_colors[4], false),
                    LaneSpec::new(Lane::Key6, 55.0, popn_colors[5], false),
                    LaneSpec::new(Lane::Key7, 55.0, popn_colors[6], false),
                    LaneSpec::new(Lane::Key8, 55.0, popn_colors[7], false),
                    LaneSpec::new(Lane::Key9, 55.0, popn_colors[8], false),
                ];
                Self::build_from_specs(&specs, None, 0.0, 150.0)
            }
        }
    }

    fn build_from_specs(
        specs: &[LaneSpec],
        gap_after: Option<usize>,
        gap: f32,
        offset_x: f32,
    ) -> Self {
        let mut layouts: [Option<LaneLayout>; LANE_COUNT] = [(); LANE_COUNT].map(|_| None);
        let mut lane_order = Vec::with_capacity(specs.len());
        let mut x = 0.0;

        for (idx, spec) in specs.iter().enumerate() {
            if let Some(gap_index) = gap_after {
                if idx == gap_index {
                    x += gap;
                }
            }

            layouts[spec.lane.index()] =
                Some(LaneLayout::new(x, spec.width, spec.color, spec.is_scratch));
            lane_order.push(spec.lane);
            x += spec.width;
        }

        Self {
            judge_line_y: 900.0,
            lane_top_y: 100.0,
            lane_height: 800.0,
            layouts,
            lane_order,
            total_width: x,
            offset_x,
        }
    }

    /// Get the layout for a specific lane.
    pub fn get_layout(&self, lane: Lane) -> Option<&LaneLayout> {
        self.layouts
            .get(lane.index())
            .and_then(|layout| layout.as_ref())
    }

    /// Get the X position for a lane (with offset).
    pub fn lane_x(&self, lane: Lane) -> f32 {
        self.layouts
            .get(lane.index())
            .and_then(|l| l.as_ref())
            .map(|l| self.offset_x + l.x)
            .unwrap_or(0.0)
    }

    /// Get the width for a lane.
    pub fn lane_width(&self, lane: Lane) -> f32 {
        self.layouts
            .get(lane.index())
            .and_then(|l| l.as_ref())
            .map(|l| l.width)
            .unwrap_or(0.0)
    }

    /// Get the ordered lanes for rendering and input.
    pub fn lanes(&self) -> &[Lane] {
        &self.lane_order
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

        assert_eq!(config.lane_order.len(), 8);
        assert!(config.get_layout(Lane::Scratch).unwrap().is_scratch);

        for lane in config.lanes().iter().skip(1) {
            assert!(!config.get_layout(*lane).unwrap().is_scratch);
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
