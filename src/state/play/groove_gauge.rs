use crate::state::play::JudgeRank;

/// Type of groove gauge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeType {
    AssistEasy,
    Easy,
    Normal,
    Hard,
    ExHard,
    Hazard,
    /// Class gauge for Dan/Course mode (similar to Hard but with gauge carry-over).
    Class,
}

/// Gauge modification values for each judge rank.
/// Order: [PG, GR, GD, BD, PR, MS]
#[derive(Debug, Clone)]
pub struct GaugeModifier {
    values: [f64; 6],
}

impl GaugeModifier {
    pub fn new(pg: f64, gr: f64, gd: f64, bd: f64, pr: f64, ms: f64) -> Self {
        Self {
            values: [pg, gr, gd, bd, pr, ms],
        }
    }

    pub fn get(&self, rank: JudgeRank) -> f64 {
        self.values[rank.index()]
    }
}

/// Gauge element property defining the behavior of a gauge type.
#[derive(Debug, Clone)]
pub struct GaugeProperty {
    pub gauge_type: GaugeType,
    pub min: f64,
    pub max: f64,
    pub init: f64,
    pub border: f64,
    pub modifier: GaugeModifier,
    pub guts: Vec<(f64, f64)>,
    pub use_total_modifier: bool,
}

impl GaugeProperty {
    /// SEVENKEYS NORMAL gauge.
    pub fn sevenkeys_normal() -> Self {
        Self {
            gauge_type: GaugeType::Normal,
            min: 2.0,
            max: 100.0,
            init: 20.0,
            border: 80.0,
            modifier: GaugeModifier::new(1.0, 1.0, 0.5, -3.0, -6.0, -2.0),
            guts: vec![],
            use_total_modifier: true,
        }
    }

    /// SEVENKEYS ASSIST EASY gauge.
    pub fn sevenkeys_assist_easy() -> Self {
        Self {
            gauge_type: GaugeType::AssistEasy,
            min: 2.0,
            max: 100.0,
            init: 20.0,
            border: 60.0,
            modifier: GaugeModifier::new(1.0, 1.0, 0.5, -1.0, -3.0, -0.5),
            guts: vec![],
            use_total_modifier: true,
        }
    }

    /// SEVENKEYS EASY gauge.
    pub fn sevenkeys_easy() -> Self {
        Self {
            gauge_type: GaugeType::Easy,
            min: 2.0,
            max: 100.0,
            init: 20.0,
            border: 80.0,
            modifier: GaugeModifier::new(1.0, 1.0, 0.5, -1.5, -4.5, -1.0),
            guts: vec![],
            use_total_modifier: true,
        }
    }

    /// SEVENKEYS HARD gauge.
    pub fn sevenkeys_hard() -> Self {
        Self {
            gauge_type: GaugeType::Hard,
            min: 0.0,
            max: 100.0,
            init: 100.0,
            border: 0.0,
            modifier: GaugeModifier::new(0.15, 0.12, 0.03, -5.0, -10.0, -5.0),
            guts: vec![
                (10.0, 0.4),
                (20.0, 0.5),
                (30.0, 0.6),
                (40.0, 0.7),
                (50.0, 0.8),
            ],
            use_total_modifier: false,
        }
    }

    /// SEVENKEYS EXHARD gauge.
    pub fn sevenkeys_exhard() -> Self {
        Self {
            gauge_type: GaugeType::ExHard,
            min: 0.0,
            max: 100.0,
            init: 100.0,
            border: 0.0,
            modifier: GaugeModifier::new(0.15, 0.06, 0.0, -8.0, -16.0, -8.0),
            guts: vec![],
            use_total_modifier: false,
        }
    }

    /// SEVENKEYS HAZARD gauge.
    pub fn sevenkeys_hazard() -> Self {
        Self {
            gauge_type: GaugeType::Hazard,
            min: 0.0,
            max: 100.0,
            init: 100.0,
            border: 0.0,
            modifier: GaugeModifier::new(0.15, 0.06, 0.0, -100.0, -100.0, -10.0),
            guts: vec![],
            use_total_modifier: false,
        }
    }

    /// SEVENKEYS CLASS gauge (for Dan/Course mode).
    /// Similar to Hard gauge but designed for gauge carry-over.
    pub fn sevenkeys_class() -> Self {
        Self {
            gauge_type: GaugeType::Class,
            min: 0.0,
            max: 100.0,
            init: 100.0,
            border: 0.0,
            modifier: GaugeModifier::new(0.15, 0.12, 0.03, -5.0, -10.0, -5.0),
            guts: vec![
                (10.0, 0.4),
                (20.0, 0.5),
                (30.0, 0.6),
                (40.0, 0.7),
                (50.0, 0.8),
            ],
            use_total_modifier: false,
        }
    }
}

/// Groove gauge that tracks player health during gameplay.
pub struct GrooveGauge {
    property: GaugeProperty,
    value: f64,
    #[allow(dead_code)]
    total: f64,
    #[allow(dead_code)]
    total_notes: usize,
    calculated_modifier: GaugeModifier,
}

impl GrooveGauge {
    /// Create a new groove gauge.
    pub fn new(property: GaugeProperty, total: f64, total_notes: usize) -> Self {
        let calculated_modifier = Self::calculate_modifier(&property, total, total_notes);
        Self {
            value: property.init,
            property,
            total,
            total_notes,
            calculated_modifier,
        }
    }

    /// Create a new ASSIST EASY gauge.
    pub fn assist_easy(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_assist_easy(), total, total_notes)
    }

    /// Create a new EASY gauge.
    pub fn easy(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_easy(), total, total_notes)
    }

    /// Create a new NORMAL gauge.
    pub fn normal(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_normal(), total, total_notes)
    }

    /// Create a new HARD gauge.
    pub fn hard(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_hard(), total, total_notes)
    }

    /// Create a new EXHARD gauge.
    pub fn exhard(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_exhard(), total, total_notes)
    }

    /// Create a new HAZARD gauge.
    pub fn hazard(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_hazard(), total, total_notes)
    }

    /// Create a new CLASS gauge (for Dan/Course mode).
    pub fn class(total: f64, total_notes: usize) -> Self {
        Self::new(GaugeProperty::sevenkeys_class(), total, total_notes)
    }

    /// Create a CLASS gauge with a specific initial value (for gauge carry-over).
    pub fn class_with_initial(total: f64, total_notes: usize, initial_value: f64) -> Self {
        let mut gauge = Self::class(total, total_notes);
        gauge.value = initial_value.clamp(0.0, 100.0);
        gauge
    }

    fn calculate_modifier(
        property: &GaugeProperty,
        total: f64,
        total_notes: usize,
    ) -> GaugeModifier {
        if property.use_total_modifier && total_notes > 0 {
            let base = &property.modifier;
            let ratio = total / total_notes as f64;
            GaugeModifier::new(
                base.get(JudgeRank::PerfectGreat) * ratio,
                base.get(JudgeRank::Great) * ratio,
                base.get(JudgeRank::Good) * ratio,
                base.get(JudgeRank::Bad),
                base.get(JudgeRank::Poor),
                base.get(JudgeRank::Miss),
            )
        } else {
            property.modifier.clone()
        }
    }

    /// Update the gauge based on a judge result.
    pub fn update(&mut self, rank: JudgeRank) {
        let mut change = self.calculated_modifier.get(rank);

        if change < 0.0 {
            change = self.apply_guts(change);
        }

        self.value = (self.value + change).clamp(self.property.min, self.property.max);
    }

    fn apply_guts(&self, change: f64) -> f64 {
        for &(threshold, multiplier) in &self.property.guts {
            if self.value < threshold {
                return change * multiplier;
            }
        }
        change
    }

    /// Get the current gauge value (0.0 - 100.0).
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Get the gauge value as a ratio (0.0 - 1.0).
    pub fn ratio(&self) -> f64 {
        self.value / 100.0
    }

    /// Check if the gauge is at or above the clear border.
    pub fn is_clear(&self) -> bool {
        self.value >= self.property.border
    }

    /// Check if the gauge has reached 0 (for HARD/EXHARD/CLASS).
    pub fn is_dead(&self) -> bool {
        matches!(
            self.property.gauge_type,
            GaugeType::Hard | GaugeType::ExHard | GaugeType::Hazard | GaugeType::Class
        ) && self.value <= 0.0
    }

    /// Get the gauge type.
    pub fn gauge_type(&self) -> GaugeType {
        self.property.gauge_type
    }

    /// Get the border value.
    pub fn border(&self) -> f64 {
        self.property.border
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_gauge_initial_value() {
        let gauge = GrooveGauge::normal(200.0, 1000);
        assert_eq!(gauge.value(), 20.0);
        assert_eq!(gauge.border(), 80.0);
    }

    #[test]
    fn test_hard_gauge_initial_value() {
        let gauge = GrooveGauge::hard(200.0, 1000);
        assert_eq!(gauge.value(), 100.0);
        assert_eq!(gauge.border(), 0.0);
    }

    #[test]
    fn test_normal_gauge_total_modifier() {
        let mut gauge = GrooveGauge::normal(200.0, 1000);
        let initial = gauge.value();
        gauge.update(JudgeRank::PerfectGreat);
        let increase = gauge.value() - initial;
        let expected = 1.0 * 200.0 / 1000.0;
        assert!((increase - expected).abs() < 0.001);
    }

    #[test]
    fn test_hard_gauge_guts_modifier() {
        let mut gauge = GrooveGauge::hard(200.0, 1000);
        gauge.value = 5.0;

        let initial = gauge.value();
        gauge.update(JudgeRank::Bad);
        let decrease = initial - gauge.value();
        let expected = 5.0 * 0.4;
        assert!((decrease - expected).abs() < 0.001);
    }

    #[test]
    fn test_normal_gauge_clear() {
        let mut gauge = GrooveGauge::normal(200.0, 100);
        assert!(!gauge.is_clear());

        for _ in 0..100 {
            gauge.update(JudgeRank::PerfectGreat);
        }
        assert!(gauge.is_clear());
    }

    #[test]
    fn test_hard_gauge_dead() {
        let mut gauge = GrooveGauge::hard(200.0, 100);
        assert!(!gauge.is_dead());

        // With guts modifier, we need more misses to reach 0
        for _ in 0..100 {
            gauge.update(JudgeRank::Miss);
        }
        assert!(gauge.is_dead());
    }

    #[test]
    fn test_gauge_clamp() {
        let mut gauge = GrooveGauge::normal(200.0, 10);

        for _ in 0..1000 {
            gauge.update(JudgeRank::PerfectGreat);
        }
        assert_eq!(gauge.value(), 100.0);
    }
}
