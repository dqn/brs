use std::path::Path;
use std::sync::{Arc, Mutex};

use macroquad::prelude::*;

use crate::config::GameSettings;
use crate::database::{SavedScore, ScoreRepository, compute_file_hash};
use crate::game::{ClearLamp, PlayResult, RandomOption};
use crate::ir::{
    IrClient, IrSubmitState, ScoreHashData, ScoreSubmission, compute_md5_hash, generate_score_hash,
    validate_score,
};
use crate::render::font::draw_text_jp;
use crate::render::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

use super::{Scene, SceneTransition};

pub struct ResultScene {
    result: PlayResult,
    is_new_record: bool,
    ir_state: Arc<Mutex<IrSubmitState>>,
    ir_rank: Arc<Mutex<Option<u32>>>,
    ir_total_players: Arc<Mutex<Option<u32>>>,
    ir_message: Arc<Mutex<Option<String>>>,
}

impl ResultScene {
    pub fn new(result: PlayResult) -> Self {
        let is_new_record = Self::save_score(&result);

        let ir_state = Arc::new(Mutex::new(IrSubmitState::Idle));
        let ir_rank = Arc::new(Mutex::new(None));
        let ir_total_players = Arc::new(Mutex::new(None));
        let ir_message = Arc::new(Mutex::new(None));

        // Start IR submission if configured
        let settings = GameSettings::load();
        if settings.ir.is_configured() && settings.ir.auto_submit {
            // Check if assist options are used
            let has_assist = result.play_options.has_assist();
            if !has_assist || settings.ir.submit_with_assist {
                Self::start_ir_submission(
                    &result,
                    &settings,
                    Arc::clone(&ir_state),
                    Arc::clone(&ir_rank),
                    Arc::clone(&ir_total_players),
                    Arc::clone(&ir_message),
                );
            } else {
                if let Ok(mut state) = ir_state.lock() {
                    *state = IrSubmitState::Disabled;
                }
                if let Ok(mut msg) = ir_message.lock() {
                    *msg = Some("Assist option used".to_string());
                }
            }
        } else if !settings.ir.enabled {
            if let Ok(mut state) = ir_state.lock() {
                *state = IrSubmitState::Disabled;
            }
        }

        Self {
            result,
            is_new_record,
            ir_state,
            ir_rank,
            ir_total_players,
            ir_message,
        }
    }

    fn start_ir_submission(
        result: &PlayResult,
        settings: &GameSettings,
        ir_state: Arc<Mutex<IrSubmitState>>,
        ir_rank: Arc<Mutex<Option<u32>>>,
        ir_total_players: Arc<Mutex<Option<u32>>>,
        ir_message: Arc<Mutex<Option<String>>>,
    ) {
        if let Ok(mut state) = ir_state.lock() {
            *state = IrSubmitState::Submitting;
        }

        // Compute MD5 hash for LR2IR compatibility
        let chart_path = result.chart_path.clone();
        let chart_md5 = match compute_md5_hash(Path::new(&chart_path)) {
            Ok(hash) => hash,
            Err(e) => {
                if let Ok(mut state) = ir_state.lock() {
                    *state = IrSubmitState::Failed;
                }
                if let Ok(mut msg) = ir_message.lock() {
                    *msg = Some(format!("Hash error: {}", e));
                }
                return;
            }
        };

        // Compute SHA256 for internal use
        let chart_hash = match compute_file_hash(Path::new(&chart_path)) {
            Ok(hash) => hash,
            Err(e) => {
                if let Ok(mut state) = ir_state.lock() {
                    *state = IrSubmitState::Failed;
                }
                if let Ok(mut msg) = ir_message.lock() {
                    *msg = Some(format!("Hash error: {}", e));
                }
                return;
            }
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Generate score hash
        let score_hash = match generate_score_hash(&ScoreHashData {
            chart_md5: &chart_md5,
            ex_score: result.ex_score,
            clear_lamp: result.clear_lamp.as_u8(),
            max_combo: result.max_combo,
            pgreat_count: result.pgreat_count,
            great_count: result.great_count,
            good_count: result.good_count,
            bad_count: result.bad_count,
            poor_count: result.poor_count,
            timestamp,
            secret_key: &settings.ir.secret_key,
        }) {
            Ok(hash) => hash,
            Err(e) => {
                if let Ok(mut state) = ir_state.lock() {
                    *state = IrSubmitState::Failed;
                }
                if let Ok(mut msg) = ir_message.lock() {
                    *msg = Some(format!("Score hash error: {}", e));
                }
                return;
            }
        };

        let submission = ScoreSubmission {
            player_id: settings.ir.player_id.clone(),
            chart_hash,
            chart_md5,
            ex_score: result.ex_score,
            clear_lamp: result.clear_lamp,
            max_combo: result.max_combo,
            pgreat_count: result.pgreat_count,
            great_count: result.great_count,
            good_count: result.good_count,
            bad_count: result.bad_count,
            poor_count: result.poor_count,
            total_notes: result.total_notes,
            play_option: result.play_options.clone(),
            timestamp,
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            score_hash,
        };

        // Validate score before submission
        let validation = validate_score(&submission);
        if !validation.is_valid {
            if let Ok(mut state) = ir_state.lock() {
                *state = IrSubmitState::Failed;
            }
            if let Ok(mut msg) = ir_message.lock() {
                *msg = Some(format!("Invalid score: {:?}", validation.errors));
            }
            return;
        }

        // Spawn async task for IR submission
        let base_url = settings.ir.effective_url();
        let player_id = settings.ir.player_id.clone();
        let secret_key = settings.ir.secret_key.clone();
        let server_type = settings.ir.server_type;

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Failed to create tokio runtime for IR submission: {}", e);
                    if let Ok(mut state) = ir_state.lock() {
                        *state = IrSubmitState::Failed;
                    }
                    if let Ok(mut msg) = ir_message.lock() {
                        *msg = Some(format!("Runtime error: {}", e));
                    }
                    return;
                }
            };
            rt.block_on(async {
                let client = match IrClient::new(base_url, player_id, secret_key, server_type) {
                    Ok(c) => c,
                    Err(e) => {
                        if let Ok(mut state) = ir_state.lock() {
                            *state = IrSubmitState::Failed;
                        }
                        if let Ok(mut msg) = ir_message.lock() {
                            *msg = Some(format!("Client error: {}", e));
                        }
                        return;
                    }
                };

                match client.submit_score(submission).await {
                    Ok(response) => {
                        if response.success {
                            if let Ok(mut state) = ir_state.lock() {
                                *state = IrSubmitState::Success;
                            }
                            if let Ok(mut rank) = ir_rank.lock() {
                                *rank = response.rank;
                            }
                            if let Ok(mut total) = ir_total_players.lock() {
                                *total = response.total_players;
                            }
                        } else if let Ok(mut state) = ir_state.lock() {
                            *state = IrSubmitState::Failed;
                        }
                        if let Ok(mut msg) = ir_message.lock() {
                            *msg = response.message;
                        }
                    }
                    Err(e) => {
                        if let Ok(mut state) = ir_state.lock() {
                            *state = IrSubmitState::Failed;
                        }
                        if let Ok(mut msg) = ir_message.lock() {
                            *msg = Some(format!("Submit error: {}", e));
                        }
                    }
                }
            });
        });
    }

    fn retry_ir_submission(&self) {
        let settings = GameSettings::load();
        if settings.ir.is_configured() {
            Self::start_ir_submission(
                &self.result,
                &settings,
                Arc::clone(&self.ir_state),
                Arc::clone(&self.ir_rank),
                Arc::clone(&self.ir_total_players),
                Arc::clone(&self.ir_message),
            );
        }
    }

    fn save_score(result: &PlayResult) -> bool {
        // Compute hash from chart file
        let hash = match compute_file_hash(Path::new(&result.chart_path)) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to compute chart hash: {}", e);
                return false;
            }
        };

        // Load repository and save score
        let mut repo = match ScoreRepository::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to load score repository: {}", e);
                return false;
            }
        };

        let new_score = SavedScore::from_play_result(hash.clone(), result);
        let is_new_record = repo.update(&hash, new_score);

        if let Err(e) = repo.save() {
            eprintln!("Failed to save scores: {}", e);
        }

        is_new_record
    }

    fn clear_lamp_color(&self) -> Color {
        match self.result.clear_lamp {
            ClearLamp::NoPlay => GRAY,
            ClearLamp::Failed => Color::new(0.5, 0.0, 0.0, 1.0),
            ClearLamp::AssistEasy => Color::new(0.6, 0.3, 0.8, 1.0),
            ClearLamp::Easy => Color::new(0.0, 0.8, 0.3, 1.0),
            ClearLamp::Normal => Color::new(0.2, 0.6, 1.0, 1.0),
            ClearLamp::Hard => Color::new(1.0, 0.5, 0.0, 1.0),
            ClearLamp::ExHard => Color::new(1.0, 0.8, 0.0, 1.0),
            ClearLamp::FullCombo => Color::new(1.0, 0.0, 0.5, 1.0),
        }
    }
}

impl Scene for ResultScene {
    fn update(&mut self) -> SceneTransition {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            return SceneTransition::Pop;
        }

        // R key for IR retry
        if is_key_pressed(KeyCode::R) {
            let state = self.ir_state.lock().ok().map(|s| *s);
            if state == Some(IrSubmitState::Failed) {
                self.retry_ir_submission();
            }
        }

        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));

        let center_x = VIRTUAL_WIDTH / 2.0;

        draw_text_jp("RESULT", center_x - 60.0, 50.0, 40.0, WHITE);

        // Clear lamp display
        let lamp_text = self.result.clear_lamp.display_name();
        let lamp_color = self.clear_lamp_color();
        draw_text_jp(lamp_text, center_x - 100.0, 80.0, 24.0, lamp_color);

        // New record indicator
        if self.is_new_record {
            draw_text_jp("NEW RECORD!", center_x + 50.0, 80.0, 20.0, GOLD);
        }

        draw_text_jp(&self.result.title, center_x - 150.0, 120.0, 28.0, YELLOW);
        draw_text_jp(&self.result.artist, center_x - 150.0, 150.0, 20.0, GRAY);

        let rank = self.result.rank();
        let rank_color = match rank {
            "MAX" => Color::new(1.0, 0.8, 0.0, 1.0),
            "AAA" => Color::new(1.0, 0.6, 0.0, 1.0),
            "AA" => Color::new(0.8, 0.8, 0.0, 1.0),
            "A" => Color::new(0.0, 1.0, 0.5, 1.0),
            "B" => Color::new(0.0, 0.8, 1.0, 1.0),
            _ => WHITE,
        };

        draw_text_jp(rank, center_x - 30.0, 220.0, 80.0, rank_color);
        draw_text_jp(
            &format!("{:.2}%", self.result.accuracy()),
            center_x - 60.0,
            270.0,
            32.0,
            WHITE,
        );

        let stats_x = center_x - 120.0;
        let stats_start_y = 320.0;
        let line_height = 30.0;

        draw_text_jp(
            &format!("EX SCORE: {}", self.result.ex_score),
            stats_x,
            stats_start_y,
            24.0,
            YELLOW,
        );
        draw_text_jp(
            &format!("MAX COMBO: {}", self.result.max_combo),
            stats_x,
            stats_start_y + line_height,
            24.0,
            WHITE,
        );

        draw_text_jp(
            &format!("PGREAT: {}", self.result.pgreat_count),
            stats_x,
            stats_start_y + line_height * 3.0,
            20.0,
            Color::new(1.0, 1.0, 0.0, 1.0),
        );
        draw_text_jp(
            &format!("GREAT: {}", self.result.great_count),
            stats_x,
            stats_start_y + line_height * 4.0,
            20.0,
            Color::new(1.0, 0.8, 0.0, 1.0),
        );
        draw_text_jp(
            &format!("GOOD: {}", self.result.good_count),
            stats_x,
            stats_start_y + line_height * 5.0,
            20.0,
            Color::new(0.0, 1.0, 0.5, 1.0),
        );
        draw_text_jp(
            &format!("BAD: {}", self.result.bad_count),
            stats_x,
            stats_start_y + line_height * 6.0,
            20.0,
            Color::new(0.5, 0.5, 1.0, 1.0),
        );
        draw_text_jp(
            &format!("POOR: {}", self.result.poor_count),
            stats_x,
            stats_start_y + line_height * 7.0,
            20.0,
            Color::new(1.0, 0.3, 0.3, 1.0),
        );

        // FAST/SLOW statistics
        draw_text_jp(
            &format!(
                "FAST:{} / SLOW:{}",
                self.result.fast_count, self.result.slow_count
            ),
            stats_x,
            stats_start_y + line_height * 9.0,
            18.0,
            GRAY,
        );

        // Random option display
        if self.result.random_option != RandomOption::Off {
            draw_text_jp(
                &format!("OPTION: {}", self.result.random_option.display_name()),
                stats_x,
                stats_start_y + line_height * 10.0,
                18.0,
                SKYBLUE,
            );
        }

        // IR status display
        let ir_state = self
            .ir_state
            .lock()
            .ok()
            .map(|s| *s)
            .unwrap_or(IrSubmitState::Idle);
        let ir_color = match ir_state {
            IrSubmitState::Idle => GRAY,
            IrSubmitState::Submitting => YELLOW,
            IrSubmitState::Success => GREEN,
            IrSubmitState::Failed => RED,
            IrSubmitState::Disabled => DARKGRAY,
        };

        let ir_text = ir_state.display_text();
        if !ir_text.is_empty() {
            draw_text_jp(ir_text, VIRTUAL_WIDTH - 200.0, 30.0, 18.0, ir_color);
        }

        // Show IR rank if available
        if ir_state == IrSubmitState::Success {
            if let Some(rank) = self.ir_rank.lock().ok().and_then(|r| *r) {
                let total = self
                    .ir_total_players
                    .lock()
                    .ok()
                    .and_then(|t| *t)
                    .unwrap_or(0);
                draw_text_jp(
                    &format!("IR Rank: #{}/{}", rank, total),
                    VIRTUAL_WIDTH - 200.0,
                    50.0,
                    16.0,
                    GREEN,
                );
            }
        }

        // Show retry hint if failed
        if ir_state == IrSubmitState::Failed {
            draw_text_jp("[R] Retry IR", VIRTUAL_WIDTH - 200.0, 50.0, 14.0, GRAY);
        }

        draw_text_jp(
            "[Enter] Continue",
            center_x - 80.0,
            VIRTUAL_HEIGHT - 30.0,
            20.0,
            GRAY,
        );
    }
}
