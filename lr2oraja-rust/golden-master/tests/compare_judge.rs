// Golden master comparison tests for JudgeManager.
//
// Loads Java-generated fixtures and runs equivalent Rust simulations,
// comparing ScoreData, maxcombo, ghost, gauge values.

use std::path::Path;

use bms_model::{BmsDecoder, BmsModel, LaneProperty};
use bms_replay::key_input_log::KeyInputLog;
use bms_rule::gauge_property::GaugeType;
use bms_rule::judge_manager::{JudgeConfig, JudgeManager};
use bms_rule::{GrooveGauge, JudgeAlgorithm, PlayerRule};
use golden_master::judge_fixtures::{JudgeFixtures, JudgeTestCase};

const NOT_SET: i64 = i64::MIN;
const FRAME_STEP: i64 = 1_000;
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

fn parse_gauge_type(s: &str) -> GaugeType {
    match s {
        "NORMAL" => GaugeType::Normal,
        "HARD" => GaugeType::Hard,
        "EXHARD" => GaugeType::ExHard,
        "EASY" => GaugeType::Easy,
        _ => panic!("Unknown gauge type: {s}"),
    }
}

struct SimulationResult {
    score: bms_rule::ScoreData,
    max_combo: i32,
    ghost: Vec<usize>,
    gauge_value: f32,
    gauge_qualified: bool,
}

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

    // Prime JudgeManager for notes at time 0
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

fn compare_result(tc: &JudgeTestCase, result: &SimulationResult) -> Vec<String> {
    let mut diffs = Vec::new();
    let expected = &tc.expected;

    // Score: 12 judge fields
    let s = &result.score;
    let e = &expected.score;
    let fields = [
        ("epg", s.epg, e.epg),
        ("lpg", s.lpg, e.lpg),
        ("egr", s.egr, e.egr),
        ("lgr", s.lgr, e.lgr),
        ("egd", s.egd, e.egd),
        ("lgd", s.lgd, e.lgd),
        ("ebd", s.ebd, e.ebd),
        ("lbd", s.lbd, e.lbd),
        ("epr", s.epr, e.epr),
        ("lpr", s.lpr, e.lpr),
        ("ems", s.ems, e.ems),
        ("lms", s.lms, e.lms),
    ];
    for (name, rust, java) in &fields {
        if rust != java {
            diffs.push(format!("score.{name}: rust={rust} java={java}"));
        }
    }

    // maxcombo, passnotes
    if result.max_combo != expected.maxcombo {
        diffs.push(format!(
            "maxcombo: rust={} java={}",
            result.max_combo, expected.maxcombo
        ));
    }

    if s.passnotes != expected.passnotes {
        diffs.push(format!(
            "passnotes: rust={} java={}",
            s.passnotes, expected.passnotes
        ));
    }

    // ghost: exact match
    if result.ghost != expected.ghost {
        let ghost_len = result.ghost.len().min(expected.ghost.len());
        let mut ghost_diff_count = 0;
        for i in 0..ghost_len {
            if result.ghost[i] != expected.ghost[i] {
                if ghost_diff_count < 5 {
                    diffs.push(format!(
                        "ghost[{i}]: rust={} java={}",
                        result.ghost[i], expected.ghost[i]
                    ));
                }
                ghost_diff_count += 1;
            }
        }
        if result.ghost.len() != expected.ghost.len() {
            diffs.push(format!(
                "ghost.len: rust={} java={}",
                result.ghost.len(),
                expected.ghost.len()
            ));
        }
        if ghost_diff_count > 5 {
            diffs.push(format!("... and {} more ghost diffs", ghost_diff_count - 5));
        }
    }

    // gauge_value: ±0.01 tolerance (f32 precision)
    if (result.gauge_value - expected.gauge_value).abs() > 0.01 {
        diffs.push(format!(
            "gauge_value: rust={:.4} java={:.4}",
            result.gauge_value, expected.gauge_value
        ));
    }

    // gauge_qualified: exact match
    if result.gauge_qualified != expected.gauge_qualified {
        diffs.push(format!(
            "gauge_qualified: rust={} java={}",
            result.gauge_qualified, expected.gauge_qualified
        ));
    }

    diffs
}

fn run_test_case(tc: &JudgeTestCase) {
    let model = load_bms(&tc.filename);
    let gauge_type = parse_gauge_type(&tc.gauge_type);

    let result = if tc.autoplay {
        run_autoplay_simulation(&model, gauge_type)
    } else if tc.input_log.is_empty() {
        // All-miss: manual simulation with no input
        run_manual_simulation(&model, &[], gauge_type)
    } else {
        let log: Vec<KeyInputLog> = tc
            .input_log
            .iter()
            .map(|e| KeyInputLog::new(e.presstime, e.keycode, e.pressed))
            .collect();
        run_manual_simulation(&model, &log, gauge_type)
    };

    let diffs = compare_result(tc, &result);
    if !diffs.is_empty() {
        panic!(
            "GM mismatch for [{}] {}:\n  {}",
            tc.group,
            tc.name,
            diffs.join("\n  ")
        );
    }
}

// =========================================================================
// Test functions: one per test case group for clear failure reporting
// =========================================================================

#[test]
fn group_a_autoplay() {
    let fixtures = JudgeFixtures::load().expect("Failed to load judge fixtures");
    for tc in &fixtures.test_cases {
        if tc.group == "A_autoplay" {
            run_test_case(tc);
        }
    }
}

#[test]
fn group_b_manual() {
    let fixtures = JudgeFixtures::load().expect("Failed to load judge fixtures");
    for tc in &fixtures.test_cases {
        if tc.group == "B_manual" {
            run_test_case(tc);
        }
    }
}

#[test]
fn group_c_gauge() {
    let fixtures = JudgeFixtures::load().expect("Failed to load judge fixtures");
    for tc in &fixtures.test_cases {
        if tc.group == "C_gauge" {
            run_test_case(tc);
        }
    }
}

#[test]
fn group_d_gauge_miss() {
    let fixtures = JudgeFixtures::load().expect("Failed to load judge fixtures");
    for tc in &fixtures.test_cases {
        if tc.group == "D_gauge_miss" {
            run_test_case(tc);
        }
    }
}

#[test]
fn group_e_longnote() {
    let fixtures = JudgeFixtures::load().expect("Failed to load judge fixtures");
    for tc in &fixtures.test_cases {
        if tc.group == "E_longnote" {
            run_test_case(tc);
        }
    }
}

#[test]
fn group_f_cross_mode() {
    let fixtures = JudgeFixtures::load().expect("Failed to load judge fixtures");
    for tc in &fixtures.test_cases {
        if tc.group == "F_cross_mode" {
            run_test_case(tc);
        }
    }
}
