use brs::audio::{AudioConfig, AudioDriver, KeysoundProcessor};
use brs::database::{Database, ScoreDatabaseAccessor};
use brs::input::{InputManager, KeyConfig};
use brs::model::{BMSModel, load_bms};
use brs::state::decide::{DecideState, DecideTransition};
use brs::state::play::{GaugeType, PlayState};
use brs::state::result::{ResultState, ResultTransition};
use brs::state::select::{SelectState, SelectTransition};
use macroquad::prelude::*;
use std::path::Path;

fn window_conf() -> Conf {
    Conf {
        window_title: "brs".to_owned(),
        window_width: 1920,
        window_height: 1080,
        fullscreen: false,
        ..Default::default()
    }
}

/// Application state machine.
enum AppState {
    Select(Box<SelectState>),
    Decide(Box<DecideState>),
    Play(Box<PlayState>, Box<brs::database::SongData>),
    Result(Box<ResultState>),
}

#[macroquad::main(window_conf)]
async fn main() {
    // Open databases
    let song_db = match Database::open_song_db(Path::new("song.db")) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open song database: {}", e);
            return;
        }
    };

    let score_db = match Database::open_score_db(Path::new("score.db")) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open score database: {}", e);
            return;
        }
    };

    // Setup input manager
    let key_config = KeyConfig::default();
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to initialize input: {}", e);
            return;
        }
    };

    // Start with select state
    let select_state = match SelectState::new(input_manager, song_db, score_db) {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to create select state: {}", e);
            return;
        }
    };

    let mut app_state = AppState::Select(Box::new(select_state));

    // Game loop
    loop {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        let delta_ms = get_frame_time() as f64 * 1000.0;

        let mut next_state = None;
        let mut should_exit = false;

        match &mut app_state {
            AppState::Select(select_state) => {
                if let Err(e) = select_state.update() {
                    eprintln!("Select error: {}", e);
                }
                select_state.draw();

                // Check for transition
                match select_state.take_transition() {
                    SelectTransition::Decide(song_data) => {
                        // Transition to decide state
                        let input_manager = select_state.take_input_manager();
                        match DecideState::new(*song_data, input_manager) {
                            Ok(decide_state) => {
                                next_state = Some(AppState::Decide(Box::new(decide_state)));
                            }
                            Err(e) => {
                                eprintln!("Failed to create decide state: {}", e);
                            }
                        }
                    }
                    SelectTransition::Exit => {
                        should_exit = true;
                    }
                    SelectTransition::None => {}
                }
            }
            AppState::Decide(decide_state) => {
                if let Err(e) = decide_state.update() {
                    eprintln!("Decide error: {}", e);
                }
                decide_state.draw();

                // Check for transition
                match decide_state.take_transition() {
                    DecideTransition::Ready(prepared) => {
                        let input_manager = decide_state.take_input_manager();
                        let play_state = PlayState::new(
                            prepared.model,
                            prepared.audio_driver,
                            prepared.keysound_processor,
                            input_manager,
                            GaugeType::Normal,
                            1.0,
                        );
                        next_state = Some(AppState::Play(
                            Box::new(play_state),
                            Box::new(prepared.song_data),
                        ));
                    }
                    DecideTransition::Cancel => {
                        next_state = Some(return_to_select());
                    }
                    DecideTransition::Error(e) => {
                        eprintln!("Decide error: {}", e);
                        next_state = Some(return_to_select());
                    }
                    DecideTransition::None => {}
                }
            }
            AppState::Play(play_state, song_data) => {
                // Handle hi-speed adjustment
                if is_key_pressed(KeyCode::Up) {
                    play_state.set_hi_speed(play_state.hi_speed() + 0.25);
                }
                if is_key_pressed(KeyCode::Down) {
                    play_state.set_hi_speed(play_state.hi_speed() - 0.25);
                }

                // Update and draw
                if let Err(e) = play_state.update(delta_ms) {
                    eprintln!("Play error: {}", e);
                }
                play_state.draw();

                // Draw controls help
                draw_play_controls_help();

                // Check for exit or return to select
                if is_key_pressed(KeyCode::Escape) {
                    // Return to select
                    next_state = Some(return_to_select());
                }

                if play_state.is_finished() && is_key_pressed(KeyCode::Enter) {
                    // Transition to result screen
                    let play_result = play_state.take_result();
                    let input_manager = play_state.take_input_manager();

                    // Open score database for saving
                    match Database::open_score_db(Path::new("score.db")) {
                        Ok(score_db) => {
                            let score_accessor = ScoreDatabaseAccessor::new(&score_db);
                            let result_state = ResultState::new(
                                play_result,
                                song_data.as_ref().clone(),
                                input_manager,
                                &score_accessor,
                            );
                            next_state = Some(AppState::Result(Box::new(result_state)));
                        }
                        Err(e) => {
                            eprintln!("Failed to open score database: {}", e);
                            next_state = Some(return_to_select());
                        }
                    }
                }
            }
            AppState::Result(result_state) => {
                if let Err(e) = result_state.update() {
                    eprintln!("Result error: {}", e);
                }
                result_state.draw();

                // Check for transition
                match result_state.take_transition() {
                    ResultTransition::Select => {
                        next_state = Some(return_to_select());
                    }
                    ResultTransition::Replay(song_data) => {
                        // Create new play state for replay
                        let input_manager = result_state.take_input_manager();
                        match create_play_state_with_input(&song_data.path, input_manager) {
                            Ok(play_state) => {
                                next_state = Some(AppState::Play(Box::new(play_state), song_data));
                            }
                            Err(e) => {
                                eprintln!("Failed to create play state for replay: {}", e);
                                next_state = Some(return_to_select());
                            }
                        }
                    }
                    ResultTransition::None => {}
                }
            }
        }

        if should_exit {
            break;
        }

        if let Some(state) = next_state {
            app_state = state;
        }

        next_frame().await;
    }
}

/// Create play state from BMS path (used for replay).
fn create_play_state_with_input(
    bms_path: &Path,
    input_manager: InputManager,
) -> anyhow::Result<PlayState> {
    // Initialize audio driver
    let audio_config = AudioConfig::default();
    let audio_driver = AudioDriver::new(audio_config)?;

    // Load BMS model
    let bms = load_bms(bms_path)?;
    let model = BMSModel::from_bms(&bms)?;

    println!("Loaded: {}", model.title);
    println!("Artist: {}", model.artist);
    println!("BPM: {}", model.initial_bpm);
    println!("Total notes: {}", model.total_notes);
    println!("BGM events: {}", model.bgm_events.len());

    // Load audio files
    let mut audio_driver = audio_driver;
    let bms_dir = bms_path.parent().unwrap_or(Path::new("."));
    match audio_driver.load_sounds(&model, bms_dir) {
        Ok(progress) => {
            println!(
                "Loaded {} of {} sounds",
                progress.loaded(),
                progress.total()
            );
        }
        Err(e) => {
            eprintln!("Failed to load sounds: {}", e);
        }
    }

    // Setup keysound processor
    let mut keysound_processor = KeysoundProcessor::new();
    keysound_processor.load_bgm_events(model.bgm_events.clone());

    // Create play state
    let play_state = PlayState::new(
        model,
        audio_driver,
        keysound_processor,
        input_manager,
        GaugeType::Normal,
        1.0,
    );

    Ok(play_state)
}

fn return_to_select() -> AppState {
    // Re-open databases
    let song_db = match Database::open_song_db(Path::new("song.db")) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open song database: {}", e);
            std::process::exit(1);
        }
    };

    let score_db = match Database::open_score_db(Path::new("score.db")) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open score database: {}", e);
            std::process::exit(1);
        }
    };

    // Create new input manager
    let key_config = KeyConfig::default();
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to initialize input: {}", e);
            std::process::exit(1);
        }
    };

    // Create new select state
    match SelectState::new(input_manager, song_db, score_db) {
        Ok(state) => AppState::Select(Box::new(state)),
        Err(e) => {
            eprintln!("Failed to create select state: {}", e);
            std::process::exit(1);
        }
    }
}

fn draw_play_controls_help() {
    let x = 550.0;
    let y = 400.0;

    draw_text("Controls:", x, y, 18.0, GRAY);
    draw_text("  Up/Down: Adjust hi-speed", x, y + 24.0, 16.0, GRAY);
    draw_text("  LShift: Scratch", x, y + 48.0, 16.0, GRAY);
    draw_text("  Z S X D C F V: Keys 1-7", x, y + 72.0, 16.0, GRAY);
    draw_text("  Escape: Return to select", x, y + 96.0, 16.0, GRAY);
    draw_text(
        "  Enter (after finish): Go to result",
        x,
        y + 120.0,
        16.0,
        GRAY,
    );
}
