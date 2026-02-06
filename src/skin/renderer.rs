use std::collections::HashMap;

use anyhow::Result;

use crate::skin::destination::{InterpolatedDst, rotation_center};
use crate::skin::skin_data::{SkinData, SkinObject};
use crate::skin::skin_property::TIMER_OFF_VALUE;
use crate::traits::render::{BlendMode, Color, DstRect, RenderBackend, SrcRect};

/// Snapshot of game state values needed by the skin renderer.
/// Populated each frame by MainStateAccessor.
#[derive(Default)]
pub struct SkinStateSnapshot {
    /// Current time in milliseconds.
    pub time_ms: i64,
    /// Timer values: timer_id -> start time in microseconds (or TIMER_OFF_VALUE).
    pub timers: HashMap<i32, i64>,
    /// Number values: number_id -> integer value.
    pub numbers: HashMap<i32, i32>,
    /// Float values: float_id -> float value.
    pub floats: HashMap<i32, f32>,
    /// String values: string_id -> string value.
    pub strings: HashMap<i32, String>,
    /// Option conditions: option_id -> bool.
    pub options: HashMap<i32, bool>,
    /// Offset values: offset_id -> (x, y, w, h, r, a).
    pub offsets: HashMap<i32, [f32; 6]>,
}

impl SkinStateSnapshot {
    /// Get a timer value in milliseconds. Returns None if off.
    pub fn timer_ms(&self, timer_id: i32) -> Option<i64> {
        let &val = self.timers.get(&timer_id)?;
        if val == TIMER_OFF_VALUE {
            return None;
        }
        Some(val / 1000) // us -> ms
    }

    /// Get a number value.
    pub fn number(&self, id: i32) -> i32 {
        self.numbers.get(&id).copied().unwrap_or(0)
    }

    /// Get a float value.
    pub fn float_value(&self, id: i32) -> f32 {
        self.floats.get(&id).copied().unwrap_or(0.0)
    }

    /// Check if an option condition is met.
    pub fn option(&self, id: i32) -> bool {
        self.options.get(&id).copied().unwrap_or(false)
    }

    /// Get a string value.
    pub fn string(&self, id: i32) -> &str {
        self.strings.get(&id).map(|s| s.as_str()).unwrap_or("")
    }
}

/// Renders a loaded skin using the RenderBackend.
pub struct SkinRenderer;

impl SkinRenderer {
    /// Render all skin objects for the current frame.
    pub fn render(
        renderer: &mut dyn RenderBackend,
        skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        for obj in &skin.objects {
            match obj {
                SkinObject::Image(img) => {
                    Self::render_image(renderer, img, skin, state)?;
                }
                SkinObject::Number(num) => {
                    Self::render_number(renderer, num, skin, state)?;
                }
                SkinObject::Bargraph(bg) => {
                    Self::render_bargraph(renderer, bg, skin, state)?;
                }
                SkinObject::Slider(sl) => {
                    Self::render_slider(renderer, sl, skin, state)?;
                }
                SkinObject::Text(txt) => {
                    Self::render_text(renderer, txt, state)?;
                }
                SkinObject::ImageSet(is) => {
                    Self::render_image_set(renderer, is, skin, state)?;
                }
                SkinObject::Graph(_) => {}
                SkinObject::Gauge(gauge) => {
                    Self::render_gauge(renderer, gauge, skin, state)?;
                }
                SkinObject::Judge(judge) => {
                    Self::render_judge(renderer, judge, skin, state)?;
                }
            }
        }
        Ok(())
    }

    /// Compute the effective time for a destination set given the timer.
    fn effective_time(
        dst: &crate::skin::destination::DestinationSet,
        state: &SkinStateSnapshot,
    ) -> Option<i64> {
        if dst.timer > 0 {
            let timer_ms = state.timer_ms(dst.timer)?;
            Some(state.time_ms - timer_ms)
        } else {
            Some(state.time_ms)
        }
    }

    /// Check option conditions for a destination set.
    fn check_options(
        dst: &crate::skin::destination::DestinationSet,
        state: &SkinStateSnapshot,
    ) -> bool {
        for &op in &dst.options {
            if op > 0 {
                if !state.option(op) {
                    return false;
                }
            } else if op < 0 && state.option(-op) {
                return false;
            }
        }
        true
    }

    fn blend_mode(blend: i32) -> BlendMode {
        match blend {
            2 => BlendMode::Additive,
            _ => BlendMode::Alpha,
        }
    }

    fn render_image(
        renderer: &mut dyn RenderBackend,
        img: &crate::skin::object::image::ImageObject,
        skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&img.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&img.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match img.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let texture = match img.texture {
            Some(t) => t,
            None => return Ok(()),
        };

        let frame = img.frame_index(time);
        let (sx, sy, sw, sh) = img.frame_src_rect(frame);

        Self::draw_sprite(
            renderer,
            texture,
            sx,
            sy,
            sw,
            sh,
            &interp,
            skin,
            img.dst.blend,
            img.dst.center,
        )?;
        Ok(())
    }

    fn render_number(
        renderer: &mut dyn RenderBackend,
        num: &crate::skin::object::number::NumberObject,
        _skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&num.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&num.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match num.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let texture = match num.texture {
            Some(t) => t,
            None => return Ok(()),
        };

        let value = state.number(num.ref_id);
        let digit_count = if num.digit > 0 { num.digit as usize } else { 1 };

        // Compute digits
        let is_negative = value < 0;
        let abs_value = value.unsigned_abs();
        let mut digits: Vec<usize> = Vec::new();
        let mut v = abs_value;
        if v == 0 {
            digits.push(0);
        } else {
            while v > 0 {
                digits.push((v % 10) as usize);
                v /= 10;
            }
        }
        digits.reverse();

        // Pad or truncate
        while digits.len() < digit_count {
            digits.insert(0, if num.padding == 0 { 0 } else { 10 }); // 10 = space
        }

        if is_negative && num.div_x > 10 {
            // Insert minus sign
            if digits.len() >= digit_count {
                digits[0] = 11; // minus
            }
        }

        let digit_w = interp.w / digit_count as f32;
        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        let blend = Self::blend_mode(num.dst.blend);

        for (i, &d) in digits.iter().enumerate() {
            if d == 10 && num.padding != 0 {
                continue; // skip space
            }
            let (sx, sy, sw, sh) = num.digit_src_rect(d);
            let dx = interp.x + i as f32 * digit_w;
            renderer.draw_sprite(
                texture,
                SrcRect {
                    x: sx,
                    y: sy,
                    w: sw,
                    h: sh,
                },
                DstRect {
                    x: dx,
                    y: interp.y,
                    w: digit_w,
                    h: interp.h,
                },
                color,
                0.0,
                blend,
            )?;
        }
        Ok(())
    }

    fn render_bargraph(
        renderer: &mut dyn RenderBackend,
        bg: &crate::skin::object::bargraph::BargraphObject,
        skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&bg.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&bg.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match bg.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let texture = match bg.texture {
            Some(t) => t,
            None => return Ok(()),
        };

        let value = state.float_value(bg.ref_id).clamp(0.0, 1.0);
        let sx = bg.src_x as f32;
        let sy = bg.src_y as f32;
        let sw = bg.src_w as f32;
        let sh = bg.src_h as f32;

        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        let blend = Self::blend_mode(bg.dst.blend);

        // Clip based on direction and value
        let (dst_x, dst_y, dst_w, dst_h, clip_sx, clip_sy, clip_sw, clip_sh) = match bg.direction {
            0 => {
                // up
                let h = interp.h * value;
                (
                    interp.x,
                    interp.y + interp.h - h,
                    interp.w,
                    h,
                    sx,
                    sy + sh * (1.0 - value),
                    sw,
                    sh * value,
                )
            }
            1 => {
                // right
                let w = interp.w * value;
                (interp.x, interp.y, w, interp.h, sx, sy, sw * value, sh)
            }
            2 => {
                // down
                let h = interp.h * value;
                (interp.x, interp.y, interp.w, h, sx, sy, sw, sh * value)
            }
            3 => {
                // left
                let w = interp.w * value;
                (
                    interp.x + interp.w - w,
                    interp.y,
                    w,
                    interp.h,
                    sx + sw * (1.0 - value),
                    sy,
                    sw * value,
                    sh,
                )
            }
            _ => (interp.x, interp.y, interp.w, interp.h, sx, sy, sw, sh),
        };

        let _ = skin; // avoid unused warning
        renderer.draw_sprite(
            texture,
            SrcRect {
                x: clip_sx,
                y: clip_sy,
                w: clip_sw,
                h: clip_sh,
            },
            DstRect {
                x: dst_x,
                y: dst_y,
                w: dst_w,
                h: dst_h,
            },
            color,
            0.0,
            blend,
        )?;
        Ok(())
    }

    fn render_slider(
        renderer: &mut dyn RenderBackend,
        sl: &crate::skin::object::slider::SliderObject,
        _skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&sl.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&sl.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match sl.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let texture = match sl.texture {
            Some(t) => t,
            None => return Ok(()),
        };

        let value = state.float_value(sl.ref_id).clamp(0.0, 1.0);
        let offset = sl.range * value;

        let (dx, dy) = match sl.direction {
            0 => (interp.x, interp.y - offset), // up
            1 => (interp.x + offset, interp.y), // right
            2 => (interp.x, interp.y + offset), // down
            3 => (interp.x - offset, interp.y), // left
            _ => (interp.x, interp.y),
        };

        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        let blend = Self::blend_mode(sl.dst.blend);

        renderer.draw_sprite(
            texture,
            SrcRect {
                x: sl.src_x as f32,
                y: sl.src_y as f32,
                w: sl.src_w as f32,
                h: sl.src_h as f32,
            },
            DstRect {
                x: dx,
                y: dy,
                w: interp.w,
                h: interp.h,
            },
            color,
            0.0,
            blend,
        )?;
        Ok(())
    }

    fn render_text(
        renderer: &mut dyn RenderBackend,
        txt: &crate::skin::object::text::TextObject,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&txt.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&txt.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match txt.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let font_id = match txt.font_id {
            Some(f) => f,
            None => return Ok(()),
        };

        let text = if let Some(ref s) = txt.static_text {
            s.as_str()
        } else {
            state.string(txt.ref_id)
        };

        if text.is_empty() {
            return Ok(());
        }

        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        renderer.draw_text(font_id, text, interp.x, interp.y, txt.size, color)?;
        Ok(())
    }

    fn render_image_set(
        renderer: &mut dyn RenderBackend,
        is: &crate::skin::object::image_set::ImageSetObject,
        skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&is.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&is.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match is.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let index = state.number(is.ref_id) as usize;
        let entry = match is.images.get(index) {
            Some(e) => e,
            None => return Ok(()),
        };
        let texture = match entry.texture {
            Some(t) => t,
            None => return Ok(()),
        };

        Self::draw_sprite(
            renderer,
            texture,
            entry.src_x as f32,
            entry.src_y as f32,
            entry.src_w as f32,
            entry.src_h as f32,
            &interp,
            skin,
            is.dst.blend,
            is.dst.center,
        )?;
        Ok(())
    }

    fn render_gauge(
        renderer: &mut dyn RenderBackend,
        gauge: &crate::skin::object::gauge::GaugeObject,
        _skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&gauge.dst, state) {
            return Ok(());
        }
        let time = match Self::effective_time(&gauge.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match gauge.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        let parts = gauge.parts.max(1) as usize;
        let gauge_value = state
            .float_value(crate::skin::skin_property::RATE_SCORE)
            .clamp(0.0, 1.0);
        let filled_parts = (gauge_value * parts as f32).ceil() as usize;

        let part_w = interp.w / parts as f32;
        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        let blend = Self::blend_mode(gauge.dst.blend);

        // Use textures[0]=empty_normal, textures[1]=filled_normal
        let empty_tex = gauge.textures.first();
        let filled_tex = gauge.textures.get(1);

        for i in 0..parts {
            let is_filled = i < filled_parts;
            let entry = if is_filled { filled_tex } else { empty_tex };
            let entry = match entry {
                Some(e) => e,
                None => continue,
            };
            let texture = match entry.texture {
                Some(t) => t,
                None => continue,
            };

            let dx = interp.x + i as f32 * part_w;
            renderer.draw_sprite(
                texture,
                SrcRect {
                    x: entry.src_x as f32,
                    y: entry.src_y as f32,
                    w: entry.src_w as f32,
                    h: entry.src_h as f32,
                },
                DstRect {
                    x: dx,
                    y: interp.y,
                    w: part_w,
                    h: interp.h,
                },
                color,
                0.0,
                blend,
            )?;
        }
        Ok(())
    }

    fn render_judge(
        renderer: &mut dyn RenderBackend,
        judge: &crate::skin::object::judge::JudgeObject,
        _skin: &SkinData,
        state: &SkinStateSnapshot,
    ) -> Result<()> {
        if !Self::check_options(&judge.dst, state) {
            return Ok(());
        }

        // Check judge timer
        let timer_id = match judge.player {
            0 => crate::skin::skin_property::TIMER_JUDGE_1P,
            1 => crate::skin::skin_property::TIMER_JUDGE_2P,
            _ => crate::skin::skin_property::TIMER_JUDGE_3P,
        };
        let judge_timer = match state.timer_ms(timer_id) {
            Some(t) => t,
            None => return Ok(()), // timer off, don't render
        };

        let time = match Self::effective_time(&judge.dst, state) {
            Some(t) => t,
            None => return Ok(()),
        };
        let interp = match judge.dst.interpolate(time) {
            Some(i) => i,
            None => return Ok(()),
        };
        if interp.a <= 0.0 {
            return Ok(());
        }

        // Get judge type from VALUE_JUDGE
        let value_id = match judge.player {
            0 => crate::skin::skin_property::VALUE_JUDGE_1P,
            1 => crate::skin::skin_property::VALUE_JUDGE_2P,
            _ => crate::skin::skin_property::VALUE_JUDGE_3P,
        };
        let judge_type = state.number(value_id).clamp(0, 5) as usize;

        let entry = match judge.textures.get(judge_type) {
            Some(e) => e,
            None => return Ok(()),
        };
        let texture = match entry.texture {
            Some(t) => t,
            None => return Ok(()),
        };

        // Animation frame calculation
        let div_x = entry.div_x.max(1);
        let div_y = entry.div_y.max(1);
        let total_frames = div_x * div_y;
        let frame = if entry.cycle > 0 && total_frames > 1 {
            let elapsed = (state.time_ms - judge_timer).max(0);
            ((elapsed / entry.cycle as i64) % total_frames as i64) as i32
        } else {
            0
        };

        let frame_w = entry.src_w as f32 / div_x as f32;
        let frame_h = entry.src_h as f32 / div_y as f32;
        let fx = (frame % div_x) as f32 * frame_w;
        let fy = (frame / div_x) as f32 * frame_h;

        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        let blend = Self::blend_mode(judge.dst.blend);

        renderer.draw_sprite(
            texture,
            SrcRect {
                x: entry.src_x as f32 + fx,
                y: entry.src_y as f32 + fy,
                w: frame_w,
                h: frame_h,
            },
            DstRect {
                x: interp.x,
                y: interp.y,
                w: interp.w,
                h: interp.h,
            },
            color,
            0.0,
            blend,
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_sprite(
        renderer: &mut dyn RenderBackend,
        texture: crate::traits::render::TextureId,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        interp: &InterpolatedDst,
        _skin: &SkinData,
        blend: i32,
        center: i32,
    ) -> Result<()> {
        let color = Color::new(interp.r, interp.g, interp.b, interp.a);
        let blend_mode = Self::blend_mode(blend);
        let angle = if interp.angle != 0 {
            interp.angle as f32
        } else {
            0.0
        };

        let _ = rotation_center(center);

        renderer.draw_sprite(
            texture,
            SrcRect {
                x: sx,
                y: sy,
                w: sw,
                h: sh,
            },
            DstRect {
                x: interp.x,
                y: interp.y,
                w: interp.w,
                h: interp.h,
            },
            color,
            angle,
            blend_mode,
        )?;
        Ok(())
    }
}
