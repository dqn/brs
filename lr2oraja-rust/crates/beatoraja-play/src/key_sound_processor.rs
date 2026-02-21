use bms_model::bms_model::BMSModel;

/// Key sound processor for BG lane playback
pub struct KeySoundProcessor {
    auto_thread_stop: bool,
}

impl Default for KeySoundProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl KeySoundProcessor {
    pub fn new() -> Self {
        KeySoundProcessor {
            auto_thread_stop: false,
        }
    }

    pub fn start_bg_play(&mut self, _model: &BMSModel, _starttime: i64) {
        // TODO: Phase 7+ dependency - requires BMSPlayer, AudioDriver, TimerManager
        // In Java, this starts an AutoplayThread for BG lane playback
        self.auto_thread_stop = false;
    }

    pub fn stop_bg_play(&mut self) {
        self.auto_thread_stop = true;
    }
}
