use super::AudioManager;
use crate::bms::Chart;

pub struct AudioScheduler {
    bgm_index: usize,
}

impl AudioScheduler {
    pub fn new() -> Self {
        Self { bgm_index: 0 }
    }

    pub fn reset(&mut self) {
        self.bgm_index = 0;
    }

    pub fn update(&mut self, chart: &Chart, audio: &mut AudioManager, current_time_ms: f64) {
        while self.bgm_index < chart.bgm_events.len() {
            let event = &chart.bgm_events[self.bgm_index];

            if event.time_ms <= current_time_ms {
                audio.play_bgm(event.keysound_id, event.time_ms);
                self.bgm_index += 1;
            } else {
                break;
            }
        }
    }
}

impl Default for AudioScheduler {
    fn default() -> Self {
        Self::new()
    }
}
