//! Screenshot comparison tool for visual regression testing.
//!
//! Compares two PNG images and outputs a text report suitable for
//! Claude Code analysis.

use clap::Parser;
use image::{GenericImageView, Pixel, Rgba};
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "screenshot_diff",
    about = "Compare two screenshots and output a diff report"
)]
struct Args {
    /// Reference screenshot path
    reference: String,

    /// Current screenshot path
    current: String,

    /// Number of grid divisions for region analysis (default: 4)
    #[arg(long, default_value = "4")]
    grid: u32,
}

fn main() {
    let args = Args::parse();

    let reference_path = Path::new(&args.reference);
    let current_path = Path::new(&args.current);

    // Load images
    let reference = match image::open(reference_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error loading reference image '{}': {}", args.reference, e);
            std::process::exit(1);
        }
    };

    let current = match image::open(current_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error loading current image '{}': {}", args.current, e);
            std::process::exit(1);
        }
    };

    println!("Screenshot Comparison Report");
    println!("============================");
    println!("Reference: {}", args.reference);
    println!("Current:   {}", args.current);
    println!();

    // Check dimensions
    let (ref_w, ref_h) = reference.dimensions();
    let (cur_w, cur_h) = current.dimensions();

    if ref_w != cur_w || ref_h != cur_h {
        println!("Size: MISMATCH");
        println!("  Reference: {}x{}", ref_w, ref_h);
        println!("  Current:   {}x{}", cur_w, cur_h);
        println!();
        println!("Cannot compare images of different sizes.");
        std::process::exit(1);
    }

    println!("Size: MATCH ({}x{})", ref_w, ref_h);
    println!();

    // Region analysis
    let grid = args.grid;
    let cell_w = ref_w / grid;
    let cell_h = ref_h / grid;

    println!("Region Analysis ({}x{} grid):", grid, grid);

    let mut total_diff_pixels = 0u64;
    let mut total_pixels = 0u64;
    let mut significant_regions = Vec::new();

    for gy in 0..grid {
        for gx in 0..grid {
            let x_start = gx * cell_w;
            let y_start = gy * cell_h;
            let x_end = if gx == grid - 1 {
                ref_w
            } else {
                x_start + cell_w
            };
            let y_end = if gy == grid - 1 {
                ref_h
            } else {
                y_start + cell_h
            };

            let (diff_count, pixel_count) =
                compare_region(&reference, &current, x_start, y_start, x_end, y_end);

            total_diff_pixels += diff_count;
            total_pixels += pixel_count;

            let diff_percent = (diff_count as f64 / pixel_count as f64) * 100.0;
            let region_label = get_region_label(gx, gy, grid);

            if diff_percent > 5.0 {
                significant_regions.push((gx, gy, diff_percent, region_label.clone()));
                println!(
                    "  [{},{}]: {:.1}% diff - SIGNIFICANT ({})",
                    gx, gy, diff_percent, region_label
                );
            } else if diff_percent > 0.1 {
                println!(
                    "  [{},{}]: {:.1}% diff ({})",
                    gx, gy, diff_percent, region_label
                );
            }
        }
    }

    println!();

    // Overall similarity
    let overall_similarity = 100.0 - (total_diff_pixels as f64 / total_pixels as f64) * 100.0;
    println!("Overall Similarity: {:.1}%", overall_similarity);

    if overall_similarity >= 99.0 {
        println!("Status: EXCELLENT - Images are nearly identical");
    } else if overall_similarity >= 95.0 {
        println!("Status: GOOD - Minor differences detected");
    } else if overall_similarity >= 80.0 {
        println!("Status: MODERATE - Noticeable differences");
    } else {
        println!("Status: SIGNIFICANT - Major differences detected");
    }

    // Summary of significant regions
    if !significant_regions.is_empty() {
        println!();
        println!("Significant Differences:");
        for (gx, gy, diff_percent, label) in &significant_regions {
            let x_start = gx * cell_w;
            let y_start = gy * cell_h;
            println!(
                "  - {} ({:.1}% diff): pixels ({}, {}) to ({}, {})",
                label,
                diff_percent,
                x_start,
                y_start,
                x_start + cell_w,
                y_start + cell_h
            );
        }
    }

    // Exit with status code based on similarity
    if overall_similarity < 95.0 {
        std::process::exit(1);
    }
}

/// Compare a region of two images and return (diff_count, total_pixels).
fn compare_region(
    reference: &image::DynamicImage,
    current: &image::DynamicImage,
    x_start: u32,
    y_start: u32,
    x_end: u32,
    y_end: u32,
) -> (u64, u64) {
    let mut diff_count = 0u64;
    let mut pixel_count = 0u64;

    for y in y_start..y_end {
        for x in x_start..x_end {
            let ref_pixel: Rgba<u8> = reference.get_pixel(x, y).to_rgba();
            let cur_pixel: Rgba<u8> = current.get_pixel(x, y).to_rgba();

            pixel_count += 1;

            // Compare with tolerance (allow small differences due to rendering)
            if !pixels_similar(&ref_pixel, &cur_pixel, 5) {
                diff_count += 1;
            }
        }
    }

    (diff_count, pixel_count)
}

/// Check if two pixels are similar within a tolerance.
fn pixels_similar(a: &Rgba<u8>, b: &Rgba<u8>, tolerance: u8) -> bool {
    let diff_r = (a[0] as i16 - b[0] as i16).unsigned_abs() as u8;
    let diff_g = (a[1] as i16 - b[1] as i16).unsigned_abs() as u8;
    let diff_b = (a[2] as i16 - b[2] as i16).unsigned_abs() as u8;
    let diff_a = (a[3] as i16 - b[3] as i16).unsigned_abs() as u8;

    diff_r <= tolerance && diff_g <= tolerance && diff_b <= tolerance && diff_a <= tolerance
}

/// Get human-readable label for a grid region.
fn get_region_label(gx: u32, gy: u32, grid: u32) -> String {
    let h_label = if gx == 0 {
        "left"
    } else if gx == grid - 1 {
        "right"
    } else {
        "center"
    };

    let v_label = if gy == 0 {
        "top"
    } else if gy == grid - 1 {
        "bottom"
    } else {
        "middle"
    };

    format!("{}-{}", v_label, h_label)
}
