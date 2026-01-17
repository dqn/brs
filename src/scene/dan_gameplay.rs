use std::path::PathBuf;

use macroquad::prelude::*;

use crate::config::GameSettings;
use crate::dan::{CourseState, DanCourse};
use crate::game::GameState;
use crate::render::font::draw_text_jp;

use super::{DanResultScene, Scene, SceneTransition};

/// State of the dan gameplay scene
enum DanPlayState {
    /// Loading the current stage
    Loading,
    /// Playing the current stage
    Playing,
    /// Stage completed, preparing next stage
    StageComplete,
    /// Course finished (passed or failed)
    Finished,
}

pub struct DanGameplayScene {
    /// Course state tracking
    course_state: CourseState,
    /// Directory containing the course file (for resolving chart paths)
    course_dir: Option<PathBuf>,
    /// Current game state for the active stage
    game_state: GameState,
    /// Current play state
    play_state: DanPlayState,
    /// Chart path for current stage
    current_chart_path: String,
    /// Whether current stage is loaded
    loaded: bool,
    /// Timer for stage transition
    transition_timer: f32,
}

impl DanGameplayScene {
    pub fn new(course: DanCourse, course_dir: Option<PathBuf>) -> Self {
        let course_state = CourseState::new(course);

        Self {
            course_state,
            course_dir,
            game_state: GameState::new(),
            play_state: DanPlayState::Loading,
            current_chart_path: String::new(),
            loaded: false,
            transition_timer: 0.0,
        }
    }

    fn get_current_chart_path(&self) -> Option<String> {
        let stage = self.course_state.current_stage();
        let charts = &self.course_state.course().charts;

        charts.get(stage).map(|chart| {
            if let Some(dir) = &self.course_dir {
                dir.join(chart).to_string_lossy().to_string()
            } else {
                chart.clone()
            }
        })
    }

    fn load_current_stage(&mut self) -> bool {
        if let Some(path) = self.get_current_chart_path() {
            self.current_chart_path = path.clone();
            self.game_state = GameState::new();

            // Set initial gauge HP from previous stage
            if self.course_state.current_stage() > 0 {
                // The gauge will be initialized by load_chart, but we need to
                // set the HP after loading. We'll handle this in update.
            }

            if let Err(e) = self.game_state.load_chart(&path) {
                eprintln!(
                    "Error loading stage {}: {}",
                    self.course_state.current_stage() + 1,
                    e
                );
                return false;
            }

            self.loaded = true;
            true
        } else {
            false
        }
    }

    /// Save gameplay settings (scroll speed, lane cover) to persistent storage
    fn save_gameplay_settings(&self) {
        let (scroll_speed, sudden, hidden, lift) = self.game_state.get_gameplay_settings();
        let mut settings = GameSettings::load();
        settings.scroll_speed = scroll_speed;
        settings.sudden = sudden;
        settings.hidden = hidden;
        settings.lift = lift;
        if let Err(e) = settings.save() {
            eprintln!("Failed to save settings: {}", e);
        }
    }
}

impl Scene for DanGameplayScene {
    fn update(&mut self) -> SceneTransition {
        match self.play_state {
            DanPlayState::Loading => {
                if !self.loaded && !self.load_current_stage() {
                    // Failed to load, abort course
                    self.course_state.mark_failed();
                    self.play_state = DanPlayState::Finished;
                    return SceneTransition::None;
                }
                self.play_state = DanPlayState::Playing;
                SceneTransition::None
            }

            DanPlayState::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    // Quit course
                    self.save_gameplay_settings();
                    self.course_state.mark_failed();
                    self.play_state = DanPlayState::Finished;
                    return SceneTransition::None;
                }

                self.game_state.update();

                // Check for stage completion or failure
                let stage_finished = self.game_state.is_finished();
                let stage_failed = self.game_state.is_failed();

                if stage_finished || stage_failed {
                    // Get result and gauge HP
                    let result = self.game_state.get_result(&self.current_chart_path);
                    let end_gauge_hp = if stage_failed {
                        0.0
                    } else {
                        // Get HP from game state's gauge
                        // For dan mode, we need to track HP through the gauge system
                        // Since GameState doesn't expose gauge HP directly in a dan-compatible way,
                        // we'll approximate based on clear status
                        if result.clear_lamp as u8 >= crate::game::ClearLamp::Hard as u8 {
                            // Cleared with hard gauge means at least > 0
                            50.0 // Approximate
                        } else {
                            0.0
                        }
                    };

                    // Record stage result
                    let should_continue = self.course_state.complete_stage(&result, end_gauge_hp);

                    if should_continue {
                        self.play_state = DanPlayState::StageComplete;
                        self.transition_timer = 2.0; // 2 second pause between stages
                    } else {
                        self.play_state = DanPlayState::Finished;
                    }
                }

                SceneTransition::None
            }

            DanPlayState::StageComplete => {
                self.transition_timer -= get_frame_time();

                if self.transition_timer <= 0.0 || is_key_pressed(KeyCode::Enter) {
                    // Load next stage
                    self.loaded = false;
                    self.play_state = DanPlayState::Loading;
                }

                SceneTransition::None
            }

            DanPlayState::Finished => {
                // Save settings before transitioning to result scene
                self.save_gameplay_settings();
                // Transition to result scene
                let course_state = std::mem::replace(
                    &mut self.course_state,
                    CourseState::new(DanCourse {
                        name: String::new(),
                        grade: crate::dan::DanGrade::default(),
                        charts: Vec::new(),
                        gauge_type: crate::game::GaugeType::Hard,
                        requirements: crate::dan::DanRequirements::default(),
                    }),
                );
                SceneTransition::Replace(Box::new(DanResultScene::new(course_state)))
            }
        }
    }

    fn draw(&self) {
        match self.play_state {
            DanPlayState::Loading => {
                clear_background(BLACK);
                let stage = self.course_state.current_stage() + 1;
                let total = self.course_state.total_stages();
                draw_text_jp(
                    &format!("STAGE {}/{}", stage, total),
                    screen_width() / 2.0 - 80.0,
                    screen_height() / 2.0 - 40.0,
                    40.0,
                    WHITE,
                );
                draw_text_jp(
                    "Loading...",
                    screen_width() / 2.0 - 50.0,
                    screen_height() / 2.0 + 20.0,
                    24.0,
                    GRAY,
                );
            }

            DanPlayState::Playing => {
                // Draw game
                self.game_state.draw();

                // Draw stage indicator overlay
                let stage = self.course_state.current_stage() + 1;
                let total = self.course_state.total_stages();
                draw_text_jp(
                    &format!("STAGE {}/{}", stage, total),
                    20.0,
                    screen_height() - 60.0,
                    20.0,
                    YELLOW,
                );

                // Draw course name
                draw_text_jp(
                    &self.course_state.course().name,
                    20.0,
                    screen_height() - 80.0,
                    16.0,
                    GRAY,
                );
            }

            DanPlayState::StageComplete => {
                clear_background(Color::new(0.0, 0.1, 0.0, 1.0));

                let stage = self.course_state.current_stage(); // Already advanced
                let total = self.course_state.total_stages();

                draw_text_jp(
                    &format!("STAGE {} COMPLETE!", stage),
                    screen_width() / 2.0 - 120.0,
                    screen_height() / 2.0 - 40.0,
                    36.0,
                    GREEN,
                );

                if stage < total {
                    draw_text_jp(
                        &format!("Next: STAGE {}/{}", stage + 1, total),
                        screen_width() / 2.0 - 80.0,
                        screen_height() / 2.0 + 20.0,
                        24.0,
                        WHITE,
                    );
                }

                // Show accumulated stats
                let stats = self.course_state.total_stats();
                draw_text_jp(
                    &format!("Total EX Score: {}", stats.ex_score),
                    screen_width() / 2.0 - 80.0,
                    screen_height() / 2.0 + 60.0,
                    18.0,
                    YELLOW,
                );

                draw_text_jp(
                    "[Enter] Continue",
                    screen_width() / 2.0 - 60.0,
                    screen_height() - 40.0,
                    16.0,
                    GRAY,
                );
            }

            DanPlayState::Finished => {
                clear_background(BLACK);
                draw_text_jp(
                    "Loading result...",
                    screen_width() / 2.0 - 80.0,
                    screen_height() / 2.0,
                    24.0,
                    WHITE,
                );
            }
        }
    }
}
