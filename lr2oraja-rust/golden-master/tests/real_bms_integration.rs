// Real BMS decode sanity tests: verify that actual BMS charts decode correctly.
//
// These tests use real BMS files (not synthetic test fixtures) to validate
// that the decoder handles production content without panics and produces
// structurally valid output.

use std::path::{Path, PathBuf};

use beatoraja_core::player_config::PlayerConfig;
use beatoraja_pattern::lane_shuffle_modifier::LaneMirrorShuffleModifier;
use beatoraja_pattern::note_shuffle_modifier::NoteShuffleModifier;
use beatoraja_pattern::pattern_modifier::PatternModifier;
use beatoraja_pattern::random::Random;
use beatoraja_pattern::scroll_speed_modifier::ScrollSpeedModifier;
use bms_model::bms_decoder::BMSDecoder;
use bms_model::bms_model::BMSModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct RealBmsFixture {
    filename: String,
    md5: String,
    sha256: String,
    mode: String,
    bpm: f64,
    total_notes: i32,
    timeline_count: usize,
}

/// Root directory containing real BMS subdirectories, relative to CARGO_MANIFEST_DIR.
fn bms_real_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bms")
}

/// Discover all .bms files under the known subdirectories.
fn discover_bms_files() -> Vec<PathBuf> {
    let base = bms_real_dir();
    let subdirs = ["bms-001", "bms-002"];

    let mut files = Vec::new();
    for subdir in &subdirs {
        let dir = base.join(subdir);
        if !dir.is_dir() {
            panic!("Expected BMS directory not found: {}", dir.display());
        }
        for entry in std::fs::read_dir(&dir).expect("Failed to read BMS directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("bms") {
                files.push(path);
            }
        }
    }

    assert!(
        !files.is_empty(),
        "No .bms files found under {}",
        base.display()
    );
    files.sort();
    files
}

/// Decode a single BMS file, returning the model. Panics with a descriptive
/// message if decoding fails.
fn decode_bms(path: &Path) -> BMSModel {
    let mut decoder = BMSDecoder::new();
    decoder
        .decode_path(path)
        .unwrap_or_else(|| panic!("BMSDecoder returned None for {}", path.display()))
}

// ============================================================================
// Test 1: All real BMS files decode without panics and pass basic sanity checks
// ============================================================================

#[test]
fn real_bms_decode_all_without_panic() {
    let files = discover_bms_files();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();

        // Mode must be set
        assert!(
            model.get_mode().is_some(),
            "{filename}: mode should be set after decode"
        );

        // BPM must be positive
        assert!(
            model.get_bpm() > 0.0,
            "{filename}: BPM should be > 0, got {}",
            model.get_bpm()
        );

        // Timelines must be non-empty
        assert!(
            !model.get_all_time_lines().is_empty(),
            "{filename}: timelines should be non-empty"
        );

        // Total notes must be > 0 (these are real playable charts)
        assert!(
            model.get_total_notes() > 0,
            "{filename}: total_notes should be > 0, got {}",
            model.get_total_notes()
        );
    }
}

// ============================================================================
// Test 2: Metadata (title and artist) is populated
// ============================================================================

#[test]
fn real_bms_metadata_is_populated() {
    let files = discover_bms_files();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();

        assert!(
            !model.get_title().is_empty(),
            "{filename}: title should be non-empty"
        );
        assert!(
            !model.get_artist().is_empty(),
            "{filename}: artist should be non-empty"
        );
    }
}

// ============================================================================
// Test 3: Timeline times are valid (>= 0 and monotonically non-decreasing)
// ============================================================================

#[test]
fn real_bms_timing_is_valid() {
    let files = discover_bms_files();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();

        let times: Vec<i64> = model
            .get_all_time_lines()
            .iter()
            .map(|tl| tl.get_micro_time())
            .collect();

        assert!(
            !times.is_empty(),
            "{filename}: should have at least one timeline"
        );

        // All times >= 0
        for (i, &t) in times.iter().enumerate() {
            assert!(
                t >= 0,
                "{filename}: timeline[{i}] time should be >= 0, got {t}"
            );
        }

        // Monotonically non-decreasing
        for i in 1..times.len() {
            assert!(
                times[i] >= times[i - 1],
                "{filename}: timeline times should be non-decreasing, but [{prev}]={t_prev} > [{i}]={t_cur}",
                prev = i - 1,
                t_prev = times[i - 1],
                t_cur = times[i]
            );
        }
    }
}

// ============================================================================
// Test 4: Hashes are stable and well-formed
// ============================================================================

#[test]
fn real_bms_hashes_are_stable() {
    let files = discover_bms_files();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();

        let md5 = model.get_md5();
        let sha256 = model.get_sha256();

        // MD5 should be 32 hex characters
        assert_eq!(
            md5.len(),
            32,
            "{filename}: MD5 should be 32 chars, got {} ('{md5}')",
            md5.len()
        );
        assert!(
            md5.chars().all(|c| c.is_ascii_hexdigit()),
            "{filename}: MD5 should be hex, got '{md5}'"
        );

        // SHA-256 should be 64 hex characters
        assert_eq!(
            sha256.len(),
            64,
            "{filename}: SHA-256 should be 64 chars, got {} ('{sha256}')",
            sha256.len()
        );
        assert!(
            sha256.chars().all(|c| c.is_ascii_hexdigit()),
            "{filename}: SHA-256 should be hex, got '{sha256}'"
        );

        // Decode a second time and verify hashes are deterministic
        let model2 = decode_bms(path);
        assert_eq!(
            md5,
            model2.get_md5(),
            "{filename}: MD5 should be stable across decodes"
        );
        assert_eq!(
            sha256,
            model2.get_sha256(),
            "{filename}: SHA-256 should be stable across decodes"
        );
    }
}

// ============================================================================
// Test 5: Golden master regression - detect any change in decoded output
// ============================================================================

#[test]
fn real_bms_golden_master_regression() {
    let files = discover_bms_files();

    let mut fixtures: Vec<RealBmsFixture> = files
        .iter()
        .map(|path| {
            let model = decode_bms(path);
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            RealBmsFixture {
                filename,
                md5: model.get_md5().to_string(),
                sha256: model.get_sha256().to_string(),
                mode: model
                    .get_mode()
                    .map(|m| m.hint().to_string())
                    .unwrap_or_default(),
                bpm: model.get_bpm(),
                total_notes: model.get_total_notes(),
                timeline_count: model.get_all_time_lines().len(),
            }
        })
        .collect();
    fixtures.sort_by(|a, b| a.filename.cmp(&b.filename));

    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/real_bms_summary.json");
    let update = std::env::var("UPDATE_REAL_BMS_FIXTURES").is_ok();

    if update {
        let json = serde_json::to_string_pretty(&fixtures).expect("Failed to serialize fixtures");
        std::fs::write(&fixture_path, json).expect("Failed to write fixture file");
        eprintln!(
            "Updated fixture: {} ({} entries)",
            fixture_path.display(),
            fixtures.len()
        );
        return;
    }

    if !fixture_path.exists() {
        // First run: auto-generate the fixture
        let json = serde_json::to_string_pretty(&fixtures).expect("Failed to serialize fixtures");
        std::fs::write(&fixture_path, json).expect("Failed to write fixture file");
        eprintln!(
            "Auto-generated fixture: {} ({} entries)",
            fixture_path.display(),
            fixtures.len()
        );
        return;
    }

    // Compare against existing fixture
    let expected_json =
        std::fs::read_to_string(&fixture_path).expect("Failed to read fixture file");
    let expected: Vec<RealBmsFixture> =
        serde_json::from_str(&expected_json).expect("Failed to parse fixture file");

    assert_eq!(
        fixtures.len(),
        expected.len(),
        "File count mismatch: got {} files, fixture has {}",
        fixtures.len(),
        expected.len()
    );

    for (actual, exp) in fixtures.iter().zip(expected.iter()) {
        assert_eq!(
            actual.filename, exp.filename,
            "Filename mismatch: got '{}', expected '{}'",
            actual.filename, exp.filename
        );
        assert_eq!(
            actual.md5, exp.md5,
            "{}: MD5 mismatch: got '{}', expected '{}'",
            actual.filename, actual.md5, exp.md5
        );
        assert_eq!(
            actual.sha256, exp.sha256,
            "{}: SHA-256 mismatch: got '{}', expected '{}'",
            actual.filename, actual.sha256, exp.sha256
        );
        assert_eq!(
            actual.mode, exp.mode,
            "{}: mode mismatch: got '{}', expected '{}'",
            actual.filename, actual.mode, exp.mode
        );
        assert!(
            (actual.bpm - exp.bpm).abs() < f64::EPSILON,
            "{}: BPM mismatch: got {}, expected {}",
            actual.filename,
            actual.bpm,
            exp.bpm
        );
        assert_eq!(
            actual.total_notes, exp.total_notes,
            "{}: total_notes mismatch: got {}, expected {}",
            actual.filename, actual.total_notes, exp.total_notes
        );
        assert_eq!(
            actual.timeline_count, exp.timeline_count,
            "{}: timeline_count mismatch: got {}, expected {}",
            actual.filename, actual.timeline_count, exp.timeline_count
        );
    }
}

// ============================================================================
// Test 6: Pattern modifiers (mirror, S-Random, H-Random) preserve note count
// ============================================================================

#[test]
fn real_bms_pattern_modifiers_no_panic() {
    let files = discover_bms_files();
    let config = PlayerConfig::default();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();
        let mode = model
            .get_mode()
            .expect(&format!("{filename}: mode should be set"));
        let original_notes = model.get_total_notes();

        // LaneMirrorShuffleModifier (mirror, player=0, is_scratch=false)
        {
            let mut model_mirror = model.clone();
            let mut modifier = LaneMirrorShuffleModifier::new(0, false);
            modifier.modify(&mut model_mirror);
            assert_eq!(
                model_mirror.get_total_notes(),
                original_notes,
                "{filename}: mirror modifier should preserve total note count"
            );
        }

        // NoteShuffleModifier with SRandom
        {
            let mut model_srandom = model.clone();
            let mut modifier = NoteShuffleModifier::new(Random::SRandom, 0, &mode, &config);
            modifier.set_seed(42);
            modifier.modify(&mut model_srandom);
            assert_eq!(
                model_srandom.get_total_notes(),
                original_notes,
                "{filename}: S-Random modifier should preserve total note count"
            );
        }

        // NoteShuffleModifier with HRandom
        {
            let mut model_hrandom = model.clone();
            let mut modifier = NoteShuffleModifier::new(Random::HRandom, 0, &mode, &config);
            modifier.set_seed(42);
            modifier.modify(&mut model_hrandom);
            assert_eq!(
                model_hrandom.get_total_notes(),
                original_notes,
                "{filename}: H-Random modifier should preserve total note count"
            );
        }
    }
}

// ============================================================================
// Test 7: Full pipeline - decode, validate timelines, build judge notes
// ============================================================================

#[test]
fn real_bms_full_pipeline_decode_validate_judge() {
    let files = discover_bms_files();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();

        // Validate: all timeline times must be >= 0
        let times: Vec<i64> = model
            .get_all_time_lines()
            .iter()
            .map(|tl| tl.get_micro_time())
            .collect();
        assert!(
            !times.is_empty(),
            "{filename}: timelines should be non-empty after decode"
        );
        for (i, &t) in times.iter().enumerate() {
            assert!(
                t >= 0,
                "{filename}: timeline[{i}] time should be >= 0, got {t}"
            );
        }

        // Build judge notes and verify they are non-empty with valid times
        let judge_notes = model.build_judge_notes();
        assert!(
            !judge_notes.is_empty(),
            "{filename}: judge notes should be non-empty for a real chart"
        );
        for (i, jn) in judge_notes.iter().enumerate() {
            assert!(
                jn.time_us >= 0,
                "{filename}: judge_note[{i}] time_us should be >= 0, got {}",
                jn.time_us
            );
        }
    }
}

// ============================================================================
// Test 8: ScrollSpeedModifier (Remove and Add modes) does not panic
// ============================================================================

#[test]
fn real_bms_scroll_speed_modifier_no_panic() {
    let files = discover_bms_files();

    for path in &files {
        let model = decode_bms(path);
        let filename = path.file_name().unwrap().to_string_lossy();

        // ScrollSpeedModifier with Mode::Remove (index 0)
        {
            let mut model_remove = model.clone();
            let mut modifier = ScrollSpeedModifier::with_params(0, 4, 0.5);
            modifier.modify(&mut model_remove);
            // After Remove mode, all timelines should have uniform BPM
            let tls = model_remove.get_all_time_lines();
            if tls.len() > 1 {
                let start_bpm = tls[0].get_bpm();
                for (i, tl) in tls.iter().enumerate().skip(1) {
                    assert!(
                        (tl.get_bpm() - start_bpm).abs() < f64::EPSILON,
                        "{filename}: after Remove, timeline[{i}] BPM should be {start_bpm}, got {}",
                        tl.get_bpm()
                    );
                }
            }
        }

        // ScrollSpeedModifier with Mode::Add (index 1)
        {
            let mut model_add = model.clone();
            let mut modifier = ScrollSpeedModifier::with_params(1, 4, 0.5);
            modifier.modify(&mut model_add);
            // Just verify no panic; scroll values are randomized so we only
            // check that timelines still exist
            assert_eq!(
                model_add.get_all_time_lines().len(),
                model.get_all_time_lines().len(),
                "{filename}: Add mode should not change timeline count"
            );
        }
    }
}
