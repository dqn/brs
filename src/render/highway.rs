use macroquad::prelude::*;

use super::config::{HighwayConfig, LANE_COUNT};
use super::lane_cover::LaneCover;
use crate::bms::{Chart, Note, NoteType};
use crate::game::GamePlayState;

pub struct Highway {
    config: HighwayConfig,
    lane_cover: LaneCover,
}

impl Highway {
    pub fn new() -> Self {
        Self::with_config(HighwayConfig::default())
    }

    pub fn with_config(config: HighwayConfig) -> Self {
        Self {
            config,
            lane_cover: LaneCover::default(),
        }
    }

    /// Get mutable reference to lane cover for adjustments
    pub fn lane_cover_mut(&mut self) -> &mut LaneCover {
        &mut self.lane_cover
    }

    /// Get current lane cover settings
    #[allow(dead_code)]
    pub fn lane_cover(&self) -> &LaneCover {
        &self.lane_cover
    }

    /// Set lane cover values
    pub fn set_lane_cover(&mut self, lane_cover: LaneCover) {
        self.lane_cover = lane_cover;
    }

    // Public API for drawing highway without game state (simple mode)
    #[allow(dead_code)]
    pub fn draw(&self, chart: &Chart, current_time_ms: f64, scroll_speed: f32) {
        let highway_x = self.highway_x();

        self.draw_lanes(highway_x);
        self.draw_notes_simple(chart, current_time_ms, scroll_speed, highway_x);
        self.draw_judge_line(highway_x);
        self.draw_info(chart);
    }

    pub fn draw_with_state(
        &self,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        scroll_speed: f32,
    ) {
        let highway_x = self.highway_x();

        self.draw_lanes(highway_x);
        self.draw_notes_with_state(chart, play_state, current_time_ms, scroll_speed, highway_x);
        self.draw_judge_line(highway_x);
        self.draw_lane_covers(highway_x);
        self.draw_info(chart);
    }

    fn highway_x(&self) -> f32 {
        (screen_width() - self.config.lane_width * LANE_COUNT as f32) / 2.0
    }

    fn draw_lanes(&self, highway_x: f32) {
        for i in 0..LANE_COUNT {
            let x = highway_x + i as f32 * self.config.lane_width;
            draw_rectangle(
                x,
                0.0,
                self.config.lane_width,
                screen_height(),
                Color::new(0.1, 0.1, 0.1, 1.0),
            );
            draw_line(
                x,
                0.0,
                x,
                screen_height(),
                1.0,
                Color::new(0.3, 0.3, 0.3, 1.0),
            );
        }
        let last_x = highway_x + LANE_COUNT as f32 * self.config.lane_width;
        draw_line(
            last_x,
            0.0,
            last_x,
            screen_height(),
            1.0,
            Color::new(0.3, 0.3, 0.3, 1.0),
        );
    }

    // Helper method for drawing notes without state (for simple mode)
    #[allow(dead_code)]
    fn draw_notes_simple(
        &self,
        chart: &Chart,
        current_time_ms: f64,
        scroll_speed: f32,
        highway_x: f32,
    ) {
        self.draw_notes_filtered(chart, None, current_time_ms, scroll_speed, highway_x);
    }

    fn draw_notes_with_state(
        &self,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        scroll_speed: f32,
        highway_x: f32,
    ) {
        self.draw_notes_filtered(
            chart,
            Some(play_state),
            current_time_ms,
            scroll_speed,
            highway_x,
        );
    }

    fn draw_notes_filtered(
        &self,
        chart: &Chart,
        play_state: Option<&GamePlayState>,
        current_time_ms: f64,
        scroll_speed: f32,
        highway_x: f32,
    ) {
        let pixels_per_ms = scroll_speed as f64 * 0.5;

        // First draw long note bars
        self.draw_long_note_bars(chart, play_state, current_time_ms, pixels_per_ms, highway_x);

        // Then draw notes on top
        for (i, note) in chart.notes.iter().enumerate() {
            if let Some(state) = play_state {
                if !state.get_state(i).is_some_and(|s| s.is_pending()) {
                    continue;
                }
            }

            let time_diff = note.time_ms - current_time_ms;

            if !(-100.0..=self.config.visible_range_ms).contains(&time_diff) {
                continue;
            }

            self.draw_note(note, time_diff, pixels_per_ms, highway_x);
        }
    }

    fn draw_long_note_bars(
        &self,
        chart: &Chart,
        play_state: Option<&GamePlayState>,
        current_time_ms: f64,
        pixels_per_ms: f64,
        highway_x: f32,
    ) {
        let long_color = Color::new(0.0, 0.8, 0.4, 0.7);

        for (i, note) in chart.notes.iter().enumerate() {
            if note.note_type != NoteType::LongStart {
                continue;
            }

            if let Some(state) = play_state {
                if !state.get_state(i).is_some_and(|s| s.is_pending()) {
                    continue;
                }
            }

            // Find corresponding LongEnd
            let end_note = chart
                .notes
                .iter()
                .skip(i + 1)
                .find(|n| n.channel == note.channel && n.note_type == NoteType::LongEnd);

            if let Some(end) = end_note {
                let start_time_diff = note.time_ms - current_time_ms;
                let end_time_diff = end.time_ms - current_time_ms;

                let start_y = self.config.judge_line_y - (start_time_diff * pixels_per_ms) as f32;
                let end_y = self.config.judge_line_y - (end_time_diff * pixels_per_ms) as f32;

                let lane = note.channel.lane_index();
                let x = highway_x + lane as f32 * self.config.lane_width;

                let bar_height = start_y - end_y;
                if bar_height > 0.0 {
                    draw_rectangle(
                        x + 4.0,
                        end_y,
                        self.config.lane_width - 8.0,
                        bar_height,
                        long_color,
                    );
                }
            }
        }
    }

    fn draw_note(&self, note: &Note, time_diff: f64, pixels_per_ms: f64, highway_x: f32) {
        let y = self.config.judge_line_y - (time_diff * pixels_per_ms) as f32;
        let lane = note.channel.lane_index();
        let x = highway_x + lane as f32 * self.config.lane_width;

        let color = match note.note_type {
            NoteType::Normal => self.config.lane_colors[lane],
            NoteType::LongStart | NoteType::LongEnd => Color::new(0.0, 1.0, 0.5, 1.0),
            NoteType::Invisible => Color::new(0.5, 0.5, 0.5, 0.5),
            NoteType::Landmine => Color::new(1.0, 0.0, 0.0, 0.8),
        };

        draw_rectangle(
            x + 2.0,
            y - self.config.note_height / 2.0,
            self.config.lane_width - 4.0,
            self.config.note_height,
            color,
        );
    }

    fn draw_judge_line(&self, highway_x: f32) {
        // Adjust judge line position based on LIFT
        let lift_offset = self.lane_cover.judge_line_position() * self.config.judge_line_y;
        let adjusted_judge_y = self.config.judge_line_y - lift_offset;

        draw_line(
            highway_x,
            adjusted_judge_y,
            highway_x + self.config.lane_width * LANE_COUNT as f32,
            adjusted_judge_y,
            3.0,
            Color::new(1.0, 0.8, 0.0, 1.0),
        );
    }

    fn draw_lane_covers(&self, highway_x: f32) {
        let highway_width = self.config.lane_width * LANE_COUNT as f32;
        let lane_height = self.config.judge_line_y; // Lane goes from top to judge line
        let cover_color = Color::new(0.0, 0.0, 0.0, 0.9);

        // Draw SUDDEN+ cover (top of lane)
        if self.lane_cover.sudden > 0 {
            let cover_height = (self.lane_cover.sudden as f32 / 1000.0) * lane_height;
            draw_rectangle(highway_x, 0.0, highway_width, cover_height, cover_color);

            // Draw white number display on cover
            draw_text(
                &format!("SUD+ {}", self.lane_cover.sudden),
                highway_x + 10.0,
                cover_height - 10.0,
                18.0,
                Color::new(0.5, 0.5, 0.5, 1.0),
            );
        }

        // Draw LIFT cover (bottom, raises judge line visually)
        if self.lane_cover.lift > 0 {
            let cover_height = (self.lane_cover.lift as f32 / 1000.0) * lane_height;
            let cover_y = self.config.judge_line_y - cover_height;
            draw_rectangle(
                highway_x,
                cover_y,
                highway_width,
                cover_height + 50.0,
                cover_color,
            );

            draw_text(
                &format!("LIFT {}", self.lane_cover.lift),
                highway_x + 10.0,
                cover_y + 20.0,
                18.0,
                Color::new(0.5, 0.5, 0.5, 1.0),
            );
        }

        // Draw HIDDEN+ cover (below judge line, covers notes after they pass)
        if self.lane_cover.hidden > 0 {
            let below_judge_height = screen_height() - self.config.judge_line_y;
            let cover_height = (self.lane_cover.hidden as f32 / 1000.0) * below_judge_height;
            draw_rectangle(
                highway_x,
                self.config.judge_line_y,
                highway_width,
                cover_height,
                cover_color,
            );

            draw_text(
                &format!("HID+ {}", self.lane_cover.hidden),
                highway_x + 10.0,
                self.config.judge_line_y + 20.0,
                18.0,
                Color::new(0.5, 0.5, 0.5, 1.0),
            );
        }
    }

    fn draw_info(&self, chart: &Chart) {
        let info_x = 10.0;
        let info_y = screen_height() - 80.0;

        draw_text(&chart.metadata.title, info_x, info_y, 24.0, WHITE);
        draw_text(&chart.metadata.artist, info_x, info_y + 25.0, 18.0, GRAY);
        draw_text(
            &format!(
                "BPM: {} | Notes: {}",
                chart.metadata.bpm as u32,
                chart.note_count()
            ),
            info_x,
            info_y + 50.0,
            16.0,
            GRAY,
        );
    }
}

impl Default for Highway {
    fn default() -> Self {
        Self::new()
    }
}
