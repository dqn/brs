// Screenshot regression tests for bms-render.
//
// All tests are #[ignore] because they require GPU access.
// Run with: cargo test -p bms-render --test screenshot_tests -- --ignored --nocapture
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

#[test]
#[ignore] // GPU required
fn test_render_blank_skin() {
    let builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    run_test(builder, "blank");
}

#[test]
#[ignore] // GPU required
fn test_render_single_image() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Red 60x40 rectangle at (100, 50)
    builder.add_image(100.0, 50.0, 60.0, 40.0, 255, 0, 0);
    run_test(builder, "single_image");
}

#[test]
#[ignore] // GPU required
fn test_render_image_alpha() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Semi-transparent blue rectangle
    builder.add_image_with_alpha(50.0, 50.0, 80.0, 60.0, 0, 0, 255, 0.5);
    run_test(builder, "image_alpha");
}

#[test]
#[ignore] // GPU required
fn test_render_z_order() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Three overlapping rectangles: red (bottom), green (middle), blue (top)
    // Later objects have higher z-order (index * 0.001)
    builder.add_image(60.0, 60.0, 80.0, 80.0, 255, 0, 0);
    builder.add_image(80.0, 70.0, 80.0, 80.0, 0, 255, 0);
    builder.add_image(100.0, 80.0, 80.0, 80.0, 0, 0, 255);
    run_test(builder, "z_order");
}

#[test]
#[ignore] // GPU required
fn test_render_animation_midpoint() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Green square moving from (0,76) to (200,76) over 1000ms
    // At t=500ms, should be at (100, 76)
    builder.add_animated_image(0.0, 76.0, 200.0, 76.0, 40.0, 40.0, 0, 255, 0, 1000);
    builder.set_time_ms(500);
    run_test(builder, "animation_midpoint");
}

#[test]
#[ignore] // GPU required
fn test_render_draw_condition_false() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // This image has a draw condition set to false => should not be visible
    builder.add_image_with_condition(50.0, 50.0, 80.0, 60.0, 255, 0, 0, 100, false);
    run_test(builder, "draw_condition_false");
}

#[test]
#[ignore] // GPU required
fn test_render_timer_inactive() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Image with timer that is inactive (no timer value set) => hidden
    builder.add_image_with_timer(50.0, 50.0, 80.0, 60.0, 255, 255, 0, 200, None);
    run_test(builder, "timer_inactive");
}

#[test]
#[ignore] // GPU required
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

#[test]
#[ignore] // GPU required
fn test_render_slider() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Horizontal slider: direction=Right(1), range=100, value=0.5
    // Base at (50, 80), 20x20 thumb
    builder.add_slider(50.0, 80.0, 20.0, 20.0, 255, 128, 0, 1, 100, 50, 0.5);
    run_test(builder, "slider");
}

#[test]
#[ignore] // GPU required
fn test_render_graph() {
    let mut builder = TestSkinBuilder::new(TEST_W as f32, TEST_H as f32);
    // Graph: direction=Right(0), value=0.5 => half-width bar
    builder.add_graph(20.0, 80.0, 200.0, 30.0, 0, 200, 100, 0, 60, 0.5);
    run_test(builder, "graph");
}
