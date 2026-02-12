// Golden master tests: compare Rust pattern modifiers against Java fixture export.
//
// Tests deterministic modifiers: AutoplayModifier, PracticeModifier, ScrollSpeedModifier (REMOVE).

use std::path::Path;

use bms_model::{BmsDecoder, NoteType};
use bms_pattern::{
    AutoplayModifier, PatternModifier, PracticeModifier, ScrollSpeedMode, ScrollSpeedModifier,
};
use golden_master::pattern_modifier_detail_fixtures::{
    ModifierNote, PatternModifierDetailFixture, PatternModifierTestCase,
};

fn fixtures_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .leak()
}

fn test_bms_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-bms")
        .leak()
}

fn load_fixture() -> PatternModifierDetailFixture {
    let path = fixtures_dir().join("pattern_modifier_detail.json");
    assert!(
        path.exists(),
        "Pattern modifier detail fixture not found: {}. Run the Java exporter first.",
        path.display()
    );
    let content = std::fs::read_to_string(&path).expect("Failed to read fixture");
    serde_json::from_str(&content).expect("Failed to parse fixture")
}

fn find_test_cases<'a>(
    fixture: &'a PatternModifierDetailFixture,
    modifier_type: &str,
    bms_file: &str,
) -> Vec<&'a PatternModifierTestCase> {
    fixture
        .test_cases
        .iter()
        .filter(|tc| tc.modifier_type == modifier_type && tc.bms_file == bms_file)
        .collect()
}

/// Convert a Rust NoteType to the Java fixture string representation.
fn note_type_to_string(nt: NoteType) -> &'static str {
    match nt {
        NoteType::Normal => "Normal",
        NoteType::LongNote => "LongNote",
        NoteType::ChargeNote => "ChargeNote",
        NoteType::HellChargeNote => "HellChargeNote",
        NoteType::Mine => "Mine",
        NoteType::Invisible => "Invisible",
    }
}

/// Capture notes from a BmsModel in the same format as the Java exporter.
/// Returns (lane, time_ms, note_type, end_time_ms) tuples sorted by (time_ms, lane).
/// Skips LN end notes (end_time_us == 0 for LN types).
fn capture_notes(model: &bms_model::BmsModel) -> Vec<ModifierNote> {
    let mut notes: Vec<ModifierNote> = model
        .notes
        .iter()
        .filter(|n| {
            // Skip LN end notes (they have end_time_us == 0 and are paired with start notes)
            if n.is_long_note() && n.end_time_us == 0 {
                return false;
            }
            true
        })
        .map(|n| {
            let time_ms = (n.time_us / 1000) as i32;
            let end_time_ms = if n.is_long_note() && n.end_time_us > 0 {
                Some((n.end_time_us / 1000) as i32)
            } else {
                None
            };
            ModifierNote {
                lane: n.lane,
                time_ms,
                note_type: note_type_to_string(n.note_type).to_string(),
                end_time_ms,
            }
        })
        .collect();

    // Sort by time then lane to match Java output order
    notes.sort_by(|a, b| a.time_ms.cmp(&b.time_ms).then_with(|| a.lane.cmp(&b.lane)));
    notes
}

/// Compare two note lists with ±2ms time tolerance.
fn compare_notes(
    rust_notes: &[ModifierNote],
    java_notes: &[ModifierNote],
    label: &str,
) -> Vec<String> {
    let mut diffs = Vec::new();

    if rust_notes.len() != java_notes.len() {
        diffs.push(format!(
            "{} note_count: rust={} java={}",
            label,
            rust_notes.len(),
            java_notes.len()
        ));
    }

    let min_len = rust_notes.len().min(java_notes.len());
    for i in 0..min_len {
        let rn = &rust_notes[i];
        let jn = &java_notes[i];

        if rn.lane != jn.lane {
            diffs.push(format!(
                "{} note[{}] lane: rust={} java={}",
                label, i, rn.lane, jn.lane
            ));
        }

        // ±2ms tolerance
        if (rn.time_ms - jn.time_ms).abs() > 2 {
            diffs.push(format!(
                "{} note[{}] time_ms: rust={} java={} (diff={})",
                label,
                i,
                rn.time_ms,
                jn.time_ms,
                rn.time_ms - jn.time_ms
            ));
        }

        if rn.note_type != jn.note_type {
            diffs.push(format!(
                "{} note[{}] note_type: rust={} java={}",
                label, i, rn.note_type, jn.note_type
            ));
        }

        // LN end time comparison with ±2ms tolerance
        match (&rn.end_time_ms, &jn.end_time_ms) {
            (Some(r_end), Some(j_end)) => {
                if (r_end - j_end).abs() > 2 {
                    diffs.push(format!(
                        "{} note[{}] end_time_ms: rust={} java={}",
                        label, i, r_end, j_end
                    ));
                }
            }
            (None, Some(j_end)) => {
                diffs.push(format!(
                    "{} note[{}] end_time_ms: rust=None java={}",
                    label, i, j_end
                ));
            }
            (Some(r_end), None) => {
                diffs.push(format!(
                    "{} note[{}] end_time_ms: rust={} java=None",
                    label, i, r_end
                ));
            }
            (None, None) => {}
        }
    }

    diffs
}

fn assert_no_diffs(diffs: &[String], test_name: &str) {
    if !diffs.is_empty() {
        panic!(
            "Pattern modifier mismatch for {} ({} differences):\n{}",
            test_name,
            diffs.len(),
            diffs
                .iter()
                .map(|d| format!("  - {d}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

fn rust_assist_level_str(level: bms_pattern::AssistLevel) -> &'static str {
    match level {
        bms_pattern::AssistLevel::None => "NONE",
        bms_pattern::AssistLevel::LightAssist => "LIGHT_ASSIST",
        bms_pattern::AssistLevel::Assist => "ASSIST",
    }
}

// =========================================================================
// Autoplay modifier tests
// =========================================================================

fn run_autoplay_test(bms_file: &str) {
    let fixture = load_fixture();
    let test_cases = find_test_cases(&fixture, "autoplay", bms_file);
    assert!(
        !test_cases.is_empty(),
        "No autoplay test case for {bms_file}"
    );

    for tc in test_cases {
        let bms_path = test_bms_dir().join(&tc.bms_file);
        assert!(
            bms_path.exists(),
            "BMS file not found: {}",
            bms_path.display()
        );

        let mut model = BmsDecoder::decode(&bms_path).expect("Failed to parse BMS");

        // Verify notes_before matches
        let notes_before = capture_notes(&model);
        let diffs = compare_notes(&notes_before, &tc.notes_before, "notes_before");
        assert_no_diffs(&diffs, &format!("autoplay/{}/before", tc.bms_file));

        // Extract config
        let lanes: Vec<usize> = tc.config["lanes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();

        // Apply modifier
        let mut modifier = AutoplayModifier::new(lanes);
        modifier.modify(&mut model);

        // Compare notes_after
        let notes_after = capture_notes(&model);
        let diffs = compare_notes(&notes_after, &tc.notes_after, "notes_after");
        assert_no_diffs(&diffs, &format!("autoplay/{}/after", tc.bms_file));

        // Compare assist level
        let rust_assist = rust_assist_level_str(modifier.assist_level());
        assert_eq!(
            rust_assist, tc.assist_level,
            "autoplay/{}: assist_level mismatch: rust={} java={}",
            tc.bms_file, rust_assist, tc.assist_level
        );
    }
}

#[test]
fn pattern_modifier_autoplay_minimal_7k() {
    run_autoplay_test("minimal_7k.bms");
}

#[test]
fn pattern_modifier_autoplay_longnote_types() {
    run_autoplay_test("longnote_types.bms");
}

// =========================================================================
// Practice modifier tests
// =========================================================================

fn run_practice_test(bms_file: &str) {
    let fixture = load_fixture();
    let test_cases = find_test_cases(&fixture, "practice", bms_file);
    assert!(
        !test_cases.is_empty(),
        "No practice test case for {bms_file}"
    );

    for tc in test_cases {
        let bms_path = test_bms_dir().join(&tc.bms_file);
        assert!(
            bms_path.exists(),
            "BMS file not found: {}",
            bms_path.display()
        );

        let mut model = BmsDecoder::decode(&bms_path).expect("Failed to parse BMS");

        // Verify notes_before matches
        let notes_before = capture_notes(&model);
        let diffs = compare_notes(&notes_before, &tc.notes_before, "notes_before");
        assert_no_diffs(&diffs, &format!("practice/{}/before", tc.bms_file));

        // Extract config
        let start_ms = tc.config["start_ms"].as_i64().unwrap();
        let end_ms = tc.config["end_ms"].as_i64().unwrap();

        // Apply modifier
        let mut modifier = PracticeModifier::new(start_ms, end_ms);
        modifier.modify(&mut model);

        // Compare notes_after
        let notes_after = capture_notes(&model);
        let diffs = compare_notes(&notes_after, &tc.notes_after, "notes_after");
        assert_no_diffs(&diffs, &format!("practice/{}/after", tc.bms_file));

        // Compare assist level
        let rust_assist = rust_assist_level_str(modifier.assist_level());
        assert_eq!(
            rust_assist, tc.assist_level,
            "practice/{}: assist_level mismatch: rust={} java={}",
            tc.bms_file, rust_assist, tc.assist_level
        );
    }
}

#[test]
fn pattern_modifier_practice_minimal_7k() {
    run_practice_test("minimal_7k.bms");
}

// =========================================================================
// ScrollSpeed REMOVE modifier tests
// =========================================================================

fn run_scroll_speed_remove_test(bms_file: &str) {
    let fixture = load_fixture();
    let test_cases = find_test_cases(&fixture, "scroll_speed_remove", bms_file);
    assert!(
        !test_cases.is_empty(),
        "No scroll_speed_remove test case for {bms_file}"
    );

    for tc in test_cases {
        let bms_path = test_bms_dir().join(&tc.bms_file);
        assert!(
            bms_path.exists(),
            "BMS file not found: {}",
            bms_path.display()
        );

        let mut model = BmsDecoder::decode(&bms_path).expect("Failed to parse BMS");

        // Verify notes_before matches
        let notes_before = capture_notes(&model);
        let diffs = compare_notes(&notes_before, &tc.notes_before, "notes_before");
        assert_no_diffs(
            &diffs,
            &format!("scroll_speed_remove/{}/before", tc.bms_file),
        );

        // Apply modifier
        let mut modifier = ScrollSpeedModifier::new(ScrollSpeedMode::Remove);
        modifier.modify(&mut model);

        // Notes should be unchanged (scroll modifier doesn't move notes)
        let notes_after = capture_notes(&model);
        let diffs = compare_notes(&notes_after, &tc.notes_after, "notes_after");
        assert_no_diffs(
            &diffs,
            &format!("scroll_speed_remove/{}/after", tc.bms_file),
        );

        // Compare assist level
        let rust_assist = rust_assist_level_str(modifier.assist_level());
        assert_eq!(
            rust_assist, tc.assist_level,
            "scroll_speed_remove/{}: assist_level mismatch: rust={} java={}",
            tc.bms_file, rust_assist, tc.assist_level
        );

        // Verify BPM normalization: all BPM changes should be set to initial_bpm
        let ref_bpm = tc.config["ref_bpm"].as_f64().unwrap();
        for change in &model.bpm_changes {
            assert!(
                (change.bpm - ref_bpm).abs() < 0.001,
                "scroll_speed_remove/{}: BPM not normalized: expected={} got={}",
                tc.bms_file,
                ref_bpm,
                change.bpm
            );
        }

        // Verify all stops are cleared
        assert!(
            model.stop_events.is_empty(),
            "scroll_speed_remove/{}: stop events not cleared: {} remain",
            tc.bms_file,
            model.stop_events.len()
        );
    }
}

#[test]
fn pattern_modifier_scroll_speed_remove_bpm_change() {
    run_scroll_speed_remove_test("bpm_change.bms");
}

#[test]
fn pattern_modifier_scroll_speed_remove_stop_sequence() {
    run_scroll_speed_remove_test("stop_sequence.bms");
}
