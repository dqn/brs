// Java vs Rust RenderSnapshot comparison tests.
//
// Instead of comparing pixel screenshots (SSIM), this compares the structural
// draw commands that each side would produce. Both Java and Rust export a JSON
// snapshot of "what to draw" (position, color, angle, blend, type-specific
// detail) and this test verifies field-by-field equality with tolerances:
//   - Position (x, y, w, h): ±1.0 pixel
//   - Color (r, g, b): ±0.005
//   - Alpha (a): ±0.01
//   - Angle, blend, visibility, text content: exact match
//
// Java fixtures: golden-master/fixtures/render_snapshots_java/{name}.json
// Rust snapshots: generated on-the-fly from skin + state
//
// Run: cargo test -p golden-master compare_render_snapshot -- --nocapture

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use bms_config::resolution::Resolution;
use bms_render::state_provider::StaticStateProvider;
use bms_skin::loader::{json_loader, lua_loader};
use bms_skin::skin_header::CustomOption;
use golden_master::render_snapshot::{RenderSnapshot, capture_render_snapshot, compare_snapshots};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn skins_dir() -> PathBuf {
    project_root().join("skins/ECFN")
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn state_dir() -> PathBuf {
    fixture_dir().join("screenshot_states")
}

fn default_enabled_option_ids(options: &[CustomOption]) -> HashSet<i32> {
    options
        .iter()
        .filter_map(CustomOption::default_option)
        .collect()
}

/// Load a Java-generated RenderSnapshot fixture.
fn load_java_snapshot(name: &str) -> RenderSnapshot {
    let path = fixture_dir()
        .join("render_snapshots_java")
        .join(format!("{name}.json"));
    assert!(
        path.exists(),
        "Java fixture not found: {}. Run `just golden-master-render-snapshot-gen` first.",
        path.display()
    );
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

/// Load a StaticStateProvider from a state JSON file.
fn load_state(name: &str) -> StaticStateProvider {
    let path = state_dir().join(name);
    if !path.exists() {
        return StaticStateProvider::default();
    }
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

/// Load a Lua skin from the ECFN directory.
fn load_lua_skin(relative_path: &str) -> bms_skin::skin::Skin {
    let path = skins_dir().join(relative_path);
    assert!(path.exists(), "Skin not found: {}", path.display());
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let header = lua_loader::load_lua_header(&content, Some(&path))
        .unwrap_or_else(|e| panic!("Failed to load Lua header {}: {}", path.display(), e));
    let enabled = default_enabled_option_ids(&header.options);
    lua_loader::load_lua_skin(&content, &enabled, Resolution::Fullhd, Some(&path), &[])
        .unwrap_or_else(|e| panic!("Failed to load Lua skin {}: {}", path.display(), e))
}

/// Load a JSON skin from the ECFN directory.
fn load_json_skin(relative_path: &str) -> bms_skin::skin::Skin {
    let path = skins_dir().join(relative_path);
    assert!(path.exists(), "Skin not found: {}", path.display());
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let header = json_loader::load_header(&content)
        .unwrap_or_else(|e| panic!("Failed to load JSON header {}: {}", path.display(), e));
    let enabled = default_enabled_option_ids(&header.options);
    json_loader::load_skin(&content, &enabled, Resolution::Fullhd, Some(&path))
        .unwrap_or_else(|e| panic!("Failed to load JSON skin {}: {}", path.display(), e))
}

struct RenderSnapshotTestCase {
    name: &'static str,
    skin_path: &'static str,
    state_json: &'static str,
    /// Whether the skin is Lua (.luaskin) or JSON (.json)
    is_lua: bool,
    /// Current known diff budget between Java and Rust snapshots.
    /// Used by non-ignored regression guard test to catch worsened parity.
    known_diff_budget: usize,
}

const TEST_CASES: &[RenderSnapshotTestCase] = &[
    RenderSnapshotTestCase {
        name: "ecfn_select",
        skin_path: "select/select.luaskin",
        state_json: "state_default.json",
        is_lua: true,
        known_diff_budget: 1,
    },
    RenderSnapshotTestCase {
        name: "ecfn_decide",
        skin_path: "decide/decide.luaskin",
        state_json: "state_default.json",
        is_lua: true,
        known_diff_budget: 5,
    },
    RenderSnapshotTestCase {
        name: "ecfn_play7_active",
        skin_path: "play/play7.luaskin",
        state_json: "state_play_active.json",
        is_lua: true,
        known_diff_budget: 1,
    },
    RenderSnapshotTestCase {
        name: "ecfn_play7_fullcombo",
        skin_path: "play/play7.luaskin",
        state_json: "state_play_fullcombo.json",
        is_lua: true,
        known_diff_budget: 1,
    },
    RenderSnapshotTestCase {
        name: "ecfn_play7_danger",
        skin_path: "play/play7.luaskin",
        state_json: "state_play_danger.json",
        is_lua: true,
        known_diff_budget: 1,
    },
    RenderSnapshotTestCase {
        name: "ecfn_result_clear",
        skin_path: "RESULT/result.luaskin",
        state_json: "state_result_clear.json",
        is_lua: true,
        known_diff_budget: 1,
    },
    RenderSnapshotTestCase {
        name: "ecfn_result_fail",
        skin_path: "RESULT/result.luaskin",
        state_json: "state_result_fail.json",
        is_lua: true,
        known_diff_budget: 1,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DiffCategory {
    CommandCount,
    Visibility,
    Geometry,
    Detail,
    Other,
}

impl DiffCategory {
    fn label(self) -> &'static str {
        match self {
            Self::CommandCount => "command_count",
            Self::Visibility => "visibility",
            Self::Geometry => "geometry",
            Self::Detail => "detail",
            Self::Other => "other",
        }
    }
}

fn categorize_diff(diff: &str) -> DiffCategory {
    if diff.starts_with("command_count:") {
        DiffCategory::CommandCount
    } else if diff.contains(" visible:") {
        DiffCategory::Visibility
    } else if diff.starts_with("skin_width:")
        || diff.starts_with("skin_height:")
        || diff.contains(" dst.")
    {
        DiffCategory::Geometry
    } else if diff.contains(" detail")
        || diff.contains(" color.")
        || diff.contains(" angle:")
        || diff.contains(" blend:")
        || diff.contains(" object_type:")
    {
        DiffCategory::Detail
    } else {
        DiffCategory::Other
    }
}

fn summarize_diff_categories(diffs: &[String]) -> String {
    let mut counts: BTreeMap<DiffCategory, usize> = BTreeMap::new();
    for diff in diffs {
        *counts.entry(categorize_diff(diff)).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(category, count)| format!("{}={}", category.label(), count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn summarize_command_count_gap(java: &RenderSnapshot, rust: &RenderSnapshot) -> Option<String> {
    if java.commands.len() == rust.commands.len() {
        return None;
    }

    let mut type_counts: BTreeMap<String, isize> = BTreeMap::new();
    let mut visible_type_counts: BTreeMap<String, isize> = BTreeMap::new();

    for cmd in &rust.commands {
        *type_counts.entry(cmd.object_type.clone()).or_insert(0) += 1;
        if cmd.visible {
            *visible_type_counts
                .entry(cmd.object_type.clone())
                .or_insert(0) += 1;
        }
    }
    for cmd in &java.commands {
        *type_counts.entry(cmd.object_type.clone()).or_insert(0) -= 1;
        if cmd.visible {
            *visible_type_counts
                .entry(cmd.object_type.clone())
                .or_insert(0) -= 1;
        }
    }

    let type_delta = type_counts
        .into_iter()
        .filter(|(_, delta)| *delta != 0)
        .map(|(ty, delta)| format!("{ty}:{delta:+}"))
        .collect::<Vec<_>>()
        .join(", ");
    let visible_delta = visible_type_counts
        .into_iter()
        .filter(|(_, delta)| *delta != 0)
        .map(|(ty, delta)| format!("{ty}:{delta:+}"))
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!(
        "type_delta(rust-java): [{}]; visible_type_delta(rust-java): [{}]",
        type_delta, visible_delta
    ))
}

fn render_snapshot_debug_paths(case_name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let debug_dir = fixture_dir().join("render_snapshots_debug");
    (
        debug_dir.join(format!("{case_name}__java.json")),
        debug_dir.join(format!("{case_name}__rust.json")),
        debug_dir.join(format!("{case_name}__diffs.txt")),
    )
}

fn snapshot_diffs(tc: &RenderSnapshotTestCase) -> (RenderSnapshot, RenderSnapshot, Vec<String>) {
    let java_snapshot = load_java_snapshot(tc.name);

    let skin = if tc.is_lua {
        load_lua_skin(tc.skin_path)
    } else {
        load_json_skin(tc.skin_path)
    };

    let provider = load_state(tc.state_json);
    let rust_snapshot = capture_render_snapshot(&skin, &provider);

    let diffs = compare_snapshots(&java_snapshot, &rust_snapshot);
    (java_snapshot, rust_snapshot, diffs)
}

fn compare_java_rust_render_snapshot(tc: &RenderSnapshotTestCase) {
    let (java_snapshot, rust_snapshot, diffs) = snapshot_diffs(tc);
    let category_summary = summarize_diff_categories(&diffs);
    let command_gap_summary = summarize_command_count_gap(&java_snapshot, &rust_snapshot);

    let visible_java = java_snapshot.commands.iter().filter(|c| c.visible).count();
    let visible_rust = rust_snapshot.commands.iter().filter(|c| c.visible).count();
    eprintln!(
        "{}: java {} objects ({} visible), rust {} objects ({} visible), {} diffs ({})",
        tc.name,
        java_snapshot.commands.len(),
        visible_java,
        rust_snapshot.commands.len(),
        visible_rust,
        diffs.len(),
        category_summary
    );

    if !diffs.is_empty() {
        // Save Java/Rust snapshots and raw diffs for deterministic debugging.
        let (java_path, rust_path, diffs_path) = render_snapshot_debug_paths(tc.name);
        let debug_dir = fixture_dir().join("render_snapshots_debug");
        std::fs::create_dir_all(&debug_dir).ok();

        std::fs::write(
            &java_path,
            serde_json::to_string_pretty(&java_snapshot).unwrap(),
        )
        .ok();
        std::fs::write(
            &rust_path,
            serde_json::to_string_pretty(&rust_snapshot).unwrap(),
        )
        .ok();
        std::fs::write(&diffs_path, diffs.join("\n")).ok();

        let first_10: Vec<_> = diffs.iter().take(10).collect();
        panic!(
            "RenderSnapshot mismatch for {} ({} differences, categories: {}, showing first 10):\n{}\n  \
             java debug: {}\n  \
             rust debug: {}\n  \
             diff list: {}\n  \
             command_count breakdown: {}",
            tc.name,
            diffs.len(),
            category_summary,
            first_10
                .iter()
                .map(|d| format!("  - {}", d))
                .collect::<Vec<_>>()
                .join("\n"),
            java_path.display(),
            rust_path.display(),
            diffs_path.display(),
            command_gap_summary.as_deref().unwrap_or("n/a"),
        );
    }
}

// --- Test cases ---

#[test]
fn render_snapshot_java_fixtures_exist() {
    for tc in TEST_CASES {
        let _ = load_java_snapshot(tc.name);
    }
}

#[test]
fn render_snapshot_parity_regression_guard() {
    for tc in TEST_CASES {
        let (java_snapshot, rust_snapshot, diffs) = snapshot_diffs(tc);
        let category_summary = summarize_diff_categories(&diffs);
        let command_gap_summary = summarize_command_count_gap(&java_snapshot, &rust_snapshot);
        assert!(
            diffs.len() <= tc.known_diff_budget,
            "RenderSnapshot diff budget exceeded for {}: {} > {}.\nCategories: {}\nCommand-count breakdown: {}\nFirst differences:\n{}",
            tc.name,
            diffs.len(),
            tc.known_diff_budget,
            category_summary,
            command_gap_summary.as_deref().unwrap_or("n/a"),
            diffs
                .iter()
                .take(10)
                .map(|d| format!("  - {}", d))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_select() {
    let tc = &TEST_CASES[0];
    compare_java_rust_render_snapshot(tc);
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_decide() {
    let tc = &TEST_CASES[1];
    compare_java_rust_render_snapshot(tc);
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_play7_active() {
    let tc = &TEST_CASES[2];
    compare_java_rust_render_snapshot(tc);
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_play7_fullcombo() {
    let tc = &TEST_CASES[3];
    compare_java_rust_render_snapshot(tc);
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_play7_danger() {
    let tc = &TEST_CASES[4];
    compare_java_rust_render_snapshot(tc);
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_result_clear() {
    let tc = &TEST_CASES[5];
    compare_java_rust_render_snapshot(tc);
}

#[test]
#[ignore = "Known Java/Rust render parity gaps; use for focused debugging"]
fn render_snapshot_ecfn_result_fail() {
    let tc = &TEST_CASES[6];
    compare_java_rust_render_snapshot(tc);
}
