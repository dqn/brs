use macroquad::prelude::*;

use crate::skin::object::{
    SkinObject, apply_offsets, check_option_visibility, get_timer_elapsed, interpolate_destinations,
};
use crate::skin::{ImageDef, ImageSetDef, MainState, SkinObjectData, SkinSourceManager};

/// Skin object that renders an image set selection.
pub struct ImageSetObject {
    /// Object data from skin definition.
    pub data: SkinObjectData,
    /// Image definitions in this set.
    pub images: Vec<ImageDef>,
    /// Integer property ID for selection.
    pub ref_id: i32,
    /// Whether the object is prepared.
    prepared: bool,
}

impl ImageSetObject {
    /// Create a new image set object.
    pub fn new(
        data: SkinObjectData,
        imageset_def: Option<ImageSetDef>,
        images: &std::collections::HashMap<String, ImageDef>,
    ) -> Self {
        let mut defs = Vec::new();
        let mut ref_id = 0;
        if let Some(set_def) = imageset_def {
            ref_id = set_def.ref_id;
            for image_id in set_def.images {
                if let Some(def) = images.get(&image_id) {
                    defs.push(def.clone());
                }
            }
        }

        Self {
            data,
            images: defs,
            ref_id,
            prepared: false,
        }
    }

    fn select_index(&self, state: &MainState) -> usize {
        if self.images.is_empty() {
            return 0;
        }
        if self.ref_id == 0 {
            return 0;
        }
        let value = state.number(self.ref_id);
        let clamped = value.clamp(0, self.images.len().saturating_sub(1) as i32);
        clamped as usize
    }

    fn get_animation_frame(&self, image_def: &ImageDef, elapsed_us: i64) -> usize {
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

    fn get_source_rect(&self, image_def: &ImageDef, frame: usize) -> Rect {
        let divx = image_def.divx.max(1) as usize;
        let divy = image_def.divy.max(1) as usize;
        let frame_w = image_def.w as f32 / divx as f32;
        let frame_h = image_def.h as f32 / divy as f32;

        let frame_x = (frame % divx) as f32 * frame_w;
        let frame_y = (frame / divx) as f32 * frame_h;

        Rect::new(
            image_def.x as f32 + frame_x,
            image_def.y as f32 + frame_y,
            frame_w,
            frame_h,
        )
    }
}

impl SkinObject for ImageSetObject {
    fn prepare(&mut self, _sources: &SkinSourceManager) {
        self.prepared = true;
    }

    fn draw(&self, state: &MainState, sources: &SkinSourceManager, now_time_us: i64) {
        if !self.is_visible(state) {
            return;
        }

        if self.images.is_empty() {
            return;
        }

        let image_index = self.select_index(state);
        let image_def = match self.images.get(image_index) {
            Some(def) => def,
            None => return,
        };

        let Some(texture) = sources.get(image_def.src) else {
            return;
        };

        // Calculate elapsed time for destination
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
        let dst = apply_offsets(dst, &self.data, state);

        // Skip if invisible
        if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }

        let animation_timer = if image_def.timer != 0 {
            image_def.timer
        } else {
            self.data.timer
        };
        let anim_elapsed = get_timer_elapsed(animation_timer, state, now_time_us);
        let frame = if anim_elapsed >= 0 {
            self.get_animation_frame(image_def, anim_elapsed)
        } else {
            0
        };
        let src_rect = self.get_source_rect(image_def, frame);

        // Draw the texture
        let color = Color::new(dst.r / 255.0, dst.g / 255.0, dst.b / 255.0, dst.a / 255.0);

        draw_texture_ex(
            &texture.texture,
            dst.x,
            dst.y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(dst.w, dst.h)),
                source: Some(src_rect),
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
