//! Screenshot capture utility for visual testing.

use anyhow::Result;
use image::{ImageBuffer, Rgba};
use macroquad::prelude::*;
use std::path::Path;

/// Capture the current screen and save to a PNG file.
/// Uses render target approach for reliable screenshot capture.
pub fn capture_screenshot(output_path: &Path) -> Result<()> {
    let width = screen_width() as u32;
    let height = screen_height() as u32;

    let screen = get_screen_data();

    // Check if we got valid data
    if screen.bytes.is_empty() {
        return Err(anyhow::anyhow!("Screen data is empty"));
    }

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

/// Capture screenshot using render target for more reliable capture.
/// Call this after drawing but before next_frame().
pub async fn capture_screenshot_via_render_target(output_path: &Path) -> Result<()> {
    let width = screen_width() as u32;
    let height = screen_height() as u32;

    // Create a render target
    let render_target = render_target(width, height);
    render_target.texture.set_filter(FilterMode::Nearest);

    // Set camera to render to our target
    set_camera(&Camera2D {
        zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
        target: vec2(screen_width() / 2.0, screen_height() / 2.0),
        render_target: Some(render_target.clone()),
        ..Default::default()
    });

    // Clear the render target
    clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

    // Draw a test pattern
    draw_rectangle(100.0, 100.0, 200.0, 100.0, RED);
    draw_text("RENDER TARGET TEST", 110.0, 160.0, 30.0, WHITE);

    // Reset camera
    set_default_camera();

    // Get texture data
    let texture = render_target.texture;
    let image = texture.get_texture_data();

    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, image.bytes.to_vec())
        .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from render target"))?;

    // Flip vertically
    let flipped = image::imageops::flip_vertical(&img);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    flipped.save(output_path)?;

    Ok(())
}
