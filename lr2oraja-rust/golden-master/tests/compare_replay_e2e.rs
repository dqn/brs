// Golden master comparison tests for replay E2E.
//
// Compares Rust replay simulation output against Java ReplayE2EExporter fixtures.
// Covers: autoplay across multiple BMS files, gauge type variants, and manual input.
//
// Run: cargo test -p golden-master compare_replay_e2e -- --nocapture

use std::path::Path;

use bms_model::{BmsDecoder, BmsModel, LaneProperty};
use bms_rule::gauge_property::GaugeType;
use bms_rule::judge_manager::{JudgeConfig, JudgeManager};
use bms_rule::{GrooveGauge, JudgeAlgorithm, PlayerRule};
use golden_master::replay_e2e_fixtures::{ExpectedScore, ReplayE2EFixtures, ReplayE2ETestCase};

/// Sentinel for "not set" timestamps (matches JudgeManager internal).
const NOT_SET: i64 = i64::MIN;

/// Frame step for simulation (1ms = 1000us).
const FRAME_STEP: i64 = 1_000;

/// Extra time after last note to finish simulation (1 second).
const TAIL_TIME: i64 = 1_000_000;

/// Gauge value comparison tolerance (f32 rounding).
const GAUGE_TOLERANCE: f32 = 0.02;

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
        "ASSIST_EASY" => GaugeType::AssistEasy,
        "EASY" => GaugeType::Easy,
        "NORMAL" => GaugeType::Normal,
        "HARD" => GaugeType::Hard,
        "EXHARD" => GaugeType::ExHard,
        "HAZARD" => GaugeType::Hazard,
        "CLASS" => GaugeType::Class,
        "EXCLASS" => GaugeType::ExClass,
        "EXHARDCLASS" => GaugeType::ExHardClass,
        _ => panic!("Unknown gauge type: {s}"),
    }
}

struct SimResult {
    score: bms_rule::ScoreData,
    max_combo: i32,
    ghost: Vec<usize>,
    gauge_value: f32,
    gauge_qualified: bool,
    pass_notes: i32,
}

fn run_simulation(model: &BmsModel, tc: &ReplayE2ETestCase) -> SimResult {
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
        autoplay: tc.autoplay,
        judge_property: &rule.judge,
        lane_property: None,
    };

    let gauge_type = parse_gauge_type(&tc.gauge_type);
    let mut jm = JudgeManager::new(&config);
    let mut gauge = GrooveGauge::new(&rule.gauge, gauge_type, total, total_notes);

    let lp = LaneProperty::new(model.mode);
    let physical_key_count = lp.physical_key_count();

    // Prime JudgeManager: set prev_time to -1 so notes at time_us=0 are not skipped.
    let empty_states = vec![false; physical_key_count];
    let empty_times = vec![NOT_SET; physical_key_count];
    jm.update(-1, &judge_notes, &empty_states, &empty_times, &mut gauge);

    let last_note_time = judge_notes
        .iter()
        .map(|n| n.time_us.max(n.end_time_us))
        .max()
        .unwrap_or(0);
    let end_time = last_note_time + TAIL_TIME;

    if tc.autoplay {
        let key_states = vec![false; physical_key_count];
        let key_times = vec![NOT_SET; physical_key_count];
        let mut time = 0i64;
        while time <= end_time {
            jm.update(time, &judge_notes, &key_states, &key_times, &mut gauge);
            time += FRAME_STEP;
        }
    } else {
        let mut sorted_log: Vec<&_> = tc.input_log.iter().collect();
        sorted_log.sort_by_key(|e| e.presstime);

        let mut key_states = vec![false; physical_key_count];
        let mut log_cursor = 0;
        let mut time = 0i64;

        while time <= end_time {
            let mut key_changed_times = vec![NOT_SET; physical_key_count];

            while log_cursor < sorted_log.len() && sorted_log[log_cursor].presstime <= time {
                let event = &sorted_log[log_cursor];
                let key = event.keycode as usize;
                if key < physical_key_count {
                    key_states[key] = event.pressed;
                    key_changed_times[key] = event.presstime;
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
    }

    SimResult {
        score: jm.score().clone(),
        max_combo: jm.max_combo(),
        ghost: jm.ghost().to_vec(),
        gauge_value: gauge.value(),
        gauge_qualified: gauge.is_qualified(),
        pass_notes: jm.past_notes(),
    }
}

fn compare_score(actual: &bms_rule::ScoreData, expected: &ExpectedScore) -> Vec<String> {
    let mut diffs = Vec::new();
    let fields = [
        ("epg", actual.epg, expected.epg),
        ("lpg", actual.lpg, expected.lpg),
        ("egr", actual.egr, expected.egr),
        ("lgr", actual.lgr, expected.lgr),
        ("egd", actual.egd, expected.egd),
        ("lgd", actual.lgd, expected.lgd),
        ("ebd", actual.ebd, expected.ebd),
        ("lbd", actual.lbd, expected.lbd),
        ("epr", actual.epr, expected.epr),
        ("lpr", actual.lpr, expected.lpr),
        ("ems", actual.ems, expected.ems),
        ("lms", actual.lms, expected.lms),
        ("score.maxcombo", actual.maxcombo, expected.maxcombo),
        ("score.passnotes", actual.passnotes, expected.passnotes),
    ];
    for (name, actual_val, expected_val) in fields {
        if actual_val != expected_val {
            diffs.push(format!("{name}: rust={actual_val} java={expected_val}"));
        }
    }
    diffs
}

#[test]
fn compare_replay_e2e() {
    let fixtures = ReplayE2EFixtures::load().expect(
        "Failed to load replay E2E fixture. Run `just golden-master-replay-e2e-gen` first.",
    );

    let mut failures: Vec<String> = Vec::new();
    let mut pass_count = 0;

    for tc in &fixtures.test_cases {
        let model = load_bms(&tc.filename);
        let result = run_simulation(&model, tc);

        let mut diffs: Vec<String> = Vec::new();

        // Compare score fields
        diffs.extend(compare_score(&result.score, &tc.expected.score));

        // Compare maxcombo
        if result.max_combo != tc.expected.maxcombo {
            diffs.push(format!(
                "maxcombo: rust={} java={}",
                result.max_combo, tc.expected.maxcombo
            ));
        }

        // Compare passnotes
        if result.pass_notes != tc.expected.passnotes {
            diffs.push(format!(
                "passnotes: rust={} java={}",
                result.pass_notes, tc.expected.passnotes
            ));
        }

        // Compare gauge_value with tolerance
        if (result.gauge_value - tc.expected.gauge_value).abs() > GAUGE_TOLERANCE {
            diffs.push(format!(
                "gauge_value: rust={} java={} (diff={})",
                result.gauge_value,
                tc.expected.gauge_value,
                (result.gauge_value - tc.expected.gauge_value).abs()
            ));
        }

        // Compare gauge_qualified
        if result.gauge_qualified != tc.expected.gauge_qualified {
            diffs.push(format!(
                "gauge_qualified: rust={} java={}",
                result.gauge_qualified, tc.expected.gauge_qualified
            ));
        }

        // Compare ghost
        if result.ghost.len() != tc.expected.ghost.len() {
            diffs.push(format!(
                "ghost.len: rust={} java={}",
                result.ghost.len(),
                tc.expected.ghost.len()
            ));
        } else {
            for (i, (r, j)) in result
                .ghost
                .iter()
                .zip(tc.expected.ghost.iter())
                .enumerate()
            {
                if r != j {
                    diffs.push(format!("ghost[{i}]: rust={r} java={j}"));
                }
            }
        }

        if diffs.is_empty() {
            pass_count += 1;
            eprintln!("  PASS: {}/{}", tc.group, tc.name);
        } else {
            failures.push(format!(
                "[{}/{}] {} differences:\n{}",
                tc.group,
                tc.name,
                diffs.len(),
                diffs
                    .iter()
                    .map(|d| format!("    - {d}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "Replay E2E GM test: {pass_count}/{} passed, {} failed:\n\n{}",
            fixtures.test_cases.len(),
            failures.len(),
            failures.join("\n\n")
        );
    }

    println!(
        "Replay E2E GM test: {pass_count}/{} all passed",
        fixtures.test_cases.len(),
    );
}
