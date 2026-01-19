//! Judge statistics display for IIDX-style layout

use macroquad::prelude::*;

use super::font::{draw_text_jp, measure_text_jp};
use crate::skin::{BpmDisplayLayout, JudgeStatsLayout, Rect};

/// Judge statistics data
pub struct JudgeData {
    pub pgreat: u32,
    pub great: u32,
    pub good: u32,
    pub bad: u32,
    pub poor: u32,
    pub fast: u32,
    pub slow: u32,
}

/// Judge statistics display
pub struct JudgeStats {
    pub pgreat: u32,
    pub great: u32,
    pub good: u32,
    pub bad: u32,
    pub poor: u32,
    pub combo_break: u32,
    pub fast: u32,
    pub slow: u32,
}

impl JudgeStats {
    pub fn new() -> Self {
        Self {
            pgreat: 0,
            great: 0,
            good: 0,
            bad: 0,
            poor: 0,
            combo_break: 0,
            fast: 0,
            slow: 0,
        }
    }

    /// Update stats from score manager
    pub fn update(&mut self, data: JudgeData) {
        self.pgreat = data.pgreat;
        self.great = data.great;
        self.good = data.good;
        self.bad = data.bad;
        self.poor = data.poor;
        self.combo_break = data.bad + data.poor;
        self.fast = data.fast;
        self.slow = data.slow;
    }

    /// Draw the judge statistics within the specified rect
    pub fn draw(&self, rect: &Rect, layout: &JudgeStatsLayout) {
        let label_x = rect.x + layout.label_x;
        let value_x = rect.x + layout.value_x;

        // Header
        draw_text_jp(
            "JUDGE",
            label_x,
            rect.y + layout.header_y,
            layout.header_font_size,
            GRAY,
        );

        let items = [
            ("PG:", self.pgreat, Color::new(0.0, 0.9, 0.9, 1.0)),
            ("GR:", self.great, Color::new(1.0, 0.8, 0.0, 1.0)),
            ("GD:", self.good, Color::new(0.0, 0.8, 0.0, 1.0)),
            ("BD:", self.bad, Color::new(0.5, 0.5, 1.0, 1.0)),
            ("PR:", self.poor, Color::new(0.8, 0.2, 0.2, 1.0)),
            ("CB:", self.combo_break, Color::new(0.5, 0.5, 0.5, 1.0)),
        ];

        for (i, (label, value, color)) in items.iter().enumerate() {
            let y = rect.y + layout.item_start_y + i as f32 * layout.item_line_height;
            draw_text_jp(label, label_x, y, layout.item_font_size, *color);
            draw_text_jp(
                &format!("{:5}", value),
                value_x,
                y,
                layout.item_font_size,
                *color,
            );
        }

        // FAST/SLOW stats - position SLOW at right edge of rect
        let fast_slow_y = rect.y + layout.fast_slow_y;
        draw_text_jp(
            &format!("FAST {:4}", self.fast),
            rect.x + layout.fast_label_x,
            fast_slow_y,
            layout.fast_slow_font_size,
            Color::new(0.0, 0.8, 1.0, 1.0),
        );
        let slow_str = format!("{:4} SLOW", self.slow);
        let slow_width = measure_text_jp(&slow_str, layout.fast_slow_font_size).width;
        let slow_x = rect.x + rect.width - layout.slow_right_margin - slow_width;
        draw_text_jp(
            &slow_str,
            slow_x,
            fast_slow_y,
            layout.fast_slow_font_size,
            Color::new(1.0, 0.5, 0.0, 1.0),
        );
    }
}

impl Default for JudgeStats {
    fn default() -> Self {
        Self::new()
    }
}

/// BPM display component
pub struct BpmDisplay {
    pub min_bpm: u32,
    pub current_bpm: u32,
    pub max_bpm: u32,
}

impl BpmDisplay {
    pub fn new(bpm: u32) -> Self {
        Self {
            min_bpm: bpm,
            current_bpm: bpm,
            max_bpm: bpm,
        }
    }

    pub fn update(&mut self, min: u32, current: u32, max: u32) {
        self.min_bpm = min;
        self.current_bpm = current;
        self.max_bpm = max;
    }

    /// Draw BPM display within the specified rect
    pub fn draw(&self, rect: &Rect, layout: &BpmDisplayLayout) {
        let center_x = rect.x + rect.width / 2.0;

        // Large current BPM in center
        let current_str = format!("{}", self.current_bpm);
        let current_width = measure_text_jp(&current_str, layout.current_font_size).width;
        let current_x = center_x - current_width / 2.0 + layout.current_center_offset_x;
        draw_text_jp(
            &current_str,
            current_x,
            rect.y + layout.current_y,
            layout.current_font_size,
            YELLOW,
        );

        // MIN / MAX labels
        let min_str = format!("{}", self.min_bpm);
        draw_text_jp(
            &min_str,
            rect.x + layout.min_x,
            rect.y + layout.min_y,
            layout.min_font_size,
            GRAY,
        );
        draw_text_jp(
            "MIN",
            rect.x + layout.min_x,
            rect.y + layout.min_max_label_y,
            layout.label_font_size,
            DARKGRAY,
        );

        let max_str = format!("{}", self.max_bpm);
        let max_width = measure_text_jp(&max_str, layout.max_font_size).width;
        let max_x = rect.x + rect.width - layout.max_right_margin - max_width;
        draw_text_jp(
            &max_str,
            max_x,
            rect.y + layout.max_y,
            layout.max_font_size,
            GRAY,
        );
        draw_text_jp(
            "MAX",
            max_x,
            rect.y + layout.min_max_label_y,
            layout.label_font_size,
            DARKGRAY,
        );

        // BPM label
        let bpm_width = measure_text_jp("BPM", layout.bpm_label_font_size).width;
        let bpm_x = center_x - bpm_width / 2.0 + layout.bpm_label_center_offset_x;
        draw_text_jp(
            "BPM",
            bpm_x,
            rect.y + layout.bpm_label_y,
            layout.bpm_label_font_size,
            GRAY,
        );
    }
}

impl Default for BpmDisplay {
    fn default() -> Self {
        Self::new(150)
    }
}
