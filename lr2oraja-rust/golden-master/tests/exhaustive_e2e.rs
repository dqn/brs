// Exhaustive E2E tests: 4 PlayModes x 6 GaugeTypes x 3 InputModes = 72 tests
//
// Validates invariants across the full combination matrix:
// - Autoplay: all PG, gauge qualified, max_combo >= total_notes
// - ManualPerfect: all PG, gauge qualified (normal notes only, 0ms offset)
// - ManualAllMiss: all PR/MS, max_combo=0, gauge not qualified

use bms_rule::gauge_property::GaugeType;
use bms_rule::{JUDGE_MS, JUDGE_PG, JUDGE_PR};
use golden_master::e2e_helpers::*;

fn run_autoplay_test(bms_file: &str, gauge_type: GaugeType, label: &str) {
    let model = load_bms(bms_file);
    let total = model.total_notes();
    assert!(total > 0, "{label}: should have playable notes");

    let result = run_autoplay_simulation(&model, gauge_type);
    assert_all_pgreat(&result, total, label);
}

fn run_manual_perfect_test(bms_file: &str, gauge_type: GaugeType, label: &str) {
    let model = load_bms(bms_file);
    let normal = count_normal_notes(&model);
    assert!(normal > 0, "{label}: should have normal notes");

    let log = create_note_press_log(&model.notes, model.mode, 0);
    let result = run_manual_simulation(&model, &log, gauge_type);

    let score = &result.score;
    assert_eq!(
        score.judge_count(JUDGE_PG),
        normal as i32,
        "{label}: all normal notes should be PG (PG={}, total_judge={})",
        score.judge_count(JUDGE_PG),
        score.total_judge_count()
    );
    assert!(
        result.gauge_qualified,
        "{label}: gauge should be qualified (value={})",
        result.gauge_value
    );
    assert!(
        result.max_combo >= normal as i32,
        "{label}: max_combo {} < normal_notes {}",
        result.max_combo,
        normal
    );
}

fn run_manual_all_miss_test(bms_file: &str, gauge_type: GaugeType, label: &str) {
    let model = load_bms(bms_file);
    let total = model.total_notes();
    assert!(total > 0, "{label}: should have playable notes");

    let result = run_manual_simulation(&model, &[], gauge_type);

    let score = &result.score;
    let miss_count = score.judge_count(JUDGE_PR) + score.judge_count(JUDGE_MS);
    assert_eq!(
        miss_count,
        total as i32,
        "{label}: all notes should be PR/MS (PR={}, MS={}, total={})",
        score.judge_count(JUDGE_PR),
        score.judge_count(JUDGE_MS),
        total
    );
    assert_eq!(result.max_combo, 0, "{label}: max_combo should be 0");
    assert!(
        !result.gauge_qualified,
        "{label}: gauge should NOT be qualified (value={})",
        result.gauge_value
    );
}

// ============================================================================
// Beat5K (5key.bms): 6 gauges x 3 inputs = 18 tests
// ============================================================================

#[test]
fn beat5k_assist_easy_autoplay() {
    run_autoplay_test(
        "5key.bms",
        GaugeType::AssistEasy,
        "beat5k_assist_easy_autoplay",
    );
}
#[test]
fn beat5k_easy_autoplay() {
    run_autoplay_test("5key.bms", GaugeType::Easy, "beat5k_easy_autoplay");
}
#[test]
fn beat5k_normal_autoplay() {
    run_autoplay_test("5key.bms", GaugeType::Normal, "beat5k_normal_autoplay");
}
#[test]
fn beat5k_hard_autoplay() {
    run_autoplay_test("5key.bms", GaugeType::Hard, "beat5k_hard_autoplay");
}
#[test]
fn beat5k_exhard_autoplay() {
    run_autoplay_test("5key.bms", GaugeType::ExHard, "beat5k_exhard_autoplay");
}
#[test]
fn beat5k_hazard_autoplay() {
    run_autoplay_test("5key.bms", GaugeType::Hazard, "beat5k_hazard_autoplay");
}

#[test]
fn beat5k_assist_easy_manual_perfect() {
    run_manual_perfect_test(
        "5key.bms",
        GaugeType::AssistEasy,
        "beat5k_assist_easy_manual_perfect",
    );
}
#[test]
fn beat5k_easy_manual_perfect() {
    run_manual_perfect_test("5key.bms", GaugeType::Easy, "beat5k_easy_manual_perfect");
}
#[test]
fn beat5k_normal_manual_perfect() {
    run_manual_perfect_test(
        "5key.bms",
        GaugeType::Normal,
        "beat5k_normal_manual_perfect",
    );
}
#[test]
fn beat5k_hard_manual_perfect() {
    run_manual_perfect_test("5key.bms", GaugeType::Hard, "beat5k_hard_manual_perfect");
}
#[test]
fn beat5k_exhard_manual_perfect() {
    run_manual_perfect_test(
        "5key.bms",
        GaugeType::ExHard,
        "beat5k_exhard_manual_perfect",
    );
}
#[test]
fn beat5k_hazard_manual_perfect() {
    run_manual_perfect_test(
        "5key.bms",
        GaugeType::Hazard,
        "beat5k_hazard_manual_perfect",
    );
}

#[test]
fn beat5k_assist_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "5key.bms",
        GaugeType::AssistEasy,
        "beat5k_assist_easy_manual_all_miss",
    );
}
#[test]
fn beat5k_easy_manual_all_miss() {
    run_manual_all_miss_test("5key.bms", GaugeType::Easy, "beat5k_easy_manual_all_miss");
}
#[test]
fn beat5k_normal_manual_all_miss() {
    run_manual_all_miss_test(
        "5key.bms",
        GaugeType::Normal,
        "beat5k_normal_manual_all_miss",
    );
}
#[test]
fn beat5k_hard_manual_all_miss() {
    run_manual_all_miss_test("5key.bms", GaugeType::Hard, "beat5k_hard_manual_all_miss");
}
#[test]
fn beat5k_exhard_manual_all_miss() {
    run_manual_all_miss_test(
        "5key.bms",
        GaugeType::ExHard,
        "beat5k_exhard_manual_all_miss",
    );
}
#[test]
fn beat5k_hazard_manual_all_miss() {
    run_manual_all_miss_test(
        "5key.bms",
        GaugeType::Hazard,
        "beat5k_hazard_manual_all_miss",
    );
}

// ============================================================================
// Beat7K (minimal_7k.bms): 6 gauges x 3 inputs = 18 tests
// ============================================================================

#[test]
fn beat7k_assist_easy_autoplay() {
    run_autoplay_test(
        "minimal_7k.bms",
        GaugeType::AssistEasy,
        "beat7k_assist_easy_autoplay",
    );
}
#[test]
fn beat7k_easy_autoplay() {
    run_autoplay_test("minimal_7k.bms", GaugeType::Easy, "beat7k_easy_autoplay");
}
#[test]
fn beat7k_normal_autoplay() {
    run_autoplay_test(
        "minimal_7k.bms",
        GaugeType::Normal,
        "beat7k_normal_autoplay",
    );
}
#[test]
fn beat7k_hard_autoplay() {
    run_autoplay_test("minimal_7k.bms", GaugeType::Hard, "beat7k_hard_autoplay");
}
#[test]
fn beat7k_exhard_autoplay() {
    run_autoplay_test(
        "minimal_7k.bms",
        GaugeType::ExHard,
        "beat7k_exhard_autoplay",
    );
}
#[test]
fn beat7k_hazard_autoplay() {
    run_autoplay_test(
        "minimal_7k.bms",
        GaugeType::Hazard,
        "beat7k_hazard_autoplay",
    );
}

#[test]
fn beat7k_assist_easy_manual_perfect() {
    run_manual_perfect_test(
        "minimal_7k.bms",
        GaugeType::AssistEasy,
        "beat7k_assist_easy_manual_perfect",
    );
}
#[test]
fn beat7k_easy_manual_perfect() {
    run_manual_perfect_test(
        "minimal_7k.bms",
        GaugeType::Easy,
        "beat7k_easy_manual_perfect",
    );
}
#[test]
fn beat7k_normal_manual_perfect() {
    run_manual_perfect_test(
        "minimal_7k.bms",
        GaugeType::Normal,
        "beat7k_normal_manual_perfect",
    );
}
#[test]
fn beat7k_hard_manual_perfect() {
    run_manual_perfect_test(
        "minimal_7k.bms",
        GaugeType::Hard,
        "beat7k_hard_manual_perfect",
    );
}
#[test]
fn beat7k_exhard_manual_perfect() {
    run_manual_perfect_test(
        "minimal_7k.bms",
        GaugeType::ExHard,
        "beat7k_exhard_manual_perfect",
    );
}
#[test]
fn beat7k_hazard_manual_perfect() {
    run_manual_perfect_test(
        "minimal_7k.bms",
        GaugeType::Hazard,
        "beat7k_hazard_manual_perfect",
    );
}

#[test]
fn beat7k_assist_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "minimal_7k.bms",
        GaugeType::AssistEasy,
        "beat7k_assist_easy_manual_all_miss",
    );
}
#[test]
fn beat7k_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "minimal_7k.bms",
        GaugeType::Easy,
        "beat7k_easy_manual_all_miss",
    );
}
#[test]
fn beat7k_normal_manual_all_miss() {
    run_manual_all_miss_test(
        "minimal_7k.bms",
        GaugeType::Normal,
        "beat7k_normal_manual_all_miss",
    );
}
#[test]
fn beat7k_hard_manual_all_miss() {
    run_manual_all_miss_test(
        "minimal_7k.bms",
        GaugeType::Hard,
        "beat7k_hard_manual_all_miss",
    );
}
#[test]
fn beat7k_exhard_manual_all_miss() {
    run_manual_all_miss_test(
        "minimal_7k.bms",
        GaugeType::ExHard,
        "beat7k_exhard_manual_all_miss",
    );
}
#[test]
fn beat7k_hazard_manual_all_miss() {
    run_manual_all_miss_test(
        "minimal_7k.bms",
        GaugeType::Hazard,
        "beat7k_hazard_manual_all_miss",
    );
}

// ============================================================================
// Beat14K (14key_dp.bms): 6 gauges x 3 inputs = 18 tests
// ============================================================================

#[test]
fn beat14k_assist_easy_autoplay() {
    run_autoplay_test(
        "14key_dp.bms",
        GaugeType::AssistEasy,
        "beat14k_assist_easy_autoplay",
    );
}
#[test]
fn beat14k_easy_autoplay() {
    run_autoplay_test("14key_dp.bms", GaugeType::Easy, "beat14k_easy_autoplay");
}
#[test]
fn beat14k_normal_autoplay() {
    run_autoplay_test("14key_dp.bms", GaugeType::Normal, "beat14k_normal_autoplay");
}
#[test]
fn beat14k_hard_autoplay() {
    run_autoplay_test("14key_dp.bms", GaugeType::Hard, "beat14k_hard_autoplay");
}
#[test]
fn beat14k_exhard_autoplay() {
    run_autoplay_test("14key_dp.bms", GaugeType::ExHard, "beat14k_exhard_autoplay");
}
#[test]
fn beat14k_hazard_autoplay() {
    run_autoplay_test("14key_dp.bms", GaugeType::Hazard, "beat14k_hazard_autoplay");
}

#[test]
fn beat14k_assist_easy_manual_perfect() {
    run_manual_perfect_test(
        "14key_dp.bms",
        GaugeType::AssistEasy,
        "beat14k_assist_easy_manual_perfect",
    );
}
#[test]
fn beat14k_easy_manual_perfect() {
    run_manual_perfect_test(
        "14key_dp.bms",
        GaugeType::Easy,
        "beat14k_easy_manual_perfect",
    );
}
#[test]
fn beat14k_normal_manual_perfect() {
    run_manual_perfect_test(
        "14key_dp.bms",
        GaugeType::Normal,
        "beat14k_normal_manual_perfect",
    );
}
#[test]
fn beat14k_hard_manual_perfect() {
    run_manual_perfect_test(
        "14key_dp.bms",
        GaugeType::Hard,
        "beat14k_hard_manual_perfect",
    );
}
#[test]
fn beat14k_exhard_manual_perfect() {
    run_manual_perfect_test(
        "14key_dp.bms",
        GaugeType::ExHard,
        "beat14k_exhard_manual_perfect",
    );
}
#[test]
fn beat14k_hazard_manual_perfect() {
    run_manual_perfect_test(
        "14key_dp.bms",
        GaugeType::Hazard,
        "beat14k_hazard_manual_perfect",
    );
}

#[test]
fn beat14k_assist_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "14key_dp.bms",
        GaugeType::AssistEasy,
        "beat14k_assist_easy_manual_all_miss",
    );
}
#[test]
fn beat14k_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "14key_dp.bms",
        GaugeType::Easy,
        "beat14k_easy_manual_all_miss",
    );
}
#[test]
fn beat14k_normal_manual_all_miss() {
    run_manual_all_miss_test(
        "14key_dp.bms",
        GaugeType::Normal,
        "beat14k_normal_manual_all_miss",
    );
}
#[test]
fn beat14k_hard_manual_all_miss() {
    run_manual_all_miss_test(
        "14key_dp.bms",
        GaugeType::Hard,
        "beat14k_hard_manual_all_miss",
    );
}
#[test]
fn beat14k_exhard_manual_all_miss() {
    run_manual_all_miss_test(
        "14key_dp.bms",
        GaugeType::ExHard,
        "beat14k_exhard_manual_all_miss",
    );
}
#[test]
fn beat14k_hazard_manual_all_miss() {
    run_manual_all_miss_test(
        "14key_dp.bms",
        GaugeType::Hazard,
        "beat14k_hazard_manual_all_miss",
    );
}

// ============================================================================
// PopN9K (9key_pms.pms): 6 gauges x 3 inputs = 18 tests
// ============================================================================

#[test]
fn popn9k_assist_easy_autoplay() {
    run_autoplay_test(
        "9key_pms.pms",
        GaugeType::AssistEasy,
        "popn9k_assist_easy_autoplay",
    );
}
#[test]
fn popn9k_easy_autoplay() {
    run_autoplay_test("9key_pms.pms", GaugeType::Easy, "popn9k_easy_autoplay");
}
#[test]
fn popn9k_normal_autoplay() {
    run_autoplay_test("9key_pms.pms", GaugeType::Normal, "popn9k_normal_autoplay");
}
#[test]
fn popn9k_hard_autoplay() {
    run_autoplay_test("9key_pms.pms", GaugeType::Hard, "popn9k_hard_autoplay");
}
#[test]
fn popn9k_exhard_autoplay() {
    run_autoplay_test("9key_pms.pms", GaugeType::ExHard, "popn9k_exhard_autoplay");
}
#[test]
fn popn9k_hazard_autoplay() {
    run_autoplay_test("9key_pms.pms", GaugeType::Hazard, "popn9k_hazard_autoplay");
}

#[test]
fn popn9k_assist_easy_manual_perfect() {
    run_manual_perfect_test(
        "9key_pms.pms",
        GaugeType::AssistEasy,
        "popn9k_assist_easy_manual_perfect",
    );
}
#[test]
fn popn9k_easy_manual_perfect() {
    run_manual_perfect_test(
        "9key_pms.pms",
        GaugeType::Easy,
        "popn9k_easy_manual_perfect",
    );
}
#[test]
fn popn9k_normal_manual_perfect() {
    run_manual_perfect_test(
        "9key_pms.pms",
        GaugeType::Normal,
        "popn9k_normal_manual_perfect",
    );
}
#[test]
fn popn9k_hard_manual_perfect() {
    run_manual_perfect_test(
        "9key_pms.pms",
        GaugeType::Hard,
        "popn9k_hard_manual_perfect",
    );
}
#[test]
fn popn9k_exhard_manual_perfect() {
    run_manual_perfect_test(
        "9key_pms.pms",
        GaugeType::ExHard,
        "popn9k_exhard_manual_perfect",
    );
}
#[test]
fn popn9k_hazard_manual_perfect() {
    run_manual_perfect_test(
        "9key_pms.pms",
        GaugeType::Hazard,
        "popn9k_hazard_manual_perfect",
    );
}

#[test]
fn popn9k_assist_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "9key_pms.pms",
        GaugeType::AssistEasy,
        "popn9k_assist_easy_manual_all_miss",
    );
}
#[test]
fn popn9k_easy_manual_all_miss() {
    run_manual_all_miss_test(
        "9key_pms.pms",
        GaugeType::Easy,
        "popn9k_easy_manual_all_miss",
    );
}
#[test]
fn popn9k_normal_manual_all_miss() {
    run_manual_all_miss_test(
        "9key_pms.pms",
        GaugeType::Normal,
        "popn9k_normal_manual_all_miss",
    );
}
#[test]
fn popn9k_hard_manual_all_miss() {
    run_manual_all_miss_test(
        "9key_pms.pms",
        GaugeType::Hard,
        "popn9k_hard_manual_all_miss",
    );
}
#[test]
fn popn9k_exhard_manual_all_miss() {
    run_manual_all_miss_test(
        "9key_pms.pms",
        GaugeType::ExHard,
        "popn9k_exhard_manual_all_miss",
    );
}
#[test]
fn popn9k_hazard_manual_all_miss() {
    run_manual_all_miss_test(
        "9key_pms.pms",
        GaugeType::Hazard,
        "popn9k_hazard_manual_all_miss",
    );
}
