// Golden master tests for skin loaders.
//
// Tests JSON, Lua, and LR2 CSV skin loaders by loading real skin files
// and verifying structural properties. Snapshot tests compare full
// SkinSnapshot JSON for regression detection.
//
// NOTE: Some ECFN skin files have issues:
// - select.json has malformed JSON (missing comma between elements)
// - select.luaskin and result.luaskin require("main_state") from play/
//   directory (cross-directory require unsupported by current Lua loader)
// These tests are marked #[ignore] until the loaders are enhanced.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bms_config::resolution::Resolution;
use bms_config::skin_type::SkinType;
use bms_skin::loader::json_loader;
use bms_skin::loader::lr2_csv_loader::load_lr2_skin;
use bms_skin::loader::lr2_header_loader::load_lr2_header;
use bms_skin::loader::lua_loader;
use bms_skin::skin_header::SkinFormat;
use golden_master::skin_fixtures::{
    load_snapshot, save_snapshot, should_update_snapshots, snapshot_from_skin,
};

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

fn test_bms_dir() -> PathBuf {
    project_root().join("test-bms")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

// ===========================================================================
// JSON Loader tests (using test_skin.json)
// ===========================================================================

#[test]
fn json_test_header() {
    let path = test_bms_dir().join("test_skin.json");
    let content = read_file(&path);
    let header = json_loader::load_header(&content).expect("Failed to load JSON header");

    assert_eq!(header.format, SkinFormat::Beatoraja);
    assert_eq!(header.skin_type, Some(SkinType::Decide));
    assert_eq!(header.name, "Test Skin");
    assert_eq!(header.author, "Test");
    assert_eq!(header.resolution.width(), 1280);
    assert_eq!(header.resolution.height(), 720);
    assert!(header.options.is_empty());
    assert!(header.files.is_empty());
}

#[test]
fn json_test_load() {
    let path = test_bms_dir().join("test_skin.json");
    let content = read_file(&path);
    let enabled: HashSet<i32> = HashSet::new();
    let skin = json_loader::load_skin(&content, &enabled, Resolution::Hd, Some(&path))
        .expect("Failed to load JSON skin");

    // Resolution: source 1280x720, dest HD 1280x720 -> scale 1.0
    assert_eq!(skin.width, 1280.0);
    assert_eq!(skin.height, 720.0);
    assert!((skin.scale_x - 1.0).abs() < 0.001);
    assert!((skin.scale_y - 1.0).abs() < 0.001);

    // Timing
    assert_eq!(skin.input, 400);
    assert_eq!(skin.scene, 3000);
    assert_eq!(skin.fadeout, 500);

    // 7 destinations -> 7 objects
    assert_eq!(skin.object_count(), 7);

    let snap = snapshot_from_skin(&skin);
    assert_eq!(snap.objects_by_type.get("Image").copied().unwrap_or(0), 3);
    assert_eq!(snap.objects_by_type.get("Number").copied().unwrap_or(0), 1);
    assert_eq!(snap.objects_by_type.get("Text").copied().unwrap_or(0), 1);
    assert_eq!(snap.objects_by_type.get("Slider").copied().unwrap_or(0), 1);
    assert_eq!(snap.objects_by_type.get("Graph").copied().unwrap_or(0), 1);
}

#[test]
fn json_test_objects() {
    let path = test_bms_dir().join("test_skin.json");
    let content = read_file(&path);
    let enabled: HashSet<i32> = HashSet::new();
    let skin = json_loader::load_skin(&content, &enabled, Resolution::Hd, Some(&path))
        .expect("Failed to load JSON skin");

    let snap = snapshot_from_skin(&skin);

    // All objects should have at least one destination
    for (i, obj) in snap.objects.iter().enumerate() {
        assert!(
            obj.destination_count > 0,
            "Object {} ({}) should have destinations",
            i,
            obj.kind
        );
    }

    // First object: bg image, 1 dst, no timer, blend 0
    assert_eq!(snap.objects[0].kind, "Image");
    assert_eq!(snap.objects[0].destination_count, 1);
    assert!(snap.objects[0].timer_id.is_none());
    assert_eq!(snap.objects[0].blend, 0);

    // Second object: icon image with timer 40
    assert_eq!(snap.objects[1].kind, "Image");
    assert_eq!(snap.objects[1].timer_id, Some(40));

    // Third object: panel with blend=2 (additive), 2 destinations
    assert_eq!(snap.objects[2].kind, "Image");
    assert_eq!(snap.objects[2].blend, 2);
    assert_eq!(snap.objects[2].destination_count, 2);
}

#[test]
fn json_test_snapshot() {
    let path = test_bms_dir().join("test_skin.json");
    let content = read_file(&path);
    let enabled: HashSet<i32> = HashSet::new();
    let skin = json_loader::load_skin(&content, &enabled, Resolution::Hd, Some(&path))
        .expect("Failed to load JSON skin");

    let snap = snapshot_from_skin(&skin);
    let fixture_path = fixtures_dir().join("skin_json_test_snapshot.json");

    if should_update_snapshots() {
        save_snapshot(&snap, &fixture_path).expect("Failed to save snapshot");
        eprintln!("Updated snapshot: {}", fixture_path.display());
        return;
    }

    if !fixture_path.exists() {
        save_snapshot(&snap, &fixture_path).expect("Failed to save initial snapshot");
        eprintln!(
            "Created initial snapshot: {}. Re-run to verify.",
            fixture_path.display()
        );
        return;
    }

    let expected = load_snapshot(&fixture_path).expect("Failed to load fixture snapshot");
    assert_eq!(snap, expected, "Full snapshot mismatch");
}

// ECFN JSON tests (malformed JSON — missing comma between elements)
#[ignore = "select.json has malformed JSON (missing comma at line 211-212)"]
#[test]
fn json_ecfn_select_header() {
    let path = skins_dir().join("select/select.json");
    let content = read_file(&path);
    let header = json_loader::load_header(&content).expect("Failed to load JSON header");

    assert_eq!(header.skin_type, Some(SkinType::MusicSelect));
    assert_eq!(header.name, "beatoraja_default");
}

// ===========================================================================
// Lua Loader tests
// ===========================================================================

#[test]
fn lua_play7_header() {
    let path = skins_dir().join("play/play7.luaskin");
    let content = read_file(&path);
    let header =
        lua_loader::load_lua_header(&content, Some(&path)).expect("Failed to load play7 header");

    // Lua loader delegates to JSON loader, so format is Beatoraja
    assert_eq!(header.format, SkinFormat::Beatoraja);
    assert_eq!(header.skin_type, Some(SkinType::Play7Keys));
    assert_eq!(header.name, "EC:FN(7K:AC)");
    assert_eq!(header.resolution.width(), 1920);
    assert_eq!(header.resolution.height(), 1080);
    assert_eq!(header.options.len(), 12, "property count");
    assert_eq!(header.files.len(), 14, "filepath count");
    assert_eq!(header.offsets.len(), 12, "offset count");
}

#[test]
fn lua_decide_header() {
    let path = skins_dir().join("decide/decide.luaskin");
    let content = read_file(&path);
    let header =
        lua_loader::load_lua_header(&content, Some(&path)).expect("Failed to load decide header");

    assert_eq!(header.format, SkinFormat::Beatoraja);
    assert_eq!(header.skin_type, Some(SkinType::Decide));
    assert_eq!(header.name, "EC:FN");
    assert_eq!(header.resolution.width(), 1920);
    assert_eq!(header.resolution.height(), 1080);
    assert!(header.options.is_empty(), "no custom options");
    assert_eq!(header.files.len(), 1, "filepath count");
}

#[test]
fn lua_decide_load() {
    let path = skins_dir().join("decide/decide.luaskin");
    let content = read_file(&path);
    let enabled: HashSet<i32> = HashSet::new();
    let skin = lua_loader::load_lua_skin(&content, &enabled, Resolution::Fullhd, Some(&path), &[])
        .expect("Failed to load Lua decide skin");

    assert_eq!(skin.width, 1920.0);
    assert_eq!(skin.height, 1080.0);
    assert_eq!(skin.scene, 2500);
    assert_eq!(skin.input, 500);
    assert_eq!(skin.fadeout, 500);
    assert!(
        skin.object_count() > 0,
        "Lua decide skin should have objects"
    );

    let snap = snapshot_from_skin(&skin);
    assert!(snap.objects_by_type.contains_key("Image"));
    assert!(snap.objects_by_type.contains_key("Text"));
}

#[test]
fn lua_decide_snapshot() {
    let path = skins_dir().join("decide/decide.luaskin");
    let content = read_file(&path);
    let enabled: HashSet<i32> = HashSet::new();
    let skin = lua_loader::load_lua_skin(&content, &enabled, Resolution::Fullhd, Some(&path), &[])
        .expect("Failed to load Lua skin");

    let snap = snapshot_from_skin(&skin);
    let fixture_path = fixtures_dir().join("skin_lua_decide_snapshot.json");

    if should_update_snapshots() {
        save_snapshot(&snap, &fixture_path).expect("Failed to save snapshot");
        eprintln!("Updated snapshot: {}", fixture_path.display());
        return;
    }

    if !fixture_path.exists() {
        save_snapshot(&snap, &fixture_path).expect("Failed to save initial snapshot");
        eprintln!(
            "Created initial snapshot: {}. Re-run to verify.",
            fixture_path.display()
        );
        return;
    }

    let expected = load_snapshot(&fixture_path).expect("Failed to load fixture snapshot");
    assert_eq!(snap, expected, "Full snapshot mismatch");
}

// Cross-directory require tests (Lua skins needing main_state from play/)
#[ignore = "select.lua requires main_state.lua from play/ (cross-directory)"]
#[test]
fn lua_select_header() {
    let path = skins_dir().join("select/select.luaskin");
    let content = read_file(&path);
    let header =
        lua_loader::load_lua_header(&content, Some(&path)).expect("Failed to load Lua header");

    assert_eq!(header.skin_type, Some(SkinType::MusicSelect));
    assert_eq!(header.name, "EC:FN / MusicSelect");
}

#[ignore = "result.lua requires main_state.lua from play/ (cross-directory)"]
#[test]
fn lua_result_header() {
    let path = skins_dir().join("RESULT/result.luaskin");
    let content = read_file(&path);
    let header =
        lua_loader::load_lua_header(&content, Some(&path)).expect("Failed to load result header");

    assert_eq!(header.skin_type, Some(SkinType::Result));
}

// ===========================================================================
// LR2 CSV Loader tests
// ===========================================================================

fn lr2_skin_path() -> PathBuf {
    test_bms_dir().join("test_skin.lr2skin")
}

#[test]
fn lr2_csv_header() {
    let content = read_file(&lr2_skin_path());
    let header = load_lr2_header(&content, None).expect("Failed to load LR2 header");

    assert_eq!(header.format, SkinFormat::Lr2);
    assert_eq!(header.resolution, Resolution::Hd);
    assert_eq!(header.options.len(), 1, "CUSTOMOPTION count");
    assert_eq!(header.options[0].name, "TestOption");
    assert_eq!(header.options[0].option_ids, vec![900, 901]);
    assert_eq!(header.options[0].contents, vec!["ON", "OFF"]);
    assert_eq!(header.files.len(), 1, "CUSTOMFILE count");
    assert_eq!(header.files[0].name, "Background");
    assert_eq!(header.offsets.len(), 1, "CUSTOMOFFSET count");
    assert_eq!(header.offsets[0].name, "LiftOffset");
    assert_eq!(header.offsets[0].id, 3);
    assert!(!header.offsets[0].editable_x);
    assert!(header.offsets[0].editable_y);
}

#[test]
fn lr2_csv_load() {
    let content = read_file(&lr2_skin_path());
    let header = load_lr2_header(&content, None).expect("Failed to load LR2 header");
    let enabled = std::collections::HashMap::new();
    let skin =
        load_lr2_skin(&content, header, &enabled, Resolution::Hd).expect("Failed to load LR2 skin");

    assert_eq!(skin.input, 500);
    assert_eq!(skin.scene, 3000);
    assert_eq!(skin.fadeout, 300);

    let snap = snapshot_from_skin(&skin);

    let image_count = snap.objects_by_type.get("Image").copied().unwrap_or(0);
    assert!(
        image_count >= 2,
        "Should have at least 2 Image objects, got {image_count}"
    );
    assert!(snap.objects_by_type.contains_key("Number"));
    assert!(snap.objects_by_type.contains_key("Text"));
    assert!(snap.objects_by_type.contains_key("Slider"));
    assert!(snap.objects_by_type.contains_key("Graph"));
}

#[test]
fn lr2_csv_conditional() {
    let content = read_file(&lr2_skin_path());

    // Load with option 900 enabled (IF branch)
    let header_on = load_lr2_header(&content, None).expect("header");
    let mut enabled_on = std::collections::HashMap::new();
    enabled_on.insert(900, 1);
    let skin_on = load_lr2_skin(&content, header_on, &enabled_on, Resolution::Hd).expect("skin on");
    let snap_on = snapshot_from_skin(&skin_on);

    // Load with no options enabled (ELSE branch)
    let header_off = load_lr2_header(&content, None).expect("header");
    let enabled_off = std::collections::HashMap::new();
    let skin_off =
        load_lr2_skin(&content, header_off, &enabled_off, Resolution::Hd).expect("skin off");
    let snap_off = snapshot_from_skin(&skin_off);

    // Both branches add exactly one image, so total should be equal
    assert_eq!(
        snap_on.object_count, snap_off.object_count,
        "IF and ELSE branches should produce same object count"
    );

    // The conditional image should differ in its first_dst (size differs: 50x50 vs 80x80)
    let on_images: Vec<_> = snap_on
        .objects
        .iter()
        .filter(|o| o.kind == "Image")
        .collect();
    let off_images: Vec<_> = snap_off
        .objects
        .iter()
        .filter(|o| o.kind == "Image")
        .collect();

    let on_last = on_images.last().expect("should have images");
    let off_last = off_images.last().expect("should have images");

    let on_w = on_last.first_dst.as_ref().map(|d| d.w).unwrap_or(0.0);
    let off_w = off_last.first_dst.as_ref().map(|d| d.w).unwrap_or(0.0);

    assert!(
        (on_w - off_w).abs() > 1.0,
        "Conditional images should differ: IF w={on_w}, ELSE w={off_w}"
    );
}
