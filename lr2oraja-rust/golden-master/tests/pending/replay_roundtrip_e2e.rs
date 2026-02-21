// Replay round-trip E2E tests: validates record → serialize → deserialize → replay.
//
// Tests that autoplay key logs can be recorded, saved as .brd files,
// loaded back, and replayed to produce identical scores.

use bms_replay::key_input_log::KeyInputLog;
use bms_replay::replay_data::{ReplayData, read_brd, write_brd};
use bms_rule::gauge_property::GaugeType;
use bms_rule::{JUDGE_MS, JUDGE_PG, JUDGE_PR};
use golden_master::e2e_helpers::*;
use tempfile::NamedTempFile;

/// Record autoplay key events from a simulation.
///
/// Generates a KeyInputLog from the BMS note data as if the player hit
/// every note perfectly (offset 0).
fn record_autoplay_keylog(model: &bms_model::BmsModel) -> Vec<KeyInputLog> {
    create_note_press_log(&model.notes, model.mode, 0)
}

// ============================================================================
// Record and replay match tests
// ============================================================================

/// Autoplay-generated keylog, when replayed manually, should produce the same score.
#[test]
fn record_and_replay_match() {
    let model = load_bms("minimal_7k.bms");
    let normal = count_normal_notes(&model);

    // Record: generate key log as if perfect manual play
    let keylog = record_autoplay_keylog(&model);
    assert!(!keylog.is_empty(), "Should have key events");

    // Replay: use the keylog in manual simulation
    let manual_result = run_manual_simulation(&model, &keylog, GaugeType::Normal);

    // Both should have all PGREAT for normal notes
    assert_eq!(
        manual_result.score.judge_count(JUDGE_PG),
        normal as i32,
        "Replayed keylog should produce all PG (PG={}, total_judge={})",
        manual_result.score.judge_count(JUDGE_PG),
        manual_result.score.total_judge_count(),
    );

    // Gauge should be qualified
    assert!(
        manual_result.gauge_qualified,
        "Replayed should be qualified (gauge={})",
        manual_result.gauge_value
    );
}

// ============================================================================
// BRD round-trip tests
// ============================================================================

/// ReplayData serialization round-trip: write_brd → read_brd produces identical keylog.
#[test]
fn replay_brd_round_trip() {
    let model = load_bms("minimal_7k.bms");
    let keylog = record_autoplay_keylog(&model);

    let mut replay = ReplayData {
        player: "test".to_string(),
        sha256: "abc123".to_string(),
        mode: model.mode as i32,
        keylog: keylog.clone(),
        gauge: GaugeType::Normal as i32,
        ..Default::default()
    };

    // Write to temp file
    let tmp = NamedTempFile::new().expect("Failed to create temp file");
    let tmp_path = tmp.path().to_path_buf();
    write_brd(&mut replay, &tmp_path).expect("Failed to write .brd");

    // Read back
    let loaded = read_brd(&tmp_path).expect("Failed to read .brd");

    // Verify key log matches
    assert_eq!(
        loaded.keylog.len(),
        keylog.len(),
        "Loaded keylog length should match original"
    );
    for (i, (original, loaded_entry)) in keylog.iter().zip(loaded.keylog.iter()).enumerate() {
        assert_eq!(
            original.presstime, loaded_entry.presstime,
            "Entry {i}: presstime mismatch"
        );
        assert_eq!(
            original.keycode, loaded_entry.keycode,
            "Entry {i}: keycode mismatch"
        );
        assert_eq!(
            original.pressed, loaded_entry.pressed,
            "Entry {i}: pressed mismatch"
        );
    }

    // Verify metadata
    assert_eq!(loaded.player, "test");
    assert_eq!(loaded.sha256, "abc123");
    assert_eq!(loaded.mode, model.mode as i32);
}

/// BRD round-trip preserves playback: loaded keylog produces same simulation result.
#[test]
fn replay_brd_playback_matches() {
    let model = load_bms("minimal_7k.bms");
    let keylog = record_autoplay_keylog(&model);

    // Original simulation
    let original_result = run_manual_simulation(&model, &keylog, GaugeType::Normal);

    // Round-trip through BRD
    let mut replay = ReplayData {
        mode: model.mode as i32,
        keylog: keylog.clone(),
        ..Default::default()
    };
    let tmp = NamedTempFile::new().unwrap();
    write_brd(&mut replay, tmp.path()).unwrap();
    let loaded = read_brd(tmp.path()).unwrap();

    // Replay with loaded keylog
    let loaded_result = run_manual_simulation(&model, &loaded.keylog, GaugeType::Normal);

    // Scores should match exactly
    assert_eq!(
        original_result.score.judge_count(JUDGE_PG),
        loaded_result.score.judge_count(JUDGE_PG),
        "PG count should match after BRD round-trip"
    );
    assert_eq!(
        original_result.max_combo, loaded_result.max_combo,
        "Max combo should match after BRD round-trip"
    );
    assert_eq!(
        original_result.ghost, loaded_result.ghost,
        "Ghost data should match after BRD round-trip"
    );
}

// ============================================================================
// Same input, different gauge tests
// ============================================================================

/// Same keylog with different gauge types should produce identical judgements
/// but potentially different gauge values and qualification results.
#[test]
fn replay_different_gauge_same_input() {
    let model = load_bms("minimal_7k.bms");
    let keylog = record_autoplay_keylog(&model);

    let normal_result = run_manual_simulation(&model, &keylog, GaugeType::Normal);
    let hard_result = run_manual_simulation(&model, &keylog, GaugeType::Hard);
    let exhard_result = run_manual_simulation(&model, &keylog, GaugeType::ExHard);

    // Judgements should be identical across all gauge types
    assert_eq!(
        normal_result.score.judge_count(JUDGE_PG),
        hard_result.score.judge_count(JUDGE_PG),
        "PG count should be same for Normal vs Hard"
    );
    assert_eq!(
        normal_result.score.judge_count(JUDGE_PG),
        exhard_result.score.judge_count(JUDGE_PG),
        "PG count should be same for Normal vs ExHard"
    );

    // Max combo should be the same
    assert_eq!(
        normal_result.max_combo, hard_result.max_combo,
        "Max combo should be same across gauge types"
    );

    // Ghost data should be identical
    assert_eq!(
        normal_result.ghost, hard_result.ghost,
        "Ghost should be same for Normal vs Hard"
    );
    assert_eq!(
        normal_result.ghost, exhard_result.ghost,
        "Ghost should be same for Normal vs ExHard"
    );

    // All should be qualified (perfect input)
    assert!(normal_result.gauge_qualified, "Normal should be qualified");
    assert!(hard_result.gauge_qualified, "Hard should be qualified");
    assert!(exhard_result.gauge_qualified, "ExHard should be qualified");
}

/// Same keylog (all-miss) with different gauge types: scores same, gauge results differ.
#[test]
fn replay_different_gauge_all_miss() {
    let model = load_bms("minimal_7k.bms");
    let total = model.total_notes();

    let normal_result = run_manual_simulation(&model, &[], GaugeType::Normal);
    let hard_result = run_manual_simulation(&model, &[], GaugeType::Hard);
    let exhard_result = run_manual_simulation(&model, &[], GaugeType::ExHard);

    // All should have same miss count
    for (label, result) in [
        ("Normal", &normal_result),
        ("Hard", &hard_result),
        ("ExHard", &exhard_result),
    ] {
        let miss = result.score.judge_count(JUDGE_PR) + result.score.judge_count(JUDGE_MS);
        assert_eq!(miss, total as i32, "{label}: all notes should be MISS/PR");
    }

    // None should be qualified
    assert!(!normal_result.gauge_qualified);
    assert!(!hard_result.gauge_qualified);
    assert!(!exhard_result.gauge_qualified);

    // Hard/ExHard should have gauge = 0 (dead)
    assert!(
        hard_result.gauge_value < 1e-6,
        "Hard gauge should be 0 on all-miss"
    );
    assert!(
        exhard_result.gauge_value < 1e-6,
        "ExHard gauge should be 0 on all-miss"
    );
}
