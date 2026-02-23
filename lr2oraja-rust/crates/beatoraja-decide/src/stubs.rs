// Stubs for external dependencies not yet available as proper imports.

use beatoraja_audio::audio_driver::AudioDriver;
use beatoraja_core::config::Config;
use beatoraja_core::player_config::PlayerConfig;
use beatoraja_core::system_sound_manager::SoundType;

// InputProcessorStub: replaced by pub use from beatoraja-input (Phase 18e-11)
pub use beatoraja_input::bms_player_input_processor::BMSPlayerInputProcessor;

// ControlKeysStub: replaced by pub use from beatoraja-input (Phase 18e-11)
pub use beatoraja_input::keyboard_input_processor::ControlKeys;

// MainControllerAccess: real trait from beatoraja-types (Phase 41b)
pub use beatoraja_types::main_controller_access::{MainControllerAccess, NullMainController};

/// Wrapper for MainController reference.
/// Delegates trait methods (change_state) to `Box<dyn MainControllerAccess>`.
/// Retains local stubs for get_input_processor
/// (types not available on MainControllerAccess trait).
/// AudioDriver is stored directly (Phase 41c) — not on MainControllerAccess trait.
pub struct MainControllerRef {
    inner: Box<dyn MainControllerAccess>,
    audio: Option<Box<dyn AudioDriver>>,
}

impl MainControllerRef {
    pub fn new(inner: Box<dyn MainControllerAccess>) -> Self {
        Self { inner, audio: None }
    }

    pub fn with_audio(inner: Box<dyn MainControllerAccess>, audio: Box<dyn AudioDriver>) -> Self {
        Self {
            inner,
            audio: Some(audio),
        }
    }

    pub fn change_state(&mut self, state: beatoraja_types::main_state_type::MainStateType) {
        self.inner.change_state(state);
    }

    pub fn get_input_processor(&mut self) -> &mut BMSPlayerInputProcessor {
        log::warn!("not yet implemented: MainController.getInputProcessor");
        // Leak a boxed value to get a &'static mut reference - stub only
        Box::leak(Box::new(BMSPlayerInputProcessor::new(
            &Config::default(),
            &PlayerConfig::default(),
        )))
    }

    pub fn get_audio_processor_mut(&mut self) -> Option<&mut dyn AudioDriver> {
        self.audio
            .as_mut()
            .map(|b| &mut **b as &mut dyn AudioDriver)
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

    #[cfg(test)]
    pub fn with_values(input: i32, scene: i32, fadeout: i32) -> Self {
        Self {
            input,
            scene,
            fadeout,
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

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::bms_model::BMSModel;
    use bms_model::note::Note;

    /// Mock AudioDriver for testing.
    struct MockAudioDriver {
        global_pitch: f32,
    }

    impl MockAudioDriver {
        fn new() -> Self {
            Self { global_pitch: 1.0 }
        }
    }

    impl AudioDriver for MockAudioDriver {
        fn play_path(&mut self, _path: &str, _volume: f32, _loop_play: bool) {}
        fn set_volume_path(&mut self, _path: &str, _volume: f32) {}
        fn is_playing_path(&self, _path: &str) -> bool {
            false
        }
        fn stop_path(&mut self, _path: &str) {}
        fn dispose_path(&mut self, _path: &str) {}
        fn set_model(&mut self, _model: &BMSModel) {}
        fn set_additional_key_sound(&mut self, _judge: i32, _fast: bool, _path: Option<&str>) {}
        fn abort(&mut self) {}
        fn get_progress(&self) -> f32 {
            1.0
        }
        fn play_note(&mut self, _n: &Note, _volume: f32, _pitch: i32) {}
        fn play_judge(&mut self, _judge: i32, _fast: bool) {}
        fn stop_note(&mut self, _n: Option<&Note>) {}
        fn set_volume_note(&mut self, _n: &Note, _volume: f32) {}
        fn set_global_pitch(&mut self, pitch: f32) {
            self.global_pitch = pitch;
        }
        fn get_global_pitch(&self) -> f32 {
            self.global_pitch
        }
        fn dispose_old(&mut self) {}
        fn dispose(&mut self) {}
    }

    #[test]
    fn test_main_controller_ref_new_has_no_audio() {
        let mut mc = MainControllerRef::new(Box::new(NullMainController));
        assert!(mc.get_audio_processor_mut().is_none());
    }

    #[test]
    fn test_main_controller_ref_with_audio_has_audio() {
        let mut mc = MainControllerRef::with_audio(
            Box::new(NullMainController),
            Box::new(MockAudioDriver::new()),
        );
        assert!(mc.get_audio_processor_mut().is_some());
    }

    #[test]
    fn test_main_controller_ref_audio_set_global_pitch() {
        let mut mc = MainControllerRef::with_audio(
            Box::new(NullMainController),
            Box::new(MockAudioDriver::new()),
        );
        if let Some(audio) = mc.get_audio_processor_mut() {
            audio.set_global_pitch(1.0);
            assert_eq!(audio.get_global_pitch(), 1.0);
        } else {
            panic!("expected audio processor to be present");
        }
    }

    #[test]
    fn test_main_controller_ref_audio_stop_note() {
        let mut mc = MainControllerRef::with_audio(
            Box::new(NullMainController),
            Box::new(MockAudioDriver::new()),
        );
        if let Some(audio) = mc.get_audio_processor_mut() {
            audio.stop_note(None);
        }
    }
}
