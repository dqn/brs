// SkinDistributionGraph ported from SkinDistributionGraph.java.
//
// Displays distribution graphs for lamp or rank statistics.

use crate::skin_object::SkinObjectBase;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const LAMP_DISTRIBUTION_COUNT: usize = 11;
pub const RANK_DISTRIBUTION_COUNT: usize = 28;

// ---------------------------------------------------------------------------
// SkinDistributionGraph
// ---------------------------------------------------------------------------

/// Distribution graph object for music select.
#[derive(Debug, Clone, Default)]
pub struct SkinDistributionGraph {
    pub base: SkinObjectBase,
    /// 0 = lamp distribution (11 entries), 1 = rank distribution (28 entries).
    pub graph_type: i32,
    /// Image sources for each distribution entry.
    pub images: Vec<Option<i32>>,
}

impl SkinDistributionGraph {
    /// Creates a lamp distribution graph with 11 entry slots.
    pub fn lamp() -> Self {
        Self {
            graph_type: 0,
            images: vec![None; LAMP_DISTRIBUTION_COUNT],
            ..Default::default()
        }
    }

    /// Creates a rank distribution graph with 28 entry slots.
    pub fn rank() -> Self {
        Self {
            graph_type: 1,
            images: vec![None; RANK_DISTRIBUTION_COUNT],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let graph = SkinDistributionGraph::default();
        assert_eq!(graph.graph_type, 0);
        assert!(graph.images.is_empty());
    }

    #[test]
    fn test_lamp_distribution() {
        let graph = SkinDistributionGraph::lamp();
        assert_eq!(graph.graph_type, 0);
        assert_eq!(graph.images.len(), LAMP_DISTRIBUTION_COUNT);
    }

    #[test]
    fn test_rank_distribution() {
        let graph = SkinDistributionGraph::rank();
        assert_eq!(graph.graph_type, 1);
        assert_eq!(graph.images.len(), RANK_DISTRIBUTION_COUNT);
    }
}
