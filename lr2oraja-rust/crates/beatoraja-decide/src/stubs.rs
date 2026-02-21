// Stubs for external dependencies not yet available as proper imports.
// These will be replaced with real imports as the corresponding crates are translated.

use beatoraja_core::main_state::MainStateType;
use beatoraja_core::system_sound_manager::SoundType;

/// Stub for MainController reference.
/// In Java, MainState holds `public final MainController main`.
/// Full MainController is in beatoraja-core but many methods are still stubbed.
pub struct MainControllerRef;

impl MainControllerRef {
    pub fn change_state(&mut self, _state: MainStateType) {
        todo!("Phase 7+ dependency: MainController.changeState")
    }

    pub fn get_input_processor(&self) -> &InputProcessorStub {
        todo!("Phase 7+ dependency: MainController.getInputProcessor")
    }

    pub fn get_audio_processor(&self) -> &AudioProcessorStub {
        todo!("Phase 7+ dependency: MainController.getAudioProcessor")
    }
}

/// Stub for BMSPlayerInputProcessor reference
pub struct InputProcessorStub;

impl InputProcessorStub {
    pub fn get_key_state(&self, _id: i32) -> bool {
        false
    }

    pub fn is_control_key_pressed(&self, _key: ControlKeysStub) -> bool {
        false
    }

    pub fn start_pressed(&self) -> bool {
        false
    }

    pub fn is_select_pressed(&self) -> bool {
        false
    }
}

/// Stub for ControlKeys enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKeysStub {
    Enter,
    Escape,
}

/// Stub for AudioProcessor reference
pub struct AudioProcessorStub;

impl AudioProcessorStub {
    pub fn set_global_pitch(&self, _pitch: f32) {
        todo!("Phase 7+ dependency: AudioProcessor.setGlobalPitch")
    }
}

/// Stub for PlayerResource reference
pub struct PlayerResourceRef;

impl PlayerResourceRef {
    pub fn set_org_gauge_option(&mut self, _gauge: i32) {
        // stub
    }

    pub fn get_player_config(&self) -> &PlayerConfigRef {
        todo!("Phase 7+ dependency: PlayerResource.getPlayerConfig")
    }
}

/// Stub for PlayerConfig reference
pub struct PlayerConfigRef {
    pub gauge: i32,
}

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

/// Stub for SkinHeader
#[derive(Clone, Debug, Default)]
pub struct SkinHeaderStub;

/// Stub for load_skin function
pub fn load_skin(_skin_type: beatoraja_skin::skin_type::SkinType) -> Option<SkinStub> {
    todo!("Phase 7+ dependency: SkinLoader.load")
}

/// Stub for play sound (MainState.play delegates to MainController.getSoundManager())
pub fn play_sound(_sound: SoundType) {
    todo!("Phase 7+ dependency: MainController.getSoundManager().play()")
}
