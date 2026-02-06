use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Result, anyhow};
use clap::Parser;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::window::Window;

use brs::app::controller::{AppController, AppStateType, AppTransition};
use brs::audio::audio_driver::AudioDriver;
use brs::config::app_config::AppConfig;
use brs::config::player_config::PlayerConfig;
use brs::database::scanner;
use brs::database::score_db::ScoreDatabase;
use brs::database::song_db::SongDatabase;
use brs::input::input_manager::InputManager;
use brs::model::bms_loader;
use brs::model::note::PlayMode;
use brs::play::play_result::PlayResult;
use brs::render::wgpu_renderer::WgpuRenderer;
use brs::render::window::{GameLoop, WindowConfig, run_app};
use brs::state::config::key_config::KeyConfig;
use brs::state::decide::decide_state::{DecideConfig, DecideState};
use brs::state::game_state::{GameState, StateTransition};
use brs::state::play::play_state::{self, PlayState};
use brs::state::result::result_state::{ResultConfig, ResultState};
use brs::state::select::bar::Bar;
use brs::state::select::select_state::{SelectInput, SelectState};
use brs::traits::input::InputProvider;
use brs::traits::render::{Color, FontId, RenderBackend};

#[derive(Parser)]
#[command(name = "brs", version, about = "BMS rhythm game player")]
struct Cli {
    /// Path to the configuration directory.
    #[arg(short, long, default_value = "~/.brs")]
    config_dir: String,

    /// BMS file to play directly (skips select screen).
    #[arg(short, long)]
    play: Option<String>,
}

/// Resolve ~ to home directory.
fn resolve_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(path)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Active game state, holding the current state implementation.
enum ActiveState {
    Select(SelectState),
    Decide(DecideState),
    Play(PlayState),
    Result(ResultState),
    None,
}

/// Main game application integrating all subsystems.
struct BrsApp {
    controller: AppController,
    state: ActiveState,

    renderer: Option<WgpuRenderer>,
    audio: Option<AudioDriver>,
    input: InputManager,
    font: Option<FontId>,

    #[allow(dead_code)]
    app_config: AppConfig,
    #[allow(dead_code)]
    player_config: PlayerConfig,

    song_db_path: String,
    score_db_path: String,
    bms_roots: Vec<String>,

    // Timing
    last_frame: Instant,
    elapsed_us: i64,
    play_start_us: i64,

    // Direct play mode
    direct_play_path: Option<String>,
    // Selected BMS path (set by select screen)
    selected_bms_path: Option<String>,
    // Last play result (for result screen)
    last_play_result: Option<PlayResult>,
    last_sha256: String,
}

impl BrsApp {
    fn new(app_config: AppConfig, player_config: PlayerConfig, config_dir: &str) -> Self {
        let key_config = KeyConfig::default_7k();
        let input = InputManager::new(key_config, PlayMode::Beat7K.key_count());

        let config_path = resolve_path(config_dir);
        let song_db_path = config_path
            .join("songdata.db")
            .to_string_lossy()
            .to_string();
        let score_db_path = config_path.join("score.db").to_string_lossy().to_string();
        let bms_roots = app_config.bms_directories.clone();

        Self {
            controller: AppController::new(),
            state: ActiveState::None,

            renderer: None,
            audio: None,
            input,
            font: None,

            app_config,
            player_config,

            song_db_path,
            score_db_path,
            bms_roots,

            last_frame: Instant::now(),
            elapsed_us: 0,
            play_start_us: 0,

            direct_play_path: None,
            selected_bms_path: None,
            last_play_result: None,
            last_sha256: String::new(),
        }
    }

    fn enter_state(&mut self, state_type: AppStateType) {
        // Dispose current state
        match &mut self.state {
            ActiveState::Select(s) => s.dispose(),
            ActiveState::Decide(s) => s.dispose(),
            ActiveState::Play(_) => {} // PlayState has no dispose trait impl
            ActiveState::Result(s) => s.dispose(),
            ActiveState::None => {}
        }

        // Create new state
        match state_type {
            AppStateType::MusicSelect => {
                if let Some(path) = self.direct_play_path.take() {
                    // Direct play mode: skip select, go straight to decide
                    self.selected_bms_path = Some(path);
                    self.controller
                        .apply_transition(&AppTransition::ChangeTo(AppStateType::Decide));
                    self.enter_state(AppStateType::Decide);
                    return;
                }

                let song_db = SongDatabase::open(&self.song_db_path)
                    .unwrap_or_else(|_| SongDatabase::open_in_memory().unwrap());
                let score_db = ScoreDatabase::open(&self.score_db_path)
                    .unwrap_or_else(|_| ScoreDatabase::open_in_memory().unwrap());

                // Scan BMS directories to populate the database.
                match scanner::scan_directories(&song_db, &self.bms_roots) {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("scanned {count} BMS files");
                        }
                    }
                    Err(e) => tracing::error!("BMS scan failed: {e}"),
                }

                let mut select = SelectState::new(song_db, score_db, self.bms_roots.clone());
                if let Err(e) = select.create() {
                    tracing::error!("failed to create select state: {e}");
                }
                self.state = ActiveState::Select(select);
            }
            AppStateType::Decide => {
                let bms_path = self.selected_bms_path.clone().unwrap_or_default();

                let config = DecideConfig {
                    bms_path: bms_path.clone(),
                    ..Default::default()
                };
                let mut decide = DecideState::new(config);
                if let Err(e) = decide.create() {
                    tracing::error!("failed to create decide state: {e}");
                }
                decide.set_loading_complete();
                self.state = ActiveState::Decide(decide);
            }
            AppStateType::Play => {
                let bms_path = self.selected_bms_path.clone().unwrap_or_default();
                self.play_start_us = self.elapsed_us;

                match load_and_create_play_state(&bms_path) {
                    Ok(mut play) => {
                        if let Err(e) = play.create() {
                            tracing::error!("failed to create play state: {e}");
                        }
                        self.state = ActiveState::Play(play);
                    }
                    Err(e) => {
                        tracing::error!("failed to load BMS {bms_path}: {e}");
                        self.controller
                            .apply_transition(&AppTransition::ChangeTo(AppStateType::MusicSelect));
                        self.enter_state(AppStateType::MusicSelect);
                        return;
                    }
                }
            }
            AppStateType::Result => {
                let play_result = self.last_play_result.take();
                let sha256 = std::mem::take(&mut self.last_sha256);

                if let Some(result_data) = play_result {
                    let mut result =
                        ResultState::new(result_data, sha256, 0, ResultConfig::default());
                    if let Err(e) = result.create() {
                        tracing::error!("failed to create result state: {e}");
                    }
                    self.state = ActiveState::Result(result);
                } else {
                    // No play result; return to select
                    self.controller
                        .apply_transition(&AppTransition::ChangeTo(AppStateType::MusicSelect));
                    self.enter_state(AppStateType::MusicSelect);
                    return;
                }
            }
            AppStateType::CourseResult | AppStateType::Config => {
                self.controller
                    .apply_transition(&AppTransition::ChangeTo(AppStateType::MusicSelect));
                self.enter_state(AppStateType::MusicSelect);
                return;
            }
        }

        tracing::info!("entered state: {:?}", state_type);
    }

    fn process_transition(&mut self, transition: StateTransition) {
        if transition == StateTransition::None {
            return;
        }

        // Capture selected BMS path from SelectState before transition
        if let ActiveState::Select(select) = &self.state
            && transition == StateTransition::Next
            && let Some(Bar::Song(song_bar)) = select.bar_manager().selected()
        {
            self.selected_bms_path = Some(song_bar.song.path.clone());
            tracing::info!("selected: {}", song_bar.title);
        }

        // Capture play result before transition
        if let ActiveState::Play(play) = &self.state {
            self.last_play_result = Some(play.build_result());
            if let Some(path) = &self.selected_bms_path {
                self.last_sha256 = path.clone();
            }
        }

        let app_transition = self.controller.resolve_transition(transition);
        self.controller.apply_transition(&app_transition);

        match app_transition {
            AppTransition::ChangeTo(new_state) => {
                self.enter_state(new_state);
            }
            AppTransition::Exit => {
                tracing::info!("exit requested");
            }
            AppTransition::None => {}
        }
    }

    /// Render debug UI showing current state and basic info.
    fn render_debug_ui(&mut self) -> Result<()> {
        let font = match self.font {
            Some(f) => f,
            None => return Ok(()),
        };
        let renderer = match &mut self.renderer {
            Some(r) => r,
            None => return Ok(()),
        };

        let white = Color::WHITE;
        let gray = Color::new(0.6, 0.6, 0.6, 1.0);

        match &self.state {
            ActiveState::Select(select) => {
                renderer.draw_text(font, "MUSIC SELECT", 20.0, 20.0, 32.0, white)?;

                let bars = select.bar_manager().bars();
                let cursor = select.bar_manager().cursor();

                if bars.is_empty() {
                    renderer.draw_text(
                        font,
                        "No songs found. Add BMS directories to ~/.brs/config.json",
                        20.0,
                        80.0,
                        20.0,
                        gray,
                    )?;
                } else {
                    // Show bars around cursor
                    let start = cursor.saturating_sub(5);
                    let end = (start + 15).min(bars.len());

                    for (i, idx) in (start..end).enumerate() {
                        let bar = &bars[idx];
                        let y = 80.0 + i as f32 * 30.0;
                        let color = if idx == cursor { white } else { gray };
                        let prefix = if idx == cursor { "> " } else { "  " };
                        let label = if bar.is_directory() {
                            format!("{prefix}[{}]", bar.title())
                        } else {
                            format!("{prefix}{}", bar.title())
                        };
                        renderer.draw_text(font, &label, 20.0, y, 20.0, color)?;
                    }

                    let info = format!(
                        "{}/{} | UP/DOWN: move  ENTER: select  ESC: exit",
                        cursor + 1,
                        bars.len()
                    );
                    renderer.draw_text(font, &info, 20.0, 560.0, 16.0, gray)?;
                }
            }
            ActiveState::Decide(decide) => {
                renderer.draw_text(font, "DECIDE", 20.0, 20.0, 32.0, white)?;
                let phase = format!("Phase: {:?}", decide.phase());
                renderer.draw_text(font, &phase, 20.0, 80.0, 20.0, gray)?;
                if let Some(path) = &self.selected_bms_path {
                    renderer.draw_text(font, path, 20.0, 120.0, 16.0, gray)?;
                }
            }
            ActiveState::Play(play) => {
                renderer.draw_text(font, "PLAYING", 20.0, 20.0, 32.0, white)?;
                let phase = format!("Phase: {:?}", play.phase());
                renderer.draw_text(font, &phase, 20.0, 80.0, 20.0, gray)?;
                let score = play.judge_score();
                let info = format!(
                    "COMBO: {}  GAUGE: {:.1}%",
                    score.combo,
                    play.gauge().value()
                );
                renderer.draw_text(font, &info, 20.0, 120.0, 20.0, white)?;
            }
            ActiveState::Result(result) => {
                renderer.draw_text(font, "RESULT", 20.0, 20.0, 32.0, white)?;
                let pr = result.play_result();
                let info = format!(
                    "Clear: {:?}  Rank: {:?}  Gauge: {:.1}%",
                    pr.clear_type, pr.rank, pr.gauge_value
                );
                renderer.draw_text(font, &info, 20.0, 80.0, 20.0, white)?;

                let s = &pr.score;
                let ex = s.exscore();
                renderer.draw_text(
                    font,
                    &format!("EXSCORE: {ex}  COMBO: {}", s.max_combo),
                    20.0,
                    120.0,
                    20.0,
                    gray,
                )?;

                renderer.draw_text(font, "Press ENTER to continue", 20.0, 560.0, 16.0, gray)?;
            }
            ActiveState::None => {}
        }

        Ok(())
    }
}

/// Load a BMS file and create a PlayState from it.
fn load_and_create_play_state(bms_path: &str) -> Result<PlayState> {
    let model = bms_loader::load_bms(Path::new(bms_path))?;
    let mode = model.mode;

    let config = play_state::PlayConfig {
        model,
        mode,
        gauge_type: 3, // Normal gauge
        autoplay_mode: brs::state::play::autoplay::AutoplayMode::Full,
    };

    Ok(PlayState::new(config))
}

impl GameLoop for BrsApp {
    fn init(&mut self, window: &Window) -> Result<()> {
        // SAFETY: We create an Arc from a raw pointer to the Window reference.
        // ManuallyDrop prevents the Arc from calling drop on the Window when it goes out of scope,
        // since the Window is owned by the event loop, not by us.
        let window_arc =
            std::mem::ManuallyDrop::new(unsafe { Arc::from_raw(window as *const Window) });

        let mut renderer = pollster::block_on(WgpuRenderer::new(Arc::clone(&window_arc)))
            .map_err(|e| anyhow!("failed to create renderer: {e}"))?;

        // Load default font
        let font_path = Path::new("assets/fonts/NotoSansJP-Regular.ttf");
        match renderer.load_font(font_path) {
            Ok(font_id) => self.font = Some(font_id),
            Err(e) => tracing::warn!("failed to load font: {e}"),
        }

        self.renderer = Some(renderer);

        match AudioDriver::new() {
            Ok(audio) => self.audio = Some(audio),
            Err(e) => tracing::warn!("audio initialization failed: {e}"),
        }

        self.last_frame = Instant::now();

        // Enter initial state
        let initial = self.controller.current_state();
        self.enter_state(initial);

        Ok(())
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame);
        let dt_us = dt.as_micros() as i64;
        self.last_frame = now;
        self.elapsed_us += dt_us;

        // Poll gamepad
        self.input.poll_gamepad(self.elapsed_us);

        // Update current state
        let transition = match &mut self.state {
            ActiveState::Select(s) => s.update(dt_us).unwrap_or(StateTransition::None),
            ActiveState::Decide(s) => s.update(dt_us).unwrap_or(StateTransition::None),
            ActiveState::Play(s) => {
                let play_time_us = self.elapsed_us - self.play_start_us;
                let events = self.input.poll_events();
                for mut event in events {
                    event.time_us -= self.play_start_us;
                    s.process_key_event(event);
                }
                s.update(play_time_us).unwrap_or(StateTransition::None)
            }
            ActiveState::Result(s) => s.update(dt_us).unwrap_or(StateTransition::None),
            ActiveState::None => StateTransition::None,
        };

        self.process_transition(transition);
    }

    fn render(&mut self) -> Result<()> {
        let renderer = match &mut self.renderer {
            Some(r) => r,
            None => return Ok(()),
        };

        renderer.begin_frame()?;
        renderer.clear(Color {
            r: 0.0,
            g: 0.0,
            b: 0.05,
            a: 1.0,
        })?;

        // Drop renderer borrow so render_debug_ui can borrow self
        let _ = renderer;
        self.render_debug_ui()?;

        let renderer = self.renderer.as_mut().unwrap();
        renderer.end_frame()?;
        Ok(())
    }

    fn should_close(&self) -> bool {
        self.controller.should_exit()
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    fn on_input(&mut self, event: &WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) {
            self.controller.request_exit();
            return;
        }

        if let WindowEvent::KeyboardInput { event, .. } = event
            && let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key
        {
            let pressed = event.state == winit::event::ElementState::Pressed;
            self.input
                .handle_keyboard(keycode, pressed, self.elapsed_us);

            if !pressed {
                return;
            }

            // State-specific key handling
            match &mut self.state {
                ActiveState::Select(select) => match keycode {
                    KeyCode::ArrowUp => select.push_input(SelectInput::Up),
                    KeyCode::ArrowDown => select.push_input(SelectInput::Down),
                    KeyCode::Enter => select.push_input(SelectInput::Decide),
                    KeyCode::Escape => {
                        select.push_input(SelectInput::Back);
                    }
                    _ => {}
                },
                ActiveState::Decide(decide) => {
                    if keycode == KeyCode::Enter {
                        decide.advance();
                    } else if keycode == KeyCode::Escape {
                        decide.cancel();
                    }
                }
                ActiveState::Play(_) => {
                    // Play input is handled via InputManager in update()
                }
                ActiveState::Result(result) => {
                    if keycode == KeyCode::Enter || keycode == KeyCode::Escape {
                        result.advance();
                    }
                }
                ActiveState::None => {}
            }

            // ESC from select exits the app
            if keycode == KeyCode::Escape
                && self.controller.current_state() == AppStateType::MusicSelect
            {
                self.controller.request_exit();
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("brs starting");

    let config_dir = resolve_path(&cli.config_dir);
    std::fs::create_dir_all(&config_dir)?;

    let mut app_config = AppConfig::load_or_default(&config_dir);
    if app_config.bms_directories.is_empty() {
        app_config.bms_directories.push("./bms".to_string());
    }
    let player_config = PlayerConfig::default();

    let window_config = WindowConfig {
        title: "brs".to_string(),
        width: app_config.width,
        height: app_config.height,
        resizable: false,
    };

    let mut app = BrsApp::new(app_config, player_config, &cli.config_dir);

    if let Some(bms_path) = cli.play {
        app.direct_play_path = Some(bms_path);
    }

    run_app(window_config, app)?;

    tracing::info!("brs exiting");
    Ok(())
}
