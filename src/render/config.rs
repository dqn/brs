use macroquad::prelude::Color;

use crate::bms::PlayMode;

/// Lane count for BMS 7-key mode (scratch + 7 keys)
pub const LANE_COUNT_BMS: usize = 8;

/// Lane count for PMS 9-key mode (9 keys)
pub const LANE_COUNT_PMS: usize = 9;

/// Lane count for DP 14-key mode (P1: scratch + 7 keys, P2: 7 keys + scratch)
pub const LANE_COUNT_DP: usize = 16;

/// Gap between P1 and P2 highways in DP mode (pixels)
pub const DP_CENTER_GAP: f32 = 20.0;

/// Legacy constant for backward compatibility
#[allow(dead_code)]
pub const LANE_COUNT: usize = LANE_COUNT_BMS;

/// Get lane count for a specific play mode
pub fn lane_count(mode: PlayMode) -> usize {
    match mode {
        PlayMode::Bms7Key => LANE_COUNT_BMS,
        PlayMode::Pms9Key => LANE_COUNT_PMS,
        PlayMode::Dp14Key => LANE_COUNT_DP,
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

/// DP 14-key lane colors (IIDX style, P1 + P2)
/// P1: S 1 2 3 4 5 6 7 | P2: 1 2 3 4 5 6 7 S
pub const DP_LANE_COLORS: [Color; LANE_COUNT_DP] = [
    // P1 side (lanes 0-7)
    Color::new(1.0, 0.3, 0.3, 1.0), // P1 Scratch - Red
    Color::new(1.0, 1.0, 1.0, 1.0), // P1 Key1 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // P1 Key2 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // P1 Key3 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // P1 Key4 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // P1 Key5 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // P1 Key6 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // P1 Key7 - White
    // P2 side (lanes 8-15)
    Color::new(1.0, 1.0, 1.0, 1.0), // P2 Key1 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // P2 Key2 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // P2 Key3 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // P2 Key4 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // P2 Key5 - White
    Color::new(0.3, 0.5, 1.0, 1.0), // P2 Key6 - Blue
    Color::new(1.0, 1.0, 1.0, 1.0), // P2 Key7 - White
    Color::new(1.0, 0.3, 0.3, 1.0), // P2 Scratch - Red
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
                PlayMode::Dp14Key => 40.0, // Narrower lanes for DP (16 lanes)
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
            PlayMode::Dp14Key => {
                if lane < LANE_COUNT_DP {
                    DP_LANE_COLORS[lane]
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                }
            }
        }
    }

    /// Get total highway width (including center gap for DP)
    pub fn total_width(&self) -> f32 {
        match self.play_mode {
            PlayMode::Dp14Key => {
                // P1 (8 lanes) + gap + P2 (8 lanes)
                self.lane_width * 8.0 + DP_CENTER_GAP + self.lane_width * 8.0
            }
            _ => self.lane_width * self.lane_count() as f32,
        }
    }

    /// Get X offset for a lane (handles DP center gap)
    pub fn lane_x_offset(&self, lane: usize) -> f32 {
        match self.play_mode {
            PlayMode::Dp14Key => {
                if lane < 8 {
                    // P1 side
                    lane as f32 * self.lane_width
                } else {
                    // P2 side (after center gap)
                    8.0 * self.lane_width + DP_CENTER_GAP + (lane - 8) as f32 * self.lane_width
                }
            }
            _ => lane as f32 * self.lane_width,
        }
    }
}

impl Default for HighwayConfig {
    fn default() -> Self {
        Self::for_mode(PlayMode::Bms7Key)
    }
}
