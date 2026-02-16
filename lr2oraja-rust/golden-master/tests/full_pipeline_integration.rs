// Full pipeline integration tests: BMS → Judge → Score → Replay round-trip.
//
// Tests the complete data flow through multiple crates:
// bms-model → bms-rule → bms-replay → bms-rule (re-simulation)

use bms_replay::replay_data::{ReplayData, read_brd, write_brd};
use bms_rule::JUDGE_PG;
use bms_rule::gauge_property::GaugeType;
use golden_master::e2e_helpers::*;
use tempfile::NamedTempFile;

// ============================================================================
// Full pipeline: BMS → Judge → Score → Replay → Re-simulate
// ============================================================================

/// Complete pipeline: parse BMS, simulate with keylog, save replay, load replay,
/// re-simulate, and verify the results match exactly.
#[test]
fn bms_to_replay_full_pipeline() {
    let model = load_bms("minimal_7k.bms");
    let normal = count_normal_notes(&model);

    // Step 1: Generate perfect keylog
    let keylog = create_note_press_log(&model.notes, model.mode, 0);
    assert!(!keylog.is_empty());

    // Step 2: Simulate with keylog
    let original = run_manual_simulation(&model, &keylog, GaugeType::Normal);
    assert_eq!(original.score.judge_count(JUDGE_PG), normal as i32);

    // Step 3: Save to BRD
    let mut replay = ReplayData {
        sha256: "test_pipeline".to_string(),
        mode: model.mode as i32,
        keylog: keylog.clone(),
        gauge: GaugeType::Normal as i32,
        ..Default::default()
    };
    let tmp = NamedTempFile::new().unwrap();
    write_brd(&mut replay, tmp.path()).unwrap();

    // Step 4: Load from BRD
    let loaded = read_brd(tmp.path()).unwrap();

    // Step 5: Re-simulate with loaded keylog
    let replayed = run_manual_simulation(&model, &loaded.keylog, GaugeType::Normal);

    // Step 6: Verify exact match
    assert_eq!(
        original.score.judge_count(JUDGE_PG),
        replayed.score.judge_count(JUDGE_PG),
        "PG should match after full pipeline round-trip"
    );
    assert_eq!(
        original.max_combo, replayed.max_combo,
        "Max combo should match"
    );
    assert_eq!(original.ghost, replayed.ghost, "Ghost should match");
    assert!(
        (original.gauge_value - replayed.gauge_value).abs() < 1e-6,
        "Gauge value should match: {} vs {}",
        original.gauge_value,
        replayed.gauge_value
    );
}

/// Full pipeline with multiple BMS files to ensure generality.
#[test]
fn full_pipeline_multiple_bms() {
    let files = ["minimal_7k.bms", "5key.bms", "bpm_change.bms"];

    for filename in files {
        let model = load_bms(filename);
        let keylog = create_note_press_log(&model.notes, model.mode, 0);

        // Simulate
        let original = run_manual_simulation(&model, &keylog, GaugeType::Normal);

        // Round-trip through BRD
        let mut replay = ReplayData {
            mode: model.mode as i32,
            keylog: keylog.clone(),
            ..Default::default()
        };
        let tmp = NamedTempFile::new().unwrap();
        write_brd(&mut replay, tmp.path()).unwrap();
        let loaded = read_brd(tmp.path()).unwrap();

        // Re-simulate
        let replayed = run_manual_simulation(&model, &loaded.keylog, GaugeType::Normal);

        // Verify
        assert_eq!(
            original.score.judge_count(JUDGE_PG),
            replayed.score.judge_count(JUDGE_PG),
            "{filename}: PG mismatch after pipeline round-trip"
        );
        assert_eq!(
            original.max_combo, replayed.max_combo,
            "{filename}: max combo mismatch"
        );
        assert_eq!(original.ghost, replayed.ghost, "{filename}: ghost mismatch");
    }
}

// ============================================================================
// Judge rank variants: same input, different RANK produces different results
// ============================================================================

/// Parse BMS files with different #RANK values and verify that the same timing
/// offset produces different judge distributions.
#[test]
fn pipeline_judge_rank_affects_distribution() {
    // minimal_7k.bms has #RANK 2 (judgerank=75, moderate)
    // defexrank.bms has custom rank settings
    let model_rank2 = load_bms("minimal_7k.bms");

    // At 25ms offset: RANK 2 → within GR window (40ms), outside PG (18ms)
    let keylog_25ms = create_note_press_log(&model_rank2.notes, model_rank2.mode, 25_000);
    let result_25ms = run_manual_simulation(&model_rank2, &keylog_25ms, GaugeType::Normal);

    // At 0ms offset: all PG regardless of rank
    let keylog_0ms = create_note_press_log(&model_rank2.notes, model_rank2.mode, 0);
    let result_0ms = run_manual_simulation(&model_rank2, &keylog_0ms, GaugeType::Normal);

    // 0ms should have more PG than 25ms
    assert!(
        result_0ms.score.judge_count(JUDGE_PG) > result_25ms.score.judge_count(JUDGE_PG),
        "0ms offset should have more PG ({}) than 25ms offset ({})",
        result_0ms.score.judge_count(JUDGE_PG),
        result_25ms.score.judge_count(JUDGE_PG)
    );

    // 0ms should have 0 GR, while 25ms should have GR > 0
    assert_eq!(
        result_0ms.score.judge_count(bms_rule::JUDGE_GR),
        0,
        "0ms should have no GR"
    );
    assert!(
        result_25ms.score.judge_count(bms_rule::JUDGE_GR) > 0,
        "25ms should have some GR"
    );
}

// ============================================================================
// Cross-gauge pipeline: verify score consistency across gauge types
// ============================================================================

/// Same BMS + same keylog through full pipeline with all gauge types:
/// scores should be identical, only gauge values differ.
#[test]
fn pipeline_cross_gauge_score_consistency() {
    let model = load_bms("minimal_7k.bms");
    let keylog = create_note_press_log(&model.notes, model.mode, 0);

    let gauge_types = [
        GaugeType::Normal,
        GaugeType::Easy,
        GaugeType::Hard,
        GaugeType::ExHard,
        GaugeType::AssistEasy,
    ];

    let results: Vec<_> = gauge_types
        .iter()
        .map(|&gt| run_manual_simulation(&model, &keylog, gt))
        .collect();

    // All should have same PG count
    let reference_pg = results[0].score.judge_count(JUDGE_PG);
    for (gt, result) in gauge_types.iter().zip(results.iter()) {
        assert_eq!(
            result.score.judge_count(JUDGE_PG),
            reference_pg,
            "{gt:?}: PG count should match Normal's"
        );
    }

    // All should have same max combo
    let reference_combo = results[0].max_combo;
    for (gt, result) in gauge_types.iter().zip(results.iter()) {
        assert_eq!(
            result.max_combo, reference_combo,
            "{gt:?}: max combo should match Normal's"
        );
    }

    // All should have identical ghost
    for (gt, result) in gauge_types.iter().zip(results.iter()) {
        assert_eq!(
            result.ghost, results[0].ghost,
            "{gt:?}: ghost should match Normal's"
        );
    }
}
