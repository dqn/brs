// Music select state skin configuration.
//
// Aggregates select-specific skin objects (song bar, distribution graph).

use crate::skin_bar::SkinBar;
use crate::skin_distribution_graph::SkinDistributionGraph;

/// Music select state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct MusicSelectSkinConfig {
    pub bar: Option<SkinBar>,
    pub distribution_graph: Option<SkinDistributionGraph>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = MusicSelectSkinConfig::default();
        assert!(config.bar.is_none());
        assert!(config.distribution_graph.is_none());
    }

    #[test]
    fn test_with_bar() {
        let config = MusicSelectSkinConfig {
            bar: Some(SkinBar::default()),
            ..Default::default()
        };
        assert!(config.bar.is_some());
    }
}
