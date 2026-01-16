//! UI layout configuration
//!
//! Contains positioning and sizing for UI elements.

use serde::{Deserialize, Serialize};

/// Rectangle position and size
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Point position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// BGA display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgaLayout {
    pub position: Point,
    pub width: f32,
    pub height: f32,
}

impl Default for BgaLayout {
    fn default() -> Self {
        Self {
            position: Point::new(10.0, 100.0),
            width: 256.0,
            height: 256.0,
        }
    }
}

/// Score/combo display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreLayout {
    /// EX Score display position (right-aligned)
    pub ex_score_position: Point,
    /// Combo display position
    pub combo_position: Point,
    /// Max combo display position
    pub max_combo_position: Point,
    /// Font size for score values
    pub font_size: f32,
}

impl Default for ScoreLayout {
    fn default() -> Self {
        Self {
            ex_score_position: Point::new(-200.0, 30.0), // Negative X means from right edge
            combo_position: Point::new(-200.0, 50.0),
            max_combo_position: Point::new(-200.0, 70.0),
            font_size: 20.0,
        }
    }
}

/// Gauge display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeLayout {
    /// Gauge text position
    pub text_position: Point,
    /// Gauge bar position
    pub bar_position: Point,
    /// Gauge bar size
    pub bar_width: f32,
    pub bar_height: f32,
    /// Font size for gauge text
    pub font_size: f32,
}

impl Default for GaugeLayout {
    fn default() -> Self {
        Self {
            text_position: Point::new(-200.0, 95.0),
            bar_position: Point::new(-200.0, 105.0),
            bar_width: 150.0,
            bar_height: 12.0,
            font_size: 20.0,
        }
    }
}

/// FAST/SLOW indicator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingLayout {
    /// Position relative to screen center
    pub position: Point,
    /// Font size
    pub font_size: f32,
    /// Statistics display position
    pub stats_position: Point,
    /// Statistics font size
    pub stats_font_size: f32,
}

impl Default for TimingLayout {
    fn default() -> Self {
        Self {
            position: Point::new(-50.0, 40.0), // Relative to center
            font_size: 24.0,
            stats_position: Point::new(-200.0, 130.0),
            stats_font_size: 16.0,
        }
    }
}

/// Judge effect display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeLayout {
    /// Position relative to screen center (0.0, 0.0 = center)
    pub position: Point,
}

impl Default for JudgeLayout {
    fn default() -> Self {
        Self {
            position: Point::new(0.0, 0.0),
        }
    }
}

/// Combo display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboLayout {
    /// Position relative to screen center
    pub position: Point,
}

impl Default for ComboLayout {
    fn default() -> Self {
        Self {
            position: Point::new(0.0, 50.0),
        }
    }
}

/// Song info display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInfoLayout {
    /// Position (from bottom-left)
    pub position: Point,
    /// Title font size
    pub title_font_size: f32,
    /// Artist font size
    pub artist_font_size: f32,
    /// Info font size (BPM, notes)
    pub info_font_size: f32,
}

impl Default for SongInfoLayout {
    fn default() -> Self {
        Self {
            position: Point::new(10.0, -80.0), // Negative Y means from bottom
            title_font_size: 24.0,
            artist_font_size: 18.0,
            info_font_size: 16.0,
        }
    }
}

/// Green number display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenNumberLayout {
    pub position: Point,
    pub font_size: f32,
}

impl Default for GreenNumberLayout {
    fn default() -> Self {
        Self {
            position: Point::new(10.0, 70.0),
            font_size: 20.0,
        }
    }
}

/// Complete layout configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutConfig {
    /// BGA display settings
    pub bga: BgaLayout,
    /// Score display settings
    pub score: ScoreLayout,
    /// Gauge display settings
    pub gauge: GaugeLayout,
    /// FAST/SLOW indicator settings
    pub timing: TimingLayout,
    /// Judge effect settings
    pub judge: JudgeLayout,
    /// Combo display settings
    pub combo: ComboLayout,
    /// Song info settings
    pub song_info: SongInfoLayout,
    /// Green number settings
    pub green_number: GreenNumberLayout,
}

#[allow(dead_code)]
impl LayoutConfig {
    /// Resolve a position that may use negative values for right/bottom anchoring
    pub fn resolve_position(&self, point: Point, screen_width: f32, screen_height: f32) -> Point {
        Point {
            x: if point.x < 0.0 {
                screen_width + point.x
            } else {
                point.x
            },
            y: if point.y < 0.0 {
                screen_height + point.y
            } else {
                point.y
            },
        }
    }

    /// Resolve position relative to screen center
    pub fn resolve_center_position(
        &self,
        offset: Point,
        screen_width: f32,
        screen_height: f32,
    ) -> Point {
        Point {
            x: screen_width / 2.0 + offset.x,
            y: screen_height / 2.0 + offset.y,
        }
    }

    /// Get BGA draw position and size
    pub fn bga_rect(&self) -> Rect {
        Rect::new(
            self.bga.position.x,
            self.bga.position.y,
            self.bga.width,
            self.bga.height,
        )
    }

    /// Get gauge bar rect resolved for screen size
    pub fn gauge_bar_rect(&self, screen_width: f32, _screen_height: f32) -> Rect {
        let pos = self.resolve_position(self.gauge.bar_position, screen_width, 0.0);
        Rect::new(
            pos.x,
            self.gauge.bar_position.y,
            self.gauge.bar_width,
            self.gauge.bar_height,
        )
    }
}
