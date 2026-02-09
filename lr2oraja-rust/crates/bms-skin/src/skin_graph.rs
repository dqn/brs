// SkinGraph ported from SkinGraph.java.
//
// Displays a simple progress bar/value graph. The source image is cropped
// based on the current float value (0.0-1.0).

use crate::image_handle::ImageHandle;
use crate::property_id::FloatId;
use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// SkinGraph
// ---------------------------------------------------------------------------

/// Growth direction for the graph bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphDirection {
    /// Bar grows from left to right (default).
    #[default]
    Right = 0,
    /// Bar grows from bottom to top.
    Up = 1,
}

impl GraphDirection {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Up,
            _ => Self::Right,
        }
    }
}

/// A skin graph object that displays a value as a cropped bar.
#[derive(Debug, Clone)]
pub struct SkinGraph {
    /// Base animation/destination properties.
    pub base: SkinObjectBase,
    /// Source image for the bar.
    pub source_images: Vec<ImageHandle>,
    pub source_timer: Option<i32>,
    pub source_cycle: i32,
    /// Alternatively, reference a global image by ID.
    pub source_image_id: Option<i32>,
    /// Float property ID for the value (0.0-1.0).
    pub ref_id: Option<FloatId>,
    /// Growth direction.
    pub direction: GraphDirection,
    /// Optional min/max range for RateProperty conversion.
    pub range_min: Option<i32>,
    pub range_max: Option<i32>,
}

impl Default for SkinGraph {
    fn default() -> Self {
        Self {
            base: SkinObjectBase::default(),
            source_images: Vec::new(),
            source_timer: None,
            source_cycle: 0,
            source_image_id: None,
            ref_id: None,
            direction: GraphDirection::Right,
            range_min: None,
            range_max: None,
        }
    }
}

impl SkinGraph {
    pub fn new(ref_id: FloatId, direction: i32) -> Self {
        Self {
            ref_id: Some(ref_id),
            direction: GraphDirection::from_i32(direction),
            ..Default::default()
        }
    }

    /// Creates a graph referencing a global image ID.
    pub fn from_image_id(image_id: i32, ref_id: FloatId, direction: i32) -> Self {
        Self {
            source_image_id: Some(image_id),
            ref_id: Some(ref_id),
            direction: GraphDirection::from_i32(direction),
            ..Default::default()
        }
    }

    /// Creates a graph with RateProperty conversion.
    pub fn with_rate(ref_id: FloatId, direction: i32, min: i32, max: i32) -> Self {
        Self {
            ref_id: Some(ref_id),
            direction: GraphDirection::from_i32(direction),
            range_min: Some(min),
            range_max: Some(max),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let graph = SkinGraph::default();
        assert_eq!(graph.direction, GraphDirection::Right);
        assert!(graph.ref_id.is_none());
    }

    #[test]
    fn test_new() {
        let graph = SkinGraph::new(FloatId(110), 1);
        assert_eq!(graph.ref_id, Some(FloatId(110)));
        assert_eq!(graph.direction, GraphDirection::Up);
    }

    #[test]
    fn test_from_image_id() {
        let graph = SkinGraph::from_image_id(5, FloatId(6), 0);
        assert_eq!(graph.source_image_id, Some(5));
        assert_eq!(graph.direction, GraphDirection::Right);
    }

    #[test]
    fn test_with_rate() {
        let graph = SkinGraph::with_rate(FloatId(71), 1, 0, 200000);
        assert_eq!(graph.range_min, Some(0));
        assert_eq!(graph.range_max, Some(200000));
    }

    #[test]
    fn test_graph_direction() {
        assert_eq!(GraphDirection::from_i32(0), GraphDirection::Right);
        assert_eq!(GraphDirection::from_i32(1), GraphDirection::Up);
        assert_eq!(GraphDirection::from_i32(99), GraphDirection::Right);
    }
}
