// Golden master tests: compare Rust BMS parser output against Java fixtures

use std::path::Path;

use bms_model::BmsDecoder;
use golden_master::{assert_model_matches_fixture, load_fixture};

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

/// Test a single BMS file against its Java fixture
fn run_golden_master_test(bms_name: &str) {
    let fixture_name = bms_name.replace(".bms", ".json");
    let fixture_path = fixtures_dir().join(&fixture_name);
    let bms_path = test_bms_dir().join(bms_name);

    assert!(
        fixture_path.exists(),
        "Fixture not found: {}",
        fixture_path.display()
    );
    assert!(
        bms_path.exists(),
        "BMS file not found: {}",
        bms_path.display()
    );

    let fixture = load_fixture(&fixture_path).expect("Failed to load fixture");
    let model = BmsDecoder::decode(&bms_path).expect("Failed to parse BMS");

    assert_model_matches_fixture(&model, &fixture);
}

#[test]
fn golden_master_minimal_7k() {
    run_golden_master_test("minimal_7k.bms");
}

#[test]
fn golden_master_5key() {
    run_golden_master_test("5key.bms");
}

#[test]
fn golden_master_longnote_types() {
    run_golden_master_test("longnote_types.bms");
}

#[test]
fn golden_master_bpm_change() {
    run_golden_master_test("bpm_change.bms");
}

#[test]
fn golden_master_stop_sequence() {
    run_golden_master_test("stop_sequence.bms");
}

#[test]
fn golden_master_mine_notes() {
    run_golden_master_test("mine_notes.bms");
}

#[test]
fn golden_master_empty_measures() {
    run_golden_master_test("empty_measures.bms");
}

#[test]
fn golden_master_9key_pms() {
    run_golden_master_test("9key_pms.bms");
}

#[test]
fn golden_master_14key_dp() {
    run_golden_master_test("14key_dp.bms");
}

#[test]
fn golden_master_scratch_bss() {
    run_golden_master_test("scratch_bss.bms");
}

#[test]
fn golden_master_encoding_sjis() {
    run_golden_master_test("encoding_sjis.bms");
}

#[test]
fn golden_master_encoding_utf8() {
    run_golden_master_test("encoding_utf8.bms");
}

#[test]
fn golden_master_random_if() {
    // Uses fixed selectedRandoms=[1] to select #IF 1 branch deterministically.
    // Matching random_seeds.json on the Java side.
    let fixture_path = fixtures_dir().join("random_if.json");
    let bms_path = test_bms_dir().join("random_if.bms");

    assert!(
        fixture_path.exists(),
        "Fixture not found: {}",
        fixture_path.display()
    );
    assert!(
        bms_path.exists(),
        "BMS file not found: {}",
        bms_path.display()
    );

    let fixture = load_fixture(&fixture_path).expect("Failed to load fixture");
    let model = BmsDecoder::decode_with_randoms(&bms_path, &[1]).expect("Failed to parse BMS");

    assert_model_matches_fixture(&model, &fixture);
}
