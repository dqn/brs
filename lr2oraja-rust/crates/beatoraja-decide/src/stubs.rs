// Stubs for external dependencies not yet available as proper imports.

use beatoraja_core::config::Config;
use beatoraja_core::main_state::MainStateType;
use beatoraja_core::player_config::PlayerConfig;
use beatoraja_core::system_sound_manager::SoundType;

// InputProcessorStub: replaced by pub use from beatoraja-input (Phase 18e-11)
pub use beatoraja_input::bms_player_input_processor::BMSPlayerInputProcessor;

// ControlKeysStub: replaced by pub use from beatoraja-input (Phase 18e-11)
pub use beatoraja_input::keyboard_input_processor::ControlKeys;

/// Stub for MainController reference.
/// Retained: get_input_processor/get_audio_processor are crate-specific and not on MainControllerAccess trait.
/// MainControllerAccess trait impl removed (unused — MusicDecide calls methods on concrete type).
pub struct MainControllerRef;

impl MainControllerRef {
    pub fn change_state(&mut self, _state: MainStateType) {
        log::warn!("not yet implemented: MainController.changeState");
    }

    pub fn get_input_processor(&mut self) -> &mut BMSPlayerInputProcessor {
        log::warn!("not yet implemented: MainController.getInputProcessor");
        // Leak a boxed value to get a &'static mut reference - stub only
        Box::leak(Box::new(BMSPlayerInputProcessor::new(
            &Config::default(),
            &PlayerConfig::default(),
        )))
    }

    pub fn get_audio_processor(&self) -> &AudioProcessorStub {
        log::warn!("not yet implemented: MainController.getAudioProcessor");
        static DEFAULT: AudioProcessorStub = AudioProcessorStub;
        &DEFAULT
    }
}

/// Stub for AudioProcessor reference
pub struct AudioProcessorStub;

impl AudioProcessorStub {
    pub fn set_global_pitch(&self, _pitch: f32) {
        log::warn!("not yet implemented: AudioProcessor.setGlobalPitch");
    }
}

/// PlayerResourceAccess — re-exported from beatoraja-types (Phase 18e-2)
pub use beatoraja_types::player_resource_access::PlayerResourceAccess;

/// NullPlayerResource — re-exported from beatoraja-types for default construction
pub use beatoraja_types::player_resource_access::NullPlayerResource;

/// Stub for Skin (base class for MusicDecideSkin)
pub struct SkinStub {
    input: i32,
    scene: i32,
    fadeout: i32,
}

impl SkinStub {
    pub fn new() -> Self {
        Self {
            input: 0,
            scene: 0,
            fadeout: 0,
        }
    }

    pub fn get_input(&self) -> i32 {
        self.input
    }

    pub fn get_scene(&self) -> i32 {
        self.scene
    }

    pub fn get_fadeout(&self) -> i32 {
        self.fadeout
    }
}

impl Default for SkinStub {
    fn default() -> Self {
        Self::new()
    }
}

/// Stub for load_skin function
pub fn load_skin(_skin_type: beatoraja_skin::skin_type::SkinType) -> Option<SkinStub> {
    log::warn!("not yet implemented: SkinLoader.load");
    None
}

/// Stub for play sound (MainState.play delegates to MainController.getSoundManager())
pub fn play_sound(_sound: SoundType) {
    log::warn!("not yet implemented: MainController.getSoundManager().play()");
}
