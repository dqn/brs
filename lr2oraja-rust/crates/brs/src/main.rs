// brs — main binary for the BMS player.
//
// Integrates all crates via Bevy app with state machine.

mod app_state;
mod bevy_keyboard;
pub mod database_manager;
pub mod external_manager;
mod game_state;
pub mod input_mapper;
mod player_resource;
mod skin_manager;
mod state;
mod system_sound;
mod timer_manager;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use clap::Parser;
use tracing::info;

use app_state::{AppStateType, StateRegistry, TickParams};
use database_manager::DatabaseManager;
use game_state::{SharedGameState, sync_timer_state};
use input_mapper::InputMapper;
use player_resource::PlayerResource;
use state::course_result::CourseResultState;
use state::decide::MusicDecideState;
use state::key_config::KeyConfigState;
use state::play::PlayState;
use state::result::ResultState;
use state::select::MusicSelectState;
use state::skin_config::SkinConfigState;
use timer_manager::TimerManager;

#[derive(Parser, Debug)]
#[command(name = "brs", about = "BMS player (Rust port of lr2oraja)")]
struct Args {
    /// Path to a BMS file to play directly (skips MusicSelect).
    #[arg(long)]
    bms: Option<PathBuf>,

    /// Path to database directory.
    #[arg(long, default_value = "db")]
    db_path: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    info!("brs starting");

    // Load BMS if specified
    let mut resource = PlayerResource::default();
    if let Some(bms_path) = &args.bms {
        info!(path = %bms_path.display(), "Loading BMS file");
        let model = bms_model::BmsDecoder::decode(bms_path)?;
        resource.play_mode = model.mode;
        resource.bms_dir = bms_path.parent().map(|p| p.to_path_buf());
        resource.bms_model = Some(model);
    }

    // Load config (use defaults for now — Phase 15-H adds file I/O)
    let config = bms_config::Config::default();
    let player_config = bms_config::PlayerConfig::default();

    // Open databases
    let database = match DatabaseManager::open(&args.db_path) {
        Ok(db) => {
            info!(path = %args.db_path.display(), "Database opened");
            Some(db)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to open database at {}: {}",
                args.db_path.display(),
                e
            );
            None
        }
    };

    // Build state registry
    let mut registry = StateRegistry::new(AppStateType::MusicSelect);
    registry.register(AppStateType::MusicSelect, Box::new(MusicSelectState::new()));
    registry.register(AppStateType::Decide, Box::new(MusicDecideState::new()));
    registry.register(AppStateType::Play, Box::new(PlayState::new()));
    registry.register(AppStateType::Result, Box::new(ResultState::new()));
    registry.register(
        AppStateType::CourseResult,
        Box::new(CourseResultState::new()),
    );
    registry.register(AppStateType::KeyConfig, Box::new(KeyConfigState::new()));
    registry.register(AppStateType::SkinConfig, Box::new(SkinConfigState::new()));

    // Shared game state for skin renderer
    let shared_state = Arc::new(RwLock::new(SharedGameState::default()));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "brs".to_string(),
                resolution: bevy::window::WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bms_render::plugin::BmsRenderPlugin)
        .insert_resource(BrsTimerManager(TimerManager::new()))
        .insert_resource(BrsPlayerResource(resource))
        .insert_resource(BrsConfig(config))
        .insert_resource(BrsPlayerConfig(player_config))
        .insert_resource(BrsStateRegistry(registry))
        .insert_resource(BrsSharedState(shared_state))
        .insert_resource(BrsDatabase(Arc::new(Mutex::new(database))))
        .insert_resource(BrsInputMapper(InputMapper::new()))
        .add_systems(Update, timer_update_system)
        .add_systems(Update, state_machine_system.after(timer_update_system))
        .add_systems(Update, state_sync_system.after(state_machine_system))
        .run();

    Ok(())
}

// Bevy resource wrappers (newtype to satisfy Resource trait)

#[derive(Resource)]
struct BrsTimerManager(TimerManager);

#[derive(Resource)]
struct BrsPlayerResource(PlayerResource);

#[derive(Resource)]
struct BrsConfig(bms_config::Config);

#[derive(Resource)]
struct BrsPlayerConfig(bms_config::PlayerConfig);

#[derive(Resource)]
struct BrsStateRegistry(StateRegistry);

#[derive(Resource)]
struct BrsSharedState(Arc<RwLock<SharedGameState>>);

/// Database wrapped in Mutex for Bevy's Send+Sync requirement
/// (rusqlite::Connection is not Sync).
#[derive(Resource)]
struct BrsDatabase(Arc<Mutex<Option<DatabaseManager>>>);

#[derive(Resource, Default)]
struct BrsInputMapper(InputMapper);

fn timer_update_system(mut timer: ResMut<BrsTimerManager>) {
    timer.0.update();
}

#[allow(clippy::too_many_arguments)] // Bevy system using dependency injection
fn state_machine_system(
    mut timer: ResMut<BrsTimerManager>,
    mut resource: ResMut<BrsPlayerResource>,
    config: Res<BrsConfig>,
    player_config: Res<BrsPlayerConfig>,
    mut registry: ResMut<BrsStateRegistry>,
    database: Res<BrsDatabase>,
    mut input_mapper: ResMut<BrsInputMapper>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut backend: Local<bevy_keyboard::BevyKeyboardBackend>,
) {
    backend.snapshot(&keyboard_input);
    let input_state = input_mapper.0.update(&*backend);
    // Lock database for this frame (states do synchronous DB access)
    let db_guard = database.0.lock().unwrap();
    let db_ref = db_guard.as_ref();
    let mut params = TickParams {
        timer: &mut timer.0,
        resource: &mut resource.0,
        config: &config.0,
        player_config: &player_config.0,
        keyboard_backend: Some(&*backend),
        database: db_ref,
        input_state: Some(&input_state),
    };
    registry.0.tick(&mut params);
}

fn state_sync_system(timer: Res<BrsTimerManager>, shared: Res<BrsSharedState>) {
    sync_timer_state(&timer.0, &shared.0);
}
