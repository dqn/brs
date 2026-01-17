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

// =============================================================================
// IIDX-style Layout
// =============================================================================

/// Screen areas for IIDX-style 3-column layout
#[derive(Debug, Clone, Copy)]
pub struct ScreenAreas {
    /// Left area: play area (highway + turntable + gauge + score)
    pub play: Rect,
    /// Center area: graph information
    pub graph: Rect,
    /// Right area: BGA + song info + judge stats
    pub info: Rect,
}

/// IIDX-style screen layout configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IidxLayout {
    /// Screen width ratio for play area (default: 0.28)
    pub play_area_ratio: f32,
    /// Screen width ratio for graph area (default: 0.23)
    pub graph_area_ratio: f32,
    // Right area ratio is calculated as: 1.0 - play_area_ratio - graph_area_ratio
}

impl Default for IidxLayout {
    fn default() -> Self {
        Self {
            play_area_ratio: 0.28,
            graph_area_ratio: 0.23,
        }
    }
}

impl IidxLayout {
    /// Calculate screen areas from screen dimensions
    pub fn calculate_areas(&self, screen_width: f32, screen_height: f32) -> ScreenAreas {
        let play_width = screen_width * self.play_area_ratio;
        let graph_width = screen_width * self.graph_area_ratio;
        let info_width = screen_width - play_width - graph_width;

        ScreenAreas {
            play: Rect::new(0.0, 0.0, play_width, screen_height),
            graph: Rect::new(play_width, 0.0, graph_width, screen_height),
            info: Rect::new(play_width + graph_width, 0.0, info_width, screen_height),
        }
    }

    /// Get info area ratio (calculated)
    #[allow(dead_code)]
    pub fn info_area_ratio(&self) -> f32 {
        1.0 - self.play_area_ratio - self.graph_area_ratio
    }
}

/// Play area internal layout
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayAreaLayout {
    /// Turntable width ratio relative to play area width (default: 0.2)
    pub turntable_ratio: f32,
    /// Highway area height ratio (default: 0.75)
    pub highway_height_ratio: f32,
    /// Gauge bar height in pixels
    pub gauge_height: f32,
    /// Bottom margin for score display
    pub score_area_height: f32,
}

impl Default for PlayAreaLayout {
    fn default() -> Self {
        Self {
            turntable_ratio: 0.2,
            highway_height_ratio: 0.75,
            gauge_height: 20.0,
            score_area_height: 50.0,
        }
    }
}

impl PlayAreaLayout {
    /// Calculate turntable rect within play area
    pub fn turntable_rect(&self, play_area: &Rect) -> Rect {
        let tt_width = play_area.width * self.turntable_ratio;
        let tt_height = play_area.height * (1.0 - self.highway_height_ratio);
        Rect::new(
            play_area.x,
            play_area.y + play_area.height * self.highway_height_ratio,
            tt_width,
            tt_height - self.gauge_height - self.score_area_height,
        )
    }

    /// Calculate highway rect within play area
    pub fn highway_rect(&self, play_area: &Rect) -> Rect {
        let tt_width = play_area.width * self.turntable_ratio;
        Rect::new(
            play_area.x + tt_width,
            play_area.y,
            play_area.width - tt_width,
            play_area.height * self.highway_height_ratio,
        )
    }

    /// Calculate gauge rect within play area
    pub fn gauge_rect(&self, play_area: &Rect) -> Rect {
        let gauge_y = play_area.y + play_area.height - self.gauge_height - self.score_area_height;
        Rect::new(play_area.x, gauge_y, play_area.width, self.gauge_height)
    }

    /// Calculate score display rect within play area
    pub fn score_rect(&self, play_area: &Rect) -> Rect {
        let score_y = play_area.y + play_area.height - self.score_area_height;
        Rect::new(
            play_area.x,
            score_y,
            play_area.width,
            self.score_area_height,
        )
    }
}

/// Info area (right side) internal layout
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InfoAreaLayout {
    /// Header height for song info
    pub header_height: f32,
    /// BGA aspect ratio (width/height, default: 4/3)
    pub bga_aspect_ratio: f32,
    /// Judge stats width ratio relative to info area width
    pub judge_stats_width_ratio: f32,
    /// Bottom panel height for judge stats and BPM
    pub bottom_panel_height: f32,
}

impl Default for InfoAreaLayout {
    fn default() -> Self {
        Self {
            header_height: 80.0,
            bga_aspect_ratio: 4.0 / 3.0,
            judge_stats_width_ratio: 0.55,
            bottom_panel_height: 180.0,
        }
    }
}

impl InfoAreaLayout {
    /// Calculate header rect for song info
    pub fn header_rect(&self, info_area: &Rect) -> Rect {
        Rect::new(
            info_area.x,
            info_area.y,
            info_area.width,
            self.header_height,
        )
    }

    /// Calculate BGA rect with aspect ratio preserved
    pub fn bga_rect(&self, info_area: &Rect) -> Rect {
        let available_height =
            info_area.height - self.header_height - self.bottom_panel_height - 20.0;
        let available_width = info_area.width - 20.0;

        let (width, height) = if available_width / available_height > self.bga_aspect_ratio {
            (available_height * self.bga_aspect_ratio, available_height)
        } else {
            (available_width, available_width / self.bga_aspect_ratio)
        };

        let x = info_area.x + (info_area.width - width) / 2.0;
        let y = info_area.y + self.header_height + 10.0;

        Rect::new(x, y, width, height)
    }

    /// Calculate judge stats rect
    pub fn judge_stats_rect(&self, info_area: &Rect) -> Rect {
        let bottom_y = info_area.y + info_area.height - self.bottom_panel_height;
        let width = info_area.width * self.judge_stats_width_ratio;
        Rect::new(
            info_area.x + 10.0,
            bottom_y,
            width - 20.0,
            self.bottom_panel_height,
        )
    }

    /// Calculate BPM display rect
    pub fn bpm_rect(&self, info_area: &Rect) -> Rect {
        let bottom_y = info_area.y + info_area.height - self.bottom_panel_height;
        let stats_width = info_area.width * self.judge_stats_width_ratio;
        let bpm_x = info_area.x + stats_width;
        let bpm_width = info_area.width - stats_width - 10.0;
        Rect::new(bpm_x, bottom_y, bpm_width, self.bottom_panel_height)
    }
}
