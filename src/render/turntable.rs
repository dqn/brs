//! Turntable rendering for IIDX-style layout

use macroquad::prelude::*;

use crate::skin::Rect;

/// Turntable visual representation
pub struct Turntable {
    /// Current rotation angle in degrees
    rotation: f32,
    /// Whether the turntable is currently active (being scratched)
    is_active: bool,
    /// Rotation speed when active
    rotation_speed: f32,
}

impl Turntable {
    pub fn new() -> Self {
        Self {
            rotation: 0.0,
            is_active: false,
            rotation_speed: 180.0, // degrees per second
        }
    }

    /// Update turntable state based on scratch input
    pub fn update(&mut self, scratch_active: bool, dt: f32) {
        self.is_active = scratch_active;
        if scratch_active {
            self.rotation += self.rotation_speed * dt;
            if self.rotation >= 360.0 {
                self.rotation -= 360.0;
            }
        }
    }

    /// Draw the turntable within the specified rect
    pub fn draw(&self, rect: &Rect) {
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        let radius = rect.width.min(rect.height) / 2.0 - 5.0;

        // Outer ring (dark blue border)
        draw_circle(center_x, center_y, radius + 3.0, Color::new(0.1, 0.2, 0.4, 1.0));

        // Main turntable body (blue gradient effect)
        let base_color = if self.is_active {
            Color::new(0.2, 0.5, 0.9, 1.0) // Brighter when active
        } else {
            Color::new(0.15, 0.35, 0.7, 1.0)
        };
        draw_circle(center_x, center_y, radius, base_color);

        // Inner circle (darker center)
        draw_circle(
            center_x,
            center_y,
            radius * 0.3,
            Color::new(0.05, 0.1, 0.2, 1.0),
        );

        // Rotation indicator line
        let angle_rad = self.rotation.to_radians();
        let line_end_x = center_x + radius * 0.7 * angle_rad.cos();
        let line_end_y = center_y + radius * 0.7 * angle_rad.sin();
        draw_line(center_x, center_y, line_end_x, line_end_y, 2.0, WHITE);

        // Additional rotation markers (3 lines at 120 degree intervals)
        for i in 0..3 {
            let marker_angle = self.rotation + (i as f32 * 120.0);
            let marker_rad = marker_angle.to_radians();
            let start_r = radius * 0.5;
            let end_r = radius * 0.8;

            let start_x = center_x + start_r * marker_rad.cos();
            let start_y = center_y + start_r * marker_rad.sin();
            let end_x = center_x + end_r * marker_rad.cos();
            let end_y = center_y + end_r * marker_rad.sin();

            draw_line(
                start_x,
                start_y,
                end_x,
                end_y,
                1.5,
                Color::new(0.3, 0.5, 0.8, 0.7),
            );
        }
    }
}

impl Default for Turntable {
    fn default() -> Self {
        Self::new()
    }
}
