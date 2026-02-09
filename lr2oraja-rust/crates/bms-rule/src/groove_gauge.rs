use crate::JUDGE_COUNT;
/// Groove gauge implementation for BMS play.
///
/// Ported from Java: `GrooveGauge.java` (Gauge inner class + GrooveGauge outer class).
///
/// The groove gauge tracks 9 gauge types simultaneously. Each gauge has its own
/// rules for increment/decrement, guts damage reduction, and death thresholds.
/// The active gauge determines the displayed value and clear status.
use crate::gauge_property::{GAUGE_TYPE_COUNT, GaugeElementProperty, GaugeProperty, GaugeType};

/// A single gauge instance with pre-computed judge values.
///
/// Once gauge value reaches 0, it can never recover (dead gauge).
#[derive(Debug, Clone)]
pub struct Gauge {
    /// Current gauge value
    value: f32,
    /// Gauge element property (defines min, max, border, death, guts)
    element: GaugeElementProperty,
    /// Pre-computed gauge change values after modifier application: [PG, GR, GD, BD, PR, MS]
    gauge_values: [f32; JUDGE_COUNT],
}

impl Gauge {
    /// Create a new gauge from an element property, pre-computing modified values.
    ///
    /// # Arguments
    /// - `element`: The gauge element configuration
    /// - `total`: Chart's TOTAL value
    /// - `total_notes`: Total number of playable notes
    pub fn new(element: GaugeElementProperty, total: f64, total_notes: usize) -> Self {
        let mut gauge_values = element.values;
        if let Some(modifier) = element.modifier {
            for v in &mut gauge_values {
                *v = modifier.modify(*v, total, total_notes);
            }
        }
        Self {
            value: element.init,
            element,
            gauge_values,
        }
    }

    /// Update the gauge based on a judge result.
    ///
    /// # Arguments
    /// - `judge`: Judge index (0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS)
    /// - `rate`: Multiplier for the gauge change (typically 1.0)
    pub fn update(&mut self, judge: usize, rate: f32) {
        debug_assert!(judge < JUDGE_COUNT, "judge index out of range: {judge}");
        let mut inc = self.gauge_values[judge] * rate;

        // Apply guts damage reduction for negative increments
        if inc < 0.0 {
            for gut in &self.element.guts {
                if self.value < gut.threshold {
                    inc *= gut.multiplier;
                    break;
                }
            }
        }

        self.set_value(self.value + inc);
    }

    /// Set the gauge value with clamping and death check.
    ///
    /// Dead gauges (value == 0) never recover. Values below death threshold
    /// cause immediate death (value set to 0).
    pub fn set_value(&mut self, value: f32) {
        if self.value > 0.0 {
            self.value = value.clamp(self.element.min, self.element.max);
            if self.value < self.element.death {
                self.value = 0.0;
            }
        }
    }

    /// Get the current gauge value.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Check if the gauge meets the clear condition.
    ///
    /// Returns true if gauge > 0 and gauge >= border.
    pub fn is_qualified(&self) -> bool {
        self.value > 0.0 && self.value >= self.element.border
    }

    /// Check if the gauge is at its maximum value.
    pub fn is_max(&self) -> bool {
        (self.value - self.element.max).abs() < f32::EPSILON
    }

    /// Check if the gauge is dead (value == 0, cannot recover).
    pub fn is_dead(&self) -> bool {
        self.value == 0.0
    }

    /// Get the gauge element property.
    pub fn element(&self) -> &GaugeElementProperty {
        &self.element
    }
}

/// Groove gauge that tracks all 9 gauge types simultaneously.
///
/// All gauges are updated on every judge event. The active gauge type determines
/// which gauge's value is displayed and used for clear status.
#[derive(Debug, Clone)]
pub struct GrooveGauge {
    /// All 9 gauges
    gauges: [Gauge; GAUGE_TYPE_COUNT],
    /// Currently active gauge type
    active_type: GaugeType,
    /// Original gauge type (for detecting type changes)
    original_type: GaugeType,
}

impl GrooveGauge {
    /// Create a new groove gauge with all 9 gauge types.
    ///
    /// # Arguments
    /// - `property`: Gauge property set for the play mode
    /// - `active_type`: Initially active gauge type
    /// - `total`: Chart's TOTAL value
    /// - `total_notes`: Total number of playable notes
    pub fn new(
        property: &GaugeProperty,
        active_type: GaugeType,
        total: f64,
        total_notes: usize,
    ) -> Self {
        let gauges =
            std::array::from_fn(|i| Gauge::new(property.elements[i].clone(), total, total_notes));
        Self {
            gauges,
            active_type,
            original_type: active_type,
        }
    }

    /// Update all gauges with a judge result.
    ///
    /// # Arguments
    /// - `judge`: Judge index (0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS)
    pub fn update(&mut self, judge: usize) {
        self.update_with_rate(judge, 1.0);
    }

    /// Update all gauges with a judge result and rate multiplier.
    ///
    /// # Arguments
    /// - `judge`: Judge index (0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS)
    /// - `rate`: Multiplier for gauge change
    pub fn update_with_rate(&mut self, judge: usize, rate: f32) {
        for gauge in &mut self.gauges {
            gauge.update(judge, rate);
        }
    }

    /// Add a fixed value to all gauges.
    pub fn add_value(&mut self, value: f32) {
        for gauge in &mut self.gauges {
            gauge.set_value(gauge.value() + value);
        }
    }

    /// Get the active gauge's current value.
    pub fn value(&self) -> f32 {
        self.gauges[self.active_type as usize].value()
    }

    /// Get a specific gauge's current value.
    pub fn value_of(&self, gauge_type: GaugeType) -> f32 {
        self.gauges[gauge_type as usize].value()
    }

    /// Set the value of all gauges.
    pub fn set_value(&mut self, value: f32) {
        for gauge in &mut self.gauges {
            gauge.set_value(value);
        }
    }

    /// Set the value of a specific gauge.
    pub fn set_value_of(&mut self, gauge_type: GaugeType, value: f32) {
        self.gauges[gauge_type as usize].set_value(value);
    }

    /// Check if the active gauge meets the clear condition.
    pub fn is_qualified(&self) -> bool {
        self.gauges[self.active_type as usize].is_qualified()
    }

    /// Get the currently active gauge type.
    pub fn active_type(&self) -> GaugeType {
        self.active_type
    }

    /// Set the active gauge type.
    pub fn set_active_type(&mut self, gauge_type: GaugeType) {
        self.active_type = gauge_type;
    }

    /// Check if the active type has changed from the original.
    pub fn is_type_changed(&self) -> bool {
        self.active_type != self.original_type
    }

    /// Check if the current gauge type is a course gauge.
    pub fn is_course_gauge(&self) -> bool {
        self.active_type.is_course_gauge()
    }

    /// Get a reference to the active gauge.
    pub fn active_gauge(&self) -> &Gauge {
        &self.gauges[self.active_type as usize]
    }

    /// Get a reference to a specific gauge.
    pub fn gauge(&self, gauge_type: GaugeType) -> &Gauge {
        &self.gauges[gauge_type as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gauge_property;
    use crate::{JUDGE_BD, JUDGE_GD, JUDGE_GR, JUDGE_MS, JUDGE_PG, JUDGE_PR};

    // -- Gauge basic tests --

    #[test]
    fn test_gauge_initial_value() {
        let prop = gauge_property::sevenkeys();
        let gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        assert!((gauge.value() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_gauge_hard_initial_value() {
        let prop = gauge_property::sevenkeys();
        let gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        assert!((gauge.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_gauge_normal_pg_recovery() {
        // Normal gauge with Total modifier, total=300, notes=1000
        // Base PG value = 1.0, modified = 1.0 * 300/1000 = 0.3
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        let initial = gauge.value();
        gauge.update(JUDGE_PG, 1.0);
        assert!((gauge.value() - (initial + 0.3)).abs() < 1e-5);
    }

    #[test]
    fn test_gauge_normal_bd_damage() {
        // Normal gauge with Total modifier
        // Base BD value = -3.0 (negative, not modified by Total)
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        let initial = gauge.value();
        gauge.update(JUDGE_BD, 1.0);
        assert!((gauge.value() - (initial - 3.0)).abs() < 1e-5);
    }

    #[test]
    fn test_gauge_clamp_to_max() {
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        gauge.set_value(200.0); // above max of 100
        assert!((gauge.value() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_gauge_clamp_to_min() {
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        gauge.set_value(-10.0); // below min of 2
        assert!((gauge.value() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_gauge_death_threshold_lr2_hard() {
        // LR2 HARD gauge has death=2
        let prop = gauge_property::lr2();
        let mut gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        // Set value to just below death threshold
        gauge.set_value(1.5);
        assert!((gauge.value() - 0.0).abs() < 1e-6, "Should be dead");
    }

    #[test]
    fn test_gauge_death_threshold_normal_no_death() {
        // Normal gauge has death=0, so it should never die from threshold
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        // Set to minimum (2.0), should survive
        gauge.set_value(2.0);
        assert!((gauge.value() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_dead_gauge_cannot_recover() {
        // Once gauge is 0, it never recovers
        let prop = gauge_property::lr2();
        let mut gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        // Kill the gauge
        gauge.set_value(1.0); // below death=2, becomes 0
        assert!(gauge.is_dead());
        // Try to recover
        gauge.set_value(50.0);
        assert!(
            (gauge.value() - 0.0).abs() < 1e-6,
            "Dead gauge should not recover"
        );
        // Try to update with PG
        gauge.update(JUDGE_PG, 1.0);
        assert!(
            (gauge.value() - 0.0).abs() < 1e-6,
            "Dead gauge should not recover via update"
        );
    }

    #[test]
    fn test_gauge_is_qualified() {
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        // Initial value 20, border 80 → not qualified
        assert!(!gauge.is_qualified());
        // Set to 80 → qualified
        gauge.set_value(80.0);
        assert!(gauge.is_qualified());
        // Set to 79.9 → not qualified
        gauge.set_value(79.9);
        assert!(!gauge.is_qualified());
    }

    #[test]
    fn test_gauge_is_qualified_hard() {
        // Hard gauge border=0, so qualified whenever alive
        let prop = gauge_property::sevenkeys();
        let gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        assert!(gauge.is_qualified());
    }

    #[test]
    fn test_gauge_is_max() {
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        assert!(!gauge.is_max());
        gauge.set_value(100.0);
        assert!(gauge.is_max());
    }

    // -- Guts damage reduction tests --

    #[test]
    fn test_guts_damage_reduction_hard_gauge() {
        // SEVENKEYS HARD: guts = [(10,0.4),(20,0.5),(30,0.6),(40,0.7),(50,0.8)]
        // At value=5 (below 10), damage is multiplied by 0.4
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        // With LimitIncrement modifier and high total, gauge_values should be
        // pg = (600-320)/1000 = 0.28 → clamped to 0.15, so pg unchanged
        // BD = -5.0 (negative, not modified)
        gauge.set_value(8.0);
        let before = gauge.value();
        gauge.update(JUDGE_BD, 1.0);
        // BD = -5.0 * 0.4 (guts at threshold 10) = -2.0
        let expected = (before - 2.0).clamp(0.0, 100.0);
        assert!(
            (gauge.value() - expected).abs() < 1e-4,
            "Expected {expected}, got {}",
            gauge.value()
        );
    }

    #[test]
    fn test_guts_no_reduction_above_threshold() {
        // At value=60 (above all guts thresholds), no reduction
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        gauge.set_value(60.0);
        let before = gauge.value();
        gauge.update(JUDGE_BD, 1.0);
        // BD = -5.0, no guts reduction
        let expected = before - 5.0;
        assert!(
            (gauge.value() - expected).abs() < 1e-4,
            "Expected {expected}, got {}",
            gauge.value()
        );
    }

    #[test]
    fn test_guts_lr2_hard() {
        // LR2 HARD: guts = [(32, 0.6)]
        // At value=30 (below 32), damage * 0.6
        let prop = gauge_property::lr2();
        let mut gauge = Gauge::new(prop.elements[GaugeType::Hard as usize].clone(), 300.0, 1000);
        // ModifyDamage modifier at total=300, notes=1000 → fix1=10/13≈0.769, fix2=1.0
        // BD base = -6.0, modified = -6.0 * max(0.769, 1.0) = -6.0
        gauge.set_value(30.0);
        let before = gauge.value();
        gauge.update(JUDGE_BD, 1.0);
        // -6.0 * 0.6 (guts) = -3.6
        let expected = before - 3.6;
        assert!(
            (gauge.value() - expected).abs() < 1e-4,
            "Expected {expected}, got {}",
            gauge.value()
        );
    }

    // -- Gauge rate multiplier test --

    #[test]
    fn test_gauge_update_with_rate() {
        let prop = gauge_property::sevenkeys();
        let mut gauge = Gauge::new(
            prop.elements[GaugeType::Normal as usize].clone(),
            300.0,
            1000,
        );
        let initial = gauge.value();
        gauge.update(JUDGE_PG, 0.5);
        // PG = 0.3, rate = 0.5, inc = 0.15
        assert!((gauge.value() - (initial + 0.15)).abs() < 1e-5);
    }

    // -- GrooveGauge tests --

    #[test]
    fn test_groove_gauge_initial_values() {
        let prop = gauge_property::sevenkeys();
        let gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        // Normal starts at 20
        assert!((gg.value() - 20.0).abs() < 1e-6);
        // Hard starts at 100
        assert!((gg.value_of(GaugeType::Hard) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_groove_gauge_update_all() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        let normal_before = gg.value();
        // Set Hard gauge to below max so we can see the increment
        gg.set_value_of(GaugeType::Hard, 95.0);
        let hard_before = gg.value_of(GaugeType::Hard);
        gg.update(JUDGE_PG);
        // All gauges should have been updated
        assert!(gg.value() > normal_before, "Normal gauge should increase");
        // Hard gauge with LimitIncrement: PG recovery depends on modifier
        // pg = ((2*300)-320)/1000 = 280/1000 = 0.28 → clamped to 0.15
        // modified value = 0.15 * 0.15 / 0.15 = 0.15
        let hard_after = gg.value_of(GaugeType::Hard);
        assert!(
            (hard_after - (hard_before + 0.15)).abs() < 1e-4,
            "Hard gauge PG recovery should be 0.15, got diff {}",
            hard_after - hard_before
        );
    }

    #[test]
    fn test_groove_gauge_active_type() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        assert_eq!(gg.active_type(), GaugeType::Normal);
        assert!(!gg.is_type_changed());
        gg.set_active_type(GaugeType::Hard);
        assert_eq!(gg.active_type(), GaugeType::Hard);
        assert!(gg.is_type_changed());
    }

    #[test]
    fn test_groove_gauge_is_qualified() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        // Normal: init=20, border=80 → not qualified
        assert!(!gg.is_qualified());
        // Switch to Hard: init=100, border=0 → qualified
        gg.set_active_type(GaugeType::Hard);
        assert!(gg.is_qualified());
    }

    #[test]
    fn test_groove_gauge_is_course_gauge() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        assert!(!gg.is_course_gauge());
        gg.set_active_type(GaugeType::Class);
        assert!(gg.is_course_gauge());
    }

    #[test]
    fn test_groove_gauge_add_value() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        let before = gg.value();
        gg.add_value(10.0);
        assert!((gg.value() - (before + 10.0)).abs() < 1e-5);
    }

    #[test]
    fn test_groove_gauge_set_value() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        gg.set_value(50.0);
        // All gauges should be set to 50
        assert!((gg.value_of(GaugeType::Normal) - 50.0).abs() < 1e-6);
        assert!((gg.value_of(GaugeType::Hard) - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_groove_gauge_sequence_normal_to_clear() {
        // Simulate a sequence of perfect judgments on Normal gauge
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        // PG recovery = 1.0 * 300/1000 = 0.3 per note
        // Need to go from 20 to 80 = 60 points, 60/0.3 = 200 PGs
        for _ in 0..200 {
            gg.update(JUDGE_PG);
        }
        assert!(gg.is_qualified(), "Should be qualified after 200 PGs");
    }

    #[test]
    fn test_groove_gauge_hard_death_sequence() {
        // EXHARD gauge has no guts, so damage is consistent: BD = -8.0
        // 100 / 8 = 12.5, so 13 BDs should kill it
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::ExHard, 300.0, 1000);
        assert!((gg.value_of(GaugeType::ExHard) - 100.0).abs() < 1e-6);
        for _ in 0..13 {
            gg.update(JUDGE_BD);
        }
        // After 13 BDs: 100 - 13*8 = -4 → clamped to min=0
        assert!(
            gg.value_of(GaugeType::ExHard) < 1e-6,
            "ExHard gauge should be dead after 13 BDs, got {}",
            gg.value_of(GaugeType::ExHard)
        );
    }

    #[test]
    fn test_groove_gauge_update_with_rate() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        let before = gg.value();
        gg.update_with_rate(JUDGE_PG, 2.0);
        // PG = 0.3, rate = 2.0, inc = 0.6
        assert!((gg.value() - (before + 0.6)).abs() < 1e-5);
    }

    #[test]
    fn test_groove_gauge_hazard_instant_kill() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Hazard, 300.0, 1000);
        assert!((gg.value_of(GaugeType::Hazard) - 100.0).abs() < 1e-6);
        // Single BD = -100 → instant death
        gg.update(JUDGE_BD);
        assert!(
            gg.value_of(GaugeType::Hazard) < 1e-6,
            "Hazard should be dead after single BD"
        );
    }

    #[test]
    fn test_groove_gauge_all_judges() {
        // Test all 6 judge types on Normal gauge
        let prop = gauge_property::sevenkeys();
        let judges = [JUDGE_PG, JUDGE_GR, JUDGE_GD, JUDGE_BD, JUDGE_PR, JUDGE_MS];
        for &judge in &judges {
            let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
            let before = gg.value();
            gg.update(judge);
            let after = gg.value();
            // PG, GR, GD should increase; BD, PR, MS should decrease
            if judge <= JUDGE_GD {
                assert!(after > before, "Judge {judge} should increase gauge");
            } else {
                assert!(after < before, "Judge {judge} should decrease gauge");
            }
        }
    }

    #[test]
    fn test_groove_gauge_set_value_of_specific() {
        let prop = gauge_property::sevenkeys();
        let mut gg = GrooveGauge::new(&prop, GaugeType::Normal, 300.0, 1000);
        gg.set_value_of(GaugeType::Hard, 75.0);
        assert!((gg.value_of(GaugeType::Hard) - 75.0).abs() < 1e-6);
        // Other gauges should not be affected
        assert!((gg.value_of(GaugeType::Normal) - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_groove_gauge_pms_max_120() {
        // PMS gauges have max=120 for AssistEasy, Easy, Normal
        let prop = gauge_property::pms();
        let mut gg = GrooveGauge::new(&prop, GaugeType::AssistEasy, 300.0, 100);
        // Set to above 120, should clamp
        gg.set_value_of(GaugeType::AssistEasy, 150.0);
        assert!((gg.value_of(GaugeType::AssistEasy) - 120.0).abs() < 1e-6);
    }
}
