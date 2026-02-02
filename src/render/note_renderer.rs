use crate::model::{LaneConfig, Note, NoteType, Timelines};
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
