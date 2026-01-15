use macroquad::prelude::*;

use crate::game::JudgeResult;

/// Time in seconds for effect animations
const JUDGE_EFFECT_DURATION: f32 = 0.3;
const COMBO_EFFECT_DURATION: f32 = 0.15;
const LANE_FLASH_DURATION: f32 = 0.1;

/// Judgment display effect
#[derive(Debug, Clone)]
pub struct JudgeEffect {
    result: JudgeResult,
    timer: f32,
    x: f32,
    y: f32,
}

impl JudgeEffect {
    pub fn new(result: JudgeResult, x: f32, y: f32) -> Self {
        Self {
            result,
            timer: JUDGE_EFFECT_DURATION,
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

    pub fn draw(&self) {
        if !self.is_active() {
            return;
        }

        let (text, base_color) = match self.result {
            JudgeResult::PGreat => ("PGREAT", Color::new(1.0, 1.0, 0.0, 1.0)),
            JudgeResult::Great => ("GREAT", Color::new(1.0, 0.8, 0.0, 1.0)),
            JudgeResult::Good => ("GOOD", Color::new(0.0, 1.0, 0.5, 1.0)),
            JudgeResult::Bad => ("BAD", Color::new(0.5, 0.5, 1.0, 1.0)),
            JudgeResult::Poor => ("POOR", Color::new(1.0, 0.3, 0.3, 1.0)),
        };

        // Animation progress (1.0 -> 0.0)
        let progress = self.timer / JUDGE_EFFECT_DURATION;

        // Scale: starts big, shrinks to normal
        let scale = 1.0 + (1.0 - progress) * 0.3;

        // Alpha fade out in last portion
        let alpha = if progress < 0.3 { progress / 0.3 } else { 1.0 };

        let color = Color::new(base_color.r, base_color.g, base_color.b, alpha);
        let font_size = 40.0 * scale;

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
    x: f32,
    y: f32,
}

impl ComboEffect {
    pub fn new(combo: u32, x: f32, y: f32) -> Self {
        Self {
            combo,
            timer: COMBO_EFFECT_DURATION,
            x,
            y,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer -= dt;
    }

    pub fn update_combo(&mut self, combo: u32) {
        self.combo = combo;
        self.timer = COMBO_EFFECT_DURATION;
    }

    pub fn draw(&self) {
        if self.combo < 2 {
            return;
        }

        // Animation progress (1.0 -> 0.0)
        let progress = (self.timer / COMBO_EFFECT_DURATION).max(0.0);

        // Bounce effect: scale up then back to normal
        let scale = if progress > 0.5 {
            1.0 + (progress - 0.5) * 0.4
        } else {
            1.0
        };

        let font_size = 36.0 * scale;
        let combo_text = format!("{} COMBO", self.combo);
        let text_width = combo_text.len() as f32 * font_size * 0.4;

        // Color based on combo count
        let color = if self.combo >= 1000 {
            Color::new(1.0, 0.8, 0.0, 1.0) // Gold
        } else if self.combo >= 500 {
            Color::new(1.0, 0.5, 0.0, 1.0) // Orange
        } else if self.combo >= 100 {
            Color::new(1.0, 1.0, 0.0, 1.0) // Yellow
        } else {
            WHITE
        };

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
#[derive(Debug, Clone, Copy, Default)]
pub struct LaneFlash {
    timer: f32,
}

impl LaneFlash {
    pub fn trigger(&mut self) {
        self.timer = LANE_FLASH_DURATION;
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
        if self.timer > 0.0 {
            self.timer / LANE_FLASH_DURATION
        } else {
            0.0
        }
    }
}

/// Effect manager for all visual effects
pub struct EffectManager {
    judge_effect: Option<JudgeEffect>,
    combo_effect: ComboEffect,
    lane_flashes: [LaneFlash; 8],
}

impl EffectManager {
    pub fn new(_judge_x: f32, _judge_y: f32, combo_x: f32, combo_y: f32) -> Self {
        Self {
            judge_effect: None,
            combo_effect: ComboEffect::new(0, combo_x, combo_y),
            lane_flashes: [LaneFlash::default(); 8],
        }
    }

    pub fn trigger_judge(&mut self, result: JudgeResult, x: f32, y: f32) {
        self.judge_effect = Some(JudgeEffect::new(result, x, y));
    }

    pub fn update_combo(&mut self, combo: u32) {
        self.combo_effect.update_combo(combo);
    }

    pub fn trigger_lane_flash(&mut self, lane: usize) {
        if lane < 8 {
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
            effect.draw();
        }
    }

    pub fn draw_combo(&self) {
        self.combo_effect.draw();
    }

    pub fn draw_lane_flashes(&self, highway_x: f32, lane_width: f32, highway_height: f32) {
        for (i, flash) in self.lane_flashes.iter().enumerate() {
            if flash.is_active() {
                let x = highway_x + i as f32 * lane_width;
                let alpha = flash.alpha() * 0.5;
                let color = Color::new(1.0, 1.0, 1.0, alpha);
                draw_rectangle(x, 0.0, lane_width, highway_height, color);
            }
        }
    }
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new(
            screen_width() / 2.0,
            screen_height() / 2.0,
            screen_width() / 2.0,
            screen_height() / 2.0 + 50.0,
        )
    }
}
