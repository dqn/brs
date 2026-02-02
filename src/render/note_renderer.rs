use crate::model::{LaneConfig, LaneCoverSettings, Note, NoteType, Timelines};
use macroquad::prelude::*;

/// Renderer for notes (normal, long, mine).
pub struct NoteRenderer<'a> {
    config: &'a LaneConfig,
}

impl<'a> NoteRenderer<'a> {
    /// Create a new note renderer.
    pub fn new(config: &'a LaneConfig) -> Self {
        Self { config }
    }

    /// Draw all notes from timelines.
    pub fn draw(&self, timelines: &Timelines, current_time_ms: f64, hi_speed: f32) {
        for note in timelines.all_notes() {
            self.draw_note(note, current_time_ms, hi_speed);
        }
    }

    /// Draw notes with a filter function that determines visibility by note index.
    pub fn draw_with_filter<F>(
        &self,
        timelines: &Timelines,
        current_time_ms: f64,
        hi_speed: f32,
        filter: F,
    ) where
        F: Fn(usize) -> bool,
    {
        for (index, note) in timelines.all_notes().enumerate() {
            if filter(index) {
                self.draw_note(note, current_time_ms, hi_speed);
            }
        }
    }

    /// Draw notes with lane cover settings and a filter function.
    pub fn draw_with_cover<F>(
        &self,
        timelines: &Timelines,
        current_time_ms: f64,
        hi_speed: f32,
        cover: &LaneCoverSettings,
        filter: F,
    ) where
        F: Fn(usize) -> bool,
    {
        for (index, note) in timelines.all_notes().enumerate() {
            if filter(index) {
                self.draw_note_with_cover(note, current_time_ms, hi_speed, cover);
            }
        }
    }

    /// Draw lane cover overlays (SUDDEN+ and HIDDEN+).
    pub fn draw_cover_overlay(&self, cover: &LaneCoverSettings) {
        if !cover.is_active() {
            return;
        }

        let lane_top = self.config.lane_top_y;
        let judge_line = self.config.judge_line_y;
        let lane_height = judge_line - lane_top;
        let lane_left = self.config.offset_x;
        let lane_width = self.config.total_width;

        let cover_color = Color::new(0.0, 0.0, 0.0, 0.9);

        // SUDDEN+ cover (top)
        if cover.sudden_plus > 0.0 {
            let sudden_height = lane_height * cover.sudden_plus;
            draw_rectangle(lane_left, lane_top, lane_width, sudden_height, cover_color);

            // Draw green line at cover edge
            draw_line(
                lane_left,
                lane_top + sudden_height,
                lane_left + lane_width,
                lane_top + sudden_height,
                2.0,
                GREEN,
            );
        }

        // HIDDEN+ cover (bottom)
        if cover.hidden_plus > 0.0 {
            let effective_judge = cover.effective_judge_line_y(judge_line, lane_top);
            let hidden_top = cover.hidden_cover_top_y(effective_judge, lane_height);
            let hidden_height = effective_judge - hidden_top;
            draw_rectangle(
                lane_left,
                hidden_top,
                lane_width,
                hidden_height,
                cover_color,
            );

            // Draw green line at cover edge
            draw_line(
                lane_left,
                hidden_top,
                lane_left + lane_width,
                hidden_top,
                2.0,
                GREEN,
            );
        }

        // Display cover values
        self.draw_cover_info(cover);
    }

    /// Draw cover information display.
    fn draw_cover_info(&self, cover: &LaneCoverSettings) {
        let x = self.config.offset_x + self.config.total_width + 20.0;
        let y = self.config.lane_top_y + 50.0;

        if cover.sudden_plus > 0.0 {
            draw_text(
                &format!("SUD+: {:.0}%", cover.sudden_plus * 100.0),
                x,
                y,
                18.0,
                GREEN,
            );
        }
        if cover.hidden_plus > 0.0 {
            draw_text(
                &format!("HID+: {:.0}%", cover.hidden_plus * 100.0),
                x,
                y + 20.0,
                18.0,
                GREEN,
            );
        }
        if cover.lift > 0.0 {
            draw_text(
                &format!("LIFT: {:.0}%", cover.lift * 100.0),
                x,
                y + 40.0,
                18.0,
                GREEN,
            );
        }
    }

    /// Draw a single note.
    fn draw_note(&self, note: &Note, current_time_ms: f64, hi_speed: f32) {
        let y = self
            .config
            .time_to_y(note.start_time_ms, current_time_ms, hi_speed);

        if note.is_long() {
            let end_y = note
                .end_time_ms
                .map(|end| self.config.time_to_y(end, current_time_ms, hi_speed));
            self.draw_long_note(note, y, end_y);
        } else {
            if !self.config.is_visible(y) {
                return;
            }
            self.draw_single_note(note, y);
        }
    }

    /// Draw a single note with lane cover settings.
    fn draw_note_with_cover(
        &self,
        note: &Note,
        current_time_ms: f64,
        hi_speed: f32,
        cover: &LaneCoverSettings,
    ) {
        let effective_judge_y =
            cover.effective_judge_line_y(self.config.judge_line_y, self.config.lane_top_y);

        // Calculate Y position with LIFT applied
        let y = self.time_to_y_with_lift(note.start_time_ms, current_time_ms, hi_speed, cover);

        if note.is_long() {
            let end_y = note
                .end_time_ms
                .map(|end| self.time_to_y_with_lift(end, current_time_ms, hi_speed, cover));
            self.draw_long_note_with_cover(note, y, end_y, cover, effective_judge_y);
        } else {
            // Check visibility with cover
            if !cover.is_y_visible(y, self.config.lane_top_y, effective_judge_y) {
                return;
            }
            if !self.config.is_visible(y) {
                return;
            }
            self.draw_single_note(note, y);
        }
    }

    /// Convert time to Y position with LIFT applied.
    fn time_to_y_with_lift(
        &self,
        time_ms: f64,
        current_time_ms: f64,
        hi_speed: f32,
        cover: &LaneCoverSettings,
    ) -> f32 {
        let effective_judge_y =
            cover.effective_judge_line_y(self.config.judge_line_y, self.config.lane_top_y);
        let delta_ms = (time_ms - current_time_ms) as f32;
        let pixels_per_ms = hi_speed * 0.5;
        effective_judge_y - delta_ms * pixels_per_ms
    }

    /// Draw a long note with lane cover settings.
    fn draw_long_note_with_cover(
        &self,
        note: &Note,
        start_y: f32,
        end_y: Option<f32>,
        cover: &LaneCoverSettings,
        effective_judge_y: f32,
    ) {
        let x = self.config.lane_x(note.lane);
        let width = self.config.lane_width(note.lane);
        let note_height = 10.0;

        let end_y = end_y.unwrap_or(self.config.lane_top_y);
        let body_top = end_y.min(start_y);
        let body_bottom = end_y.max(start_y);

        // Clip body to visible area
        let lane_height = effective_judge_y - self.config.lane_top_y;
        let visible_top = cover.sudden_cover_bottom_y(self.config.lane_top_y, lane_height);
        let visible_bottom = cover.hidden_cover_top_y(effective_judge_y, lane_height);

        let clipped_top = body_top.max(visible_top);
        let clipped_bottom = body_bottom.min(visible_bottom);

        if clipped_top >= clipped_bottom {
            return; // Entirely covered
        }

        let lane_layout = self.config.get_layout(note.lane);
        let base_color = lane_layout
            .map(|l| l.color)
            .unwrap_or(Color::new(0.8, 0.8, 0.8, 1.0));

        // Draw body (clipped)
        let body_color = Color::new(
            base_color.r * 0.7,
            base_color.g * 0.7,
            base_color.b * 0.7,
            0.6,
        );
        draw_rectangle(
            x + 3.0,
            clipped_top,
            width - 6.0,
            clipped_bottom - clipped_top,
            body_color,
        );

        // Draw start note if visible
        let start_color = Color::new(base_color.r, base_color.g, base_color.b, 1.0);
        if cover.is_y_visible(start_y, self.config.lane_top_y, effective_judge_y)
            && self.config.is_visible(start_y)
        {
            draw_rectangle(
                x + 1.0,
                start_y - note_height / 2.0,
                width - 2.0,
                note_height,
                start_color,
            );
        }

        // Draw end note if visible
        let end_color = Color::new(
            base_color.r * 0.8,
            base_color.g * 0.8,
            base_color.b * 0.8,
            1.0,
        );
        if cover.is_y_visible(end_y, self.config.lane_top_y, effective_judge_y)
            && self.config.is_visible(end_y)
        {
            draw_rectangle(
                x + 1.0,
                end_y - note_height / 2.0,
                width - 2.0,
                note_height,
                end_color,
            );
        }
    }

    /// Draw a single (non-long) note.
    fn draw_single_note(&self, note: &Note, y: f32) {
        let x = self.config.lane_x(note.lane);
        let width = self.config.lane_width(note.lane);
        let height = 10.0;

        let color = self.get_note_color(note);

        draw_rectangle(x + 1.0, y - height / 2.0, width - 2.0, height, color);

        if note.note_type == NoteType::Normal {
            let highlight_color = Color::new(1.0, 1.0, 1.0, 0.5);
            draw_rectangle(
                x + 2.0,
                y - height / 2.0 + 1.0,
                width - 4.0,
                2.0,
                highlight_color,
            );
        }
    }

    /// Draw a long note with start, body, and end.
    fn draw_long_note(&self, note: &Note, start_y: f32, end_y: Option<f32>) {
        let x = self.config.lane_x(note.lane);
        let width = self.config.lane_width(note.lane);
        let note_height = 10.0;

        let end_y = end_y.unwrap_or(self.config.lane_top_y);
        let body_top = end_y.min(start_y);
        let body_bottom = end_y.max(start_y);

        let lane_layout = self.config.get_layout(note.lane);
        let base_color = lane_layout
            .map(|l| l.color)
            .unwrap_or(Color::new(0.8, 0.8, 0.8, 1.0));

        let body_color = Color::new(
            base_color.r * 0.7,
            base_color.g * 0.7,
            base_color.b * 0.7,
            0.6,
        );
        draw_rectangle(
            x + 3.0,
            body_top,
            width - 6.0,
            body_bottom - body_top,
            body_color,
        );

        let start_color = Color::new(base_color.r, base_color.g, base_color.b, 1.0);
        if self.config.is_visible(start_y) {
            draw_rectangle(
                x + 1.0,
                start_y - note_height / 2.0,
                width - 2.0,
                note_height,
                start_color,
            );
        }

        let end_color = Color::new(
            base_color.r * 0.8,
            base_color.g * 0.8,
            base_color.b * 0.8,
            1.0,
        );
        if self.config.is_visible(end_y) {
            draw_rectangle(
                x + 1.0,
                end_y - note_height / 2.0,
                width - 2.0,
                note_height,
                end_color,
            );
        }
    }

    /// Get the color for a note based on its type.
    fn get_note_color(&self, note: &Note) -> Color {
        match note.note_type {
            NoteType::Mine => Color::new(1.0, 0.8, 0.0, 1.0),
            NoteType::Invisible => Color::new(0.5, 0.5, 0.5, 0.3),
            _ => {
                let lane_layout = self.config.get_layout(note.lane);
                lane_layout
                    .map(|l| l.color)
                    .unwrap_or(Color::new(0.8, 0.8, 0.8, 1.0))
            }
        }
    }
}
