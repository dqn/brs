// SkinGaugeGraph ported from SkinGaugeGraphObject.java.
//
// Displays gauge history transitions as a graph on result/course result screens.
// 6 gauge types × 4 colors per type (below-border BG, below-border line,
// above-border BG, above-border line).

use crate::skin_object::{Color, SkinObjectBase};

// ---------------------------------------------------------------------------
// Gauge type constants
// ---------------------------------------------------------------------------

/// Number of gauge color types.
pub const GAUGE_TYPE_COUNT: usize = 6;

/// Maps the 10 internal gauge types to the 6 color categories.
/// Index: gauge type ID (0-9). Value: color category (0-5).
pub const GAUGE_TYPE_TABLE: [usize; 10] = [0, 1, 2, 3, 4, 5, 3, 4, 5, 3];

// ---------------------------------------------------------------------------
// GaugeGraphColors
// ---------------------------------------------------------------------------

/// Colors for a single gauge type.
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeTypeColors {
    /// Line color above the border.
    pub border_line: Color,
    /// Background color above the border.
    pub border_bg: Color,
    /// Line color below the border.
    pub graph_line: Color,
    /// Background color below the border.
    pub graph_bg: Color,
}

impl Default for GaugeTypeColors {
    fn default() -> Self {
        Self {
            border_line: hex_color("000000"),
            border_bg: hex_color("000000"),
            graph_line: hex_color("000000"),
            graph_bg: hex_color("000000"),
        }
    }
}

/// All gauge graph colors (6 gauge types).
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeGraphColors {
    pub types: [GaugeTypeColors; GAUGE_TYPE_COUNT],
}

impl Default for GaugeGraphColors {
    fn default() -> Self {
        // Default colors matching Java SkinGaugeGraphObject default constructor.
        //
        // Type 0 (Assist Clear): border=red/dark_red, graph=magenta/dark_magenta
        // Type 1 (Assist & Easy Fail): border=red/dark_red, graph=cyan/dark_cyan
        // Type 2 (Groove Fail): border=red/dark_red, graph=green/dark_green
        // Type 3 (Groove Clear & Hard): border=red/dark_red, graph=black/black
        // Type 4 (Ex Hard): border=yellow/dark_yellow, graph=black/black
        // Type 5 (Hazard): border=gray/dark_gray, graph=black/black
        Self {
            types: [
                GaugeTypeColors {
                    border_line: hex_color("ff0000"),
                    border_bg: hex_color("440000"),
                    graph_line: hex_color("ff00ff"),
                    graph_bg: hex_color("440044"),
                },
                GaugeTypeColors {
                    border_line: hex_color("ff0000"),
                    border_bg: hex_color("440000"),
                    graph_line: hex_color("00ffff"),
                    graph_bg: hex_color("004444"),
                },
                GaugeTypeColors {
                    border_line: hex_color("ff0000"),
                    border_bg: hex_color("440000"),
                    graph_line: hex_color("00ff00"),
                    graph_bg: hex_color("004400"),
                },
                GaugeTypeColors {
                    border_line: hex_color("ff0000"),
                    border_bg: hex_color("440000"),
                    graph_line: hex_color("000000"),
                    graph_bg: hex_color("000000"),
                },
                GaugeTypeColors {
                    border_line: hex_color("ffff00"),
                    border_bg: hex_color("444400"),
                    graph_line: hex_color("000000"),
                    graph_bg: hex_color("000000"),
                },
                GaugeTypeColors {
                    border_line: hex_color("cccccc"),
                    border_bg: hex_color("444444"),
                    graph_line: hex_color("000000"),
                    graph_bg: hex_color("000000"),
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// SkinGaugeGraph
// ---------------------------------------------------------------------------

/// A skin gauge graph object that displays gauge history transitions.
#[derive(Debug, Clone)]
pub struct SkinGaugeGraph {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Animation delay before graph is fully drawn (ms).
    pub delay: i32,
    /// Line thickness in pixels.
    pub line_width: i32,
    /// Color configuration for each gauge type.
    pub colors: GaugeGraphColors,
}

impl Default for SkinGaugeGraph {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            delay: 1500,
            line_width: 2,
            colors: GaugeGraphColors::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parses a 6-char hex string to a Color. Panics on invalid input.
fn hex_color(s: &str) -> Color {
    let r = u8::from_str_radix(&s[0..2], 16).unwrap();
    let g = u8::from_str_radix(&s[2..4], 16).unwrap();
    let b = u8::from_str_radix(&s[4..6], 16).unwrap();
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let graph = SkinGaugeGraph::default();
        assert_eq!(graph.delay, 1500);
        assert_eq!(graph.line_width, 2);
    }

    #[test]
    fn test_gauge_type_table() {
        assert_eq!(GAUGE_TYPE_TABLE[0], 0);
        assert_eq!(GAUGE_TYPE_TABLE[3], 3);
        assert_eq!(GAUGE_TYPE_TABLE[6], 3); // maps to same as 3
        assert_eq!(GAUGE_TYPE_TABLE[9], 3);
    }

    #[test]
    fn test_default_colors_type0() {
        let colors = GaugeGraphColors::default();
        // Type 0 border line should be red
        assert!((colors.types[0].border_line.r - 1.0).abs() < 0.01);
        assert!((colors.types[0].border_line.g - 0.0).abs() < 0.01);
        // Type 0 graph line should be magenta
        assert!((colors.types[0].graph_line.r - 1.0).abs() < 0.01);
        assert!((colors.types[0].graph_line.b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_default_colors_type5() {
        let colors = GaugeGraphColors::default();
        // Type 5 (hazard) border line should be gray
        assert!((colors.types[5].border_line.r - 0.8).abs() < 0.01);
        assert!((colors.types[5].border_line.g - 0.8).abs() < 0.01);
        assert!((colors.types[5].border_line.b - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_hex_color() {
        let c = hex_color("ff8000");
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.502).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }
}
