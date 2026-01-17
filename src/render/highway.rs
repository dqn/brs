use macroquad::prelude::*;

use super::config::HighwayConfig;
use super::font::draw_text_jp;
use super::lane_cover::LaneCover;
use super::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::bms::{Chart, Note, NoteType, PlayMode};
use crate::game::GamePlayState;

pub struct Highway {
    config: HighwayConfig,
    lane_cover: LaneCover,
}

impl Highway {
    pub fn new() -> Self {
        Self::with_config(HighwayConfig::default())
    }

    /// Create Highway for a specific play mode
    #[allow(dead_code)]
    pub fn for_mode(mode: PlayMode) -> Self {
        Self::with_config(HighwayConfig::for_mode(mode))
    }

    pub fn with_config(config: HighwayConfig) -> Self {
        Self {
            config,
            lane_cover: LaneCover::default(),
        }
    }

    /// Get current play mode
    #[allow(dead_code)]
    pub fn play_mode(&self) -> PlayMode {
        self.config.play_mode
    }

    /// Set play mode (updates config accordingly)
    #[allow(dead_code)]
    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.config = HighwayConfig::for_mode(mode);
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

    #[allow(dead_code)]
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

    /// Draw highway within a specified rect (for IIDX-style layout)
    #[allow(clippy::too_many_arguments)]
    pub fn draw_in_rect(
        &self,
        rect: &crate::skin::Rect,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        scroll_speed: f32,
        measure_times: &[f64],
    ) {
        // Calculate scale factor based on rect width vs original total width
        let original_width = self.config.total_width();
        let scale = rect.width / original_width;

        // Calculate judge line Y position within the rect
        // LIFT=0: judge line at bottom, LIFT>0: offset upward
        let lift_ratio = self.lane_cover.lift as f32 / 1000.0;
        let judge_y = rect.y + rect.height * (1.0 - lift_ratio);
        let pixels_per_ms = scroll_speed as f64 * 0.5;

        self.draw_lanes_in_rect(rect, scale);
        self.draw_measure_lines_in_rect(
            rect,
            scale,
            judge_y,
            current_time_ms,
            pixels_per_ms,
            measure_times,
        );
        self.draw_notes_in_rect(
            rect,
            scale,
            judge_y,
            chart,
            play_state,
            current_time_ms,
            scroll_speed,
        );
        self.draw_lane_covers_in_rect(rect, scale, judge_y);
        self.draw_judge_line_in_rect(rect, scale, judge_y);
    }

    fn draw_lanes_in_rect(&self, rect: &crate::skin::Rect, scale: f32) {
        let lane_count = self.config.lane_count();
        let background_color = self.config.background_color();
        let border_color = self.config.border_color();

        for i in 0..lane_count {
            let x = rect.x + self.config.lane_x_offset(i) * scale;
            let width = self.config.lane_width_for_lane(i) * scale;
            draw_rectangle(x, rect.y, width, rect.height, background_color);
            draw_line(x, rect.y, x, rect.y + rect.height, 1.0, border_color);
        }

        // Draw right edge of last lane
        let last_lane = lane_count - 1;
        let last_x = rect.x
            + (self.config.lane_x_offset(last_lane) + self.config.lane_width_for_lane(last_lane))
                * scale;
        draw_line(
            last_x,
            rect.y,
            last_x,
            rect.y + rect.height,
            1.0,
            border_color,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_notes_in_rect(
        &self,
        rect: &crate::skin::Rect,
        scale: f32,
        judge_y: f32,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        scroll_speed: f32,
    ) {
        let pixels_per_ms = scroll_speed as f64 * 0.5;

        // Draw long note bars first
        self.draw_long_note_bars_in_rect(
            rect,
            scale,
            judge_y,
            chart,
            play_state,
            current_time_ms,
            pixels_per_ms,
        );

        // Then draw notes
        for (i, note) in chart.notes.iter().enumerate() {
            if !play_state.get_state(i).is_some_and(|s| s.is_pending()) {
                continue;
            }

            let time_diff = note.time_ms - current_time_ms;

            if !(-100.0..=self.config.visible_range_ms).contains(&time_diff) {
                continue;
            }

            self.draw_note_in_rect(rect, scale, judge_y, note, time_diff, pixels_per_ms);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_long_note_bars_in_rect(
        &self,
        rect: &crate::skin::Rect,
        scale: f32,
        judge_y: f32,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        pixels_per_ms: f64,
    ) {
        let play_mode = self.config.play_mode;

        for (i, note) in chart.notes.iter().enumerate() {
            if note.note_type != NoteType::LongStart {
                continue;
            }

            let end_note_info = chart
                .notes
                .iter()
                .enumerate()
                .skip(i + 1)
                .find(|(_, n)| n.channel == note.channel && n.note_type == NoteType::LongEnd);

            let Some((end_idx, end)) = end_note_info else {
                continue;
            };

            if !play_state
                .get_state(end_idx)
                .is_some_and(|s| s.is_pending())
            {
                continue;
            }

            let start_time_diff = note.time_ms - current_time_ms;
            let end_time_diff = end.time_ms - current_time_ms;

            let start_y = judge_y - (start_time_diff * pixels_per_ms) as f32;
            let end_y = judge_y - (end_time_diff * pixels_per_ms) as f32;

            if end_y > judge_y && start_y > judge_y {
                continue;
            }

            let clamped_start_y = start_y.min(judge_y);

            let lane = note.channel.lane_index_for_mode(play_mode);
            let x = rect.x + self.config.lane_x_offset(lane) * scale;
            let width = self.config.lane_width_for_lane(lane) * scale;

            let bar_height = clamped_start_y - end_y;
            if bar_height > 0.0 {
                let lane_color = self.config.lane_color(lane);
                draw_rectangle(
                    x + 4.0 * scale,
                    end_y,
                    width - 8.0 * scale,
                    bar_height,
                    lane_color,
                );
            }
        }
    }

    fn draw_note_in_rect(
        &self,
        rect: &crate::skin::Rect,
        scale: f32,
        judge_y: f32,
        note: &Note,
        time_diff: f64,
        pixels_per_ms: f64,
    ) {
        let y = judge_y - (time_diff * pixels_per_ms) as f32;

        if y > judge_y {
            return;
        }

        let lane = note.channel.lane_index_for_mode(self.config.play_mode);
        let x = rect.x + self.config.lane_x_offset(lane) * scale;
        let width = self.config.lane_width_for_lane(lane) * scale;
        let note_height = self.config.note_height * scale.min(1.0);

        let color = match note.note_type {
            NoteType::Normal | NoteType::LongStart | NoteType::LongEnd => {
                self.config.lane_color(lane)
            }
            NoteType::Invisible => self.config.invisible_note_color(),
            NoteType::Landmine => self.config.landmine_note_color(),
        };

        draw_rectangle(
            x + 2.0 * scale,
            y - note_height / 2.0,
            width - 4.0 * scale,
            note_height,
            color,
        );
    }

    fn draw_judge_line_in_rect(&self, rect: &crate::skin::Rect, scale: f32, judge_y: f32) {
        let highway_width = self.config.total_width() * scale;
        draw_line(
            rect.x,
            judge_y,
            rect.x + highway_width,
            judge_y,
            self.config.judge_line_thickness(),
            self.config.judge_line_color(),
        );
    }

    /// Draw measure lines within a specified rect
    fn draw_measure_lines_in_rect(
        &self,
        rect: &crate::skin::Rect,
        scale: f32,
        judge_y: f32,
        current_time_ms: f64,
        pixels_per_ms: f64,
        measure_times: &[f64],
    ) {
        let highway_width = self.config.total_width() * scale;
        let measure_line_color = Color::new(0.5, 0.5, 0.5, 0.6);

        for &time_ms in measure_times.iter() {
            let time_diff = time_ms - current_time_ms;

            // Skip if not in visible range
            if !(-100.0..=self.config.visible_range_ms).contains(&time_diff) {
                continue;
            }

            let y = judge_y - (time_diff * pixels_per_ms) as f32;

            // Skip if below judge line or above rect
            if y > judge_y || y < rect.y {
                continue;
            }

            // Draw the measure line
            draw_line(
                rect.x,
                y,
                rect.x + highway_width,
                y,
                1.0,
                measure_line_color,
            );
        }
    }

    fn draw_lane_covers_in_rect(&self, rect: &crate::skin::Rect, scale: f32, judge_y: f32) {
        let highway_width = self.config.total_width() * scale;
        let cover_color = self.config.lane_cover_color();
        let text_color = self.config.lane_cover_text_color();

        // SUDDEN+ cover - use rect.height to be independent of LIFT
        if self.lane_cover.sudden > 0 {
            let cover_height = (self.lane_cover.sudden as f32 / 1000.0) * rect.height;
            draw_rectangle(rect.x, rect.y, highway_width, cover_height, cover_color);
            draw_text_jp(
                &format!("SUD+ {}", self.lane_cover.sudden),
                rect.x + 10.0,
                rect.y + cover_height - 10.0,
                16.0,
                text_color,
            );
        }

        // LIFT cover - draw from judge line to bottom
        if self.lane_cover.lift > 0 {
            let lift_color = self.config.lift_cover_color();
            draw_rectangle(
                rect.x,
                judge_y,
                highway_width,
                rect.y + rect.height - judge_y,
                lift_color,
            );
            draw_text_jp(
                &format!("LIFT {}", self.lane_cover.lift),
                rect.x + 10.0,
                judge_y + 15.0,
                16.0,
                text_color,
            );
        }

        // HIDDEN+ cover
        if self.lane_cover.hidden > 0 {
            let below_judge_height = rect.y + rect.height - judge_y;
            let cover_height = (self.lane_cover.hidden as f32 / 1000.0) * below_judge_height;
            draw_rectangle(rect.x, judge_y, highway_width, cover_height, cover_color);
            draw_text_jp(
                &format!("HID+ {}", self.lane_cover.hidden),
                rect.x + 10.0,
                judge_y + 15.0,
                16.0,
                text_color,
            );
        }
    }

    /// Get highway X position
    pub fn highway_x(&self) -> f32 {
        let total_width = self.config.total_width();
        (VIRTUAL_WIDTH - total_width) / 2.0
    }

    /// Get lane widths for all lanes
    pub fn get_lane_widths(&self) -> Vec<f32> {
        let lane_count = self.config.lane_count();
        (0..lane_count)
            .map(|i| self.config.lane_width_for_lane(i))
            .collect()
    }

    /// Get judge line Y position
    #[allow(dead_code)]
    pub fn judge_line_y(&self) -> f32 {
        self.config.judge_line_y
    }

    /// Get lane colors for all lanes
    pub fn get_lane_colors(&self) -> Vec<Color> {
        let lane_count = self.config.lane_count();
        (0..lane_count).map(|i| self.config.lane_color(i)).collect()
    }

    /// Get total highway width
    pub fn total_width(&self) -> f32 {
        self.config.total_width()
    }

    /// Calculate judge line Y position within a rect, accounting for LIFT
    pub fn judge_y_in_rect(&self, rect: &crate::skin::Rect) -> f32 {
        let lift_ratio = self.lane_cover.lift as f32 / 1000.0;
        rect.y + rect.height * (1.0 - lift_ratio)
    }

    /// Get judge line Y position adjusted for LIFT
    fn adjusted_judge_line_y(&self) -> f32 {
        let lift_offset = self.lane_cover.judge_line_position() * self.config.judge_line_y;
        self.config.judge_line_y - lift_offset
    }

    fn draw_lanes(&self, highway_x: f32) {
        let lane_count = self.config.lane_count();
        let background_color = self.config.background_color();
        let border_color = self.config.border_color();

        for i in 0..lane_count {
            let x = highway_x + self.config.lane_x_offset(i);
            let width = self.config.lane_width_for_lane(i);
            draw_rectangle(x, 0.0, width, VIRTUAL_HEIGHT, background_color);
            draw_line(x, 0.0, x, VIRTUAL_HEIGHT, 1.0, border_color);
        }
        // Draw right edge of last lane
        let last_lane = lane_count - 1;
        let last_x = highway_x
            + self.config.lane_x_offset(last_lane)
            + self.config.lane_width_for_lane(last_lane);
        draw_line(last_x, 0.0, last_x, VIRTUAL_HEIGHT, 1.0, border_color);

        // For DP mode, draw P1 right edge and P2 left edge
        if self.config.play_mode == PlayMode::Dp14Key {
            // P1 right edge (after lane 7)
            let p1_right_x =
                highway_x + self.config.lane_x_offset(7) + self.config.lane_width_for_lane(7);
            draw_line(
                p1_right_x,
                0.0,
                p1_right_x,
                VIRTUAL_HEIGHT,
                1.0,
                border_color,
            );
            // P2 left edge (before lane 8)
            let p2_left_x = highway_x + self.config.lane_x_offset(8);
            draw_line(p2_left_x, 0.0, p2_left_x, VIRTUAL_HEIGHT, 1.0, border_color);
        }
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

    #[allow(dead_code)]
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
        let play_mode = self.config.play_mode;

        for (i, note) in chart.notes.iter().enumerate() {
            if note.note_type != NoteType::LongStart {
                continue;
            }

            // Find corresponding LongEnd with its index
            let end_note_info = chart
                .notes
                .iter()
                .enumerate()
                .skip(i + 1)
                .find(|(_, n)| n.channel == note.channel && n.note_type == NoteType::LongEnd);

            let Some((end_idx, end)) = end_note_info else {
                continue;
            };

            // Skip if LongEnd is already judged (LN is complete)
            if let Some(state) = play_state {
                if !state.get_state(end_idx).is_some_and(|s| s.is_pending()) {
                    continue;
                }
            }

            let start_time_diff = note.time_ms - current_time_ms;
            let end_time_diff = end.time_ms - current_time_ms;

            let judge_y = self.adjusted_judge_line_y();
            let start_y = judge_y - (start_time_diff * pixels_per_ms) as f32;
            let end_y = judge_y - (end_time_diff * pixels_per_ms) as f32;

            // Skip if entire bar is below judge line
            if end_y > judge_y && start_y > judge_y {
                continue;
            }

            // Clamp start_y to judge_y (don't draw below judge line)
            let clamped_start_y = start_y.min(judge_y);

            let lane = note.channel.lane_index_for_mode(play_mode);
            let x = highway_x + self.config.lane_x_offset(lane);
            let width = self.config.lane_width_for_lane(lane);

            let bar_height = clamped_start_y - end_y;
            if bar_height > 0.0 {
                let lane_color = self.config.lane_color(lane);
                draw_rectangle(x + 4.0, end_y, width - 8.0, bar_height, lane_color);
            }
        }
    }

    fn draw_note(&self, note: &Note, time_diff: f64, pixels_per_ms: f64, highway_x: f32) {
        let judge_y = self.adjusted_judge_line_y();
        let y = judge_y - (time_diff * pixels_per_ms) as f32;

        // Skip notes below judge line
        if y > judge_y {
            return;
        }

        let lane = note.channel.lane_index_for_mode(self.config.play_mode);
        let x = highway_x + self.config.lane_x_offset(lane);
        let width = self.config.lane_width_for_lane(lane);

        let color = match note.note_type {
            NoteType::Normal | NoteType::LongStart | NoteType::LongEnd => {
                self.config.lane_color(lane)
            }
            NoteType::Invisible => self.config.invisible_note_color(),
            NoteType::Landmine => self.config.landmine_note_color(),
        };

        draw_rectangle(
            x + 2.0,
            y - self.config.note_height / 2.0,
            width - 4.0,
            self.config.note_height,
            color,
        );
    }

    fn draw_judge_line(&self, highway_x: f32) {
        let adjusted_judge_y = self.adjusted_judge_line_y();
        let highway_width = self.config.total_width();
        draw_line(
            highway_x,
            adjusted_judge_y,
            highway_x + highway_width,
            adjusted_judge_y,
            self.config.judge_line_thickness(),
            self.config.judge_line_color(),
        );
    }

    #[allow(dead_code)]
    fn draw_lane_covers(&self, highway_x: f32) {
        let highway_width = self.config.total_width();
        let lane_height = self.config.judge_line_y; // Lane goes from top to judge line
        let cover_color = self.config.lane_cover_color();
        let text_color = self.config.lane_cover_text_color();

        // Draw SUDDEN+ cover (top of lane)
        if self.lane_cover.sudden > 0 {
            let cover_height = (self.lane_cover.sudden as f32 / 1000.0) * lane_height;
            draw_rectangle(highway_x, 0.0, highway_width, cover_height, cover_color);

            // Draw white number display on cover
            draw_text_jp(
                &format!("SUD+ {}", self.lane_cover.sudden),
                highway_x + 10.0,
                cover_height - 10.0,
                18.0,
                text_color,
            );
        }

        // Draw LIFT cover (bottom, raises judge line visually, opaque)
        if self.lane_cover.lift > 0 {
            let lift_color = self.config.lift_cover_color();
            let cover_height = (self.lane_cover.lift as f32 / 1000.0) * lane_height;
            let cover_y = self.config.judge_line_y - cover_height;
            draw_rectangle(
                highway_x,
                cover_y,
                highway_width,
                cover_height + 50.0,
                lift_color,
            );

            draw_text_jp(
                &format!("LIFT {}", self.lane_cover.lift),
                highway_x + 10.0,
                cover_y + 20.0,
                18.0,
                text_color,
            );
        }

        // Draw HIDDEN+ cover (below judge line, covers notes after they pass)
        if self.lane_cover.hidden > 0 {
            let below_judge_height = VIRTUAL_HEIGHT - self.config.judge_line_y;
            let cover_height = (self.lane_cover.hidden as f32 / 1000.0) * below_judge_height;
            draw_rectangle(
                highway_x,
                self.config.judge_line_y,
                highway_width,
                cover_height,
                cover_color,
            );

            draw_text_jp(
                &format!("HID+ {}", self.lane_cover.hidden),
                highway_x + 10.0,
                self.config.judge_line_y + 20.0,
                18.0,
                text_color,
            );
        }
    }

    fn draw_info(&self, chart: &Chart) {
        let info_x = 10.0;
        let info_y = VIRTUAL_HEIGHT - 80.0;

        draw_text_jp(&chart.metadata.title, info_x, info_y, 24.0, WHITE);
        draw_text_jp(&chart.metadata.artist, info_x, info_y + 25.0, 18.0, GRAY);
        draw_text_jp(
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
