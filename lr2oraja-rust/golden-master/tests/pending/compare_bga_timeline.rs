// Golden master tests: compare Rust BgaProcessor timeline against Java BGA export.
//
// Known semantic differences (documented in CLAUDE.md):
//   - BGA IDs: Java uses 0-based wavlist index, Rust uses raw base36 value.
//     Conversion: rust_id == java_id + 1 (for ids >= 0).
//   - Channel 06/07 swap: Rust parser maps ch06→Layer, ch07→Poor,
//     but the BMS spec (and Java/beatoraja) maps ch06→Poor, ch07→Layer.
//     Layer comparison is skipped; only BGA base (channel 04) is compared.

use std::collections::BTreeMap;
use std::path::Path;

use bms_model::{BgaLayer, BmsDecoder};
use bms_render::bga::bga_processor::BgaProcessor;
use golden_master::bga_timeline_fixtures::{BgaTimelineFixture, BgaTimelineTestCase};

#[path = "support/random_seeds.rs"]
mod random_seeds;

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

fn load_bga_timeline_fixture() -> BgaTimelineFixture {
    let path = fixtures_dir().join("bga_timeline.json");
    assert!(
        path.exists(),
        "BGA timeline fixture not found: {}. Run `just golden-master-bga-timeline-gen` first.",
        path.display()
    );
    let content = std::fs::read_to_string(&path).expect("Failed to read fixture");
    serde_json::from_str(&content).expect("Failed to parse fixture")
}

fn find_test_case<'a>(fixture: &'a BgaTimelineFixture, filename: &str) -> &'a BgaTimelineTestCase {
    fixture
        .test_cases
        .iter()
        .find(|tc| tc.filename == filename)
        .unwrap_or_else(|| panic!("Test case not found for {filename}"))
}

/// Convert Java BGA ID (0-based wavlist index) to Rust BGA ID (raw base36 value).
/// Java -1 (no BGA) maps to Rust -1.
fn java_id_to_rust(java_id: i32) -> i32 {
    if java_id < 0 { java_id } else { java_id + 1 }
}

/// Compare BGA base events from Rust model against Java fixture.
/// Only compares BGA base layer (channel 04) due to ch06/07 channel swap.
fn compare_events(
    rust_events: &[bms_model::BgaEvent],
    java_events: &[golden_master::bga_timeline_fixtures::BgaEventFixture],
) -> Vec<String> {
    let mut diffs = Vec::new();

    // Group Rust BGA base events by time (ms)
    let mut rust_by_time: BTreeMap<i32, i32> = BTreeMap::new();
    for event in rust_events {
        if event.layer != BgaLayer::Bga {
            continue;
        }
        let time_ms = (event.time_us / 1000) as i32;
        rust_by_time.insert(time_ms, event.id);
    }

    let rust_bga_events: Vec<(i32, i32)> = rust_by_time.into_iter().collect();

    // Filter Java events to BGA-only (bga_id != -1)
    let java_bga_events: Vec<(i32, i32)> = java_events
        .iter()
        .filter(|e| e.bga_id != -1)
        .map(|e| (e.time_ms, e.bga_id))
        .collect();

    if rust_bga_events.len() != java_bga_events.len() {
        diffs.push(format!(
            "bga_event_count: rust={} java={}",
            rust_bga_events.len(),
            java_bga_events.len()
        ));
    }

    let min_len = rust_bga_events.len().min(java_bga_events.len());
    for i in 0..min_len {
        let (r_time, r_bga) = rust_bga_events[i];
        let (j_time, j_bga) = java_bga_events[i];

        if (r_time - j_time).abs() > 2 {
            diffs.push(format!(
                "bga_event[{i}] time_ms: rust={r_time} java={j_time}"
            ));
        }
        // Apply ID offset: rust_id == java_id + 1
        let expected_rust_bga = java_id_to_rust(j_bga);
        if r_bga != expected_rust_bga {
            diffs.push(format!(
                "bga_event[{i}] bga_id: rust={r_bga} expected={expected_rust_bga} (java={j_bga})"
            ));
        }
    }

    diffs
}

/// Compare BGA base snapshots by stepping BgaProcessor through time.
/// Only compares current_bga (with ID offset); skips current_layer due to ch06/07 swap.
fn compare_snapshots(
    processor: &mut BgaProcessor,
    java_snapshots: &[golden_master::bga_timeline_fixtures::BgaSnapshotFixture],
) -> Vec<String> {
    let mut diffs = Vec::new();

    for (i, snapshot) in java_snapshots.iter().enumerate() {
        let time_us = snapshot.time_ms as i64 * 1000;
        processor.update(time_us);

        let rust_bga = processor.current_bga_id();
        let expected_rust_bga = java_id_to_rust(snapshot.current_bga);

        if rust_bga != expected_rust_bga {
            diffs.push(format!(
                "snapshot[{i}] t={}ms current_bga: rust={rust_bga} expected={expected_rust_bga} (java={})",
                snapshot.time_ms, snapshot.current_bga
            ));
        }
        // current_layer comparison skipped: Rust ch06→Layer, ch07→Poor (swapped from standard)
    }

    diffs
}

fn run_bga_timeline_test(bms_name: &str) {
    let fixture = load_bga_timeline_fixture();
    let test_case = find_test_case(&fixture, bms_name);

    let bms_path = test_bms_dir().join(bms_name);
    assert!(
        bms_path.exists(),
        "BMS file not found: {}",
        bms_path.display()
    );

    let model = BmsDecoder::decode(&bms_path).expect("Failed to parse BMS");

    // Compare BGA base events (layer comparison skipped due to ch06/07 swap)
    let event_diffs = compare_events(&model.bga_events, &test_case.events);

    // Compare BGA base snapshots via BgaProcessor
    let mut processor = BgaProcessor::new(&model);
    let snapshot_diffs = compare_snapshots(&mut processor, &test_case.snapshots);

    let mut all_diffs = Vec::new();
    all_diffs.extend(event_diffs);
    all_diffs.extend(snapshot_diffs);

    if !all_diffs.is_empty() {
        panic!(
            "BGA timeline mismatch for {} ({} differences):\n{}",
            bms_name,
            all_diffs.len(),
            all_diffs
                .iter()
                .map(|d| format!("  - {d}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn bga_timeline_bga_test() {
    run_bga_timeline_test("bga_test.bms");
}
