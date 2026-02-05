//! Screenshot capture utility for visual testing.

use anyhow::Result;
use image::{ImageBuffer, Rgba};
use macroquad::prelude::*;
use std::path::Path;

/// Capture the current screen and save to a PNG file.
pub fn capture_screenshot(output_path: &Path) -> Result<()> {
    let screen = get_screen_data();
    let width = screen.width as u32;
    let height = screen.height as u32;

    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, screen.bytes.clone())
        .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

    // Flip vertically (OpenGL has origin at bottom-left)
    let flipped = image::imageops::flip_vertical(&img);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    flipped.save(output_path)?;

    Ok(())
}
