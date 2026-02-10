// E2E integration tests: BMS parse → JudgeManager → GrooveGauge → ScoreData
//
// Validates that the full pipeline works correctly with real BMS files.
// Uses invariant-based assertions rather than golden-master comparison.
//
// Notes:
// - JudgeManager.prev_time starts at 0, so notes at time_us=0 are skipped on
//   the first frame. We prime the JudgeManager with update(-1) to work around this.
// - LN notes are split into start+end pairs via build_judge_notes() for JudgeManager.

use std::path::Path;

use bms_model::{BmsDecoder, BmsModel, LaneProperty};
use bms_replay::key_input_log::KeyInputLog;
use bms_rule::gauge_property::GaugeType;
use bms_rule::judge_manager::{JudgeConfig, JudgeManager};
use bms_rule::{
    GrooveGauge, JUDGE_BD, JUDGE_GD, JUDGE_GR, JUDGE_MS, JUDGE_PG, JUDGE_PR, JudgeAlgorithm,
    PlayerRule,
};

/// Sentinel for "not set" timestamps (matches JudgeManager internal).
const NOT_SET: i64 = i64::MIN;

/// Frame step for simulation (1ms = 1000μs).
const FRAME_STEP: i64 = 1_000;

/// Extra time after last note to finish simulation (1 second).
const TAIL_TIME: i64 = 1_000_000;

fn test_bms_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-bms")
        .leak()
}

fn load_bms(filename: &str) -> BmsModel {
    let path = test_bms_dir().join(filename);
    BmsDecoder::decode(&path).unwrap_or_else(|e| panic!("Failed to parse {filename}: {e}"))
}

struct SimulationResult {
    score: bms_rule::ScoreData,
    max_combo: i32,
    ghost: Vec<usize>,
    gauge_value: f32,
    gauge_qualified: bool,
}

/// Count normal (non-LN) playable notes in the model.
fn count_normal_notes(model: &BmsModel) -> usize {
    model
        .notes
        .iter()
        .filter(|n| n.is_playable() && !n.is_long_note())
        .count()
}

/// Run autoplay simulation: JudgeManager with autoplay=true, empty key inputs.
fn run_autoplay_simulation(model: &BmsModel, gauge_type: GaugeType) -> SimulationResult {
    let judge_notes = model.build_judge_notes();
    let rule = PlayerRule::lr2();
    let total_notes = judge_notes.iter().filter(|n| n.is_playable()).count();
    let total = if model.total > 0.0 {
        model.total
    } else {
        PlayerRule::default_total(total_notes)
    };

    let judge_rank = rule
        .judge
        .window_rule
        .resolve_judge_rank(model.judge_rank, model.judge_rank_type);
    let config = JudgeConfig {
        notes: &judge_notes,
        play_mode: model.mode,
        ln_type: model.ln_type,
        judge_rank,
        judge_window_rate: [100, 100, 100],
        scratch_judge_window_rate: [100, 100, 100],
        algorithm: JudgeAlgorithm::Combo,
        autoplay: true,
        judge_property: &rule.judge,
        lane_property: None,
    };

    let mut jm = JudgeManager::new(&config);
    let mut gauge = GrooveGauge::new(&rule.gauge, gauge_type, total, total_notes);

    let lp = LaneProperty::new(model.mode);
    let physical_key_count = lp.physical_key_count();
    let key_states = vec![false; physical_key_count];
    let key_times = vec![NOT_SET; physical_key_count];

    // Prime JudgeManager: set prev_time to -1 so notes at time_us=0 are not skipped.
    // JudgeManager.prev_time starts at 0, causing `note.time_us <= self.prev_time`
    // to be true for notes at time 0. This priming update sets prev_time = -1.
    jm.update(-1, &judge_notes, &key_states, &key_times, &mut gauge);

    let last_note_time = judge_notes
        .iter()
        .map(|n| n.time_us.max(n.end_time_us))
        .max()
        .unwrap_or(0);
    let end_time = last_note_time + TAIL_TIME;

    let mut time = 0i64;
    while time <= end_time {
        jm.update(time, &judge_notes, &key_states, &key_times, &mut gauge);
        time += FRAME_STEP;
    }

    SimulationResult {
        score: jm.score().clone(),
        max_combo: jm.max_combo(),
        ghost: jm.ghost().to_vec(),
        gauge_value: gauge.value(),
        gauge_qualified: gauge.is_qualified(),
    }
}

/// Create simple press+release input events for each playable note.
/// For normal notes: press at note time, release 80ms later.
/// LN notes are not generated here (handled by autoplay in JudgeManager).
fn create_note_press_log(notes: &[bms_model::Note], offset_us: i64) -> Vec<KeyInputLog> {
    let mut log = Vec::new();
    for note in notes {
        if !note.is_playable() {
            continue;
        }
        if note.is_long_note() {
            // Skip LN start/end notes for manual tests — manual LN tests
            // would need coordinated press/release timing
            continue;
        }
        log.push(KeyInputLog::new(
            note.time_us + offset_us,
            note.lane as i32,
            true,
        ));
        // Release 80ms after press
        log.push(KeyInputLog::new(
            note.time_us + offset_us + 80_000,
            note.lane as i32,
            false,
        ));
    }
    log
}

/// Run manual simulation with per-frame key state conversion.
fn run_manual_simulation(
    model: &BmsModel,
    input_log: &[KeyInputLog],
    gauge_type: GaugeType,
) -> SimulationResult {
    let judge_notes = model.build_judge_notes();
    let rule = PlayerRule::lr2();
    let total_notes = judge_notes.iter().filter(|n| n.is_playable()).count();
    let total = if model.total > 0.0 {
        model.total
    } else {
        PlayerRule::default_total(total_notes)
    };

    let judge_rank = rule
        .judge
        .window_rule
        .resolve_judge_rank(model.judge_rank, model.judge_rank_type);
    let config = JudgeConfig {
        notes: &judge_notes,
        play_mode: model.mode,
        ln_type: model.ln_type,
        judge_rank,
        judge_window_rate: [100, 100, 100],
        scratch_judge_window_rate: [100, 100, 100],
        algorithm: JudgeAlgorithm::Combo,
        autoplay: false,
        judge_property: &rule.judge,
        lane_property: None,
    };

    let mut jm = JudgeManager::new(&config);
    let mut gauge = GrooveGauge::new(&rule.gauge, gauge_type, total, total_notes);

    let lp = LaneProperty::new(model.mode);
    let physical_key_count = lp.physical_key_count();

    let mut sorted_log: Vec<&KeyInputLog> = input_log.iter().collect();
    sorted_log.sort_by_key(|e| e.get_time());

    let last_note_time = judge_notes
        .iter()
        .map(|n| n.time_us.max(n.end_time_us))
        .max()
        .unwrap_or(0);
    let end_time = last_note_time + TAIL_TIME;

    let mut key_states = vec![false; physical_key_count];
    let mut log_cursor = 0;

    // Prime JudgeManager for notes at time 0
    let empty_key_times = vec![NOT_SET; physical_key_count];
    jm.update(-1, &judge_notes, &key_states, &empty_key_times, &mut gauge);

    let mut time = 0i64;
    while time <= end_time {
        let mut key_changed_times = vec![NOT_SET; physical_key_count];

        // Input log uses lane indices (keycodes); map directly to physical key indices.
        // For Beat7K, lanes 0-6 map 1:1, lane 7 maps to physical key 7.
        while log_cursor < sorted_log.len() && sorted_log[log_cursor].get_time() <= time {
            let event = sorted_log[log_cursor];
            let key = event.keycode as usize;
            if key < physical_key_count {
                key_states[key] = event.pressed;
                key_changed_times[key] = event.get_time();
            }
            log_cursor += 1;
        }

        jm.update(
            time,
            &judge_notes,
            &key_states,
            &key_changed_times,
            &mut gauge,
        );
        time += FRAME_STEP;
    }

    SimulationResult {
        score: jm.score().clone(),
        max_combo: jm.max_combo(),
        ghost: jm.ghost().to_vec(),
        gauge_value: gauge.value(),
        gauge_qualified: gauge.is_qualified(),
    }
}

/// Assert the autoplay invariant: all notes are PGREAT.
fn assert_all_pgreat(result: &SimulationResult, total_notes: usize, label: &str) {
    let score = &result.score;
    let pg_count = score.judge_count(JUDGE_PG);

    assert!(
        pg_count > 0,
        "{label}: expected PG count > 0, got {pg_count}"
    );

    for &judge in &[JUDGE_GR, JUDGE_GD, JUDGE_BD, JUDGE_PR, JUDGE_MS] {
        assert_eq!(
            score.judge_count(judge),
            0,
            "{label}: judge {judge} should be 0, got {} \
             (PG={}, GR={}, GD={}, BD={}, PR={}, MS={})",
            score.judge_count(judge),
            score.judge_count(JUDGE_PG),
            score.judge_count(JUDGE_GR),
            score.judge_count(JUDGE_GD),
            score.judge_count(JUDGE_BD),
            score.judge_count(JUDGE_PR),
            score.judge_count(JUDGE_MS),
        );
    }

    assert!(
        result.max_combo >= total_notes as i32,
        "{label}: max_combo {} < total_notes {}",
        result.max_combo,
        total_notes
    );

    for (i, &g) in result.ghost.iter().enumerate() {
        assert_eq!(g, JUDGE_PG, "{label}: ghost[{i}] = {g}, expected PG (0)");
    }

    assert!(
        result.gauge_qualified,
        "{label}: gauge not qualified (value={})",
        result.gauge_value
    );
}

// ============================================================================
// Group A: Autoplay tests — perfect play invariants (normal notes only)
// ============================================================================

#[test]
fn autoplay_minimal_7k() {
    let model = load_bms("minimal_7k.bms");
    let total = model.total_notes();
    assert!(total > 0, "minimal_7k should have playable notes");
    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert_all_pgreat(&result, total, "autoplay_minimal_7k");
}

#[test]
fn autoplay_5key() {
    let model = load_bms("5key.bms");
    let total = model.total_notes();
    assert!(total > 0);
    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert_all_pgreat(&result, total, "autoplay_5key");
}

#[test]
fn autoplay_14key_dp() {
    let model = load_bms("14key_dp.bms");
    let total = model.total_notes();
    assert!(total > 0);
    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert_all_pgreat(&result, total, "autoplay_14key_dp");
}

#[test]
fn autoplay_9key_pms() {
    let model = load_bms("9key_pms.bms");
    let total = model.total_notes();
    assert!(total > 0);
    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert_all_pgreat(&result, total, "autoplay_9key_pms");
}

#[test]
fn autoplay_bpm_change() {
    let model = load_bms("bpm_change.bms");
    let total = model.total_notes();
    assert!(total > 0);
    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert_all_pgreat(&result, total, "autoplay_bpm_change");
}

#[test]
fn autoplay_mine_no_damage() {
    let model = load_bms("mine_notes.bms");
    let total = model.total_notes();
    assert!(total > 0, "mine_notes should have playable notes");

    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert_all_pgreat(&result, total, "autoplay_mine_no_damage");

    assert!(
        result.gauge_value > 0.0,
        "Gauge should be alive (no mine damage in autoplay)"
    );
}

// LN autoplay tests: build_judge_notes() splits LN into start+end pairs with
// pair_index, so autoplay correctly tracks LN start→end as all PGREAT.
// Pure LN end notes are not independently judged (1 judgment per LN pair),
// so we use ghost.len() as the expected total rather than raw playable count.

#[test]
fn autoplay_longnote() {
    let model = load_bms("longnote_types.bms");
    let long_notes = model.total_long_notes();
    assert!(long_notes > 0, "longnote_types should have LN notes");

    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    // ghost.len() reflects actually-judged notes (excludes pure LN end)
    let total = result.ghost.len();
    assert!(total > 0);
    assert_all_pgreat(&result, total, "autoplay_longnote");
}

#[test]
fn autoplay_scratch_bss() {
    let model = load_bms("scratch_bss.bms");

    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    // ghost.len() reflects actually-judged notes
    let total = result.ghost.len();
    assert!(total > 0);
    assert_all_pgreat(&result, total, "autoplay_scratch_bss");
}

// ============================================================================
// Group B: Manual input tests — timing offset affects judgment
// ============================================================================

#[test]
fn manual_perfect() {
    let model = load_bms("minimal_7k.bms");
    let normal = count_normal_notes(&model);
    // Create press events at exact note times (0 offset)
    let log = create_note_press_log(&model.notes, 0);
    let result = run_manual_simulation(&model, &log, GaugeType::Normal);

    let score = &result.score;
    assert_eq!(
        score.judge_count(JUDGE_PG),
        normal as i32,
        "All normal notes should be PG with exact timing"
    );
}

#[test]
fn manual_great() {
    let model = load_bms("minimal_7k.bms");
    // minimal_7k.bms has #RANK 2 → resolved judgerank=75 (LR2). Scaled windows:
    //   PG ±18ms, GR ±40ms, GD ±100ms, BD ±200ms
    // Offset by 25ms — within GR window (±40ms) but outside PG window (±18ms)
    let log = create_note_press_log(&model.notes, 25_000);
    let result = run_manual_simulation(&model, &log, GaugeType::Normal);

    let score = &result.score;
    assert_eq!(
        score.judge_count(JUDGE_BD),
        0,
        "No BAD expected at 25ms offset"
    );
    assert_eq!(
        score.judge_count(JUDGE_MS),
        0,
        "No MISS expected at 25ms offset"
    );
    assert!(
        score.judge_count(JUDGE_GR) > 0,
        "Expected some GR at 25ms offset (PG={}, GR={}, GD={})",
        score.judge_count(JUDGE_PG),
        score.judge_count(JUDGE_GR),
        score.judge_count(JUDGE_GD)
    );
}

#[test]
fn manual_good() {
    let model = load_bms("minimal_7k.bms");
    // minimal_7k.bms has #RANK 2 → resolved judgerank=75 (LR2). Scaled windows:
    //   PG ±18ms, GR ±40ms, GD ±100ms, BD ±200ms
    // Offset by 50ms — within GD window (±100ms) but outside GR window (±40ms)
    let log = create_note_press_log(&model.notes, 50_000);
    let result = run_manual_simulation(&model, &log, GaugeType::Normal);

    let score = &result.score;
    assert_eq!(
        score.judge_count(JUDGE_MS),
        0,
        "No MISS expected at 50ms offset"
    );
    assert!(
        score.judge_count(JUDGE_GD) > 0,
        "Expected some GD at 50ms offset (PG={}, GR={}, GD={}, BD={}, PR={}, MS={})",
        score.judge_count(JUDGE_PG),
        score.judge_count(JUDGE_GR),
        score.judge_count(JUDGE_GD),
        score.judge_count(JUDGE_BD),
        score.judge_count(JUDGE_PR),
        score.judge_count(JUDGE_MS),
    );
}

#[test]
fn manual_bad() {
    let model = load_bms("minimal_7k.bms");
    // minimal_7k.bms has #RANK 2 → resolved judgerank=75 (LR2). Scaled windows:
    //   PG ±18ms, GR ±40ms, GD ±100ms, BD ±200ms
    // Offset by 150ms — within BD window (±200ms) but outside GD window (±100ms)
    let log = create_note_press_log(&model.notes, 150_000);
    let result = run_manual_simulation(&model, &log, GaugeType::Normal);

    let score = &result.score;
    assert!(
        score.judge_count(JUDGE_BD) > 0 || score.judge_count(JUDGE_PR) > 0,
        "Expected some BD/PR at 150ms offset (PG={}, GR={}, GD={}, BD={}, PR={}, MS={})",
        score.judge_count(JUDGE_PG),
        score.judge_count(JUDGE_GR),
        score.judge_count(JUDGE_GD),
        score.judge_count(JUDGE_BD),
        score.judge_count(JUDGE_PR),
        score.judge_count(JUDGE_MS),
    );
}

#[test]
fn manual_all_miss() {
    let model = load_bms("minimal_7k.bms");
    let total = model.total_notes();
    // No input at all — all notes should be MISS
    let result = run_manual_simulation(&model, &[], GaugeType::Normal);

    let score = &result.score;
    let miss_count = score.judge_count(JUDGE_PR) + score.judge_count(JUDGE_MS);
    assert_eq!(
        miss_count,
        total as i32,
        "All notes should be MISS/PR with no input (PG={}, GR={}, GD={}, BD={}, PR={}, MS={})",
        score.judge_count(JUDGE_PG),
        score.judge_count(JUDGE_GR),
        score.judge_count(JUDGE_GD),
        score.judge_count(JUDGE_BD),
        score.judge_count(JUDGE_PR),
        score.judge_count(JUDGE_MS),
    );
    assert_eq!(result.max_combo, 0, "Max combo should be 0 with no input");
}

// ============================================================================
// Group C: Gauge integration tests
// ============================================================================

#[test]
fn gauge_normal_autoplay() {
    let model = load_bms("minimal_7k.bms");
    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    assert!(
        result.gauge_qualified,
        "Normal gauge should be qualified on autoplay (value={})",
        result.gauge_value
    );
}

#[test]
fn gauge_hard_autoplay() {
    let model = load_bms("minimal_7k.bms");
    let result = run_autoplay_simulation(&model, GaugeType::Hard);
    // Hard gauge starts at 100 and should not decrease with all PG
    assert!(
        (result.gauge_value - 100.0).abs() < 1e-3,
        "Hard gauge should stay at 100.0 on autoplay, got {}",
        result.gauge_value
    );
    assert!(result.gauge_qualified);
}

#[test]
fn gauge_exhard_all_miss() {
    let model = load_bms("minimal_7k.bms");
    let result = run_manual_simulation(&model, &[], GaugeType::ExHard);
    assert!(
        result.gauge_value < 1e-6,
        "ExHard gauge should be dead (0.0) with all misses, got {}",
        result.gauge_value
    );
    assert!(!result.gauge_qualified);
}

#[test]
fn gauge_all_types_autoplay() {
    let model = load_bms("minimal_7k.bms");
    for gauge_type in GaugeType::ALL {
        if gauge_type.is_course_gauge() {
            continue;
        }
        let result = run_autoplay_simulation(&model, gauge_type);
        assert!(
            result.gauge_qualified,
            "{gauge_type:?} gauge should be qualified on autoplay (value={})",
            result.gauge_value
        );
    }
}

// ============================================================================
// Group D: LN special tests
// ============================================================================

#[test]
fn ln_autoplay_judge_count() {
    let model = load_bms("longnote_types.bms");

    let result = run_autoplay_simulation(&model, GaugeType::Normal);

    // Pure LN: 1 judgment per LN pair (end not independently judged).
    // ghost.len() matches the number of actually-judged notes.
    let expected = result.ghost.len() as i32;
    assert_eq!(
        result.score.total_judge_count(),
        expected,
        "Judge count should match ghost length (expected={expected}, got={})",
        result.score.total_judge_count()
    );
}

#[test]
fn scratch_autoplay_judge_count() {
    let model = load_bms("scratch_bss.bms");

    let result = run_autoplay_simulation(&model, GaugeType::Normal);

    // BSS (CN type): start + end independently judged.
    // ghost.len() matches the number of actually-judged notes.
    let expected = result.ghost.len() as i32;
    assert_eq!(
        result.score.total_judge_count(),
        expected,
        "Judge count should match ghost length (expected={expected}, got={})",
        result.score.total_judge_count()
    );
}

// ============================================================================
// Group E: Cross-mode invariants
// ============================================================================

#[test]
fn cross_mode_invariants() {
    // Test all BMS files that contain only normal notes (no LN) across different modes.
    // Note: 9key_pms.bms is parsed as Beat7K because PMS detection requires .pms extension.
    let test_files = ["5key.bms", "minimal_7k.bms", "14key_dp.bms", "9key_pms.bms"];

    for filename in test_files {
        let model = load_bms(filename);
        let total = model.total_notes();
        assert!(total > 0, "{filename} should have playable notes");

        let result = run_autoplay_simulation(&model, GaugeType::Normal);
        assert_all_pgreat(&result, total, &format!("cross_mode_{filename}"));
    }
}
