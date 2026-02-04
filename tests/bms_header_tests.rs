//! Tests for BMS header parsing (#RANK, #TOTAL, #DEFEXRANK).

use brs::model::JudgeRankType;
use brs::state::play::JudgeWindow;

/// Test that #RANK 0 (VERY HARD) applies 0.7x scaling.
#[test]
fn test_rank_0_very_hard_scaling() {
    let window = JudgeWindow::from_rank(0, JudgeRankType::BmsRank);

    // Base: PG=20, GR=50, GD=100, BD=150, PR=200
    // Scale: 0.7x
    assert!((window.pg - 14.0).abs() < 0.001);
    assert!((window.gr - 35.0).abs() < 0.001);
    assert!((window.gd - 70.0).abs() < 0.001);
    assert!((window.bd - 105.0).abs() < 0.001);
    assert!((window.pr - 140.0).abs() < 0.001);
}

/// Test that #RANK 1 (HARD) applies 0.85x scaling.
#[test]
fn test_rank_1_hard_scaling() {
    let window = JudgeWindow::from_rank(1, JudgeRankType::BmsRank);

    assert!((window.pg - 17.0).abs() < 0.001);
    assert!((window.gr - 42.5).abs() < 0.001);
    assert!((window.gd - 85.0).abs() < 0.001);
    assert!((window.bd - 127.5).abs() < 0.001);
    assert!((window.pr - 170.0).abs() < 0.001);
}

/// Test that #RANK 2 (NORMAL) applies 1.0x scaling.
#[test]
fn test_rank_2_normal_scaling() {
    let window = JudgeWindow::from_rank(2, JudgeRankType::BmsRank);

    assert!((window.pg - 20.0).abs() < 0.001);
    assert!((window.gr - 50.0).abs() < 0.001);
    assert!((window.gd - 100.0).abs() < 0.001);
    assert!((window.bd - 150.0).abs() < 0.001);
    assert!((window.pr - 200.0).abs() < 0.001);
}

/// Test that #RANK 3 (EASY) applies 1.2x scaling.
#[test]
fn test_rank_3_easy_scaling() {
    let window = JudgeWindow::from_rank(3, JudgeRankType::BmsRank);

    assert!((window.pg - 24.0).abs() < 0.001);
    assert!((window.gr - 60.0).abs() < 0.001);
    assert!((window.gd - 120.0).abs() < 0.001);
    assert!((window.bd - 180.0).abs() < 0.001);
    assert!((window.pr - 240.0).abs() < 0.001);
}

/// Test that unknown #RANK values default to 1.0x scaling.
#[test]
fn test_rank_unknown_defaults_to_normal() {
    let window_negative = JudgeWindow::from_rank(-1, JudgeRankType::BmsRank);
    let window_large = JudgeWindow::from_rank(99, JudgeRankType::BmsRank);

    // Both should use 1.0x scaling
    assert!((window_negative.pg - 20.0).abs() < 0.001);
    assert!((window_large.pg - 20.0).abs() < 0.001);
}

/// Test #DEFEXRANK linear scaling (value / 100).
#[test]
fn test_defexrank_100_equals_normal() {
    let window = JudgeWindow::from_rank(100, JudgeRankType::BmsDefExRank);

    assert!((window.pg - 20.0).abs() < 0.001);
    assert!((window.gr - 50.0).abs() < 0.001);
}

/// Test #DEFEXRANK with value 150 (1.5x scaling).
#[test]
fn test_defexrank_150_scaling() {
    let window = JudgeWindow::from_rank(150, JudgeRankType::BmsDefExRank);

    assert!((window.pg - 30.0).abs() < 0.001);
    assert!((window.gr - 75.0).abs() < 0.001);
    assert!((window.gd - 150.0).abs() < 0.001);
}

/// Test #DEFEXRANK with value 50 (0.5x scaling, minimum).
#[test]
fn test_defexrank_50_minimum_scaling() {
    let window = JudgeWindow::from_rank(50, JudgeRankType::BmsDefExRank);

    assert!((window.pg - 10.0).abs() < 0.001);
    assert!((window.gr - 25.0).abs() < 0.001);
}

/// Test #DEFEXRANK clamping at minimum (0.5x).
#[test]
fn test_defexrank_clamp_minimum() {
    let window = JudgeWindow::from_rank(30, JudgeRankType::BmsDefExRank);

    // 30/100 = 0.3, but clamped to 0.5
    assert!((window.pg - 10.0).abs() < 0.001);
}

/// Test #DEFEXRANK clamping at maximum (2.0x).
#[test]
fn test_defexrank_clamp_maximum() {
    let window = JudgeWindow::from_rank(300, JudgeRankType::BmsDefExRank);

    // 300/100 = 3.0, but clamped to 2.0
    assert!((window.pg - 40.0).abs() < 0.001);
    assert!((window.gr - 100.0).abs() < 0.001);
}

/// Test bmson judge_rank scaling (value / 18).
#[test]
fn test_bmson_judge_rank_18_equals_normal() {
    let window = JudgeWindow::from_rank(18, JudgeRankType::BmsonJudgeRank);

    // 18 / 18 = 1.0
    assert!((window.pg - 20.0).abs() < 0.001);
    assert!((window.gr - 50.0).abs() < 0.001);
}

/// Test bmson judge_rank scaling with value 36 (2.0x, maximum).
#[test]
fn test_bmson_judge_rank_36_scaling() {
    let window = JudgeWindow::from_rank(36, JudgeRankType::BmsonJudgeRank);

    // 36 / 18 = 2.0
    assert!((window.pg - 40.0).abs() < 0.001);
    assert!((window.gr - 100.0).abs() < 0.001);
}

/// Test bmson judge_rank clamping.
#[test]
fn test_bmson_judge_rank_clamp() {
    let window_min = JudgeWindow::from_rank(5, JudgeRankType::BmsonJudgeRank);
    let window_max = JudgeWindow::from_rank(100, JudgeRankType::BmsonJudgeRank);

    // 5 / 18 = 0.28, clamped to 0.5
    assert!((window_min.pg - 10.0).abs() < 0.001);

    // 100 / 18 = 5.56, clamped to 2.0
    assert!((window_max.pg - 40.0).abs() < 0.001);
}

#[cfg(test)]
mod total_tests {
    use brs::state::play::{GaugeProperty, GrooveGauge, JudgeRank};

    /// Test that #TOTAL affects normal gauge increase rate.
    #[test]
    fn test_total_affects_gauge_increase() {
        // Higher TOTAL = faster gauge increase
        let mut gauge_low = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 100.0, 100);
        let mut gauge_high = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 300.0, 100);

        let initial_low = gauge_low.value();
        let initial_high = gauge_high.value();

        gauge_low.update(JudgeRank::PerfectGreat);
        gauge_high.update(JudgeRank::PerfectGreat);

        let increase_low = gauge_low.value() - initial_low;
        let increase_high = gauge_high.value() - initial_high;

        // TOTAL 300 should give 3x the increase of TOTAL 100
        assert!((increase_high / increase_low - 3.0).abs() < 0.01);
    }

    /// Test TOTAL with different note counts.
    #[test]
    fn test_total_note_ratio() {
        // Same TOTAL, different note counts
        let mut gauge_few = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 200.0, 100);
        let mut gauge_many = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 200.0, 200);

        let initial_few = gauge_few.value();
        let initial_many = gauge_many.value();

        gauge_few.update(JudgeRank::PerfectGreat);
        gauge_many.update(JudgeRank::PerfectGreat);

        let increase_few = gauge_few.value() - initial_few;
        let increase_many = gauge_many.value() - initial_many;

        // With same TOTAL, more notes means less increase per note
        assert!((increase_few / increase_many - 2.0).abs() < 0.01);
    }

    /// Test that TOTAL does not affect hard gauge (fixed modifier).
    #[test]
    fn test_total_does_not_affect_hard_gauge() {
        let mut gauge_low = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 100.0, 100);
        let mut gauge_high = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 300.0, 100);

        let initial_low = gauge_low.value();
        let initial_high = gauge_high.value();

        gauge_low.update(JudgeRank::PerfectGreat);
        gauge_high.update(JudgeRank::PerfectGreat);

        let increase_low = gauge_low.value() - initial_low;
        let increase_high = gauge_high.value() - initial_high;

        // Hard gauge uses fixed modifier, so both should increase the same
        assert!((increase_low - increase_high).abs() < 0.001);
    }

    /// Test expected gauge value after full combo with various TOTAL values.
    #[test]
    fn test_full_combo_gauge_with_total() {
        let total = 200.0;
        let notes = 100;
        let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), total, notes);

        // All PerfectGreat
        for _ in 0..notes {
            gauge.update(JudgeRank::PerfectGreat);
        }

        // With TOTAL=200, 100 notes, base modifier 1.0:
        // Increase per note = 1.0 * (200/100) = 2.0
        // Total increase = 2.0 * 100 = 200
        // But clamped to max (100)
        assert_eq!(gauge.value(), 100.0);
        assert!(gauge.is_clear());
    }
}
