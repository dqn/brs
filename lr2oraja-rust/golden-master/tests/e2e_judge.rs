// E2E integration tests: BMS parse → JudgeManager → GrooveGauge → ScoreData
//
// Validates that the full pipeline works correctly with real BMS files.
// Uses invariant-based assertions rather than golden-master comparison.
//
// Known limitations:
// - LN pair_index is not set by the parser (always usize::MAX), so autoplay
//   cannot properly track LN start→end. Pure LN notes are judged as PR (miss)
//   by the miss phase. This will be fixed when the parser sets pair_index.
// - JudgeManager.prev_time starts at 0, so notes at time_us=0 are skipped on
//   the first frame. We prime the JudgeManager with update(-1) to work around this.

use std::path::Path;

use bms_model::{BmsDecoder, BmsModel, NoteType};
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
    let rule = PlayerRule::lr2();
    let total_notes = model.total_notes();
    let total = if model.total > 0.0 {
        model.total
    } else {
        PlayerRule::default_total(total_notes)
    };

    let config = JudgeConfig {
        notes: &model.notes,
        play_mode: model.mode,
        ln_type: model.ln_type,
        judge_rank: model.judge_rank,
        judge_window_rate: [100, 100, 100],
        scratch_judge_window_rate: [100, 100, 100],
        algorithm: JudgeAlgorithm::Combo,
        autoplay: true,
        judge_property: &rule.judge,
    };

    let mut jm = JudgeManager::new(&config);
    let mut gauge = GrooveGauge::new(&rule.gauge, gauge_type, total, total_notes);

    let key_count = model.mode.key_count();
    let key_states = vec![false; key_count];
    let key_times = vec![NOT_SET; key_count];

    // Prime JudgeManager: set prev_time to -1 so notes at time_us=0 are not skipped.
    // JudgeManager.prev_time starts at 0, causing `note.time_us <= self.prev_time`
    // to be true for notes at time 0. This priming update sets prev_time = -1.
    jm.update(-1, &model.notes, &key_states, &key_times, &mut gauge);

    let last_note_time = model
        .notes
        .iter()
        .map(|n| n.time_us.max(n.end_time_us))
        .max()
        .unwrap_or(0);
    let end_time = last_note_time + TAIL_TIME;

    let mut time = 0i64;
    while time <= end_time {
        jm.update(time, &model.notes, &key_states, &key_times, &mut gauge);
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

/// Create simple press+release input events for each playable normal note.
/// This avoids the complexity of the full autoplay log (which includes release
/// events for empty lanes at every timeline).
fn create_note_press_log(model: &BmsModel, offset_us: i64) -> Vec<KeyInputLog> {
    let mut log = Vec::new();
    for note in &model.notes {
        if !note.is_playable() {
            continue;
        }
        match note.note_type {
            NoteType::Normal => {
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
            _ => {
                // Skip LN/CN/HCN for manual tests (pair_index limitation)
            }
        }
    }
    log
}

/// Run manual simulation with per-frame key state conversion.
fn run_manual_simulation(
    model: &BmsModel,
    input_log: &[KeyInputLog],
    gauge_type: GaugeType,
) -> SimulationResult {
    let rule = PlayerRule::lr2();
    let total_notes = model.total_notes();
    let total = if model.total > 0.0 {
        model.total
    } else {
        PlayerRule::default_total(total_notes)
    };

    let config = JudgeConfig {
        notes: &model.notes,
        play_mode: model.mode,
        ln_type: model.ln_type,
        judge_rank: model.judge_rank,
        judge_window_rate: [100, 100, 100],
        scratch_judge_window_rate: [100, 100, 100],
        algorithm: JudgeAlgorithm::Combo,
        autoplay: false,
        judge_property: &rule.judge,
    };

    let mut jm = JudgeManager::new(&config);
    let mut gauge = GrooveGauge::new(&rule.gauge, gauge_type, total, total_notes);

    let key_count = model.mode.key_count();

    let mut sorted_log: Vec<&KeyInputLog> = input_log.iter().collect();
    sorted_log.sort_by_key(|e| e.get_time());

    let last_note_time = model
        .notes
        .iter()
        .map(|n| n.time_us.max(n.end_time_us))
        .max()
        .unwrap_or(0);
    let end_time = last_note_time + TAIL_TIME;

    let mut key_states = vec![false; key_count];
    let mut log_cursor = 0;

    // Prime JudgeManager for notes at time 0
    let empty_key_times = vec![NOT_SET; key_count];
    jm.update(-1, &model.notes, &key_states, &empty_key_times, &mut gauge);

    let mut time = 0i64;
    while time <= end_time {
        let mut key_changed_times = vec![NOT_SET; key_count];

        while log_cursor < sorted_log.len() && sorted_log[log_cursor].get_time() <= time {
            let event = sorted_log[log_cursor];
            let lane = event.keycode as usize;
            if lane < key_count {
                key_states[lane] = event.pressed;
                key_changed_times[lane] = event.get_time();
            }
            log_cursor += 1;
        }

        jm.update(
            time,
            &model.notes,
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

// LN autoplay tests: pair_index is not set by the parser, so pure LN autoplay
// doesn't work correctly. These tests verify normal notes are PG and LN notes
// are judged (as PR/miss). Full LN autoplay requires pair_index to be set.

#[test]
fn autoplay_longnote() {
    let model = load_bms("longnote_types.bms");
    let total = model.total_notes();
    let normal = count_normal_notes(&model);
    let long_notes = model.total_long_notes();
    assert!(total > 0);
    assert!(long_notes > 0, "longnote_types should have LN notes");

    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    let score = &result.score;

    // Normal notes should all be PG
    assert_eq!(
        score.judge_count(JUDGE_PG),
        normal as i32,
        "Normal notes should be PG: expected {normal}, got {}",
        score.judge_count(JUDGE_PG)
    );

    // LN notes are judged as PR due to pair_index limitation
    // TODO: Once parser sets pair_index, change this to assert_all_pgreat
    assert_eq!(
        score.total_judge_count(),
        total as i32,
        "Each playable note should be judged exactly once"
    );
}

#[test]
fn autoplay_scratch_bss() {
    let model = load_bms("scratch_bss.bms");
    let total = model.total_notes();
    let normal = count_normal_notes(&model);
    let long_notes = model.total_long_notes();
    assert!(total > 0);

    let result = run_autoplay_simulation(&model, GaugeType::Normal);
    let score = &result.score;

    // Normal notes should all be PG
    assert_eq!(
        score.judge_count(JUDGE_PG),
        normal as i32,
        "Normal notes should be PG: expected {normal}, got {} \
         (total={total}, long={long_notes})",
        score.judge_count(JUDGE_PG)
    );

    // TODO: Once parser sets pair_index, change this to assert_all_pgreat
    assert_eq!(
        score.total_judge_count(),
        total as i32,
        "Each playable note should be judged exactly once"
    );
}

// ============================================================================
// Group B: Manual input tests — timing offset affects judgment
// ============================================================================

#[test]
fn manual_perfect() {
    let model = load_bms("minimal_7k.bms");
    let normal = count_normal_notes(&model);
    // Create press events at exact note times (0 offset)
    let log = create_note_press_log(&model, 0);
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
    // minimal_7k.bms has #RANK 2 → judge_rank=50. LR2 scaled windows:
    //   PG ±15ms, GR ±30ms, GD ±60ms, BD ±200ms
    // Offset by 25ms — within GR window (±30ms) but outside PG window (±15ms)
    let log = create_note_press_log(&model, 25_000);
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
    // minimal_7k.bms has #RANK 2 → judge_rank=50. LR2 scaled windows:
    //   PG ±15ms, GR ±30ms, GD ±60ms, BD ±200ms
    // Offset by 50ms — within GD window (±60ms) but outside GR window (±30ms)
    let log = create_note_press_log(&model, 50_000);
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
    // minimal_7k.bms has #RANK 2 → judge_rank=50. LR2 scaled windows:
    //   PG ±15ms, GR ±30ms, GD ±60ms, BD ±200ms
    // Offset by 100ms — within BD window (±200ms) but outside GD window (±60ms)
    let log = create_note_press_log(&model, 100_000);
    let result = run_manual_simulation(&model, &log, GaugeType::Normal);

    let score = &result.score;
    assert!(
        score.judge_count(JUDGE_BD) > 0 || score.judge_count(JUDGE_PR) > 0,
        "Expected some BD/PR at 100ms offset (PG={}, GR={}, GD={}, BD={}, PR={}, MS={})",
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
    let total = model.total_notes();

    let result = run_autoplay_simulation(&model, GaugeType::Normal);

    // Each playable note (including LN starts) gets exactly 1 judgment.
    // Note: LN end judgments are not generated because pair_index is not set.
    // TODO: When pair_index is set, LN should produce start+end judgments.
    assert_eq!(
        result.score.total_judge_count(),
        total as i32,
        "Each playable note should be judged exactly once (total={total}, got={})",
        result.score.total_judge_count()
    );
}

#[test]
fn scratch_autoplay_judge_count() {
    let model = load_bms("scratch_bss.bms");
    let total = model.total_notes();

    let result = run_autoplay_simulation(&model, GaugeType::Normal);

    // Same as ln_autoplay_judge_count
    assert_eq!(
        result.score.total_judge_count(),
        total as i32,
        "Each playable note should be judged exactly once (total={total}, got={})",
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
