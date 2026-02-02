use anyhow::Result;
use macroquad::prelude::*;
use std::time::Instant;
use tracing::{error, info};

use crate::database::{ClearType, ScoreData, ScoreDatabaseAccessor, SongData};
use crate::input::{InputManager, KeyInputLog};
use crate::replay::{ReplayRecorder, ReplaySlot, find_empty_slot, save_replay};
use crate::state::play::{GaugeType, PlayResult, Rank};

/// Actions available on the result screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultAction {
    ReturnToSelect,
    Replay,
}

impl ResultAction {
    fn next(self) -> Self {
        match self {
            Self::ReturnToSelect => Self::Replay,
            Self::Replay => Self::ReturnToSelect,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ReturnToSelect => "RETURN TO SELECT",
            Self::Replay => "REPLAY",
        }
    }
}

/// Transition from the result screen.
#[derive(Debug, Clone)]
pub enum ResultTransition {
    None,
    Select,
    Replay(Box<SongData>),
}

/// Result screen state.
pub struct ResultState {
    play_result: PlayResult,
    song_data: SongData,
    is_new_record: bool,
    clear_type: ClearType,
    selected_action: ResultAction,
    input_manager: InputManager,
    transition: ResultTransition,
    start_time: Instant,
}

impl ResultState {
    /// Create a new result state.
    pub fn new(
        play_result: PlayResult,
        song_data: SongData,
        input_manager: InputManager,
        input_logs: Option<Vec<KeyInputLog>>,
        hi_speed: f32,
        score_db: &ScoreDatabaseAccessor,
    ) -> Self {
        let clear_type = Self::determine_clear_type(&play_result);
        let is_new_record = Self::save_score(score_db, &play_result, &song_data, clear_type);

        // Save replay if input logs are available
        if let Some(logs) = input_logs {
            Self::save_replay_data(&song_data, &play_result, clear_type, logs, hi_speed);
        }

        Self {
            play_result,
            song_data,
            is_new_record,
            clear_type,
            selected_action: ResultAction::ReturnToSelect,
            input_manager,
            transition: ResultTransition::None,
            start_time: Instant::now(),
        }
    }

    /// Update the result state.
    pub fn update(&mut self) -> Result<()> {
        self.input_manager.update();

        // Navigation
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Down) {
            self.selected_action = self.selected_action.next();
        }

        // Confirm action
        if is_key_pressed(KeyCode::Enter) {
            match self.selected_action {
                ResultAction::ReturnToSelect => {
                    self.transition = ResultTransition::Select;
                }
                ResultAction::Replay => {
                    self.transition = ResultTransition::Replay(Box::new(self.song_data.clone()));
                }
            }
        }

        // Quick return to select
        if is_key_pressed(KeyCode::Escape) {
            self.transition = ResultTransition::Select;
        }

        Ok(())
    }

    /// Draw the result screen.
    pub fn draw(&self) {
        let x = 100.0;
        let mut y = 100.0;

        // Title
        draw_text("=== RESULT ===", x, y, 48.0, WHITE);
        y += 60.0;

        // Song info
        draw_text(
            &format!("Title: {}", self.song_data.title),
            x,
            y,
            24.0,
            WHITE,
        );
        y += 30.0;
        draw_text(
            &format!("Artist: {}", self.song_data.artist),
            x,
            y,
            20.0,
            GRAY,
        );
        y += 50.0;

        // Clear status
        let (clear_text, clear_color) = self.clear_text_and_color();
        draw_text(clear_text, x, y, 36.0, clear_color);
        y += 50.0;

        // Rank
        let rank = self.play_result.rank();
        draw_text(
            &format!("Rank: {}", rank.as_str()),
            x,
            y,
            32.0,
            self.rank_color(rank),
        );
        y += 50.0;

        // Score section
        draw_text(
            &format!("EX-SCORE: {}", self.play_result.ex_score()),
            x,
            y,
            28.0,
            YELLOW,
        );
        y += 35.0;
        draw_text(
            &format!("Score Rate: {:.2}%", self.play_result.score.clear_rate()),
            x,
            y,
            24.0,
            WHITE,
        );
        y += 35.0;
        draw_text(
            &format!("MAX COMBO: {}", self.play_result.max_combo()),
            x,
            y,
            24.0,
            ORANGE,
        );
        y += 35.0;
        draw_text(
            &format!("BP: {}", self.play_result.bp()),
            x,
            y,
            24.0,
            Color::new(0.7, 0.7, 1.0, 1.0),
        );
        y += 50.0;

        // Judge counts
        self.draw_judge_counts(x, y);
        y += 80.0;

        // New record indicator
        if self.is_new_record {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let alpha = 0.5 + 0.5 * (elapsed * 4.0).sin();
            draw_text(
                "NEW RECORD!",
                x + 400.0,
                200.0,
                40.0,
                Color::new(1.0, 0.8, 0.0, alpha),
            );
        }

        // Fast/Slow
        draw_text(
            &format!(
                "FAST: {} / SLOW: {}",
                self.play_result.fast_count, self.play_result.slow_count
            ),
            x,
            y,
            18.0,
            GRAY,
        );
        y += 60.0;

        // Action selection
        self.draw_action_selection(x, y);
    }

    fn draw_judge_counts(&self, x: f32, y: f32) {
        let score = &self.play_result.score;

        draw_text(
            &format!("PG: {}", score.pg_count),
            x,
            y,
            20.0,
            Color::new(0.0, 1.0, 1.0, 1.0),
        );
        draw_text(
            &format!("GR: {}", score.gr_count),
            x + 120.0,
            y,
            20.0,
            Color::new(1.0, 1.0, 0.0, 1.0),
        );
        draw_text(
            &format!("GD: {}", score.gd_count),
            x + 240.0,
            y,
            20.0,
            Color::new(0.0, 1.0, 0.0, 1.0),
        );

        draw_text(
            &format!("BD: {}", score.bd_count),
            x,
            y + 25.0,
            20.0,
            Color::new(0.5, 0.5, 1.0, 1.0),
        );
        draw_text(
            &format!("PR: {}", score.pr_count),
            x + 120.0,
            y + 25.0,
            20.0,
            GRAY,
        );
        draw_text(
            &format!("MS: {}", score.ms_count),
            x + 240.0,
            y + 25.0,
            20.0,
            Color::new(1.0, 0.3, 0.3, 1.0),
        );
    }

    fn draw_action_selection(&self, x: f32, y: f32) {
        draw_text("Select Action:", x, y, 20.0, GRAY);

        let actions = [ResultAction::ReturnToSelect, ResultAction::Replay];

        for (i, action) in actions.iter().enumerate() {
            let action_y = y + 30.0 + i as f32 * 30.0;
            let is_selected = *action == self.selected_action;
            let color = if is_selected { YELLOW } else { WHITE };
            let prefix = if is_selected { "> " } else { "  " };

            draw_text(
                &format!("{}{}", prefix, action.as_str()),
                x,
                action_y,
                24.0,
                color,
            );
        }

        draw_text(
            "Enter: Confirm / Escape: Return to Select",
            x,
            y + 100.0,
            16.0,
            DARKGRAY,
        );
    }

    fn clear_text_and_color(&self) -> (&'static str, Color) {
        match self.clear_type {
            ClearType::Failed => ("FAILED", Color::new(0.8, 0.2, 0.2, 1.0)),
            ClearType::AssistEasy | ClearType::LightAssistEasy => {
                ("ASSIST CLEAR", Color::new(0.6, 0.3, 0.8, 1.0))
            }
            ClearType::Easy => ("EASY CLEAR", Color::new(0.3, 0.8, 0.3, 1.0)),
            ClearType::Normal => ("CLEAR", Color::new(0.3, 0.5, 1.0, 1.0)),
            ClearType::Hard => ("HARD CLEAR", Color::new(1.0, 0.5, 0.3, 1.0)),
            ClearType::ExHard => ("EX-HARD CLEAR", Color::new(1.0, 0.8, 0.0, 1.0)),
            ClearType::FullCombo => ("FULL COMBO", Color::new(0.0, 1.0, 1.0, 1.0)),
            ClearType::Perfect => ("PERFECT", Color::new(1.0, 0.9, 0.5, 1.0)),
            ClearType::Max => ("MAX", Color::new(1.0, 1.0, 1.0, 1.0)),
            ClearType::NoPlay => ("NO PLAY", GRAY),
        }
    }

    fn rank_color(&self, rank: Rank) -> Color {
        match rank {
            Rank::Max => WHITE,
            Rank::AAA => Color::new(1.0, 0.8, 0.0, 1.0),
            Rank::AA => Color::new(1.0, 0.6, 0.0, 1.0),
            Rank::A => Color::new(0.3, 0.8, 0.3, 1.0),
            Rank::B => Color::new(0.3, 0.5, 1.0, 1.0),
            Rank::C => Color::new(0.6, 0.3, 0.8, 1.0),
            Rank::D => Color::new(0.8, 0.4, 0.4, 1.0),
            Rank::E => Color::new(0.6, 0.3, 0.3, 1.0),
            Rank::F => Color::new(0.4, 0.2, 0.2, 1.0),
        }
    }

    /// Take the current transition.
    pub fn take_transition(&mut self) -> ResultTransition {
        std::mem::replace(&mut self.transition, ResultTransition::None)
    }

    /// Take the input manager for reuse.
    pub fn take_input_manager(&mut self) -> InputManager {
        let key_config = self.input_manager.key_config().clone();
        let dummy = InputManager::new(key_config).unwrap();
        std::mem::replace(&mut self.input_manager, dummy)
    }

    fn determine_clear_type(play_result: &PlayResult) -> ClearType {
        if !play_result.is_clear {
            return ClearType::Failed;
        }

        // Check for full combo / perfect
        let score = &play_result.score;
        if score.bp() == 0 {
            if score.gr_count == 0 && score.gd_count == 0 {
                return ClearType::Max;
            } else if score.gd_count == 0 {
                return ClearType::Perfect;
            } else {
                return ClearType::FullCombo;
            }
        }

        // Based on gauge type
        match play_result.gauge_type {
            GaugeType::AssistEasy => ClearType::AssistEasy,
            GaugeType::Easy => ClearType::Easy,
            GaugeType::Normal => ClearType::Normal,
            GaugeType::Hard | GaugeType::Class => ClearType::Hard,
            GaugeType::ExHard | GaugeType::Hazard => ClearType::ExHard,
        }
    }

    fn save_replay_data(
        song_data: &SongData,
        play_result: &PlayResult,
        clear_type: ClearType,
        logs: Vec<KeyInputLog>,
        hi_speed: f32,
    ) {
        let gauge_type = match play_result.gauge_type {
            GaugeType::AssistEasy => 0,
            GaugeType::Easy => 1,
            GaugeType::Normal => 2,
            GaugeType::Hard => 3,
            GaugeType::ExHard => 4,
            GaugeType::Hazard => 5,
            GaugeType::Class => 6,
        };

        let mut recorder = ReplayRecorder::new(
            song_data.sha256.clone(),
            "Player".to_string(), // TODO: Get player name from config
            gauge_type,
            hi_speed,
        );
        recorder.set_input_logs(logs);
        recorder.set_score(play_result, clear_type);

        // Find empty slot or overwrite slot 0
        let slot = find_empty_slot(&song_data.sha256).unwrap_or(ReplaySlot::SLOT_0);
        let replay_data = recorder.into_replay_data();

        match save_replay(&replay_data, slot) {
            Ok(path) => {
                info!("Replay saved to: {}", path.display());
            }
            Err(e) => {
                error!("Failed to save replay: {}", e);
            }
        }
    }

    fn save_score(
        score_db: &ScoreDatabaseAccessor,
        play_result: &PlayResult,
        song_data: &SongData,
        clear_type: ClearType,
    ) -> bool {
        // Check if this is a new EX-SCORE record
        let existing = score_db.get_score(&song_data.sha256, 0).ok().flatten();
        let is_new_record = existing
            .as_ref()
            .is_none_or(|old| play_result.ex_score() as i32 > old.ex_score);

        let score_data = ScoreData {
            sha256: song_data.sha256.clone(),
            mode: 0, // Default LN mode
            clear: clear_type,
            ex_score: play_result.ex_score() as i32,
            max_combo: play_result.max_combo() as i32,
            min_bp: play_result.bp() as i32,
            pg: play_result.score.pg_count as i32,
            gr: play_result.score.gr_count as i32,
            gd: play_result.score.gd_count as i32,
            bd: play_result.score.bd_count as i32,
            pr: play_result.score.pr_count as i32,
            ms: play_result.score.ms_count as i32,
            notes: play_result.score.total_notes() as i32,
            play_count: 0, // Will be updated by save_score
            clear_count: 0,
            date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };

        if let Err(e) = score_db.save_score(&score_data) {
            error!("Failed to save score: {}", e);
        }

        is_new_record
    }
}
