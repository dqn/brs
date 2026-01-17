use macroquad::prelude::*;

use crate::game::JudgeResult;
use crate::skin::EffectConfig;

/// Judgment display effect
#[derive(Debug, Clone)]
pub struct JudgeEffect {
    result: JudgeResult,
    timer: f32,
    duration: f32,
    x: f32,
    y: f32,
}

impl JudgeEffect {
    pub fn new(result: JudgeResult, x: f32, y: f32, duration: f32) -> Self {
        Self {
            result,
            timer: duration,
            duration,
            x,
            y,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    pub fn draw(&self, config: &EffectConfig) {
        if !self.is_active() {
            return;
        }

        let text = match self.result {
            JudgeResult::PGreat => "PGREAT",
            JudgeResult::Great => "GREAT",
            JudgeResult::Good => "GOOD",
            JudgeResult::Bad => "BAD",
            JudgeResult::Poor => "POOR",
        };
        let base_color = config.judge_color(self.result);

        // Animation progress (1.0 -> 0.0)
        let progress = self.timer / self.duration;

        // Scale: starts big, shrinks to normal
        let scale = 1.0 + (1.0 - progress) * 0.3;

        // Alpha fade out in last portion
        let alpha = if progress < 0.3 { progress / 0.3 } else { 1.0 };

        let color = Color::new(base_color.r, base_color.g, base_color.b, alpha);
        let font_size = config.judge_font_size * scale;

        // Calculate centered position
        let text_width = text.len() as f32 * font_size * 0.5;
        let draw_x = self.x - text_width / 2.0;

        draw_text(text, draw_x, self.y, font_size, color);
    }
}

/// Combo display effect
#[derive(Debug, Clone)]
pub struct ComboEffect {
    combo: u32,
    timer: f32,
    duration: f32,
    x: f32,
    y: f32,
}

impl ComboEffect {
    pub fn new(combo: u32, x: f32, y: f32, duration: f32) -> Self {
        Self {
            combo,
            timer: duration,
            duration,
            x,
            y,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
    }

    pub fn update_combo(&mut self, combo: u32, duration: f32) {
        self.combo = combo;
        self.timer = duration;
    }

    pub fn draw(&self, config: &EffectConfig) {
        if self.combo < 2 {
            return;
        }

        // Animation progress (1.0 -> 0.0)
        let progress = (self.timer / self.duration).max(0.0);

        // Bounce effect: scale up then back to normal
        let scale = if progress > 0.5 {
            1.0 + (progress - 0.5) * 0.4
        } else {
            1.0
        };

        let font_size = config.combo_font_size * scale;
        let combo_text = format!("{} COMBO", self.combo);
        let text_width = combo_text.len() as f32 * font_size * 0.4;

        // Color based on combo count (from skin config)
        let color = config.combo_color(self.combo);

        draw_text(
            &combo_text,
            self.x - text_width / 2.0,
            self.y,
            font_size,
            color,
        );
    }
}

/// Lane flash effect when key is pressed
#[derive(Debug, Clone, Copy)]
pub struct LaneFlash {
    timer: f32,
    duration: f32,
}

impl LaneFlash {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: 0.0,
            duration,
        }
    }

    pub fn trigger(&mut self) {
        self.timer = self.duration;
    }

    pub fn update(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer -= dt;
        }
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    pub fn alpha(&self) -> f32 {
        if self.timer > 0.0 && self.duration > 0.0 {
            self.timer / self.duration
        } else {
            0.0
        }
    }
}

impl Default for LaneFlash {
    fn default() -> Self {
        Self::new(0.1) // Default 100ms
    }
}

/// Key beam effect displayed while key is held
#[derive(Debug, Clone, Copy)]
pub struct KeyBeam {
    is_held: bool,
}

impl KeyBeam {
    pub fn new() -> Self {
        Self { is_held: false }
    }

    pub fn set_held(&mut self, held: bool) {
        self.is_held = held;
    }

    pub fn is_active(&self) -> bool {
        self.is_held
    }
}

impl Default for KeyBeam {
    fn default() -> Self {
        Self::new()
    }
}

use crate::bms::MAX_LANE_COUNT;

/// Effect manager for all visual effects
pub struct EffectManager {
    judge_effect: Option<JudgeEffect>,
    combo_effect: ComboEffect,
    lane_flashes: [LaneFlash; MAX_LANE_COUNT],
    key_beams: [KeyBeam; MAX_LANE_COUNT],
    effect_config: EffectConfig,
}

impl EffectManager {
    pub fn new(combo_x: f32, combo_y: f32) -> Self {
        Self::with_config(combo_x, combo_y, EffectConfig::default())
    }

    pub fn with_config(combo_x: f32, combo_y: f32, config: EffectConfig) -> Self {
        let lane_flash = LaneFlash::new(config.lane_flash_duration);
        Self {
            judge_effect: None,
            combo_effect: ComboEffect::new(0, combo_x, combo_y, config.combo_duration),
            lane_flashes: [lane_flash; MAX_LANE_COUNT],
            key_beams: [KeyBeam::new(); MAX_LANE_COUNT],
            effect_config: config,
        }
    }

    /// Get the effect config
    #[allow(dead_code)]
    pub fn config(&self) -> &EffectConfig {
        &self.effect_config
    }

    pub fn trigger_judge(&mut self, result: JudgeResult, x: f32, y: f32) {
        self.judge_effect = Some(JudgeEffect::new(
            result,
            x,
            y,
            self.effect_config.judge_duration,
        ));
    }

    pub fn update_combo(&mut self, combo: u32) {
        self.combo_effect
            .update_combo(combo, self.effect_config.combo_duration);
    }

    pub fn trigger_lane_flash(&mut self, lane: usize) {
        if lane < MAX_LANE_COUNT {
            self.lane_flashes[lane].trigger();
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(ref mut effect) = self.judge_effect {
            effect.update(dt);
            if !effect.is_active() {
                self.judge_effect = None;
            }
        }
        self.combo_effect.update(dt);
        for flash in &mut self.lane_flashes {
            flash.update(dt);
        }
    }

    pub fn draw_judge(&self) {
        if let Some(ref effect) = self.judge_effect {
            effect.draw(&self.effect_config);
        }
    }

    pub fn draw_combo(&self) {
        self.combo_effect.draw(&self.effect_config);
    }

    pub fn draw_lane_flashes(&self, highway_x: f32, lane_widths: &[f32], highway_height: f32) {
        let max_alpha = self.effect_config.lane_flash_alpha;
        let mut x = highway_x;
        for (i, flash) in self.lane_flashes.iter().enumerate() {
            let width = lane_widths.get(i).copied().unwrap_or(50.0);
            if flash.is_active() {
                let alpha = flash.alpha() * max_alpha;
                let color = Color::new(1.0, 1.0, 1.0, alpha);
                draw_rectangle(x, 0.0, width, highway_height, color);
            }
            x += width;
        }
    }

    /// Set key held state for a lane
    pub fn set_key_held(&mut self, lane: usize, held: bool) {
        if lane < MAX_LANE_COUNT {
            self.key_beams[lane].set_held(held);
        }
    }

    /// Draw key beams for held keys
    /// Draws a gradient beam from judge_y upward with configurable alpha
    pub fn draw_key_beams(
        &self,
        highway_x: f32,
        lane_widths: &[f32],
        judge_y: f32,
        lane_colors: &[Color],
    ) {
        let config = &self.effect_config.key_beam;
        if !config.enabled {
            return;
        }

        let beam_height = judge_y * config.height_ratio;

        let mut x = highway_x;
        for (i, beam) in self.key_beams.iter().enumerate() {
            let width = lane_widths.get(i).copied().unwrap_or(50.0);

            if beam.is_active() {
                let base_color = lane_colors.get(i).copied().unwrap_or(WHITE);

                // Draw gradient: more opaque at bottom (judge line), transparent at top
                let segments = 20;
                let segment_height = beam_height / segments as f32;

                for seg in 0..segments {
                    let y = judge_y - (seg + 1) as f32 * segment_height;
                    let progress = seg as f32 / segments as f32;
                    // Interpolate alpha from max (bottom) to min (top)
                    let alpha = config.max_alpha * (1.0 - progress) + config.min_alpha * progress;
                    let color = Color::new(base_color.r, base_color.g, base_color.b, alpha);
                    draw_rectangle(x, y, width, segment_height, color);
                }
            }

            x += width;
        }
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new(screen_width() / 2.0, screen_height() / 2.0 + 50.0)
    }
}
