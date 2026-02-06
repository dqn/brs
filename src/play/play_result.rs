use super::clear_type::ClearType;
use super::gauge::gauge_property::GaugeType;
use super::gauge::groove_gauge::GrooveGauge;
use super::score::{ScoreData, ScoreRank};

/// Determine the clear type from gauge state and score data.
///
/// Logic from beatoraja:
/// - If gauge is not qualified -> Failed
/// - If score is max (all PG, full combo) -> Max
/// - If score is perfect (all PG) -> Perfect
/// - If score is full combo (no BD/PR/MS) -> FullCombo
/// - Otherwise -> based on gauge type mapping (from_gauge_type)
pub fn determine_clear_type(gauge: &GrooveGauge, score: &ScoreData) -> ClearType {
    if !gauge.is_qualified() {
        return ClearType::Failed;
    }

    if score.is_max() {
        return ClearType::Max;
    }
    if score.is_perfect() {
        return ClearType::Perfect;
    }
    if score.is_full_combo() {
        return ClearType::FullCombo;
    }

    ClearType::from_gauge_type(gauge.gauge_type())
}

/// Complete play result combining score and clear information.
#[derive(Debug, Clone)]
pub struct PlayResult {
    /// The final score data.
    pub score: ScoreData,
    /// The clear type achieved.
    pub clear_type: ClearType,
    /// The score rank.
    pub rank: ScoreRank,
    /// The final gauge value.
    pub gauge_value: f32,
    /// The gauge type used.
    pub gauge_type: GaugeType,
}

impl PlayResult {
    /// Create a PlayResult from gauge and score state.
    pub fn new(gauge: &GrooveGauge, score: ScoreData) -> Self {
        let clear_type = determine_clear_type(gauge, &score);
        let rank = score.rank();
        Self {
            score,
            clear_type,
            rank,
            gauge_value: gauge.value(),
            gauge_type: gauge.gauge_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::gauge::gauge_property::*;

    fn make_gauge(gauge_type: GaugeType) -> GrooveGauge {
        GrooveGauge::new(gauge_type, GaugePropertyType::SevenKeys, 300.0, 1000)
    }

    // =========================================================================
    // determine_clear_type tests
    // =========================================================================

    #[test]
    fn clear_type_failed_when_not_qualified() {
        let gauge = make_gauge(GAUGE_NORMAL);
        // Normal gauge starts at 20%, border is 80% -> not qualified
        let score = ScoreData::new(1000);
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Failed);
    }

    #[test]
    fn clear_type_max_when_all_pg_full_combo() {
        let mut gauge = make_gauge(GAUGE_NORMAL);
        gauge.set_value(100.0); // Ensure qualified
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 10; // All PG
        score.max_combo = 10; // Full combo
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Max);
    }

    #[test]
    fn clear_type_perfect_when_all_pg_but_combo_broken() {
        let mut gauge = make_gauge(GAUGE_NORMAL);
        gauge.set_value(100.0);
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 10;
        score.max_combo = 5; // Combo was broken
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Perfect);
    }

    #[test]
    fn clear_type_full_combo_with_gr() {
        let mut gauge = make_gauge(GAUGE_NORMAL);
        gauge.set_value(100.0);
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[1] = 5; // Some GR
        score.max_combo = 10;
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::FullCombo);
    }

    #[test]
    fn clear_type_normal_gauge() {
        let mut gauge = make_gauge(GAUGE_NORMAL);
        gauge.set_value(80.0); // Qualified (border = 80)
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5; // Some BD -> not full combo
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Normal);
    }

    #[test]
    fn clear_type_easy_gauge() {
        let mut gauge = make_gauge(GAUGE_EASY);
        gauge.set_value(80.0); // EASY border is 80
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5;
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Easy);
    }

    #[test]
    fn clear_type_hard_gauge() {
        // Hard gauge: starts at 100, border is 0 (any value > 0 qualifies)
        let gauge = make_gauge(GAUGE_HARD);
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5;
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Hard);
    }

    #[test]
    fn clear_type_exhard_gauge() {
        let gauge = make_gauge(GAUGE_EXHARD);
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5;
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::ExHard);
    }

    #[test]
    fn clear_type_assist_easy_gauge() {
        let mut gauge = make_gauge(GAUGE_ASSIST_EASY);
        gauge.set_value(60.0); // ASSIST_EASY border is 60
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5;
        assert_eq!(
            determine_clear_type(&gauge, &score),
            ClearType::LightAssistEasy
        );
    }

    #[test]
    fn clear_type_hazard_gauge() {
        // Hazard gauge: starts at 100, any miss = death
        let gauge = make_gauge(GAUGE_HAZARD);
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5;
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::FullCombo);
    }

    #[test]
    fn clear_type_class_gauge() {
        let gauge = make_gauge(GAUGE_CLASS);
        // CLASS gauge init is 100, border is 0
        let mut score = ScoreData::new(10);
        score.early_counts[0] = 5;
        score.early_counts[3] = 5;
        assert_eq!(determine_clear_type(&gauge, &score), ClearType::Normal);
    }

    // =========================================================================
    // PlayResult tests
    // =========================================================================

    #[test]
    fn play_result_new() {
        let mut gauge = make_gauge(GAUGE_NORMAL);
        gauge.set_value(90.0);
        let mut score = ScoreData::new(100);
        score.early_counts[0] = 80;
        score.early_counts[1] = 15;
        score.early_counts[3] = 5;
        score.max_combo = 50;

        let result = PlayResult::new(&gauge, score);
        assert_eq!(result.clear_type, ClearType::Normal);
        // exscore = 80*2 + 15 = 175, rate = 175/200 = 0.875
        // 0.875 * 27 = 23.625, >= 21 -> AAA
        assert_eq!(result.rank, ScoreRank::AAA);
        assert!((result.gauge_value - 90.0).abs() < 0.001);
        assert_eq!(result.gauge_type, GAUGE_NORMAL);
    }

    #[test]
    fn play_result_failed() {
        let gauge = make_gauge(GAUGE_NORMAL);
        let score = ScoreData::new(100);
        let result = PlayResult::new(&gauge, score);
        assert_eq!(result.clear_type, ClearType::Failed);
    }
}
