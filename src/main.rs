use brs::audio::{AudioConfig, AudioDriver, KeysoundProcessor};
use brs::config::AppConfig;
use brs::database::{Database, Mode, ScoreDatabaseAccessor, SongData};
use brs::input::{HotkeyConfig, InputManager, KeyConfig, PlayHotkey};
use brs::model::{BMSModel, ChartFormat, JudgeRankType, LongNoteMode, PlayMode, load_chart};
use brs::replay::{ReplayPlayer, ReplaySlot, load_replay};
use brs::skin::path as skin_path;
use brs::skin::SkinRenderer;
use brs::state::config::{ConfigState, ConfigTransition};
use brs::state::decide::{DecideState, DecideTransition};
use brs::state::play::{GaugeType, PlayResult, PlayState, Score};
use brs::state::result::{ResultState, ResultTransition};
use brs::state::select::{SelectScanRequest, SelectState, SelectTransition};
use brs::util::logging::init_logging;
use brs::util::screenshot::capture_screenshot;
use clap::Parser;
use macroquad::prelude::*;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Command line arguments for brs.
#[derive(Parser)]
#[command(name = "brs", about = "BMS player in Rust")]
struct Args {
    /// Capture screenshot of specified state and exit.
    /// Valid values: "select", "play", "result"
    #[arg(long)]
    screenshot: Option<String>,

    /// Output directory for screenshot.
    #[arg(long, default_value = ".agent/screenshots/current")]
    screenshot_output: String,

    /// BMS file path for play screenshot.
    #[arg(long)]
    bms: Option<String>,
}

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
    Config(Box<ConfigState>),
    Decide(Box<DecideState>),
    Play(Box<PlayState>, Box<brs::database::SongData>),
    Result(Box<ResultState>),
}

#[macroquad::main(window_conf)]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    if let Err(e) = init_logging(Some(Path::new("logs")), false) {
        eprintln!("Failed to initialize logging: {}", e);
    }
    info!("brs starting...");

    // Handle screenshot mode
    if let Some(ref state_name) = args.screenshot {
        run_screenshot_mode(state_name, &args.screenshot_output, args.bms.as_deref()).await;
        return;
    }

    // Open databases
    let song_db = match Database::open_song_db(Path::new("song.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open song database: {}", e);
            return;
        }
    };

    let score_db = match Database::open_score_db(Path::new("score.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open score database: {}", e);
            return;
        }
    };

    // Setup input manager
    let key_config = KeyConfig::load().unwrap_or_else(|e| {
        warn!(
            "Failed to load key config / キー設定の読み込みに失敗: {}",
            e
        );
        KeyConfig::default()
    });
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize input: {}", e);
            return;
        }
    };

    // Start with select state
    let select_state =
        match SelectState::new(input_manager, song_db, score_db, SelectScanRequest::Auto) {
            Ok(state) => state,
            Err(e) => {
                error!("Failed to create select state: {}", e);
                return;
            }
        };

    let mut app_state = AppState::Select(Box::new(select_state));
    let hotkeys = HotkeyConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load hotkeys / ホットキーの読み込みに失敗: {}", e);
        HotkeyConfig::default()
    });

    // Game loop
    loop {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        let delta_ms = get_frame_time() as f64 * 1000.0;

        let mut next_state = None;
        let mut should_exit = false;

        match &mut app_state {
            AppState::Select(select_state) => {
                if let Err(e) = select_state.update() {
                    error!("Select error: {}", e);
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
                                error!("Failed to create decide state: {}", e);
                            }
                        }
                    }
                    SelectTransition::Config => {
                        let input_manager = select_state.take_input_manager();
                        let config_state = ConfigState::new(input_manager);
                        next_state = Some(AppState::Config(Box::new(config_state)));
                    }
                    SelectTransition::Exit => {
                        should_exit = true;
                    }
                    SelectTransition::None => {}
                }
            }
            AppState::Config(config_state) => {
                if let Err(e) = config_state.update() {
                    error!("Config error: {}", e);
                }
                config_state.draw();

                match config_state.take_transition() {
                    ConfigTransition::Back { rescan } => {
                        let input_manager = config_state.take_input_manager();
                        let request = if rescan {
                            SelectScanRequest::Manual
                        } else {
                            SelectScanRequest::None
                        };
                        next_state = Some(return_to_select_with_input(input_manager, request));
                    }
                    ConfigTransition::None => {}
                }
            }
            AppState::Decide(decide_state) => {
                if let Err(e) = decide_state.update() {
                    error!("Decide error: {}", e);
                }
                decide_state.draw();

                // Check for transition
                match decide_state.take_transition() {
                    DecideTransition::Ready(prepared) => {
                        let input_manager = decide_state.take_input_manager();
                        let mut play_state = PlayState::new(
                            prepared.model,
                            prepared.audio_driver,
                            prepared.keysound_processor,
                            input_manager,
                            GaugeType::Normal,
                            1.0,
                        );
                        apply_play_skin(&mut play_state).await;
                        let bms_dir = prepared.song_data.path.parent().unwrap_or(Path::new("."));
                        play_state.load_bga(bms_dir).await;
                        next_state = Some(AppState::Play(
                            Box::new(play_state),
                            Box::new(prepared.song_data),
                        ));
                    }
                    DecideTransition::Cancel => {
                        let input_manager = decide_state.take_input_manager();
                        next_state = Some(return_to_select_with_input(
                            input_manager,
                            SelectScanRequest::None,
                        ));
                    }
                    DecideTransition::Error(e) => {
                        error!("Decide error: {}", e);
                        let input_manager = decide_state.take_input_manager();
                        next_state = Some(return_to_select_with_input(
                            input_manager,
                            SelectScanRequest::None,
                        ));
                    }
                    DecideTransition::None => {}
                }
            }
            AppState::Play(play_state, song_data) => {
                // Handle hi-speed adjustment
                if hotkeys.pressed_play(PlayHotkey::HiSpeedUp) {
                    play_state.set_hi_speed(play_state.hi_speed() + 0.25);
                }
                if hotkeys.pressed_play(PlayHotkey::HiSpeedDown) {
                    play_state.set_hi_speed(play_state.hi_speed() - 0.25);
                }
                if hotkeys.pressed_play(PlayHotkey::SpeedReset) {
                    play_state.set_playback_speed(1.0);
                }
                if hotkeys.pressed_play(PlayHotkey::SpeedDown) {
                    play_state.set_playback_speed(play_state.playback_speed() - 0.25);
                }
                if hotkeys.pressed_play(PlayHotkey::SpeedUp) {
                    play_state.set_playback_speed(play_state.playback_speed() + 0.25);
                }
                if hotkeys.pressed_play(PlayHotkey::PracticeStart) {
                    play_state.set_practice_start();
                }
                if hotkeys.pressed_play(PlayHotkey::PracticeEnd) {
                    play_state.set_practice_end();
                }
                if hotkeys.pressed_play(PlayHotkey::PracticeClear) {
                    play_state.clear_practice();
                }
                if hotkeys.pressed_play(PlayHotkey::ToggleBga) {
                    play_state.toggle_bga();
                }

                // Update and draw
                if let Err(e) = play_state.update(delta_ms) {
                    error!("Play error: {}", e);
                }
                play_state.draw();

                // Draw controls help
                draw_play_controls_help();

                // Check for exit or return to select
                if is_key_pressed(KeyCode::Escape) {
                    // Return to select
                    let input_manager = play_state.take_input_manager();
                    next_state = Some(return_to_select_with_input(
                        input_manager,
                        SelectScanRequest::None,
                    ));
                }

                if play_state.is_finished() && is_key_pressed(KeyCode::Enter) {
                    // Transition to result screen
                    let play_result = play_state.take_result();
                    let hi_speed = play_state.hi_speed();
                    let save_result = play_state.should_save_result();
                    let mut input_manager = play_state.take_input_manager();

                    // Extract input logs for replay saving (only for live play)
                    let input_logs = if save_result {
                        input_manager.take_logger().map(|l| l.into_logs())
                    } else {
                        input_manager.take_logger();
                        None
                    };

                    // Open score database for saving
                    match Database::open_score_db(Path::new("score.db")) {
                        Ok(score_db) => {
                            let score_accessor = ScoreDatabaseAccessor::new(&score_db);
                            let result_state = ResultState::new(
                                play_result,
                                song_data.as_ref().clone(),
                                input_manager,
                                input_logs,
                                hi_speed,
                                save_result,
                                &score_accessor,
                            );
                            next_state = Some(AppState::Result(Box::new(result_state)));
                        }
                        Err(e) => {
                            error!("Failed to open score database: {}", e);
                            next_state = Some(return_to_select_with_input(
                                input_manager,
                                SelectScanRequest::None,
                            ));
                        }
                    }
                }
            }
            AppState::Result(result_state) => {
                if let Err(e) = result_state.update() {
                    error!("Result error: {}", e);
                }
                result_state.draw();

                // Check for transition
                match result_state.take_transition() {
                    ResultTransition::Select => {
                        let input_manager = result_state.take_input_manager();
                        next_state = Some(return_to_select_with_input(
                            input_manager,
                            SelectScanRequest::None,
                        ));
                    }
                    ResultTransition::Replay(song_data) => {
                        // Try to load replay data for this song
                        let input_manager = result_state.take_input_manager();
                        match load_replay(&song_data.sha256, ReplaySlot::SLOT_0) {
                            Ok(Some(replay_data)) => {
                                // Play with replay data
                                match create_replay_play_state(
                                    &song_data,
                                    input_manager,
                                    replay_data,
                                )
                                .await
                                {
                                    Ok(play_state) => {
                                        next_state =
                                            Some(AppState::Play(Box::new(play_state), song_data));
                                    }
                                    Err(e) => {
                                        error!("Failed to create replay play state: {}", e);
                                        next_state = Some(return_to_select());
                                    }
                                }
                            }
                            Ok(None) => {
                                // No replay available, just replay the song normally
                                match create_play_state_with_input(&song_data.path, input_manager)
                                    .await
                                {
                                    Ok(play_state) => {
                                        next_state =
                                            Some(AppState::Play(Box::new(play_state), song_data));
                                    }
                                    Err(e) => {
                                        error!("Failed to create play state: {}", e);
                                        next_state = Some(return_to_select());
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to load replay: {}", e);
                                next_state = Some(return_to_select_with_input(
                                    input_manager,
                                    SelectScanRequest::None,
                                ));
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

async fn apply_play_skin(play_state: &mut PlayState) {
    let config = AppConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config / 設定の読み込みに失敗: {}", e);
        AppConfig::default()
    });

    let Some(skin_path) = config.play_skin_path.as_deref() else {
        return;
    };

    let path = Path::new(skin_path);
    let Some(resolved_path) = skin_path::resolve_skin_path(path) else {
        warn!(
            "Skin not found: {} / スキンが見つかりません: {}",
            path.display(),
            path.display()
        );
        return;
    };

    match SkinRenderer::load(&resolved_path).await {
        Ok(renderer) => play_state.set_skin_renderer(renderer),
        Err(e) => warn!("Failed to load skin: {} / スキンの読み込みに失敗: {}", e, e),
    }
}

/// Create play state from BMS path (used for replay).
async fn create_play_state_with_input(
    bms_path: &Path,
    input_manager: InputManager,
) -> anyhow::Result<PlayState> {
    // Initialize audio driver
    let audio_config = AudioConfig::default();
    let audio_driver = AudioDriver::new(audio_config)?;

    // Load BMS model
    let loaded = load_chart(bms_path)?;
    let model = BMSModel::from_bms(&loaded.bms, loaded.format, Some(bms_path))?;

    info!("Loaded: {}", model.title);
    debug!("Artist: {}", model.artist);
    debug!("BPM: {}", model.initial_bpm);
    debug!("Total notes: {}", model.total_notes);
    debug!("BGM events: {}", model.bgm_events.len());

    // Load audio files
    let mut audio_driver = audio_driver;
    let bms_dir = bms_path.parent().unwrap_or(Path::new("."));
    match audio_driver.load_sounds(&model, bms_dir) {
        Ok(progress) => {
            info!(
                "Loaded {} of {} sounds",
                progress.loaded(),
                progress.total()
            );
        }
        Err(e) => {
            warn!("Failed to load sounds: {}", e);
        }
    }

    // Setup keysound processor
    let mut keysound_processor = KeysoundProcessor::new();
    keysound_processor.load_bgm_events(model.bgm_events.clone());

    // Create play state
    let mut play_state = PlayState::new(
        model,
        audio_driver,
        keysound_processor,
        input_manager,
        GaugeType::Normal,
        1.0,
    );
    apply_play_skin(&mut play_state).await;
    play_state.load_bga(bms_dir).await;

    Ok(play_state)
}

fn return_to_select() -> AppState {
    let key_config = KeyConfig::load().unwrap_or_else(|e| {
        warn!(
            "Failed to load key config / キー設定の読み込みに失敗: {}",
            e
        );
        KeyConfig::default()
    });
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize input: {}", e);
            std::process::exit(1);
        }
    };

    return_to_select_with_input(input_manager, SelectScanRequest::None)
}

fn return_to_select_with_input(
    input_manager: InputManager,
    scan_request: SelectScanRequest,
) -> AppState {
    // Re-open databases
    let song_db = match Database::open_song_db(Path::new("song.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open song database: {}", e);
            std::process::exit(1);
        }
    };

    let score_db = match Database::open_score_db(Path::new("score.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open score database: {}", e);
            std::process::exit(1);
        }
    };

    // Create new select state
    match SelectState::new(input_manager, song_db, score_db, scan_request) {
        Ok(state) => AppState::Select(Box::new(state)),
        Err(e) => {
            error!("Failed to create select state: {}", e);
            std::process::exit(1);
        }
    }
}

/// Create play state with replay data for playback.
async fn create_replay_play_state(
    song_data: &SongData,
    input_manager: InputManager,
    replay_data: brs::replay::ReplayData,
) -> anyhow::Result<PlayState> {
    // Initialize audio driver
    let audio_config = AudioConfig::default();
    let audio_driver = AudioDriver::new(audio_config)?;

    // Load BMS model
    let loaded = load_chart(&song_data.path)?;
    let model = BMSModel::from_bms(&loaded.bms, loaded.format, Some(&song_data.path))?;

    info!("Loading replay for: {}", model.title);
    debug!("Replay recorded at: {}", replay_data.metadata.recorded_at);
    debug!("Replay inputs: {}", replay_data.input_logs.len());

    // Load audio files
    let mut audio_driver = audio_driver;
    let bms_dir = song_data.path.parent().unwrap_or(Path::new("."));
    match audio_driver.load_sounds(&model, bms_dir) {
        Ok(progress) => {
            info!(
                "Loaded {} of {} sounds",
                progress.loaded(),
                progress.total()
            );
        }
        Err(e) => {
            warn!("Failed to load sounds: {}", e);
        }
    }

    // Setup keysound processor
    let mut keysound_processor = KeysoundProcessor::new();
    keysound_processor.load_bgm_events(model.bgm_events.clone());

    // Create replay player
    let replay_player = ReplayPlayer::new(replay_data.input_logs);

    // Create play state with replay
    let mut play_state = PlayState::new_replay(
        model,
        audio_driver,
        keysound_processor,
        input_manager,
        GaugeType::Normal, // TODO: Use gauge type from replay data
        replay_data.metadata.hi_speed,
        replay_player,
    );
    apply_play_skin(&mut play_state).await;
    play_state
        .load_bga(song_data.path.parent().unwrap_or(Path::new(".")))
        .await;

    Ok(play_state)
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

/// Run screenshot capture mode.
/// Renders the specified state for a few frames, captures a screenshot, and exits.
async fn run_screenshot_mode(state_name: &str, output_dir: &str, bms_path: Option<&str>) {
    const WARMUP_FRAMES: u32 = 5;

    info!("Screenshot mode: capturing '{}' state", state_name);

    match state_name {
        "select" => {
            run_screenshot_select(output_dir, WARMUP_FRAMES).await;
        }
        "play" => {
            let Some(bms) = bms_path else {
                error!("--bms option is required for play screenshot");
                eprintln!("Error: --bms option is required for play screenshot");
                return;
            };
            run_screenshot_play(output_dir, bms, WARMUP_FRAMES).await;
        }
        "result" => {
            run_screenshot_result(output_dir, WARMUP_FRAMES).await;
        }
        other => {
            error!(
                "Unknown screenshot state: '{}'. Valid states: select, play, result",
                other
            );
            eprintln!(
                "Unknown screenshot state: '{}'. Valid states: select, play, result",
                other
            );
        }
    }
}

/// Capture screenshot of select state.
async fn run_screenshot_select(output_dir: &str, warmup_frames: u32) {
    // Open databases
    let song_db = match Database::open_song_db(Path::new("song.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open song database: {}", e);
            return;
        }
    };

    let score_db = match Database::open_score_db(Path::new("score.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open score database: {}", e);
            return;
        }
    };

    // Setup input manager
    let key_config = KeyConfig::load().unwrap_or_default();
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize input: {}", e);
            return;
        }
    };

    // Create select state
    let mut select_state =
        match SelectState::new(input_manager, song_db, score_db, SelectScanRequest::None) {
            Ok(state) => state,
            Err(e) => {
                error!("Failed to create select state: {}", e);
                return;
            }
        };

    // Render warmup frames to stabilize UI
    for _ in 0..warmup_frames {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));
        let _ = select_state.update();
        select_state.draw();
        next_frame().await;
    }

    // Capture screenshot
    save_screenshot(output_dir, "select.png");
}

/// Capture screenshot of play state.
async fn run_screenshot_play(output_dir: &str, bms_path: &str, warmup_frames: u32) {
    let bms_path = Path::new(bms_path);
    if !bms_path.exists() {
        error!("BMS file not found: {}", bms_path.display());
        eprintln!("Error: BMS file not found: {}", bms_path.display());
        return;
    }

    // Initialize audio driver
    let audio_config = AudioConfig::default();
    let audio_driver = match AudioDriver::new(audio_config) {
        Ok(driver) => driver,
        Err(e) => {
            error!("Failed to initialize audio: {}", e);
            return;
        }
    };

    // Load BMS model
    let loaded = match load_chart(bms_path) {
        Ok(loaded) => loaded,
        Err(e) => {
            error!("Failed to load BMS: {}", e);
            return;
        }
    };

    let model = match BMSModel::from_bms(&loaded.bms, loaded.format, Some(bms_path)) {
        Ok(model) => model,
        Err(e) => {
            error!("Failed to parse BMS: {}", e);
            return;
        }
    };

    info!("Loaded: {}", model.title);

    // Load audio files
    let mut audio_driver = audio_driver;
    let bms_dir = bms_path.parent().unwrap_or(Path::new("."));
    if let Err(e) = audio_driver.load_sounds(&model, bms_dir) {
        warn!("Failed to load sounds: {}", e);
    }

    // Setup keysound processor
    let mut keysound_processor = KeysoundProcessor::new();
    keysound_processor.load_bgm_events(model.bgm_events.clone());

    // Setup input manager
    let key_config = KeyConfig::load().unwrap_or_default();
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize input: {}", e);
            return;
        }
    };

    // Create play state
    let mut play_state = PlayState::new(
        model,
        audio_driver,
        keysound_processor,
        input_manager,
        GaugeType::Normal,
        1.0,
    );

    // For screenshot mode, use built-in beatoraja-style renderer
    // (External skins require texture assets which may not be available)
    // No skin renderer is set, so draw_beatoraja_ui() will be used

    play_state.load_bga(bms_dir).await;

    // Simulate time to pass countdown and show notes
    // Run enough frames to get past countdown (3000ms default) and into gameplay
    let frames_to_simulate = 250 + warmup_frames as usize; // ~4 seconds at 60fps + warmup
    for i in 0..frames_to_simulate {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));
        let _ = play_state.update(16.67); // ~60fps
        play_state.draw();

        // Capture screenshot on the last frame before next_frame()
        if i == frames_to_simulate - 1 {
            save_screenshot(output_dir, "play.png");
        }

        next_frame().await;
    }
}

/// Capture screenshot of result state with mock data.
async fn run_screenshot_result(output_dir: &str, warmup_frames: u32) {
    // Create mock play result
    let mut score = Score::new(1000);
    score.pg_count = 850;
    score.gr_count = 120;
    score.gd_count = 20;
    score.bd_count = 5;
    score.pr_count = 3;
    score.ms_count = 2;
    score.max_combo = 500;

    let play_result = PlayResult::new(
        score,
        80.0,                   // gauge_value
        GaugeType::Normal,      // gauge_type
        true,                   // is_clear
        180000.0,               // play_time_ms (3 minutes)
        15,                     // fast_count
        12,                     // slow_count
        PlayMode::Beat7K,       // play_mode
        LongNoteMode::Ln,       // long_note_mode
        2,                      // judge_rank (NORMAL)
        JudgeRankType::BmsRank, // judge_rank_type
        300.0,                  // total
        ChartFormat::Bms,       // source_format
    );

    // Create mock song data
    let song_data = SongData {
        sha256: "mock_sha256_for_screenshot".to_string(),
        md5: "mock_md5".to_string(),
        path: PathBuf::from("/mock/path/song.bms"),
        folder: "Mock Folder".to_string(),
        title: "Sample Song Title".to_string(),
        subtitle: "~Subtitle~".to_string(),
        artist: "Sample Artist".to_string(),
        subartist: "feat. Guest".to_string(),
        genre: "Sample Genre".to_string(),
        mode: Mode::Beat7K,
        level: 12,
        difficulty: 4, // ANOTHER
        max_bpm: 180,
        min_bpm: 180,
        notes: 1000,
        date: 0,
        add_date: 0,
    };

    // Setup input manager
    let key_config = KeyConfig::load().unwrap_or_default();
    let input_manager = match InputManager::new(key_config) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize input: {}", e);
            return;
        }
    };

    // Open a temporary score database (we won't actually save)
    let score_db = match Database::open_score_db(Path::new("score.db")) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to open score database: {}", e);
            return;
        }
    };
    let score_accessor = ScoreDatabaseAccessor::new(&score_db);

    // Create result state (save_score=false to avoid writing mock data)
    let mut result_state = ResultState::new(
        play_result,
        song_data,
        input_manager,
        None,  // input_logs
        2.0,   // hi_speed
        false, // save_score
        &score_accessor,
    );

    // Render warmup frames
    for _ in 0..warmup_frames {
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));
        let _ = result_state.update();
        result_state.draw();
        next_frame().await;
    }

    // Capture screenshot
    save_screenshot(output_dir, "result.png");
}

/// Helper to save screenshot and log result.
fn save_screenshot(output_dir: &str, filename: &str) {
    let output_path = Path::new(output_dir).join(filename);

    // Create directory if needed
    if let Some(parent) = output_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("Failed to create directory: {}", e);
            eprintln!("Failed to create directory: {}", e);
            return;
        }
    }

    // Use macroquad's built-in screenshot function first
    let screen = get_screen_data();
    eprintln!(
        "Screen data: {}x{}, {} bytes",
        screen.width,
        screen.height,
        screen.bytes.len()
    );

    match capture_screenshot(&output_path) {
        Ok(()) => {
            info!("Screenshot saved to: {}", output_path.display());
            println!("Screenshot saved: {}", output_path.display());
        }
        Err(e) => {
            error!("Failed to capture screenshot: {}", e);
            eprintln!("Failed to capture screenshot: {}", e);
        }
    }
}
