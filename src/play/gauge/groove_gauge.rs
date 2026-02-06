use super::gauge_property::{GaugeElementProperty, GaugePropertyType};

/// A single gauge instance with current value and pre-computed gauge changes.
/// Corresponds to GrooveGauge.Gauge inner class in beatoraja.
#[derive(Debug, Clone)]
pub struct Gauge {
    /// Current gauge value.
    value: f32,
    /// Gauge element property (static reference).
    element: &'static GaugeElementProperty,
    /// Pre-computed gauge change values (after modifier applied).
    gauge: [f32; 6],
}

impl Gauge {
    /// Create a new gauge from element property and model parameters.
    pub fn new(element: &'static GaugeElementProperty, total: f64, total_notes: usize) -> Self {
        let mut gauge = element.value;
        if let Some(modifier) = element.modifier {
            for v in &mut gauge {
                *v = modifier.modify(*v, total, total_notes);
            }
        }
        Self {
            value: element.init,
            element,
            gauge,
        }
    }

    /// Get the current gauge value.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set the gauge value (clamped to [min, max]).
    /// Only updates when value > 0 (no update after death).
    pub fn set_value(&mut self, value: f32) {
        if self.value > 0.0 {
            self.value = value.clamp(self.element.min, self.element.max);
        }
    }

    /// Update gauge based on judge result.
    /// `judge`: judge index (0=PG, 1=GR, 2=GD, 3=BD, 4=PR, 5=MS).
    /// `rate`: multiplier for the gauge change (typically 1.0, 0.5 for HCN).
    pub fn update(&mut self, judge: usize, rate: f32) {
        let mut inc = self.gauge[judge] * rate;
        if inc < 0.0 {
            for gut in self.element.guts {
                if self.value < gut[0] {
                    inc *= gut[1];
                    break;
                }
            }
        }
        self.set_value(self.value + inc);
    }

    /// Whether the gauge meets the clear condition.
    pub fn is_qualified(&self) -> bool {
        self.value > 0.0 && self.value >= self.element.border
    }

    /// Whether the gauge is at maximum.
    pub fn is_max(&self) -> bool {
        (self.value - self.element.max).abs() < f32::EPSILON
    }

    /// Get the element property.
    pub fn property(&self) -> &GaugeElementProperty {
        self.element
    }
}

/// GrooveGauge manages all 9 gauge types simultaneously.
/// Corresponds to GrooveGauge class in beatoraja.
#[derive(Debug, Clone)]
pub struct GrooveGauge {
    /// The currently selected gauge type index.
    gauge_type: usize,
    /// The original gauge type at construction time.
    original_type: usize,
    /// All 9 gauges.
    gauges: Vec<Gauge>,
}

impl GrooveGauge {
    /// Create a new GrooveGauge.
    /// `gauge_type`: initial gauge type index (0-8).
    /// `property`: gauge property set for the mode.
    /// `total`: BMS model TOTAL value.
    /// `total_notes`: total playable notes.
    pub fn new(
        gauge_type: usize,
        property: GaugePropertyType,
        total: f64,
        total_notes: usize,
    ) -> Self {
        let elements = property.elements();
        let gauges = elements
            .iter()
            .map(|elem| Gauge::new(elem, total, total_notes))
            .collect();
        Self {
            gauge_type,
            original_type: gauge_type,
            gauges,
        }
    }

    /// Update all gauges with a judge result.
    pub fn update(&mut self, judge: usize, rate: f32) {
        for gauge in &mut self.gauges {
            gauge.update(judge, rate);
        }
    }

    /// Update all gauges with a judge result (rate = 1.0).
    pub fn update_judge(&mut self, judge: usize) {
        self.update(judge, 1.0);
    }

    /// Add a direct value to all gauges (e.g., mine damage).
    pub fn add_value(&mut self, value: f32) {
        for gauge in &mut self.gauges {
            gauge.set_value(gauge.value() + value);
        }
    }

    /// Get the current gauge value for the selected type.
    pub fn value(&self) -> f32 {
        self.gauges[self.gauge_type].value()
    }

    /// Get the gauge value for a specific type.
    pub fn value_of(&self, gauge_type: usize) -> f32 {
        self.gauges[gauge_type].value()
    }

    /// Set the gauge value for all gauges.
    pub fn set_value(&mut self, value: f32) {
        for gauge in &mut self.gauges {
            gauge.set_value(value);
        }
    }

    /// Set the gauge value for a specific type.
    pub fn set_value_of(&mut self, gauge_type: usize, value: f32) {
        self.gauges[gauge_type].set_value(value);
    }

    /// Whether the current gauge meets the clear condition.
    pub fn is_qualified(&self) -> bool {
        self.gauges[self.gauge_type].is_qualified()
    }

    /// Get the current gauge type index.
    pub fn gauge_type(&self) -> usize {
        self.gauge_type
    }

    /// Set the current gauge type.
    pub fn set_gauge_type(&mut self, gauge_type: usize) {
        self.gauge_type = gauge_type;
    }

    /// Whether the gauge type has changed from the original.
    pub fn is_type_changed(&self) -> bool {
        self.original_type != self.gauge_type
    }

    /// Whether the current gauge is a course gauge (CLASS/EXCLASS/EXHARDCLASS).
    pub fn is_course_gauge(&self) -> bool {
        self.gauge_type >= 6 && self.gauge_type <= 8
    }

    /// Get the number of gauge types.
    pub fn gauge_type_count(&self) -> usize {
        self.gauges.len()
    }

    /// Get a reference to a specific gauge.
    pub fn gauge(&self, gauge_type: usize) -> &Gauge {
        &self.gauges[gauge_type]
    }

    /// Get a mutable reference to a specific gauge.
    pub fn gauge_mut(&mut self, gauge_type: usize) -> &mut Gauge {
        &mut self.gauges[gauge_type]
    }

    /// Determine gauge type for grade (course) play.
    /// Corresponds to GrooveGauge.create() logic in beatoraja.
    pub fn grade_gauge_type(base_type: usize) -> usize {
        if base_type <= 2 {
            6 // CLASS
        } else if base_type == 3 {
            7 // EXCLASS
        } else {
            8 // EXHARDCLASS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::gauge::gauge_property::*;

    fn make_sevenkeys_gauge(gauge_type: usize) -> GrooveGauge {
        // total=300, total_notes=1000 (typical chart)
        GrooveGauge::new(gauge_type, GaugePropertyType::SevenKeys, 300.0, 1000)
    }

    // =========================================================================
    // Gauge basic tests
    // =========================================================================

    #[test]
    fn gauge_init_value() {
        let gauge = Gauge::new(&NORMAL, 300.0, 1000);
        assert!((gauge.value() - 20.0).abs() < 0.001);
    }

    #[test]
    fn gauge_hard_init() {
        let gauge = Gauge::new(&HARD, 300.0, 1000);
        assert!((gauge.value() - 100.0).abs() < 0.001);
    }

    #[test]
    fn gauge_set_value_clamps() {
        let mut gauge = Gauge::new(&NORMAL, 300.0, 1000);
        gauge.set_value(150.0);
        assert!((gauge.value() - 100.0).abs() < 0.001); // Clamped to max

        gauge.set_value(-10.0);
        assert!((gauge.value() - 2.0).abs() < 0.001); // Clamped to min
    }

    #[test]
    fn gauge_set_value_no_update_after_death() {
        let mut gauge = Gauge::new(&HARD, 300.0, 1000);
        // Hard gauge starts at 100, min is 0
        gauge.set_value(0.0); // Set to 0 (dead)
        gauge.set_value(50.0); // Should not update
        assert!((gauge.value() - 0.0).abs() < 0.001);
    }

    #[test]
    fn gauge_update_pg_normal() {
        let mut gauge = Gauge::new(&NORMAL, 300.0, 1000);
        // PG: base 1.0, modified by Total: 1.0 * 300 / 1000 = 0.3
        let old_value = gauge.value();
        gauge.update(0, 1.0);
        assert!((gauge.value() - (old_value + 0.3)).abs() < 0.01);
    }

    #[test]
    fn gauge_update_poor_normal() {
        let mut gauge = Gauge::new(&NORMAL, 300.0, 1000);
        gauge.set_value(50.0);
        let old_value = gauge.value();
        // PR: -6.0 (no modifier applied to negative)
        gauge.update(4, 1.0);
        assert!((gauge.value() - (old_value - 6.0)).abs() < 0.01);
    }

    #[test]
    fn gauge_guts_reduce_damage() {
        // HARD gauge with guts: when value < 10, damage * 0.4
        let mut gauge = Gauge::new(&HARD, 300.0, 1000);
        gauge.set_value(5.0); // Below first gut threshold (10)

        // BD: -5.0, with gut: -5.0 * 0.4 = -2.0
        let old_value = gauge.value();
        gauge.update(3, 1.0);
        let expected = old_value - 5.0 * 0.4;
        assert!(
            (gauge.value() - expected).abs() < 0.01,
            "Expected {}, got {}",
            expected,
            gauge.value()
        );
    }

    #[test]
    fn gauge_is_qualified_normal() {
        let mut gauge = Gauge::new(&NORMAL, 300.0, 1000);
        assert!(!gauge.is_qualified()); // 20 < 80
        gauge.set_value(80.0);
        assert!(gauge.is_qualified()); // 80 >= 80
    }

    #[test]
    fn gauge_is_qualified_hard() {
        let gauge = Gauge::new(&HARD, 300.0, 1000);
        assert!(gauge.is_qualified()); // 100 > 0, border is 0
    }

    #[test]
    fn gauge_is_max() {
        let mut gauge = Gauge::new(&NORMAL, 300.0, 1000);
        assert!(!gauge.is_max());
        gauge.set_value(100.0);
        assert!(gauge.is_max());
    }

    // =========================================================================
    // GrooveGauge tests
    // =========================================================================

    #[test]
    fn groove_gauge_new() {
        let gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        assert_eq!(gg.gauge_type(), GAUGE_NORMAL);
        assert_eq!(gg.gauge_type_count(), 9);
        assert!(!gg.is_type_changed());
    }

    #[test]
    fn groove_gauge_value() {
        let gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        assert!((gg.value() - 20.0).abs() < 0.001); // NORMAL init
    }

    #[test]
    fn groove_gauge_update_all() {
        let mut gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        gg.update_judge(0); // PG
        // All 9 gauges should be updated
        assert!(gg.value() > 20.0); // Normal gauge increased
    }

    #[test]
    fn groove_gauge_type_change() {
        let mut gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        assert!(!gg.is_type_changed());
        gg.set_gauge_type(GAUGE_HARD);
        assert!(gg.is_type_changed());
        assert_eq!(gg.gauge_type(), GAUGE_HARD);
    }

    #[test]
    fn groove_gauge_is_course_gauge() {
        let gg = make_sevenkeys_gauge(GAUGE_CLASS);
        assert!(gg.is_course_gauge());

        let gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        assert!(!gg.is_course_gauge());
    }

    #[test]
    fn groove_gauge_add_value() {
        let mut gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        let old = gg.value();
        gg.add_value(-5.0);
        assert!((gg.value() - (old - 5.0)).abs() < 0.001);
    }

    #[test]
    fn groove_gauge_set_value() {
        let mut gg = make_sevenkeys_gauge(GAUGE_NORMAL);
        gg.set_value(50.0);
        // All gauges should be 50 (or clamped)
        for i in 0..9 {
            assert!(gg.value_of(i) <= 100.0);
        }
    }

    #[test]
    fn groove_gauge_grade_type() {
        assert_eq!(GrooveGauge::grade_gauge_type(0), 6); // ASSIST_EASY -> CLASS
        assert_eq!(GrooveGauge::grade_gauge_type(1), 6); // EASY -> CLASS
        assert_eq!(GrooveGauge::grade_gauge_type(2), 6); // NORMAL -> CLASS
        assert_eq!(GrooveGauge::grade_gauge_type(3), 7); // HARD -> EXCLASS
        assert_eq!(GrooveGauge::grade_gauge_type(4), 8); // EXHARD -> EXHARDCLASS
        assert_eq!(GrooveGauge::grade_gauge_type(5), 8); // HAZARD -> EXHARDCLASS
    }

    // =========================================================================
    // Modifier integration tests
    // =========================================================================

    #[test]
    fn total_modifier_applied_to_normal_gauge() {
        // NORMAL: PG base=1.0, Total modifier: 1.0 * 300 / 1000 = 0.3
        let gauge = Gauge::new(&NORMAL, 300.0, 1000);
        let modified_pg = gauge.gauge[0];
        assert!((modified_pg - 0.3).abs() < 0.01);
    }

    #[test]
    fn limit_increment_applied_to_hard_gauge() {
        // HARD: PG base=0.15, LimitIncrement with total=300, notes=1000
        // pg = min(0.15, (2*300-320)/1000) = min(0.15, 0.28) = 0.15
        // modified = 0.15 * 0.15 / 0.15 = 0.15
        let gauge = Gauge::new(&HARD, 300.0, 1000);
        assert!((gauge.gauge[0] - 0.15).abs() < 0.01);
    }

    #[test]
    fn no_modifier_class_gauge() {
        // CLASS: no modifier, values unchanged
        let gauge = Gauge::new(&CLASS, 300.0, 1000);
        assert_eq!(gauge.gauge, CLASS.value);
    }

    #[test]
    fn hazard_instant_death() {
        let mut gg = make_sevenkeys_gauge(GAUGE_HAZARD);
        assert!((gg.value() - 100.0).abs() < 0.001);
        gg.update_judge(3); // BD: -100.0
        assert!((gg.value_of(GAUGE_HAZARD) - 0.0).abs() < 0.001);
    }
}
