//! Judge window compatibility tests with beatoraja.

use brs::model::JudgeRankType;
use brs::state::play::{JudgeRank, JudgeWindow};

/// Test base judge windows match beatoraja defaults.
#[test]
fn test_base_windows() {
    let window = JudgeWindow::base();

    // beatoraja base windows (RANK 2 / NORMAL)
    assert_eq!(window.pg, 20.0);
    assert_eq!(window.gr, 50.0);
    assert_eq!(window.gd, 100.0);
    assert_eq!(window.bd, 150.0);
    assert_eq!(window.pr, 200.0);
}

/// Test judge boundary values.
#[test]
fn test_judge_boundaries() {
    let window = JudgeWindow::base();

    // Exact boundaries should hit the better judgment
    assert_eq!(window.judge(20.0), Some(JudgeRank::PerfectGreat));
    assert_eq!(window.judge(50.0), Some(JudgeRank::Great));
    assert_eq!(window.judge(100.0), Some(JudgeRank::Good));
    assert_eq!(window.judge(150.0), Some(JudgeRank::Bad));
    assert_eq!(window.judge(200.0), Some(JudgeRank::Poor));

    // Just over boundaries should hit the worse judgment
    assert_eq!(window.judge(20.001), Some(JudgeRank::Great));
    assert_eq!(window.judge(50.001), Some(JudgeRank::Good));
    assert_eq!(window.judge(100.001), Some(JudgeRank::Bad));
    assert_eq!(window.judge(150.001), Some(JudgeRank::Poor));
    assert_eq!(window.judge(200.001), None);
}

/// Test scaled windows maintain proportions.
#[test]
fn test_scaled_windows_proportions() {
    let base = JudgeWindow::base();
    let scaled = JudgeWindow::from_rank(0, JudgeRankType::BmsRank); // 0.7x

    // All windows should scale by the same factor
    let ratio = scaled.pg / base.pg;
    assert!((scaled.gr / base.gr - ratio).abs() < 0.001);
    assert!((scaled.gd / base.gd - ratio).abs() < 0.001);
    assert!((scaled.bd / base.bd - ratio).abs() < 0.001);
    assert!((scaled.pr / base.pr - ratio).abs() < 0.001);
}

/// Test timing difference is symmetric (early vs late).
#[test]
fn test_symmetric_timing() {
    let window = JudgeWindow::base();

    // Same absolute difference should give same judgment
    assert_eq!(window.judge(30.0), window.judge(30.0)); // Both should be Great
    assert_eq!(window.judge(75.0), window.judge(75.0)); // Both should be Good
}

/// Test RANK to scale factor mapping.
#[test]
fn test_rank_scale_factors() {
    let base = JudgeWindow::base();

    let rank0 = JudgeWindow::from_rank(0, JudgeRankType::BmsRank);
    let rank1 = JudgeWindow::from_rank(1, JudgeRankType::BmsRank);
    let rank2 = JudgeWindow::from_rank(2, JudgeRankType::BmsRank);
    let rank3 = JudgeWindow::from_rank(3, JudgeRankType::BmsRank);

    // Verify scale factors
    assert!((rank0.pg / base.pg - 0.7).abs() < 0.001);
    assert!((rank1.pg / base.pg - 0.85).abs() < 0.001);
    assert!((rank2.pg / base.pg - 1.0).abs() < 0.001);
    assert!((rank3.pg / base.pg - 1.2).abs() < 0.001);
}

/// Test DEFEXRANK continuous scaling.
#[test]
fn test_defexrank_continuous_scaling() {
    let base = JudgeWindow::base();

    // Test various DEFEXRANK values
    for value in [50, 75, 100, 125, 150, 175, 200] {
        let window = JudgeWindow::from_rank(value, JudgeRankType::BmsDefExRank);
        let expected_scale = (value as f64 / 100.0).clamp(0.5, 2.0);
        let actual_scale = window.pg / base.pg;
        assert!(
            (actual_scale - expected_scale).abs() < 0.001,
            "DEFEXRANK {} expected scale {}, got {}",
            value,
            expected_scale,
            actual_scale
        );
    }
}
