use macroquad::prelude::Color;

pub const LANE_COUNT: usize = 8;

#[derive(Debug, Clone)]
pub struct HighwayConfig {
    pub lane_width: f32,
    pub note_height: f32,
    pub judge_line_y: f32,
    pub visible_range_ms: f64,
    pub lane_colors: [Color; LANE_COUNT],
}

impl Default for HighwayConfig {
    fn default() -> Self {
        Self {
            lane_width: 50.0,
            note_height: 10.0,
            judge_line_y: 500.0,
            visible_range_ms: 2000.0,
            lane_colors: [
                Color::new(1.0, 0.3, 0.3, 1.0), // Scratch - Red
                Color::new(1.0, 1.0, 1.0, 1.0), // Key1 - White
                Color::new(0.3, 0.5, 1.0, 1.0), // Key2 - Blue
                Color::new(1.0, 1.0, 1.0, 1.0), // Key3 - White
                Color::new(0.3, 0.5, 1.0, 1.0), // Key4 - Blue
                Color::new(1.0, 1.0, 1.0, 1.0), // Key5 - White
                Color::new(0.3, 0.5, 1.0, 1.0), // Key6 - Blue
                Color::new(1.0, 1.0, 1.0, 1.0), // Key7 - White
            ],
        }
    }
}
