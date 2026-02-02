use macroquad::prelude::*;

use crate::skin::object::{
    SkinObject, check_option_visibility, get_timer_elapsed, interpolate_destinations,
};
use crate::skin::{ImageDef, MainState, SkinObjectData, SkinSourceManager};

/// Skin object that renders a single image.
pub struct ImageObject {
    /// Object data from skin definition.
    pub data: SkinObjectData,
    /// Image definition.
    pub image_def: Option<ImageDef>,
    /// Whether the object is prepared.
    prepared: bool,
}

impl ImageObject {
    /// Create a new image object.
    pub fn new(data: SkinObjectData, image_def: Option<ImageDef>) -> Self {
        Self {
            data,
            image_def,
            prepared: false,
        }
    }

    /// Get the current animation frame based on timer.
    fn get_animation_frame(&self, elapsed_us: i64) -> usize {
        let Some(ref image_def) = self.image_def else {
            return 0;
        };

        let total_frames = (image_def.divx * image_def.divy) as usize;
        if total_frames <= 1 || image_def.cycle <= 0 {
            return 0;
        }

        let cycle_us = (image_def.cycle * 1000) as i64;
        let elapsed_in_cycle = if cycle_us > 0 {
            elapsed_us % cycle_us
        } else {
            0
        };

        let frame_duration = cycle_us / total_frames as i64;
        if frame_duration <= 0 {
            return 0;
        }

        ((elapsed_in_cycle / frame_duration) as usize).min(total_frames - 1)
    }

    /// Calculate source rectangle for a specific frame.
    fn get_source_rect(&self, frame: usize) -> Option<Rect> {
        let image_def = self.image_def.as_ref()?;

        let divx = image_def.divx.max(1) as usize;
        let divy = image_def.divy.max(1) as usize;
        let frame_w = image_def.w as f32 / divx as f32;
        let frame_h = image_def.h as f32 / divy as f32;

        let frame_x = (frame % divx) as f32 * frame_w;
        let frame_y = (frame / divx) as f32 * frame_h;

        Some(Rect::new(
            image_def.x as f32 + frame_x,
            image_def.y as f32 + frame_y,
            frame_w,
            frame_h,
        ))
    }
}

impl SkinObject for ImageObject {
    fn prepare(&mut self, _sources: &SkinSourceManager) {
        self.prepared = true;
    }

    fn draw(&self, state: &MainState, sources: &SkinSourceManager, now_time_us: i64) {
        if !self.is_visible(state) {
            return;
        }

        let Some(ref image_def) = self.image_def else {
            return;
        };

        let Some(texture) = sources.get(image_def.src) else {
            return;
        };

        // Calculate elapsed time for animation
        let elapsed_us = get_timer_elapsed(self.data.timer, state, now_time_us);
        if elapsed_us < 0 {
            return; // Timer not active
        }

        // Get interpolated destination
        let elapsed_ms = elapsed_us / 1000;
        let Some(dst) = interpolate_destinations(&self.data.dst, elapsed_ms, self.data.loop_count)
        else {
            return;
        };

        // Skip if invisible
        if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }

        // Get animation frame
        let frame = self.get_animation_frame(elapsed_us);
        let src_rect = self.get_source_rect(frame);

        // Draw the texture
        let color = Color::new(dst.r / 255.0, dst.g / 255.0, dst.b / 255.0, dst.a / 255.0);

        draw_texture_ex(
            &texture.texture,
            dst.x,
            dst.y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(dst.w, dst.h)),
                source: src_rect,
                rotation: dst.angle.to_radians(),
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }

    fn is_visible(&self, state: &MainState) -> bool {
        check_option_visibility(&self.data.op, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::Destination;

    #[test]
    fn test_image_object_creation() {
        let data = SkinObjectData {
            id: "test".to_string(),
            dst: vec![Destination {
                x: 100.0,
                y: 100.0,
                w: 50.0,
                h: 50.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let obj = ImageObject::new(data, None);
        assert!(!obj.prepared);
    }

    #[test]
    fn test_animation_frame_calculation() {
        let data = SkinObjectData::default();
        let image_def = ImageDef {
            id: "test".to_string(),
            divx: 4,
            divy: 1,
            cycle: 400, // 400ms for 4 frames = 100ms per frame
            ..Default::default()
        };
        let obj = ImageObject::new(data, Some(image_def));

        // At t=0, frame 0
        assert_eq!(obj.get_animation_frame(0), 0);
        // At t=150ms (150000us), frame 1
        assert_eq!(obj.get_animation_frame(150_000), 1);
        // At t=250ms (250000us), frame 2
        assert_eq!(obj.get_animation_frame(250_000), 2);
    }
}
