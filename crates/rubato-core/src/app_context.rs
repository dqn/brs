use std::sync::atomic::AtomicBool;

use rubato_audio::audio_system::AudioSystem;

use crate::config::Config;
use crate::main_controller::{DatabaseState, IntegrationState, LifecycleState, SkinOffset};
use crate::player_config::PlayerConfig;
use crate::system_sound_manager::SystemSoundManager;
use crate::timer_manager::TimerManager;
use rubato_input::bms_player_input_processor::BMSPlayerInputProcessor;

/// Shared application context holding config, audio, input, timer, database,
/// display, integration, and lifecycle state. Extracted from `MainController`
/// to separate application-wide concerns from state-machine mechanics.
pub struct AppContext {
    // --- Config ---
    pub config: Config,
    pub player: PlayerConfig,

    // --- Audio ---
    pub audio: Option<AudioSystem>,
    pub sound: Option<SystemSoundManager>,
    pub loudness_analyzer: Option<rubato_audio::bms_loudness_analyzer::BMSLoudnessAnalyzer>,

    // --- Timer ---
    pub timer: TimerManager,

    // --- Input ---
    pub input: Option<BMSPlayerInputProcessor>,
    pub input_poll_quit: std::sync::Arc<AtomicBool>,

    // --- Database ---
    pub db: DatabaseState,

    // --- Display ---
    pub offset: Vec<SkinOffset>,
    pub showfps: bool,
    pub debug: bool,

    // --- Integration ---
    pub integration: IntegrationState,

    // --- Lifecycle ---
    pub lifecycle: LifecycleState,
    pub exit_requested: AtomicBool,
}
