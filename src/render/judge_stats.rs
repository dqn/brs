//! Judge statistics display for IIDX-style layout

use macroquad::prelude::*;

use super::font::draw_text_jp;
use crate::skin::Rect;

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
    pub fn update(
        &mut self,
        pgreat: u32,
        great: u32,
        good: u32,
        bad: u32,
        poor: u32,
        fast: u32,
        slow: u32,
    ) {
        self.pgreat = pgreat;
        self.great = great;
        self.good = good;
        self.bad = bad;
        self.poor = poor;
        self.combo_break = bad + poor;
        self.fast = fast;
        self.slow = slow;
    }

    /// Draw the judge statistics within the specified rect
    pub fn draw(&self, rect: &Rect) {
        let line_height = 22.0;
        let label_x = rect.x + 10.0;
        let value_x = rect.x + 60.0;
        let fast_slow_x = rect.x + 110.0;

        // Header
        draw_text_jp("JUDGE", label_x, rect.y + 20.0, 16.0, GRAY);

        let items = [
            ("PG:", self.pgreat, Color::new(0.0, 0.9, 0.9, 1.0)),
            ("GR:", self.great, Color::new(1.0, 0.8, 0.0, 1.0)),
            ("GD:", self.good, Color::new(0.0, 0.8, 0.0, 1.0)),
            ("BD:", self.bad, Color::new(0.5, 0.5, 1.0, 1.0)),
            ("PR:", self.poor, Color::new(0.8, 0.2, 0.2, 1.0)),
            ("CB:", self.combo_break, Color::new(0.5, 0.5, 0.5, 1.0)),
        ];

        for (i, (label, value, color)) in items.iter().enumerate() {
            let y = rect.y + 45.0 + i as f32 * line_height;
            draw_text_jp(label, label_x, y, 16.0, *color);
            draw_text_jp(&format!("{:5}", value), value_x, y, 16.0, *color);
        }

        // FAST/SLOW stats
        let fast_slow_y = rect.y + 45.0 + 6.0 * line_height + 10.0;
        draw_text_jp(
            &format!("FAST {:4}", self.fast),
            label_x,
            fast_slow_y,
            14.0,
            Color::new(0.0, 0.8, 1.0, 1.0),
        );
        draw_text_jp(
            &format!("{:4} SLOW", self.slow),
            fast_slow_x,
            fast_slow_y,
            14.0,
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
    pub fn draw(&self, rect: &Rect) {
        let center_x = rect.x + rect.width / 2.0;
        let y_start = rect.y + 30.0;
        let line_height = 35.0;

        // Large current BPM in center
        let current_str = format!("{}", self.current_bpm);
        let text_width = current_str.len() as f32 * 25.0;
        draw_text_jp(
            &current_str,
            center_x - text_width / 2.0,
            y_start + line_height,
            48.0,
            YELLOW,
        );

        // MIN / MAX labels
        draw_text_jp(
            &format!("{}", self.min_bpm),
            rect.x + 10.0,
            y_start + line_height,
            20.0,
            GRAY,
        );
        draw_text_jp("MIN", rect.x + 10.0, y_start + line_height + 20.0, 12.0, DARKGRAY);

        draw_text_jp(
            &format!("{}", self.max_bpm),
            rect.x + rect.width - 50.0,
            y_start + line_height,
            20.0,
            GRAY,
        );
        draw_text_jp(
            "MAX",
            rect.x + rect.width - 50.0,
            y_start + line_height + 20.0,
            12.0,
            DARKGRAY,
        );

        // BPM label
        draw_text_jp("BPM", center_x - 20.0, y_start + line_height + 45.0, 14.0, GRAY);
    }
}

impl Default for BpmDisplay {
    fn default() -> Self {
        Self::new(150)
    }
}
