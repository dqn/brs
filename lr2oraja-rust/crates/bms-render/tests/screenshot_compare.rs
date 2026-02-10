// SSIM comparison utility for screenshot tests.
//
// Compares actual rendered screenshots against fixture PNGs.
// Set UPDATE_SCREENSHOTS=1 to regenerate fixtures.

use std::path::Path;

/// Compare an actual screenshot against a fixture, or update the fixture.
///
/// - If `UPDATE_SCREENSHOTS=1` env var is set: saves actual as the new fixture.
/// - If fixture doesn't exist: saves actual as fixture and prints a message.
/// - Otherwise: loads fixture, computes SSIM, panics if below threshold.
///
/// Returns the SSIM score (1.0 if fixture was created/updated).
pub fn compare_or_update(actual: &image::RgbaImage, fixture_path: &Path, threshold: f64) -> f64 {
    let update_mode = std::env::var("UPDATE_SCREENSHOTS")
        .map(|v| v == "1")
        .unwrap_or(false);

    if update_mode {
        if let Some(parent) = fixture_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create fixture directory");
        }
        actual
            .save(fixture_path)
            .expect("Failed to save fixture screenshot");
        eprintln!("Updated fixture: {}", fixture_path.display());
        return 1.0;
    }

    if !fixture_path.exists() {
        if let Some(parent) = fixture_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create fixture directory");
        }
        actual
            .save(fixture_path)
            .expect("Failed to save initial fixture");
        eprintln!(
            "Created initial fixture (re-run to validate): {}",
            fixture_path.display()
        );
        return 1.0;
    }

    // Load fixture and compare
    let fixture = image::open(fixture_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", fixture_path.display(), e))
        .to_rgba8();

    assert_eq!(
        (actual.width(), actual.height()),
        (fixture.width(), fixture.height()),
        "Screenshot dimensions mismatch: actual {}x{} vs fixture {}x{}",
        actual.width(),
        actual.height(),
        fixture.width(),
        fixture.height(),
    );

    // Convert to grayscale for SSIM comparison
    let actual_gray = image::DynamicImage::ImageRgba8(actual.clone()).to_luma8();
    let fixture_gray = image::DynamicImage::ImageRgba8(fixture.clone()).to_luma8();

    let result = image_compare::gray_similarity_structure(
        &image_compare::Algorithm::MSSIMSimple,
        &actual_gray,
        &fixture_gray,
    )
    .expect("SSIM comparison failed");

    let ssim = result.score;

    if ssim < threshold {
        // Save diff image for debugging
        let diff_path = fixture_path.with_extension("diff.png");
        let actual_path = fixture_path.with_extension("actual.png");

        actual
            .save(&actual_path)
            .expect("Failed to save actual screenshot");

        let diff_img = generate_diff(actual, &fixture);
        diff_img
            .save(&diff_path)
            .expect("Failed to save diff image");

        panic!(
            "SSIM {:.4} below threshold {:.4} for {}\n  actual: {}\n  diff: {}",
            ssim,
            threshold,
            fixture_path.display(),
            actual_path.display(),
            diff_path.display(),
        );
    }

    ssim
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
                diff.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            } else {
                diff.put_pixel(x, y, image::Rgba([pa[0] / 4, pa[1] / 4, pa[2] / 4, 255]));
            }
        }
    }

    diff
}
