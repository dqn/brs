// Result / Decide / CourseResult state skin configurations.
//
// Contains state-specific skin data for result screens including
// gauge graphs, note distribution, BPM, and timing graphs.

use crate::skin_bpm_graph::SkinBpmGraph;
use crate::skin_gauge_graph::SkinGaugeGraph;
use crate::skin_visualizer::{SkinNoteDistributionGraph, SkinTimingDistributionGraph};

/// Result state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct ResultSkinConfig {
    /// Gauge history transition graph.
    pub gauge_graph: Option<SkinGaugeGraph>,
    /// Note distribution graph.
    pub note_graph: Option<SkinNoteDistributionGraph>,
    /// BPM timeline graph.
    pub bpm_graph: Option<SkinBpmGraph>,
    /// Timing distribution graph.
    pub timing_graph: Option<SkinTimingDistributionGraph>,
}

/// Decide state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct DecideSkinConfig {}

/// Course result state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct CourseResultSkinConfig {
    /// Gauge history transition graph.
    pub gauge_graph: Option<SkinGaugeGraph>,
    /// Note distribution graph.
    pub note_graph: Option<SkinNoteDistributionGraph>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let result = ResultSkinConfig::default();
        assert!(result.gauge_graph.is_none());
        assert!(result.note_graph.is_none());
        assert!(result.bpm_graph.is_none());
        assert!(result.timing_graph.is_none());

        let _decide = DecideSkinConfig::default();

        let course = CourseResultSkinConfig::default();
        assert!(course.gauge_graph.is_none());
        assert!(course.note_graph.is_none());
    }
}
