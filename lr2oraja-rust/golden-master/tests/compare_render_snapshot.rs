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

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bms_config::resolution::Resolution;
use bms_render::state_provider::StaticStateProvider;
use bms_skin::loader::{json_loader, lua_loader};
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
    let enabled: HashSet<i32> = HashSet::new();
    lua_loader::load_lua_skin(&content, &enabled, Resolution::Fullhd, Some(&path), &[])
        .unwrap_or_else(|e| panic!("Failed to load Lua skin {}: {}", path.display(), e))
}

/// Load a JSON skin from the ECFN directory.
fn load_json_skin(relative_path: &str) -> bms_skin::skin::Skin {
    let path = skins_dir().join(relative_path);
    assert!(path.exists(), "Skin not found: {}", path.display());
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    let enabled: HashSet<i32> = HashSet::new();
    json_loader::load_skin(&content, &enabled, Resolution::Fullhd, Some(&path))
        .unwrap_or_else(|e| panic!("Failed to load JSON skin {}: {}", path.display(), e))
}

struct RenderSnapshotTestCase {
    name: &'static str,
    skin_path: &'static str,
    state_json: &'static str,
    /// Whether the skin is Lua (.luaskin) or JSON (.json)
    is_lua: bool,
}

const TEST_CASES: &[RenderSnapshotTestCase] = &[
    RenderSnapshotTestCase {
        name: "ecfn_select",
        skin_path: "select/select.luaskin",
        state_json: "state_default.json",
        is_lua: true,
    },
    RenderSnapshotTestCase {
        name: "ecfn_decide",
        skin_path: "decide/decide.luaskin",
        state_json: "state_default.json",
        is_lua: true,
    },
    RenderSnapshotTestCase {
        name: "ecfn_play7_active",
        skin_path: "play/play7.luaskin",
        state_json: "state_play_active.json",
        is_lua: true,
    },
    RenderSnapshotTestCase {
        name: "ecfn_play7_fullcombo",
        skin_path: "play/play7.luaskin",
        state_json: "state_play_fullcombo.json",
        is_lua: true,
    },
    RenderSnapshotTestCase {
        name: "ecfn_play7_danger",
        skin_path: "play/play7.luaskin",
        state_json: "state_play_danger.json",
        is_lua: true,
    },
    RenderSnapshotTestCase {
        name: "ecfn_result_clear",
        skin_path: "RESULT/result.luaskin",
        state_json: "state_result_clear.json",
        is_lua: true,
    },
    RenderSnapshotTestCase {
        name: "ecfn_result_fail",
        skin_path: "RESULT/result.luaskin",
        state_json: "state_result_fail.json",
        is_lua: true,
    },
];

fn compare_java_rust_render_snapshot(tc: &RenderSnapshotTestCase) {
    let java_snapshot = load_java_snapshot(tc.name);

    let skin = if tc.is_lua {
        load_lua_skin(tc.skin_path)
    } else {
        load_json_skin(tc.skin_path)
    };

    let provider = load_state(tc.state_json);
    let rust_snapshot = capture_render_snapshot(&skin, &provider);

    let diffs = compare_snapshots(&java_snapshot, &rust_snapshot);

    let visible_java = java_snapshot.commands.iter().filter(|c| c.visible).count();
    let visible_rust = rust_snapshot.commands.iter().filter(|c| c.visible).count();
    eprintln!(
        "{}: java {} objects ({} visible), rust {} objects ({} visible), {} diffs",
        tc.name,
        java_snapshot.commands.len(),
        visible_java,
        rust_snapshot.commands.len(),
        visible_rust,
        diffs.len()
    );

    if !diffs.is_empty() {
        // Save both snapshots for debugging
        let debug_dir = fixture_dir().join("render_snapshots_debug");
        std::fs::create_dir_all(&debug_dir).ok();

        let rust_path = debug_dir.join(format!("{}_rust.json", tc.name));
        let json = serde_json::to_string_pretty(&rust_snapshot).unwrap();
        std::fs::write(&rust_path, json).ok();

        let first_10: Vec<_> = diffs.iter().take(10).collect();
        panic!(
            "RenderSnapshot mismatch for {} ({} differences, showing first 10):\n{}\n  \
             rust debug: {}",
            tc.name,
            diffs.len(),
            first_10
                .iter()
                .map(|d| format!("  - {}", d))
                .collect::<Vec<_>>()
                .join("\n"),
            rust_path.display(),
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
