use macroquad::prelude::*;

use crate::skin::object::{
    SkinObject, apply_offsets, check_option_visibility, get_timer_elapsed, interpolate_destinations,
};
use crate::skin::{MainState, SkinObjectData, SkinSourceManager, SliderDef};

/// Skin object that renders a slider image with a movable offset.
pub struct SliderObject {
    /// Object data from skin definition.
    pub data: SkinObjectData,
    /// Slider definition.
    pub slider_def: Option<SliderDef>,
    /// Whether the object is prepared.
    prepared: bool,
}

impl SliderObject {
    /// Create a new slider object.
    pub fn new(data: SkinObjectData, slider_def: Option<SliderDef>) -> Self {
        Self {
            data,
            slider_def,
            prepared: false,
        }
    }

    /// Get the current animation frame based on timer.
    fn get_animation_frame(&self, elapsed_us: i64) -> usize {
        let Some(ref slider_def) = self.slider_def else {
            return 0;
        };

        let total_frames = (slider_def.divx * slider_def.divy) as usize;
        if total_frames <= 1 || slider_def.cycle <= 0 {
            return 0;
        }

        let cycle_us = (slider_def.cycle * 1000) as i64;
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
        let slider_def = self.slider_def.as_ref()?;

        let divx = slider_def.divx.max(1) as usize;
        let divy = slider_def.divy.max(1) as usize;
        let frame_w = slider_def.w as f32 / divx as f32;
        let frame_h = slider_def.h as f32 / divy as f32;

        let frame_x = (frame % divx) as f32 * frame_w;
        let frame_y = (frame / divx) as f32 * frame_h;

        Some(Rect::new(
            slider_def.x as f32 + frame_x,
            slider_def.y as f32 + frame_y,
            frame_w,
            frame_h,
        ))
    }

    fn slider_value(&self, state: &MainState) -> f32 {
        let Some(ref slider_def) = self.slider_def else {
            return 0.0;
        };

        let mut value = state.float_number(slider_def.slider_type) as f32;
        if let (Some(min), Some(max)) = (slider_def.min, slider_def.max) {
            if max > min {
                value = ((value - min as f32) / (max - min) as f32).clamp(0.0, 1.0);
            }
        }
        value.clamp(0.0, 1.0)
    }
}

impl SkinObject for SliderObject {
    fn prepare(&mut self, _sources: &SkinSourceManager) {
        self.prepared = true;
    }

    fn draw(&self, state: &MainState, sources: &SkinSourceManager, now_time_us: i64) {
        if !self.is_visible(state) {
            return;
        }

        let Some(ref slider_def) = self.slider_def else {
            return;
        };

        let Some(texture) = sources.get(slider_def.src) else {
            return;
        };

        let elapsed_us = get_timer_elapsed(self.data.timer, state, now_time_us);
        if elapsed_us < 0 {
            return;
        }

        let elapsed_ms = elapsed_us / 1000;
        let Some(dst) = interpolate_destinations(&self.data.dst, elapsed_ms, self.data.loop_count)
        else {
            return;
        };
        let mut dst = apply_offsets(dst, &self.data, state);

        if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }

        let value = self.slider_value(state);
        let range = slider_def.range as f32;
        match slider_def.angle {
            0 => dst.y += value * range,
            1 => dst.x += value * range,
            2 => dst.y -= value * range,
            3 => dst.x -= value * range,
            _ => {}
        }

        let animation_timer = if slider_def.timer != 0 {
            slider_def.timer
        } else {
            self.data.timer
        };
        let anim_elapsed_us = get_timer_elapsed(animation_timer, state, now_time_us);
        let frame = if anim_elapsed_us >= 0 {
            self.get_animation_frame(anim_elapsed_us)
        } else {
            0
        };
        let src_rect = self.get_source_rect(frame);

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
