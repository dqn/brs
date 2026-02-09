// SkinBpmGraph ported from SkinBPMGraph.java.
//
// Displays BPM changes over time as a logarithmic graph. The actual graph
// rendering is deferred to Phase 10. This module stores configuration only.

use crate::skin_object::{Color, SkinObjectBase};

// ---------------------------------------------------------------------------
// SkinBpmGraph
// ---------------------------------------------------------------------------

/// BPM graph color configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct BpmGraphColors {
    /// Main BPM line color (default: green #00ff00).
    pub main_bpm: Color,
    /// Minimum BPM line color (default: blue #0000ff).
    pub min_bpm: Color,
    /// Maximum BPM line color (default: red #ff0000).
    pub max_bpm: Color,
    /// Other BPM line color (default: yellow #ffff00).
    pub other_bpm: Color,
    /// Stop line color (default: purple #ff00ff).
    pub stop: Color,
    /// Transition line color (default: gray #7f7f7f).
    pub transition: Color,
}

impl Default for BpmGraphColors {
    fn default() -> Self {
        Self {
            main_bpm: Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            min_bpm: Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
            max_bpm: Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            other_bpm: Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            stop: Color {
                r: 1.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
            transition: Color {
                r: 0.498,
                g: 0.498,
                b: 0.498,
                a: 1.0,
            },
        }
    }
}

/// A skin BPM graph object that displays BPM changes over time.
#[derive(Debug, Clone)]
pub struct SkinBpmGraph {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Animation delay before graph is fully drawn (ms).
    pub delay: i32,
    /// Line thickness in pixels.
    pub line_width: i32,
    /// Color configuration.
    pub colors: BpmGraphColors,
}

impl Default for SkinBpmGraph {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            delay: 0,
            line_width: 2,
            colors: BpmGraphColors::default(),
        }
    }
}

impl SkinBpmGraph {
    pub fn new(delay: i32, line_width: i32, colors: BpmGraphColors) -> Self {
        Self {
            delay: delay.max(0),
            line_width: line_width.max(1),
            colors,
            ..Default::default()
        }
    }
}

/// Parses a hex color string, stripping non-hex characters and
/// clamping to 6 chars. Returns None if the result is empty.
pub fn parse_hex_color(s: &str) -> Option<Color> {
    let hex: String = s
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(6)
        .collect();
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let graph = SkinBpmGraph::default();
        assert_eq!(graph.delay, 0);
        assert_eq!(graph.line_width, 2);
    }

    #[test]
    fn test_new_clamps() {
        let graph = SkinBpmGraph::new(-10, 0, BpmGraphColors::default());
        assert_eq!(graph.delay, 0);
        assert_eq!(graph.line_width, 1);
    }

    #[test]
    fn test_parse_hex_color() {
        let c = parse_hex_color("ff8000").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.502).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_parse_hex_color_with_junk() {
        // Non-hex chars are stripped
        let c = parse_hex_color("#FF-00-FF").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_color_too_short() {
        assert!(parse_hex_color("ff").is_none());
    }

    #[test]
    fn test_default_colors() {
        let colors = BpmGraphColors::default();
        assert!((colors.main_bpm.g - 1.0).abs() < 0.01); // green
        assert!((colors.min_bpm.b - 1.0).abs() < 0.01); // blue
        assert!((colors.max_bpm.r - 1.0).abs() < 0.01); // red
    }
}
