// Result / Decide / CourseResult state skin configurations.
//
// Placeholder structs for state-specific skin data. Fields will be added
// as rendering logic is ported in later phases.

/// Result state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct ResultSkinConfig {}

/// Decide state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct DecideSkinConfig {}

/// Course result state skin configuration.
#[derive(Debug, Clone, Default)]
pub struct CourseResultSkinConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let _result = ResultSkinConfig::default();
        let _decide = DecideSkinConfig::default();
        let _course = CourseResultSkinConfig::default();
    }
}
