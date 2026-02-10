// Screenshot regression tests for bms-render.
//
// Uses harness = false (custom main) because Bevy headless rendering
// may need special initialization. Tests are skipped by default; pass
// --ignored to run them (matches cargo test convention for GPU tests).
//
// Run: cargo test -p bms-render --test screenshot_tests -- --ignored --nocapture
// Update fixtures: UPDATE_SCREENSHOTS=1 cargo test -p bms-render --test screenshot_tests -- --ignored --nocapture

mod screenshot_compare;
mod screenshot_harness;
mod test_skin_builder;

use std::path::PathBuf;

use screenshot_compare::compare_or_update;
use screenshot_harness::RenderTestHarness;
use test_skin_builder::TestSkinBuilder;

/// Test resolution: small for fast rendering.
const TEST_W: u32 = 256;
const TEST_H: u32 = 192;

/// SSIM threshold for screenshot comparison.
const SSIM_THRESHOLD: f64 = 0.99;

/// Path to fixture directory.
fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("golden-master")
        .join("fixtures")
        .join("screenshots")
}

fn fixture_path(name: &str) -> PathBuf {
    fixture_dir().join(format!("{}.png", name))
}

/// Helper: build harness, upload images, setup skin, capture frame.
fn run_test(builder: TestSkinBuilder, fixture_name: &str) {
    let (skin, images, provider) = builder.build();
    let mut harness = RenderTestHarness::new(TEST_W, TEST_H);

    for img in &images {
        harness.upload_image(&img.rgba);
    }

    harness.setup_skin(skin, Box::new(provider));

    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = tmp_dir.path().join("screenshot.png");

    harness.capture_frame(&output_path);

    let actual = image::open(&output_path)
        .expect("Failed to read captured screenshot")
        .to_rgba8();

    compare_or_update(&actual, &fixture_path(fixture_name), SSIM_THRESHOLD);
}

// ---------------------------------------------------------------------------
// Test cases
// ---------------------------------------------------------------------------

fn test_render_blank_skin() {
    let builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    run_test(builder, "blank");
}

fn test_render_single_image() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Red 60x40 rectangle at (100, 50)
    builder.add_image(100.0, 50.0, 60.0, 40.0, 255, 0, 0);
    run_test(builder, "single_image");
}

fn test_render_image_alpha() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Semi-transparent blue rectangle
    builder.add_image_with_alpha(50.0, 50.0, 80.0, 60.0, 0, 0, 255, 0.5);
    run_test(builder, "image_alpha");
}

fn test_render_z_order() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Three overlapping rectangles: red (bottom), green (middle), blue (top)
    // Later objects have higher z-order (index * 0.001)
    builder.add_image(60.0, 60.0, 80.0, 80.0, 255, 0, 0);
    builder.add_image(80.0, 70.0, 80.0, 80.0, 0, 255, 0);
    builder.add_image(100.0, 80.0, 80.0, 80.0, 0, 0, 255);
    run_test(builder, "z_order");
}

fn test_render_animation_midpoint() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Green square moving from (0,76) to (200,76) over 1000ms
    // At t=500ms, should be at (100, 76)
    builder.add_animated_image(0.0, 76.0, 200.0, 76.0, 40.0, 40.0, 0, 255, 0, 1000);
    builder.set_time_ms(500);
    run_test(builder, "animation_midpoint");
}

fn test_render_draw_condition_false() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // This image has a draw condition set to false => should not be visible
    builder.add_image_with_condition(50.0, 50.0, 80.0, 60.0, 255, 0, 0, 100, false);
    run_test(builder, "draw_condition_false");
}

fn test_render_timer_inactive() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Image with timer that is inactive (no timer value set) => hidden
    builder.add_image_with_timer(50.0, 50.0, 80.0, 60.0, 255, 255, 0, 200, None);
    run_test(builder, "timer_inactive");
}

fn test_render_four_corners() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    let s = 30.0; // square size
    // Top-left: red
    builder.add_image(0.0, 0.0, s, s, 255, 0, 0);
    // Top-right: green
    builder.add_image(TEST_W as f32 - s, 0.0, s, s, 0, 255, 0);
    // Bottom-left: blue
    builder.add_image(0.0, TEST_H as f32 - s, s, s, 0, 0, 255);
    // Bottom-right: yellow
    builder.add_image(TEST_W as f32 - s, TEST_H as f32 - s, s, s, 255, 255, 0);
    run_test(builder, "four_corners");
}

fn test_render_slider() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Horizontal slider: direction=Right(1), range=100, value=0.5
    // Base at (50, 80), 20x20 thumb
    builder.add_slider(50.0, 80.0, 20.0, 20.0, 255, 128, 0, 1, 100, 50, 0.5);
    run_test(builder, "slider");
}

fn test_render_graph() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Graph: direction=Right(0), value=0.5 => half-width bar
    builder.add_graph(20.0, 80.0, 200.0, 30.0, 0, 200, 100, 0, 60, 0.5);
    run_test(builder, "graph");
}

// ---------------------------------------------------------------------------
// JSON skin file tests
// ---------------------------------------------------------------------------

/// Path to test-skin directory.
fn test_skin_dir() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = lr2oraja-rust/crates/bms-render
    // test-bms is at the project root (3 levels up)
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-bms")
        .join("test-skin")
}

/// Helper: load a JSON skin file, set up state, capture frame.
fn run_json_skin_test(
    skin_json_name: &str,
    provider: bms_render::state_provider::StaticStateProvider,
    fixture_name: &str,
) {
    let skin_path = test_skin_dir().join(skin_json_name);
    let mut harness = RenderTestHarness::new(TEST_W, TEST_H);

    harness.load_json_skin(&skin_path, Box::new(provider));

    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = tmp_dir.path().join("screenshot.png");

    harness.capture_frame(&output_path);

    let actual = image::open(&output_path)
        .expect("Failed to read captured screenshot")
        .to_rgba8();

    screenshot_compare::compare_or_update(&actual, &fixture_path(fixture_name), SSIM_THRESHOLD);
}

fn test_render_json_skin() {
    let mut provider = bms_render::state_provider::StaticStateProvider::default();
    // slider FloatId(17) = 0.5, graph FloatId(100) = 0.6
    provider.floats.insert(17, 0.5);
    provider.floats.insert(100, 0.6);
    run_json_skin_test("skin.json", provider, "json_skin");
}

fn test_render_json_skin_with_condition() {
    let mut provider = bms_render::state_provider::StaticStateProvider::default();
    // BooleanId(900) = false → accent image should be hidden
    provider.booleans.insert(900, false);
    run_json_skin_test(
        "skin_with_condition.json",
        provider,
        "json_skin_with_condition",
    );
}

// ---------------------------------------------------------------------------
// ECFN skin tests (real-world skins, skipped if not present)
// ---------------------------------------------------------------------------

/// Path to ECFN skin directory.
fn ecfn_skin_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("skins")
        .join("ECFN")
}

fn test_render_ecfn_select() {
    let skin_path = ecfn_skin_dir().join("select/select.json");
    if !skin_path.exists() {
        eprintln!("ECFN select skin not found, skipping");
        return;
    }

    let provider = bms_render::state_provider::StaticStateProvider::default();
    let mut harness = RenderTestHarness::new(1280, 720);

    harness.load_json_skin_with_resolution(
        &skin_path,
        Box::new(provider),
        bms_config::resolution::Resolution::Hd,
    );

    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = tmp_dir.path().join("screenshot.png");

    harness.capture_frame(&output_path);

    let actual = image::open(&output_path)
        .expect("Failed to read captured screenshot")
        .to_rgba8();

    screenshot_compare::compare_or_update(&actual, &fixture_path("ecfn_select"), SSIM_THRESHOLD);
}

// ---------------------------------------------------------------------------
// Custom test runner
// ---------------------------------------------------------------------------

fn get_tests() -> Vec<(&'static str, fn())> {
    vec![
        ("test_render_blank_skin", test_render_blank_skin as fn()),
        ("test_render_single_image", test_render_single_image),
        ("test_render_image_alpha", test_render_image_alpha),
        ("test_render_z_order", test_render_z_order),
        (
            "test_render_animation_midpoint",
            test_render_animation_midpoint,
        ),
        (
            "test_render_draw_condition_false",
            test_render_draw_condition_false,
        ),
        ("test_render_timer_inactive", test_render_timer_inactive),
        ("test_render_four_corners", test_render_four_corners),
        ("test_render_slider", test_render_slider),
        ("test_render_graph", test_render_graph),
        ("test_render_json_skin", test_render_json_skin),
        (
            "test_render_json_skin_with_condition",
            test_render_json_skin_with_condition,
        ),
        ("test_render_ecfn_select", test_render_ecfn_select),
    ]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Support --list for test discovery
    if args.iter().any(|a| a == "--list") {
        for (name, _) in get_tests() {
            println!("{}: test", name);
        }
        return;
    }

    // Match cargo test convention: skip unless --ignored is passed
    if !args.iter().any(|a| a == "--ignored") {
        eprintln!("Screenshot tests skipped (GPU required). Run with --ignored to execute.");
        return;
    }

    // Optional name filter: first non-flag arg after binary name
    let filter: Option<&str> = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str());

    let tests = get_tests();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (name, test_fn) in &tests {
        if let Some(f) = filter {
            if !name.contains(f) {
                skipped += 1;
                continue;
            }
        }

        eprint!("test {} ... ", name);
        match std::panic::catch_unwind(test_fn) {
            Ok(_) => {
                eprintln!("ok");
                passed += 1;
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                eprintln!("FAILED\n  {}", msg);
                failed += 1;
            }
        }
    }

    eprintln!(
        "\ntest result: {}. {} passed; {} failed; {} filtered out",
        if failed == 0 { "ok" } else { "FAILED" },
        passed,
        failed,
        skipped
    );

    if failed > 0 {
        std::process::exit(1);
    }
}
