use crate::audio::AudioDriver;
use anyhow::Result;

/// A BGM event representing a keysound to play at a specific time.
#[derive(Debug, Clone)]
pub struct BgmEvent {
    /// Time in milliseconds when the keysound should play.
    pub time_ms: f64,
    /// WAV ID of the keysound.
    pub wav_id: u16,
}

impl BgmEvent {
    /// Create a new BGM event.
    pub fn new(time_ms: f64, wav_id: u16) -> Self {
        Self { time_ms, wav_id }
    }
}

/// Processes keysound playback during gameplay.
pub struct KeysoundProcessor {
    /// BGM events sorted by time.
    bgm_events: Vec<BgmEvent>,
    /// Next BGM event index to process.
    next_bgm_index: usize,
    /// Volume multiplier for all keysounds.
    volume: f64,
}

impl KeysoundProcessor {
    /// Create a new KeysoundProcessor.
    pub fn new() -> Self {
        Self {
            bgm_events: Vec::new(),
            next_bgm_index: 0,
            volume: 1.0,
        }
    }

    /// Load BGM events and sort them by time.
    pub fn load_bgm_events(&mut self, mut events: Vec<BgmEvent>) {
        events.sort_by(|a, b| {
            a.time_ms
                .partial_cmp(&b.time_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.bgm_events = events;
        self.next_bgm_index = 0;
    }

    /// Update and play any BGM keysounds that should play at the current time.
    pub fn update(&mut self, audio_driver: &mut AudioDriver, current_time_ms: f64) -> Result<()> {
        while self.next_bgm_index < self.bgm_events.len() {
            let event = &self.bgm_events[self.next_bgm_index];
            if event.time_ms <= current_time_ms {
                audio_driver.play_keysound(event.wav_id, self.volume)?;
                self.next_bgm_index += 1;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Play a keysound triggered by player input.
    pub fn play_player_keysound(&self, audio_driver: &mut AudioDriver, wav_id: u16) -> Result<()> {
        audio_driver.play_keysound(wav_id, self.volume)
    }

    /// Play a keysound with panning based on lane position.
    /// pan: -1.0 (left) to 1.0 (right)
    pub fn play_player_keysound_with_pan(
        &self,
        audio_driver: &mut AudioDriver,
        wav_id: u16,
        pan: f64,
    ) -> Result<()> {
        // kira uses 0.0-1.0 for panning, convert from -1.0..1.0
        let kira_pan = (pan + 1.0) / 2.0;
        audio_driver.play_keysound_with_pan(wav_id, self.volume, kira_pan)
    }

    /// Set the volume multiplier.
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get the current volume multiplier.
    pub fn volume(&self) -> f64 {
        self.volume
    }

    /// Reset playback to the beginning.
    pub fn reset(&mut self) {
        self.next_bgm_index = 0;
    }

    /// Seek to a specific time position.
    pub fn seek(&mut self, time_ms: f64) {
        self.next_bgm_index = self
            .bgm_events
            .iter()
            .position(|e| e.time_ms > time_ms)
            .unwrap_or(self.bgm_events.len());
    }

    /// Get the total number of BGM events.
    pub fn bgm_event_count(&self) -> usize {
        self.bgm_events.len()
    }

    /// Get the index of the next BGM event to play.
    pub fn next_bgm_index(&self) -> usize {
        self.next_bgm_index
    }
}

impl Default for KeysoundProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keysound_processor_seek() {
        let mut processor = KeysoundProcessor::new();
        processor.load_bgm_events(vec![
            BgmEvent::new(100.0, 1),
            BgmEvent::new(200.0, 2),
            BgmEvent::new(300.0, 3),
        ]);

        processor.seek(150.0);
        assert_eq!(processor.next_bgm_index(), 1);

        processor.seek(0.0);
        assert_eq!(processor.next_bgm_index(), 0);

        processor.seek(500.0);
        assert_eq!(processor.next_bgm_index(), 3);
    }

    #[test]
    fn test_keysound_processor_reset() {
        let mut processor = KeysoundProcessor::new();
        processor.load_bgm_events(vec![BgmEvent::new(100.0, 1), BgmEvent::new(200.0, 2)]);

        processor.seek(150.0);
        assert_eq!(processor.next_bgm_index(), 1);

        processor.reset();
        assert_eq!(processor.next_bgm_index(), 0);
    }

    #[test]
    fn test_volume() {
        let mut processor = KeysoundProcessor::new();
        processor.set_volume(0.5);
        assert!((processor.volume() - 0.5).abs() < f64::EPSILON);

        processor.set_volume(1.5);
        assert!((processor.volume() - 1.0).abs() < f64::EPSILON);

        processor.set_volume(-0.5);
        assert!((processor.volume() - 0.0).abs() < f64::EPSILON);
    }
}
