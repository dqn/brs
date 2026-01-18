use super::AudioManager;
use crate::bms::Chart;

pub struct AudioScheduler {
    bgm_index: usize,
    lookahead_ms: f64,
}

impl AudioScheduler {
    pub fn new() -> Self {
        Self {
            bgm_index: 0,
            lookahead_ms: 100.0,
        }
    }

    pub fn reset(&mut self) {
        self.bgm_index = 0;
    }

    pub fn update(
        &mut self,
        chart: &Chart,
        audio: &mut AudioManager,
        current_time_ms: f64,
        start_delay_ms: f64,
    ) {
        let schedule_until = current_time_ms + self.lookahead_ms;
        while self.bgm_index < chart.bgm_events.len() {
            let event = &chart.bgm_events[self.bgm_index];

            if event.time_ms <= schedule_until {
                audio.play_bgm_at(event.keysound_id, event.time_ms + start_delay_ms);
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
