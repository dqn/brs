// SkinGauge — groove gauge skin object.
//
// Groove gauge display with 4-part or 6-part animation modes.
// This is constructed by the LR2 CSV loader's SRC_GROOVEGAUGE / DST_GROOVEGAUGE
// commands and by the JSON loader's gauge object.

use crate::image_handle::ImageHandle;
use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// SkinGauge
// ---------------------------------------------------------------------------

/// Gauge part type (front/back, active/inactive colors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugePartType {
    /// Active gauge (front, red/survival zone).
    FrontRed = 0,
    /// Active gauge (front, green/normal zone).
    FrontGreen = 1,
    /// Inactive gauge (back, red zone).
    BackRed = 2,
    /// Inactive gauge (back, green zone).
    BackGreen = 3,
    /// EX active (front, red, extended).
    ExFrontRed = 4,
    /// EX active (front, green, extended).
    ExFrontGreen = 5,
}

/// A single gauge part with animation frames.
#[derive(Debug, Clone)]
pub struct GaugePart {
    /// Part type identifier.
    pub part_type: GaugePartType,
    /// Animation frames (typically 6 frames for blinking).
    pub images: Vec<ImageHandle>,
    /// Animation timer ID.
    pub timer: Option<i32>,
    /// Animation cycle in milliseconds.
    pub cycle: i32,
}

/// A skin gauge object that displays groove gauge with animated parts.
#[derive(Debug, Clone)]
pub struct SkinGauge {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Gauge parts (4 for standard, 6 for EX mode).
    pub parts: Vec<GaugePart>,
    /// Number of gauge nodes (typically 50).
    pub nodes: i32,
}

impl Default for SkinGauge {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            parts: Vec::new(),
            nodes: 50,
        }
    }
}

impl SkinGauge {
    pub fn new(nodes: i32) -> Self {
        Self {
            nodes,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let gauge = SkinGauge::default();
        assert_eq!(gauge.nodes, 50);
        assert!(gauge.parts.is_empty());
    }

    #[test]
    fn test_new() {
        let gauge = SkinGauge::new(100);
        assert_eq!(gauge.nodes, 100);
    }
}
