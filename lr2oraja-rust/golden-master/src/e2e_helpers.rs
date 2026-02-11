// E2E simulation helpers: shared between e2e_judge.rs and exhaustive_e2e.rs
//
// Provides BMS loading, autoplay/manual simulation, and assertion utilities
// for integration tests that validate the full pipeline:
// BMS parse -> JudgeManager -> GrooveGauge -> ScoreData

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
pub const NOT_SET: i64 = i64::MIN;

/// Frame step for simulation (1ms = 1000us).
pub const FRAME_STEP: i64 = 1_000;

/// Extra time after last note to finish simulation (1 second).
pub const TAIL_TIME: i64 = 1_000_000;

pub fn test_bms_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-bms")
        .leak()
}

pub fn load_bms(filename: &str) -> BmsModel {
    let path = test_bms_dir().join(filename);
    BmsDecoder::decode(&path).unwrap_or_else(|e| panic!("Failed to parse {filename}: {e}"))
}

pub struct SimulationResult {
    pub score: bms_rule::ScoreData,
    pub max_combo: i32,
    pub ghost: Vec<usize>,
    pub gauge_value: f32,
    pub gauge_qualified: bool,
}

/// Count normal (non-LN) playable notes in the model.
pub fn count_normal_notes(model: &BmsModel) -> usize {
    model
        .notes
        .iter()
        .filter(|n| n.is_playable() && !n.is_long_note())
        .count()
}

/// Run autoplay simulation: JudgeManager with autoplay=true, empty key inputs.
pub fn run_autoplay_simulation(model: &BmsModel, gauge_type: GaugeType) -> SimulationResult {
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
///
/// Uses LaneProperty to map note lanes to physical key indices, which is
/// required for DP modes (Beat14K) where lane indices differ from key indices.
pub fn create_note_press_log(
    notes: &[bms_model::Note],
    mode: bms_model::PlayMode,
    offset_us: i64,
) -> Vec<KeyInputLog> {
    let lp = LaneProperty::new(mode);
    let mut log = Vec::new();
    for note in notes {
        if !note.is_playable() {
            continue;
        }
        if note.is_long_note() {
            // Skip LN start/end notes for manual tests
            continue;
        }
        // Use lane_to_keys to get the correct physical key index
        let keys = lp.lane_to_keys(note.lane);
        let key = keys[0] as i32;
        log.push(KeyInputLog::new(note.time_us + offset_us, key, true));
        // Release 80ms after press
        log.push(KeyInputLog::new(
            note.time_us + offset_us + 80_000,
            key,
            false,
        ));
    }
    log
}

/// Run manual simulation with per-frame key state conversion.
pub fn run_manual_simulation(
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
pub fn assert_all_pgreat(result: &SimulationResult, total_notes: usize, label: &str) {
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
