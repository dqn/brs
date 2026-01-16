use macroquad::prelude::Color;

use crate::bms::PlayMode;
use crate::skin::SkinTheme;

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

/// BMS lane type for color lookup
/// 0 = scratch, 1 = white key, 2 = black key
const BMS_LANE_TYPES: [u8; LANE_COUNT_BMS] = [
    0, // Scratch
    1, // Key1 - White
    2, // Key2 - Black (Blue)
    1, // Key3 - White
    2, // Key4 - Black (Blue)
    1, // Key5 - White
    2, // Key6 - Black (Blue)
    1, // Key7 - White
];

/// DP lane types (P1: S 1 2 3 4 5 6 7 | P2: 1 2 3 4 5 6 7 S)
const DP_LANE_TYPES: [u8; LANE_COUNT_DP] = [
    // P1 side (lanes 0-7)
    0, // P1 Scratch
    1, // P1 Key1 - White
    2, // P1 Key2 - Black
    1, // P1 Key3 - White
    2, // P1 Key4 - Black
    1, // P1 Key5 - White
    2, // P1 Key6 - Black
    1, // P1 Key7 - White
    // P2 side (lanes 8-15)
    1, // P2 Key1 - White
    2, // P2 Key2 - Black
    1, // P2 Key3 - White
    2, // P2 Key4 - Black
    1, // P2 Key5 - White
    2, // P2 Key6 - Black
    1, // P2 Key7 - White
    0, // P2 Scratch
];

#[derive(Debug, Clone)]
pub struct HighwayConfig {
    pub lane_width: f32,
    pub note_height: f32,
    pub judge_line_y: f32,
    pub visible_range_ms: f64,
    pub play_mode: PlayMode,
    pub skin_theme: SkinTheme,
}

impl HighwayConfig {
    /// Create a new config for a specific play mode with default skin
    pub fn for_mode(mode: PlayMode) -> Self {
        Self::for_mode_with_skin(mode, SkinTheme::default())
    }

    /// Create a new config for a specific play mode with custom skin
    pub fn for_mode_with_skin(mode: PlayMode, skin_theme: SkinTheme) -> Self {
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
            skin_theme,
        }
    }

    /// Get lane count for current play mode
    pub fn lane_count(&self) -> usize {
        lane_count(self.play_mode)
    }

    /// Get lane color for the given lane index (uses skin theme)
    pub fn lane_color(&self, lane: usize) -> Color {
        match self.play_mode {
            PlayMode::Bms7Key => {
                if lane < LANE_COUNT_BMS {
                    self.skin_theme.bms_lane_color(BMS_LANE_TYPES[lane])
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                }
            }
            PlayMode::Pms9Key => {
                if lane < LANE_COUNT_PMS {
                    self.skin_theme.pms_lane_color(lane)
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                }
            }
            PlayMode::Dp14Key => {
                if lane < LANE_COUNT_DP {
                    self.skin_theme.bms_lane_color(DP_LANE_TYPES[lane])
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                }
            }
        }
    }

    /// Get background color for lanes
    pub fn background_color(&self) -> Color {
        self.skin_theme.background_color(self.play_mode)
    }

    /// Get border color for lanes
    pub fn border_color(&self) -> Color {
        self.skin_theme.border_color(self.play_mode)
    }

    /// Get long note body color
    pub fn long_note_color(&self) -> Color {
        self.skin_theme.long_note_color()
    }

    /// Get long note edge color
    pub fn long_note_edge_color(&self) -> Color {
        self.skin_theme.long_note_edge_color()
    }

    /// Get invisible note color
    pub fn invisible_note_color(&self) -> Color {
        self.skin_theme.invisible_note_color()
    }

    /// Get landmine note color
    pub fn landmine_note_color(&self) -> Color {
        self.skin_theme.landmine_note_color()
    }

    /// Get judge line color
    pub fn judge_line_color(&self) -> Color {
        self.skin_theme.judge_line_color()
    }

    /// Get judge line thickness
    pub fn judge_line_thickness(&self) -> f32 {
        self.skin_theme.judge_line_thickness()
    }

    /// Get lane cover color
    pub fn lane_cover_color(&self) -> Color {
        self.skin_theme.lane_cover_color()
    }

    /// Get lane cover text color
    pub fn lane_cover_text_color(&self) -> Color {
        self.skin_theme.lane_cover_text_color()
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
