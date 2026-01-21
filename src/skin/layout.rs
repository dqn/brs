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
    #[serde(default)]
    pub bga: BgaLayout,
    /// Score display settings
    #[serde(default)]
    pub score: ScoreLayout,
    /// Gauge display settings
    #[serde(default)]
    pub gauge: GaugeLayout,
    /// FAST/SLOW indicator settings
    #[serde(default)]
    pub timing: TimingLayout,
    /// Judge effect settings
    #[serde(default)]
    pub judge: JudgeLayout,
    /// Combo display settings
    #[serde(default)]
    pub combo: ComboLayout,
    /// Song info settings
    #[serde(default)]
    pub song_info: SongInfoLayout,
    /// Green number settings
    #[serde(default)]
    pub green_number: GreenNumberLayout,
    /// IIDX-style layout settings
    #[serde(default)]
    pub iidx: IidxLayoutConfig,
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
#[serde(default)]
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
#[serde(default)]
pub struct PlayAreaLayout {
    /// Turntable width ratio relative to play area width (default: 0.2)
    pub turntable_ratio: f32,
    /// Highway area height ratio (default: 0.75)
    pub highway_height_ratio: f32,
    /// Gauge bar height in pixels
    pub gauge_height: f32,
    /// Bottom margin for score display
    pub score_area_height: f32,
    /// Progress bar width in pixels
    pub progress_bar_width: f32,
    /// Progress bar horizontal offset from play area left edge
    pub progress_bar_offset_x: f32,
    /// Keyboard key horizontal padding
    pub key_padding_x: f32,
    /// Keyboard key vertical padding
    pub key_padding_y: f32,
    /// Judge/combo effect position layout
    #[serde(default)]
    pub effects: PlayEffectsLayout,
    /// Gauge display layout
    #[serde(default)]
    pub gauge_display: GaugeDisplayLayout,
    /// Score/hi-speed display layout
    #[serde(default)]
    pub score_display: ScoreAreaLayout,
}

impl Default for PlayAreaLayout {
    fn default() -> Self {
        Self {
            turntable_ratio: 0.2,
            highway_height_ratio: 0.75,
            gauge_height: 20.0,
            score_area_height: 50.0,
            progress_bar_width: 8.0,
            progress_bar_offset_x: 0.0,
            key_padding_x: 2.0,
            key_padding_y: 5.0,
            effects: PlayEffectsLayout::default(),
            gauge_display: GaugeDisplayLayout::default(),
            score_display: ScoreAreaLayout::default(),
        }
    }
}

/// Judge/combo effect position layout
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayEffectsLayout {
    /// Offset from highway center (X) and judge line (Y)
    pub offset: Point,
}

impl Default for PlayEffectsLayout {
    fn default() -> Self {
        Self {
            offset: Point::new(0.0, -120.0),
        }
    }
}

/// Gauge display layout
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct GaugeDisplayLayout {
    /// Label position relative to gauge rect
    pub label_position: Point,
    /// Label font size
    pub label_font_size: f32,
    /// Percentage right margin from gauge rect right
    pub value_right_margin: f32,
    /// Percentage bottom margin from gauge rect bottom
    pub value_bottom_margin: f32,
    /// Percentage font size
    pub value_font_size: f32,
}

impl Default for GaugeDisplayLayout {
    fn default() -> Self {
        Self {
            label_position: Point::new(5.0, -2.0),
            label_font_size: 12.0,
            value_right_margin: 30.0,
            value_bottom_margin: 4.0,
            value_font_size: 14.0,
        }
    }
}

/// Score/hi-speed display layout
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct ScoreAreaLayout {
    /// Score label position
    pub score_label_position: Point,
    /// Score value position
    pub score_value_position: Point,
    /// Score label font size
    pub score_label_font_size: f32,
    /// Score value font size
    pub score_value_font_size: f32,
    /// Hi-speed label position
    pub hispeed_label_offset: Point,
    /// Hi-speed value position
    pub hispeed_value_offset: Point,
    /// Hi-speed label font size
    pub hispeed_label_font_size: f32,
    /// Hi-speed value font size
    pub hispeed_value_font_size: f32,
}

impl Default for ScoreAreaLayout {
    fn default() -> Self {
        Self {
            score_label_position: Point::new(10.0, 20.0),
            score_value_position: Point::new(10.0, 40.0),
            score_label_font_size: 14.0,
            score_value_font_size: 24.0,
            hispeed_label_offset: Point::new(0.0, 20.0),
            hispeed_value_offset: Point::new(0.0, 40.0),
            hispeed_label_font_size: 14.0,
            hispeed_value_font_size: 24.0,
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
#[serde(default)]
pub struct InfoAreaLayout {
    /// Header height for song info
    pub header_height: f32,
    /// Header layout
    #[serde(default)]
    pub header_layout: InfoHeaderLayout,
    /// BGA aspect ratio (width/height, default: 4/3)
    pub bga_aspect_ratio: f32,
    /// Horizontal margin for BGA area
    pub bga_margin_x: f32,
    /// Top margin for BGA area (below header)
    pub bga_margin_top: f32,
    /// Bottom margin for BGA area (above bottom panel)
    pub bga_margin_bottom: f32,
    /// Judge stats width ratio relative to info area width
    pub judge_stats_width_ratio: f32,
    /// Bottom panel height for judge stats and BPM
    pub bottom_panel_height: f32,
    /// Bottom panel horizontal padding
    pub bottom_panel_padding: f32,
    /// Gap between judge stats and BPM panels
    pub bottom_panel_gap: f32,
    /// Bottom panel border thickness
    pub bottom_panel_border_thickness: f32,
    /// Bottom panel divider thickness (0.0 to disable)
    pub bottom_panel_divider_thickness: f32,
    /// Divider X offset from judge stats right edge
    pub bottom_panel_divider_offset_x: f32,
    /// Divider vertical padding from panel top/bottom
    pub bottom_panel_divider_padding_y: f32,
    /// Judge stats layout
    #[serde(default)]
    pub judge_stats: JudgeStatsLayout,
    /// BPM display layout
    #[serde(default)]
    pub bpm: BpmDisplayLayout,
}

impl Default for InfoAreaLayout {
    fn default() -> Self {
        Self {
            header_height: 80.0,
            header_layout: InfoHeaderLayout::default(),
            bga_aspect_ratio: 4.0 / 3.0,
            bga_margin_x: 10.0,
            bga_margin_top: 10.0,
            bga_margin_bottom: 10.0,
            judge_stats_width_ratio: 0.55,
            bottom_panel_height: 180.0,
            bottom_panel_padding: 10.0,
            bottom_panel_gap: 10.0,
            bottom_panel_border_thickness: 1.0,
            bottom_panel_divider_thickness: 1.0,
            bottom_panel_divider_offset_x: 5.0,
            bottom_panel_divider_padding_y: 6.0,
            judge_stats: JudgeStatsLayout::default(),
            bpm: BpmDisplayLayout::default(),
        }
    }
}

/// Song info header layout
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct InfoHeaderLayout {
    /// Difficulty badge position (relative to header rect)
    pub badge_position: Point,
    /// Difficulty badge size
    pub badge_size: Point,
    /// Difficulty badge text offset from badge top-left
    pub badge_text_offset: Point,
    /// Difficulty badge font size
    pub badge_font_size: f32,
    /// Title position (relative to header rect)
    pub title_position: Point,
    /// Title font size
    pub title_font_size: f32,
    /// Artist position (relative to header rect)
    pub artist_position: Point,
    /// Artist font size
    pub artist_font_size: f32,
}

impl Default for InfoHeaderLayout {
    fn default() -> Self {
        Self {
            badge_position: Point::new(10.0, 15.0),
            badge_size: Point::new(80.0, 25.0),
            badge_text_offset: Point::new(5.0, 20.0),
            badge_font_size: 16.0,
            title_position: Point::new(100.0, 25.0),
            title_font_size: 18.0,
            artist_position: Point::new(100.0, 50.0),
            artist_font_size: 14.0,
        }
    }
}

/// Judge stats layout configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct JudgeStatsLayout {
    /// Header baseline offset from rect top
    pub header_y: f32,
    /// Header font size
    pub header_font_size: f32,
    /// Label offset from rect left
    pub label_x: f32,
    /// Value offset from rect left
    pub value_x: f32,
    /// First item baseline offset from rect top
    pub item_start_y: f32,
    /// Line height for items
    pub item_line_height: f32,
    /// Item font size
    pub item_font_size: f32,
    /// FAST/SLOW baseline offset from rect top
    pub fast_slow_y: f32,
    /// FAST label offset from rect left
    pub fast_label_x: f32,
    /// SLOW label right margin from rect right
    pub slow_right_margin: f32,
    /// FAST/SLOW font size
    pub fast_slow_font_size: f32,
}

impl Default for JudgeStatsLayout {
    fn default() -> Self {
        Self {
            header_y: 20.0,
            header_font_size: 16.0,
            label_x: 10.0,
            value_x: 60.0,
            item_start_y: 45.0,
            item_line_height: 22.0,
            item_font_size: 16.0,
            fast_slow_y: 45.0 + 6.0 * 22.0 + 10.0,
            fast_label_x: 10.0,
            slow_right_margin: 80.0,
            fast_slow_font_size: 14.0,
        }
    }
}

/// BPM display layout configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct BpmDisplayLayout {
    /// Current BPM baseline offset from rect top
    pub current_y: f32,
    /// Current BPM font size
    pub current_font_size: f32,
    /// Current BPM horizontal offset from rect center
    pub current_center_offset_x: f32,
    /// Min BPM baseline offset from rect top
    pub min_y: f32,
    /// Min BPM font size
    pub min_font_size: f32,
    /// Min BPM offset from rect left
    pub min_x: f32,
    /// Max BPM baseline offset from rect top
    pub max_y: f32,
    /// Max BPM font size
    pub max_font_size: f32,
    /// Max BPM right margin from rect right
    pub max_right_margin: f32,
    /// MIN/MAX label baseline offset from rect top
    pub min_max_label_y: f32,
    /// MIN/MAX label font size
    pub label_font_size: f32,
    /// BPM label baseline offset from rect top
    pub bpm_label_y: f32,
    /// BPM label font size
    pub bpm_label_font_size: f32,
    /// BPM label horizontal offset from rect center
    pub bpm_label_center_offset_x: f32,
}

impl Default for BpmDisplayLayout {
    fn default() -> Self {
        Self {
            current_y: 65.0,
            current_font_size: 48.0,
            current_center_offset_x: 0.0,
            min_y: 65.0,
            min_font_size: 20.0,
            min_x: 10.0,
            max_y: 65.0,
            max_font_size: 20.0,
            max_right_margin: 50.0,
            min_max_label_y: 85.0,
            label_font_size: 12.0,
            bpm_label_y: 110.0,
            bpm_label_font_size: 14.0,
            bpm_label_center_offset_x: 0.0,
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
        let available_height = info_area.height
            - self.header_height
            - self.bottom_panel_height
            - self.bga_margin_top
            - self.bga_margin_bottom;
        let available_width = info_area.width - self.bga_margin_x * 2.0;

        let (width, height) = if available_width / available_height > self.bga_aspect_ratio {
            (available_height * self.bga_aspect_ratio, available_height)
        } else {
            (available_width, available_width / self.bga_aspect_ratio)
        };

        let x = info_area.x + (info_area.width - width) / 2.0;
        let y = info_area.y + self.header_height + self.bga_margin_top;

        Rect::new(x, y, width, height)
    }

    /// Calculate judge stats rect
    pub fn judge_stats_rect(&self, info_area: &Rect) -> Rect {
        let bottom_y = info_area.y + info_area.height - self.bottom_panel_height;
        let width = info_area.width * self.judge_stats_width_ratio;
        Rect::new(
            info_area.x + self.bottom_panel_padding,
            bottom_y,
            width - self.bottom_panel_padding,
            self.bottom_panel_height,
        )
    }

    /// Calculate BPM display rect
    pub fn bpm_rect(&self, info_area: &Rect) -> Rect {
        let bottom_y = info_area.y + info_area.height - self.bottom_panel_height;
        let stats_width = info_area.width * self.judge_stats_width_ratio;
        let bpm_x = info_area.x + stats_width + self.bottom_panel_gap;
        let bpm_width =
            info_area.width - stats_width - self.bottom_panel_gap - self.bottom_panel_padding;
        Rect::new(bpm_x, bottom_y, bpm_width, self.bottom_panel_height)
    }
}

/// Graph area layout configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphAreaLayout {
    /// Score graph height ratio relative to the graph area height
    pub score_graph_height_ratio: f32,
    /// Header bar height
    pub header_height: f32,
    /// Header text X offset from graph area
    pub header_text_x: f32,
    /// Header text baseline offset from top of graph area
    pub header_text_y: f32,
    /// Header text font size
    pub header_text_font_size: f32,
    /// Graph padding inside the area
    pub graph_padding: f32,
    /// Grade label X offset from graph area
    pub grade_label_x: f32,
    /// Grade label Y offset from grade line
    pub grade_label_offset_y: f32,
    /// Grade label font size
    pub grade_label_font_size: f32,
    /// Grade line thickness
    pub grade_line_thickness: f32,
    /// Current score bar width ratio (relative to graph width)
    pub bar_width_ratio: f32,
    /// Current score bar X ratio (relative to graph width)
    pub bar_x_ratio: f32,
    /// Target score bar width ratio (relative to graph width)
    pub target_bar_width_ratio: f32,
    /// Target score bar X ratio (relative to graph width)
    pub target_bar_x_ratio: f32,
    /// Score label position X (relative to graph area)
    pub score_label_x: f32,
    /// Score label baseline Y (relative to graph area)
    pub score_label_y: f32,
    /// Score value baseline Y (relative to graph area)
    pub score_value_y: f32,
    /// Score line gap for stacked labels
    pub score_line_gap: f32,
    /// Score label font size
    pub score_label_font_size: f32,
    /// Score value font size
    pub score_value_font_size: f32,
    /// Right margin for score value alignment
    pub score_value_right_margin: f32,
    /// Option text position (relative to graph area origin)
    pub option_position: Point,
    /// Green number text position (relative to graph area origin)
    pub green_number_position: Point,
    /// Option text font size
    pub option_font_size: f32,
    /// Green number font size
    pub green_number_font_size: f32,
}

impl Default for GraphAreaLayout {
    fn default() -> Self {
        Self {
            score_graph_height_ratio: 0.7,
            header_height: 40.0,
            header_text_x: 10.0,
            header_text_y: 25.0,
            header_text_font_size: 16.0,
            graph_padding: 20.0,
            grade_label_x: 2.0,
            grade_label_offset_y: -2.0,
            grade_label_font_size: 12.0,
            grade_line_thickness: 1.0,
            bar_width_ratio: 0.3,
            bar_x_ratio: 0.35,
            target_bar_width_ratio: 0.24,
            target_bar_x_ratio: 0.65,
            score_label_x: 15.0,
            score_label_y: 0.0,
            score_value_y: 0.0,
            score_line_gap: 25.0,
            score_label_font_size: 14.0,
            score_value_font_size: 20.0,
            score_value_right_margin: 80.0,
            option_position: Point::new(10.0, 810.0),
            green_number_position: Point::new(10.0, 835.0),
            option_font_size: 16.0,
            green_number_font_size: 14.0,
        }
    }
}

impl GraphAreaLayout {
    pub fn score_graph_rect(&self, area: &Rect) -> Rect {
        Rect::new(
            area.x,
            area.y,
            area.width,
            area.height * self.score_graph_height_ratio,
        )
    }

    pub fn resolve_position(&self, area: &Rect, point: Point) -> Point {
        Point {
            x: if point.x < 0.0 {
                area.x + area.width + point.x
            } else {
                area.x + point.x
            },
            y: if point.y < 0.0 {
                area.y + area.height + point.y
            } else {
                area.y + point.y
            },
        }
    }
}

/// IIDX layout configuration for skins
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct IidxLayoutConfig {
    pub screen: IidxLayout,
    pub play: PlayAreaLayout,
    pub graph: GraphAreaLayout,
    pub info: InfoAreaLayout,
}
