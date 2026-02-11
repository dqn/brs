// Java vs Rust screenshot golden master comparison.
//
// Compares Java-generated screenshots (from beatoraja LWJGL3 rendering)
// against Rust-generated screenshots (from Bevy rendering) using SSIM.
//
// Java fixtures: golden-master/fixtures/screenshots_java/*.png
// Rust fixtures: golden-master/fixtures/screenshots/*.png
//
// Run: cargo test -p golden-master compare_screenshots -- --nocapture

use std::path::PathBuf;

struct ScreenshotTestCase {
    name: &'static str,
    threshold: f64,
}

const TEST_CASES: &[ScreenshotTestCase] = &[
    ScreenshotTestCase {
        name: "ecfn_select",
        threshold: 0.85,
    },
    ScreenshotTestCase {
        name: "ecfn_decide",
        threshold: 0.85,
    },
    ScreenshotTestCase {
        name: "ecfn_play7_active",
        threshold: 0.85,
    },
    ScreenshotTestCase {
        name: "ecfn_play7_fullcombo",
        threshold: 0.85,
    },
    ScreenshotTestCase {
        name: "ecfn_play7_danger",
        threshold: 0.85,
    },
    ScreenshotTestCase {
        name: "ecfn_result_clear",
        threshold: 0.85,
    },
    ScreenshotTestCase {
        name: "ecfn_result_fail",
        threshold: 0.85,
    },
];

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn get_test_case(name: &str) -> &'static ScreenshotTestCase {
    TEST_CASES
        .iter()
        .find(|tc| tc.name == name)
        .unwrap_or_else(|| panic!("Unknown test case: {name}"))
}

fn compare_java_rust_screenshot(test_name: &str, threshold: f64) {
    let java_path = fixture_dir()
        .join("screenshots_java")
        .join(format!("{test_name}.png"));
    let rust_path = fixture_dir()
        .join("screenshots")
        .join(format!("{test_name}.png"));

    if !java_path.exists() {
        eprintln!("Java fixture not found: {}, skipping", java_path.display());
        return;
    }
    if !rust_path.exists() {
        eprintln!("Rust fixture not found: {}, skipping", rust_path.display());
        return;
    }

    let java_img = image::open(&java_path)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to load Java screenshot {}: {}",
                java_path.display(),
                e
            )
        })
        .to_rgba8();
    let rust_img = image::open(&rust_path)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to load Rust screenshot {}: {}",
                rust_path.display(),
                e
            )
        })
        .to_rgba8();

    // Dimensions may differ; resize if needed (Java and Rust should use same resolution)
    assert_eq!(
        (java_img.width(), java_img.height()),
        (rust_img.width(), rust_img.height()),
        "Resolution mismatch for {test_name}: Java {}x{} vs Rust {}x{}",
        java_img.width(),
        java_img.height(),
        rust_img.width(),
        rust_img.height(),
    );

    // Convert to grayscale for SSIM
    let java_gray = image::DynamicImage::ImageRgba8(java_img.clone()).to_luma8();
    let rust_gray = image::DynamicImage::ImageRgba8(rust_img.clone()).to_luma8();

    let result = image_compare::gray_similarity_structure(
        &image_compare::Algorithm::MSSIMSimple,
        &java_gray,
        &rust_gray,
    )
    .expect("SSIM comparison failed");

    let ssim = result.score;
    eprintln!("{test_name}: SSIM = {ssim:.4} (threshold = {threshold:.2})");

    if ssim < threshold {
        // Save diff image for debugging
        let diff_dir = fixture_dir().join("screenshots_diff");
        std::fs::create_dir_all(&diff_dir).ok();

        let diff_path = diff_dir.join(format!("{test_name}.diff.png"));
        let diff_img = generate_diff(&java_img, &rust_img);
        diff_img.save(&diff_path).ok();

        panic!(
            "SSIM {ssim:.4} below threshold {threshold} for {test_name}\n  \
             java: {}\n  rust: {}\n  diff: {}",
            java_path.display(),
            rust_path.display(),
            diff_path.display(),
        );
    }
}

/// Generate a visual diff image highlighting pixel differences.
fn generate_diff(a: &image::RgbaImage, b: &image::RgbaImage) -> image::RgbaImage {
    let w = a.width();
    let h = a.height();
    let mut diff = image::RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let pa = a.get_pixel(x, y);
            let pb = b.get_pixel(x, y);
            let dr = (pa[0] as i32 - pb[0] as i32).unsigned_abs() as u8;
            let dg = (pa[1] as i32 - pb[1] as i32).unsigned_abs() as u8;
            let db = (pa[2] as i32 - pb[2] as i32).unsigned_abs() as u8;
            let da = (pa[3] as i32 - pb[3] as i32).unsigned_abs() as u8;

            let max_diff = dr.max(dg).max(db).max(da);
            if max_diff > 0 {
                // Highlight differences in red, intensity proportional to diff
                diff.put_pixel(
                    x,
                    y,
                    image::Rgba([255, 0, 0, (max_diff as u16 * 2).min(255) as u8]),
                );
            } else {
                // Matching pixels: dimmed original
                diff.put_pixel(x, y, image::Rgba([pa[0] / 4, pa[1] / 4, pa[2] / 4, 255]));
            }
        }
    }

    diff
}

// --- Test cases: one per ECFN skin screenshot ---
// NOTE: These pixel SSIM tests are superseded by compare_render_snapshot.rs
// which compares draw commands structurally. Kept as #[ignore] for reference.

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_select() {
    let tc = get_test_case("ecfn_select");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_decide() {
    let tc = get_test_case("ecfn_decide");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_play7_active() {
    let tc = get_test_case("ecfn_play7_active");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_play7_fullcombo() {
    let tc = get_test_case("ecfn_play7_fullcombo");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_play7_danger() {
    let tc = get_test_case("ecfn_play7_danger");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_result_clear() {
    let tc = get_test_case("ecfn_result_clear");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}

#[test]
#[ignore = "Superseded by RenderSnapshot structural comparison (compare_render_snapshot.rs)"]
fn compare_screenshots_ecfn_result_fail() {
    let tc = get_test_case("ecfn_result_fail");
    compare_java_rust_screenshot(tc.name, tc.threshold);
}
