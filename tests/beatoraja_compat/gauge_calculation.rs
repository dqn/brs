//! Gauge calculation compatibility tests with beatoraja.

use brs::state::play::{GaugeProperty, GrooveGauge, JudgeRank};

/// Test data from beatoraja for gauge verification.
struct GaugeTestCase {
    gauge_type: GaugeType,
    total: f64,
    total_notes: usize,
    judge_sequence: Vec<JudgeRank>,
    expected_final_gauge: f64,
    tolerance: f64,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum GaugeType {
    Normal,
    Hard,
    ExHard,
}

impl GaugeTestCase {
    fn run(&self) {
        let property = match self.gauge_type {
            GaugeType::Normal => GaugeProperty::sevenkeys_normal(),
            GaugeType::Hard => GaugeProperty::sevenkeys_hard(),
            GaugeType::ExHard => GaugeProperty::sevenkeys_exhard(),
        };

        let mut gauge = GrooveGauge::new(property, self.total, self.total_notes);

        for rank in &self.judge_sequence {
            gauge.update(*rank);
        }

        let diff = (gauge.value() - self.expected_final_gauge).abs();
        assert!(
            diff <= self.tolerance,
            "Gauge mismatch: expected {:.2}, got {:.2} (diff: {:.2})",
            self.expected_final_gauge,
            gauge.value(),
            diff
        );
    }
}

/// Test normal gauge basic behavior.
#[test]
fn test_normal_gauge_all_perfect() {
    GaugeTestCase {
        gauge_type: GaugeType::Normal,
        total: 200.0,
        total_notes: 100,
        judge_sequence: vec![JudgeRank::PerfectGreat; 100],
        expected_final_gauge: 100.0, // Clamped to max
        tolerance: 0.1,
    }
    .run();
}

/// Test normal gauge with some misses.
#[test]
fn test_normal_gauge_with_misses() {
    // Initial: 20.0
    // 50 PG: +50 * (1.0 * 200/100) = +100 → clamped to 100
    // 10 Miss: -10 * 2.0 = -20 → 80
    let mut sequence = vec![JudgeRank::PerfectGreat; 50];
    sequence.extend(vec![JudgeRank::Miss; 10]);

    GaugeTestCase {
        gauge_type: GaugeType::Normal,
        total: 200.0,
        total_notes: 100,
        judge_sequence: sequence,
        expected_final_gauge: 80.0,
        tolerance: 0.1,
    }
    .run();
}

/// Test hard gauge starts at 100%.
#[test]
fn test_hard_gauge_starts_at_100() {
    let gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 200.0, 100);
    assert_eq!(gauge.value(), 100.0);
}

/// Test hard gauge with single miss.
#[test]
fn test_hard_gauge_single_miss() {
    // Hard gauge: Miss = -5.0
    GaugeTestCase {
        gauge_type: GaugeType::Hard,
        total: 200.0,
        total_notes: 100,
        judge_sequence: vec![JudgeRank::Miss],
        expected_final_gauge: 95.0,
        tolerance: 0.1,
    }
    .run();
}

/// Test hard gauge guts system at low gauge.
#[test]
fn test_hard_gauge_guts_at_5_percent() {
    let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 200.0, 100);
    gauge.value = 5.0; // Set directly for test

    let before = gauge.value();
    gauge.update(JudgeRank::Bad);
    let damage = before - gauge.value();

    // At 5%, guts multiplier is 0.4 (below 10% threshold)
    // Bad damage = 5.0 * 0.4 = 2.0
    assert!((damage - 2.0).abs() < 0.01);
}

/// Test hard gauge guts system at 15%.
#[test]
fn test_hard_gauge_guts_at_15_percent() {
    let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 200.0, 100);
    gauge.value = 15.0;

    let before = gauge.value();
    gauge.update(JudgeRank::Bad);
    let damage = before - gauge.value();

    // At 15%, guts multiplier is 0.5 (below 20% threshold)
    // Bad damage = 5.0 * 0.5 = 2.5
    assert!((damage - 2.5).abs() < 0.01);
}

/// Test hard gauge death.
#[test]
fn test_hard_gauge_death() {
    let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 200.0, 100);

    // Apply many misses to reach 0
    for _ in 0..100 {
        gauge.update(JudgeRank::Miss);
        if gauge.is_dead() {
            break;
        }
    }

    assert!(gauge.is_dead());
    assert!(gauge.value() <= 0.0);
}

/// Test exhard gauge has no guts.
#[test]
fn test_exhard_no_guts() {
    let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_exhard(), 200.0, 100);
    gauge.value = 5.0;

    let before = gauge.value();
    gauge.update(JudgeRank::Bad);
    let damage = before - gauge.value();

    // ExHard has no guts, full 8.0 damage
    assert!((damage - 5.0).abs() < 0.01); // Clamped at min 0
}

/// Test clear conditions.
#[test]
fn test_clear_conditions() {
    // Normal gauge: border at 80%
    let mut normal = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 200.0, 100);
    normal.value = 79.9;
    assert!(!normal.is_clear());
    normal.value = 80.0;
    assert!(normal.is_clear());

    // Hard gauge: border at 0%, must be alive
    let mut hard = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 200.0, 100);
    hard.value = 0.1;
    assert!(hard.is_clear());
    assert!(!hard.is_dead());
    hard.value = 0.0;
    assert!(hard.is_clear()); // At exactly 0, at border
    assert!(hard.is_dead()); // But considered dead
}
