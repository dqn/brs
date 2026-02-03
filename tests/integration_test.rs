//! Integration tests for brs.

use brs::audio::AudioConfig;
use brs::input::KeyConfig;
use brs::model::note::LANE_COUNT;
use brs::state::play::{GaugeProperty, GrooveGauge, JudgeRank, Score};

/// Test that all gauge types initialize correctly.
#[test]
fn test_all_gauge_types_initialize() {
    let properties = [
        GaugeProperty::sevenkeys_assist_easy(),
        GaugeProperty::sevenkeys_easy(),
        GaugeProperty::sevenkeys_normal(),
        GaugeProperty::sevenkeys_hard(),
        GaugeProperty::sevenkeys_exhard(),
        GaugeProperty::sevenkeys_hazard(),
    ];

    for property in properties {
        let gauge = GrooveGauge::new(property, 100.0, 100);
        // All gauges should start with valid values
        assert!(gauge.value() >= 0.0 && gauge.value() <= 100.0);
    }
}

/// Test that normal gauge increases on perfect great.
#[test]
fn test_normal_gauge_increases_on_pg() {
    let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_normal(), 100.0, 100);
    let initial = gauge.value();

    gauge.update(JudgeRank::PerfectGreat);

    assert!(gauge.value() > initial);
}

/// Test that hard gauge decreases on miss.
#[test]
fn test_hard_gauge_decreases_on_miss() {
    let mut gauge = GrooveGauge::new(GaugeProperty::sevenkeys_hard(), 100.0, 100);
    let initial = gauge.value();

    gauge.update(JudgeRank::Miss);

    assert!(gauge.value() < initial);
}

/// Test that score tracks all judge results.
#[test]
fn test_score_tracking() {
    let mut score = Score::new(6);

    score.update(JudgeRank::PerfectGreat);
    score.update(JudgeRank::Great);
    score.update(JudgeRank::Good);
    score.update(JudgeRank::Bad);
    score.update(JudgeRank::Poor);
    score.update(JudgeRank::Miss);

    assert_eq!(score.pg_count, 1);
    assert_eq!(score.gr_count, 1);
    assert_eq!(score.gd_count, 1);
    assert_eq!(score.bd_count, 1);
    assert_eq!(score.pr_count, 1);
    assert_eq!(score.ms_count, 1);
    assert_eq!(score.judged_count(), 6);
}

/// Test that audio config has sensible defaults.
#[test]
fn test_audio_config_defaults() {
    let config = AudioConfig::default();

    assert!(config.master_volume > 0.0);
    assert!(config.master_volume <= 1.0);
    assert!(config.buffer_size > 0);
    assert!(config.sample_rate > 0);
}

/// Test that key config has all lanes configured.
/// キー設定が全レーン分定義されていることを確認する。
#[test]
fn test_key_config_has_all_lanes() {
    let config = KeyConfig::default();

    assert_eq!(config.keyboard.lanes.len(), LANE_COUNT);
}
