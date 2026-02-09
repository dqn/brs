// Skin visualizer types ported from Java:
// - SkinHitErrorVisualizer.java
// - SkinNoteDistributionGraph.java
// - SkinTimingDistributionGraph.java
// - SkinTimingVisualizer.java
//
// These are specialized objects for gameplay feedback visualization.
// Rendering logic is deferred to Phase 10. This module stores configuration.

use crate::skin_object::{Color, SkinObjectBase};

// ---------------------------------------------------------------------------
// Common
// ---------------------------------------------------------------------------

/// Validates a hex color string. Returns the validated string, or a default
/// opaque red ("FF0000FF") if the input is invalid (non-hex chars or < 6 chars).
pub fn validate_color_string(cs: &str) -> String {
    let hex_only: String = cs.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex_only.len() == cs.len() && cs.len() >= 6 {
        cs.to_string()
    } else {
        "FF0000FF".to_string()
    }
}

/// Parses a validated hex color string (6 or 8 chars) into a Color.
pub fn parse_color(hex: &str) -> Color {
    let validated = validate_color_string(hex);
    let bytes: Vec<u8> = (0..validated.len())
        .step_by(2)
        .filter_map(|i| {
            if i + 2 <= validated.len() {
                u8::from_str_radix(&validated[i..i + 2], 16).ok()
            } else {
                None
            }
        })
        .collect();
    match bytes.len() {
        4 => Color {
            r: bytes[0] as f32 / 255.0,
            g: bytes[1] as f32 / 255.0,
            b: bytes[2] as f32 / 255.0,
            a: bytes[3] as f32 / 255.0,
        },
        3 => Color {
            r: bytes[0] as f32 / 255.0,
            g: bytes[1] as f32 / 255.0,
            b: bytes[2] as f32 / 255.0,
            a: 1.0,
        },
        _ => Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    }
}

// ---------------------------------------------------------------------------
// SkinHitErrorVisualizer
// ---------------------------------------------------------------------------

/// EMA (Exponential Moving Average) rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmaMode {
    /// No EMA display.
    #[default]
    None = 0,
    /// EMA shown as a vertical line.
    Line = 1,
    /// EMA shown as a triangle.
    Triangle = 2,
    /// Both line and triangle.
    Both = 3,
}

impl EmaMode {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Line,
            2 => Self::Triangle,
            3 => Self::Both,
            _ => Self::None,
        }
    }
}

/// Configuration for early/late hit error visualization.
#[derive(Debug, Clone)]
pub struct SkinHitErrorVisualizer {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Display width in pixels.
    pub width: i32,
    /// Judge width in milliseconds (half-range).
    pub judge_width_millis: i32,
    /// Line width in pixels (clamped 1-4).
    pub line_width: i32,
    /// Whether to show individual note hit errors.
    pub hiterror_mode: bool,
    /// Whether to color-code by judge result.
    pub color_mode: bool,
    /// EMA rendering mode.
    pub ema_mode: EmaMode,
    /// EMA smoothing factor (alpha).
    pub ema_alpha: f32,
    /// Number of recent judgments to display (max 100).
    pub window_length: i32,
    /// Whether to shorten older error lines (decay effect).
    pub draw_decay: bool,
    /// Line color for hit error indicators.
    pub line_color: Color,
    /// Center line color.
    pub center_color: Color,
    /// EMA indicator color.
    pub ema_color: Color,
    /// Judge colors: [PG, GR, GD, BD, PR].
    pub judge_colors: [Color; 5],
}

impl Default for SkinHitErrorVisualizer {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            width: 100,
            judge_width_millis: 150,
            line_width: 2,
            hiterror_mode: true,
            color_mode: false,
            ema_mode: EmaMode::None,
            ema_alpha: 0.1,
            window_length: 30,
            draw_decay: false,
            line_color: Color::white(),
            center_color: Color::white(),
            ema_color: Color::white(),
            judge_colors: [Color::white(); 5],
        }
    }
}

// ---------------------------------------------------------------------------
// SkinNoteDistributionGraph
// ---------------------------------------------------------------------------

/// Note distribution graph type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteDistributionType {
    /// Normal: note type breakdown (7 cols: normal, LN head, scratch, LN body, etc.)
    #[default]
    Normal = 0,
    /// Judge: per-judge result counts (6 cols).
    Judge = 1,
    /// Early/Late: early/late split for each judge (10 cols).
    EarlyLate = 2,
}

impl NoteDistributionType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Judge,
            2 => Self::EarlyLate,
            _ => Self::Normal,
        }
    }

    /// Number of data columns for this graph type.
    pub fn data_length(self) -> usize {
        match self {
            Self::Normal => 7,
            Self::Judge => 6,
            Self::EarlyLate => 10,
        }
    }
}

/// Configuration for note distribution graph over time.
#[derive(Debug, Clone)]
pub struct SkinNoteDistributionGraph {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Graph type.
    pub graph_type: NoteDistributionType,
    /// Animation delay in milliseconds.
    pub delay: i32,
    /// Whether to hide the background grid.
    pub back_tex_off: bool,
    /// Whether to reverse the draw order of note types.
    pub order_reverse: bool,
    /// Whether to remove vertical gaps between cells.
    pub no_gap: bool,
    /// Whether to remove horizontal gaps between cells.
    pub no_gap_x: bool,
}

impl Default for SkinNoteDistributionGraph {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            graph_type: NoteDistributionType::Normal,
            delay: 500,
            back_tex_off: false,
            order_reverse: false,
            no_gap: false,
            no_gap_x: false,
        }
    }
}

impl SkinNoteDistributionGraph {
    pub fn new(graph_type: i32, delay: i32) -> Self {
        Self {
            graph_type: NoteDistributionType::from_i32(graph_type),
            delay,
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// SkinTimingDistributionGraph
// ---------------------------------------------------------------------------

/// Configuration for timing distribution histogram (shown on result screen).
#[derive(Debug, Clone)]
pub struct SkinTimingDistributionGraph {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Graph width divided by line width = number of histogram bins.
    pub graph_width: i32,
    /// Line width in pixels.
    pub line_width: i32,
    /// Whether to draw the average timing line.
    pub draw_average: bool,
    /// Whether to draw the standard deviation range.
    pub draw_dev: bool,
    /// Graph bar color.
    pub graph_color: Color,
    /// Average line color.
    pub average_color: Color,
    /// Deviation line color.
    pub dev_color: Color,
    /// Judge area colors: [PG, GR, GD, BD, PR].
    pub judge_colors: [Color; 5],
}

impl Default for SkinTimingDistributionGraph {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            graph_width: 100,
            line_width: 1,
            draw_average: false,
            draw_dev: false,
            graph_color: Color::white(),
            average_color: Color::white(),
            dev_color: Color::white(),
            judge_colors: [Color::white(); 5],
        }
    }
}

// ---------------------------------------------------------------------------
// SkinTimingVisualizer
// ---------------------------------------------------------------------------

/// Configuration for real-time judge timing feedback during play.
#[derive(Debug, Clone)]
pub struct SkinTimingVisualizer {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Display width in pixels.
    pub width: i32,
    /// Judge width in milliseconds (half-range).
    pub judge_width_millis: i32,
    /// Line width in pixels (clamped 1-4).
    pub line_width: i32,
    /// Whether to shorten older timing lines (decay effect).
    pub draw_decay: bool,
    /// Line color for timing indicators.
    pub line_color: Color,
    /// Center line color.
    pub center_color: Color,
    /// Judge area colors: [PG, GR, GD, BD, PR].
    pub judge_colors: [Color; 5],
}

impl Default for SkinTimingVisualizer {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            width: 100,
            judge_width_millis: 150,
            line_width: 2,
            draw_decay: false,
            line_color: Color::white(),
            center_color: Color::white(),
            judge_colors: [Color::white(); 5],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_color_string_valid() {
        assert_eq!(validate_color_string("ff0000"), "ff0000");
        assert_eq!(validate_color_string("FF00FFAA"), "FF00FFAA");
    }

    #[test]
    fn test_validate_color_string_invalid() {
        assert_eq!(validate_color_string("gg0000"), "FF0000FF");
        assert_eq!(validate_color_string("ff"), "FF0000FF");
        assert_eq!(validate_color_string("#ff0000"), "FF0000FF");
    }

    #[test]
    fn test_parse_color_rgb() {
        let c = parse_color("00FF00");
        assert!((c.r - 0.0).abs() < 0.01);
        assert!((c.g - 1.0).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_parse_color_rgba() {
        let c = parse_color("FF000080");
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.a - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_ema_mode() {
        assert_eq!(EmaMode::from_i32(0), EmaMode::None);
        assert_eq!(EmaMode::from_i32(1), EmaMode::Line);
        assert_eq!(EmaMode::from_i32(2), EmaMode::Triangle);
        assert_eq!(EmaMode::from_i32(3), EmaMode::Both);
        assert_eq!(EmaMode::from_i32(99), EmaMode::None);
    }

    #[test]
    fn test_note_distribution_type() {
        assert_eq!(
            NoteDistributionType::from_i32(0),
            NoteDistributionType::Normal
        );
        assert_eq!(
            NoteDistributionType::from_i32(1),
            NoteDistributionType::Judge
        );
        assert_eq!(
            NoteDistributionType::from_i32(2),
            NoteDistributionType::EarlyLate
        );
        assert_eq!(NoteDistributionType::Normal.data_length(), 7);
        assert_eq!(NoteDistributionType::Judge.data_length(), 6);
        assert_eq!(NoteDistributionType::EarlyLate.data_length(), 10);
    }

    #[test]
    fn test_hit_error_default() {
        let v = SkinHitErrorVisualizer::default();
        assert_eq!(v.width, 100);
        assert_eq!(v.line_width, 2);
        assert!(v.hiterror_mode);
        assert!(!v.color_mode);
    }

    #[test]
    fn test_note_distribution_new() {
        let g = SkinNoteDistributionGraph::new(2, 1000);
        assert_eq!(g.graph_type, NoteDistributionType::EarlyLate);
        assert_eq!(g.delay, 1000);
    }

    #[test]
    fn test_timing_distribution_default() {
        let g = SkinTimingDistributionGraph::default();
        assert_eq!(g.graph_width, 100);
        assert!(!g.draw_average);
        assert!(!g.draw_dev);
    }

    #[test]
    fn test_timing_visualizer_default() {
        let v = SkinTimingVisualizer::default();
        assert_eq!(v.width, 100);
        assert_eq!(v.judge_width_millis, 150);
        assert!(!v.draw_decay);
    }
}
