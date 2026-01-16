use macroquad::prelude::Color;

use crate::bms::PlayMode;

/// Lane count for BMS 7-key mode (scratch + 7 keys)
pub const LANE_COUNT_BMS: usize = 8;

/// Lane count for PMS 9-key mode (9 keys)
pub const LANE_COUNT_PMS: usize = 9;

/// Legacy constant for backward compatibility
#[allow(dead_code)]
pub const LANE_COUNT: usize = LANE_COUNT_BMS;

/// Get lane count for a specific play mode
pub fn lane_count(mode: PlayMode) -> usize {
    match mode {
        PlayMode::Bms7Key => LANE_COUNT_BMS,
        PlayMode::Pms9Key => LANE_COUNT_PMS,
    }
}

/// BMS 7-key lane colors (IIDX style)
pub const BMS_LANE_COLORS: [Color; LANE_COUNT_BMS] = [
    Color::new(1.0, 0.3, 0.3, 1.0), // Scratch - Red
    Color::new(1.0, 1.0, 1.0, 1.0), // Key1 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // Key2 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // Key3 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // Key4 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // Key5 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // Key6 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // Key7 - White
];

/// PMS 9-key lane colors (Pop'n style rainbow)
/// Key1  Key2  Key3  Key4  Key5  Key6  Key7  Key8  Key9
/// White Yellow Green Blue  Red   Blue  Green Yellow White
pub const PMS_LANE_COLORS: [Color; LANE_COUNT_PMS] = [
    Color::new(1.0, 1.0, 1.0, 1.0), // Key1 - White
    Color::new(1.0, 0.9, 0.2, 1.0), // Key2 - Yellow
    Color::new(0.2, 0.9, 0.3, 1.0), // Key3 - Green
    Color::new(0.3, 0.5, 1.0, 1.0), // Key4 - Blue
    Color::new(1.0, 0.2, 0.2, 1.0), // Key5 - Red (center)
    Color::new(0.3, 0.5, 1.0, 1.0), // Key6 - Blue
    Color::new(0.2, 0.9, 0.3, 1.0), // Key7 - Green
    Color::new(1.0, 0.9, 0.2, 1.0), // Key8 - Yellow
    Color::new(1.0, 1.0, 1.0, 1.0), // Key9 - White
];

#[derive(Debug, Clone)]
pub struct HighwayConfig {
    pub lane_width: f32,
    pub note_height: f32,
    pub judge_line_y: f32,
    pub visible_range_ms: f64,
    pub play_mode: PlayMode,
}

impl HighwayConfig {
    /// Create a new config for a specific play mode
    pub fn for_mode(mode: PlayMode) -> Self {
        Self {
            lane_width: match mode {
                PlayMode::Bms7Key => 50.0,
                PlayMode::Pms9Key => 44.0, // Narrower lanes for 9 keys
            },
            note_height: 10.0,
            judge_line_y: 500.0,
            visible_range_ms: 2000.0,
            play_mode: mode,
        }
    }

    /// Get lane count for current play mode
    pub fn lane_count(&self) -> usize {
        lane_count(self.play_mode)
    }

    /// Get lane color for the given lane index
    pub fn lane_color(&self, lane: usize) -> Color {
        match self.play_mode {
            PlayMode::Bms7Key => {
                if lane < LANE_COUNT_BMS {
                    BMS_LANE_COLORS[lane]
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                }
            }
            PlayMode::Pms9Key => {
                if lane < LANE_COUNT_PMS {
                    PMS_LANE_COLORS[lane]
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                }
            }
        }
    }
}

impl Default for HighwayConfig {
    fn default() -> Self {
        Self::for_mode(PlayMode::Bms7Key)
    }
}
