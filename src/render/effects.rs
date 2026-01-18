use macroquad::prelude::*;

use super::font::draw_text_jp;
use super::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::game::JudgeResult;
use crate::skin::EffectConfig;

/// Judgment display effect
#[derive(Debug, Clone)]
pub struct JudgeEffect {
    result: JudgeResult,
    timer: f32,
    duration: f32,
    #[allow(dead_code)]
    x: f32,
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

        draw_text_jp(text, draw_x, self.y, font_size, color);
    }
}

/// Combo display effect
#[derive(Debug, Clone)]
pub struct ComboEffect {
    combo: u32,
    timer: f32,
    #[allow(dead_code)]
    duration: f32,
    #[allow(dead_code)]
    x: f32,
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

        draw_text_jp(
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

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.is_held
    }
}

impl Default for KeyBeam {
    fn default() -> Self {
        Self::new()
    }
}

/// Bomb effect displayed when a note is hit
#[derive(Debug, Clone, Copy)]
pub struct BombEffect {
    lane: usize,
    timer: f32,
    duration: f32,
}

impl BombEffect {
    pub fn new(lane: usize, duration: f32) -> Self {
        Self {
            lane,
            timer: duration,
            duration,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (1.0 - (self.timer / self.duration)).clamp(0.0, 1.0)
    }

    pub fn draw(&self, x: f32, y: f32, scale: f32, config: &EffectConfig) {
        let bomb = &config.bomb;
        if !bomb.enabled {
            return;
        }

        let t = self.progress();
        let eased = 1.0 - (1.0 - t) * (1.0 - t);
        let radius = (bomb.start_radius + (bomb.end_radius - bomb.start_radius) * eased) * scale;
        let alpha = (1.0 - t) * bomb.max_alpha;
        if alpha <= 0.0 || radius <= 0.0 {
            return;
        }

        let base_color: Color = bomb.color.into();
        let color = Color::new(base_color.r, base_color.g, base_color.b, alpha);
        let thickness = (bomb.line_thickness * scale).max(1.0);

        draw_circle_lines(x, y, radius, thickness, color);

        let glow_alpha = alpha * 0.25;
        if glow_alpha > 0.0 {
            let glow_color = Color::new(base_color.r, base_color.g, base_color.b, glow_alpha);
            draw_circle(x, y, radius * 0.4, glow_color);
        }
    }
}

use crate::bms::MAX_LANE_COUNT;

/// Effect manager for all visual effects
pub struct EffectManager {
    judge_effect: Option<JudgeEffect>,
    combo_effect: ComboEffect,
    lane_flashes: [LaneFlash; MAX_LANE_COUNT],
    key_beams: [KeyBeam; MAX_LANE_COUNT],
    bombs: Vec<BombEffect>,
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
            bombs: Vec::new(),
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

    pub fn trigger_bomb(&mut self, lane: usize) {
        if lane >= MAX_LANE_COUNT || !self.effect_config.bomb.enabled {
            return;
        }

        const MAX_BOMBS: usize = 128;
        if self.bombs.len() >= MAX_BOMBS {
            self.bombs.remove(0);
        }

        self.bombs
            .push(BombEffect::new(lane, self.effect_config.bomb.duration));
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
        for bomb in &mut self.bombs {
            bomb.update(dt);
        }
        self.bombs.retain(|bomb| bomb.is_active());
    }

    #[allow(dead_code)]
    pub fn draw_judge(&self) {
        if let Some(ref effect) = self.judge_effect {
            effect.draw(&self.effect_config);
        }
    }

    #[allow(dead_code)]
    pub fn draw_combo(&self) {
        self.combo_effect.draw(&self.effect_config);
    }

    /// Draw judge text, combo, and fast/slow at the specified position
    /// Format: "PGREAT 100" with FAST/SLOW displayed above in smaller font
    /// center_x: horizontal center of the highway
    /// y: Y position to draw (typically judge_y - 120.0)
    /// timing_diff_ms: timing difference in ms (positive = early/FAST, negative = late/SLOW)
    pub fn draw_judge_and_combo_at(&self, center_x: f32, y: f32, timing_diff_ms: Option<f64>) {
        let Some(ref effect) = self.judge_effect else {
            return;
        };
        if !effect.is_active() {
            return;
        }

        let config = &self.effect_config;

        // Calculate alpha for fade out
        let progress = effect.timer / effect.duration;
        let alpha = if progress < 0.3 { progress / 0.3 } else { 1.0 };

        let font_size = config.judge_font_size;

        // Draw FAST/SLOW above the judge text (skip for PGREAT)
        if effect.result != JudgeResult::PGreat {
            if let Some(diff) = timing_diff_ms {
                let (fs_text, fs_color) = if diff > 0.0 {
                    ("FAST", Color::new(0.3, 0.5, 1.0, alpha)) // Blue
                } else if diff < 0.0 {
                    ("SLOW", Color::new(1.0, 0.3, 0.3, alpha)) // Red
                } else {
                    ("", Color::default())
                };

                if !fs_text.is_empty() {
                    let fs_font_size = font_size * 0.6;
                    let fs_width = fs_text.len() as f32 * fs_font_size * 0.5;
                    let fs_x = center_x - fs_width / 2.0;
                    let fs_y = y - font_size;
                    draw_text_jp(fs_text, fs_x, fs_y, fs_font_size, fs_color);
                }
            }
        }

        // Build judge text: "PGREAT 100"
        let judge_text = match effect.result {
            JudgeResult::PGreat => "PGREAT",
            JudgeResult::Great => "GREAT",
            JudgeResult::Good => "GOOD",
            JudgeResult::Bad => "BAD",
            JudgeResult::Poor => "POOR",
        };

        // Always show combo count (>= 1)
        let text = if self.combo_effect.combo >= 1 {
            format!("{} {}", judge_text, self.combo_effect.combo)
        } else {
            judge_text.to_string()
        };

        let base_color = config.judge_color(effect.result);
        let color = Color::new(base_color.r, base_color.g, base_color.b, alpha);

        let text_width = text.len() as f32 * font_size * 0.5;
        let x = center_x - text_width / 2.0;

        draw_text_jp(&text, x, y, font_size, color);
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// Draw key beams within a specified rect (for IIDX-style layout)
    /// lane_widths should be scaled to the rect dimensions
    pub fn draw_key_beams_in_rect(
        &self,
        rect: &crate::skin::Rect,
        lane_widths: &[f32],
        lane_colors: &[Color],
        scale: f32,
        judge_y: f32,
    ) {
        let config = &self.effect_config.key_beam;
        if !config.enabled {
            return;
        }

        let beam_height = (judge_y - rect.y) * config.height_ratio;

        let mut x = rect.x;
        for (i, beam) in self.key_beams.iter().enumerate() {
            let base_width = lane_widths.get(i).copied().unwrap_or(50.0);
            let width = base_width * scale;

            if beam.is_active() {
                let base_color = lane_colors.get(i).copied().unwrap_or(WHITE);

                // Draw gradient: more opaque at bottom (judge line), transparent at top
                let segments = 20;
                let segment_height = beam_height / segments as f32;

                for seg in 0..segments {
                    let y = judge_y - (seg + 1) as f32 * segment_height;
                    // Don't draw above the rect
                    if y < rect.y {
                        continue;
                    }
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

    /// Draw bomb effects on the judge line for each lane
    pub fn draw_bombs_in_rect(
        &self,
        rect: &crate::skin::Rect,
        lane_widths: &[f32],
        scale: f32,
        judge_y: f32,
    ) {
        if self.bombs.is_empty() {
            return;
        }

        let mut lane_centers = Vec::with_capacity(lane_widths.len());
        let mut x = rect.x;
        for width in lane_widths.iter().copied() {
            let scaled_width = width * scale;
            lane_centers.push(x + scaled_width * 0.5);
            x += scaled_width;
        }

        for bomb in &self.bombs {
            let Some(&center_x) = lane_centers.get(bomb.lane) else {
                continue;
            };
            bomb.draw(center_x, judge_y, scale, &self.effect_config);
        }
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0 + 50.0)
    }
}
