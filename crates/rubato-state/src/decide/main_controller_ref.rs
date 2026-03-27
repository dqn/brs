use rubato_core::system_sound_manager::SoundType;
use rubato_input::bms_player_input_processor::BMSPlayerInputProcessor;

/// Wrapper for MainController reference.
/// Delegates trait methods (change_state) to `Box<dyn MainControllerAccess>`.
/// Stores BMSPlayerInputProcessor locally (type not available on MainControllerAccess trait).
pub struct MainControllerRef {
    inner: Box<dyn rubato_types::main_controller_access::MainControllerAccess>,
    input_processor: BMSPlayerInputProcessor,
}

impl MainControllerRef {
    pub fn new(inner: Box<dyn rubato_types::main_controller_access::MainControllerAccess>) -> Self {
        let config = inner.config();
        let input_processor = BMSPlayerInputProcessor::new_without_midi(config);
        Self {
            inner,
            input_processor,
        }
    }

    pub fn config(&self) -> &rubato_types::config::Config {
        self.inner.config()
    }

    pub fn player_config(&self) -> &rubato_types::player_config::PlayerConfig {
        self.inner.player_config()
    }

    pub fn change_state(&mut self, state: rubato_types::main_state_type::MainStateType) {
        self.inner.change_state(state);
    }

    pub fn input_processor(&mut self) -> &mut BMSPlayerInputProcessor {
        &mut self.input_processor
    }

    pub fn sync_input_from(&mut self, input: &BMSPlayerInputProcessor) {
        self.input_processor.sync_runtime_state_from(input);
    }

    pub fn sync_input_back_to(&mut self, input: &mut BMSPlayerInputProcessor) {
        input.sync_runtime_state_from(&self.input_processor);
    }

    pub fn play_sound(&mut self, sound: &SoundType, loop_sound: bool) {
        self.inner.play_sound(sound, loop_sound);
    }

    pub fn update_audio_config(&self, audio: rubato_types::audio_config::AudioConfig) {
        self.inner.update_audio_config(audio);
    }

    pub fn offset_value(&self, id: i32) -> Option<&rubato_types::skin_offset::SkinOffset> {
        self.inner.offset_value(id)
    }

    pub fn play_audio_path(&mut self, path: &str, volume: f32, loop_play: bool) {
        self.inner.play_audio_path(path, volume, loop_play);
    }

    pub fn stop_audio_path(&mut self, path: &str) {
        self.inner.stop_audio_path(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubato_types::main_controller_access::NullMainController;

    #[test]
    fn test_main_controller_ref_new() {
        let mc = MainControllerRef::new(Box::new(NullMainController));
        assert_eq!(
            mc.config().display.window_width,
            rubato_types::config::Config::default().display.window_width,
        );
    }
}
