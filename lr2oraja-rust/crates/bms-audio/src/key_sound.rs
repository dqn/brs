/// Key sound processor for BGM autoplay.
///
/// Ports Java `KeySoundProcessor.java`.
/// Manages BG note autoplay using a timeline pointer approach.
use bms_model::{BgNote, BmsModel};

use crate::driver::AudioDriver;

/// Processes BG notes in time order, triggering audio playback.
pub struct KeySoundProcessor {
    /// BG notes sorted by time, with their timeline index.
    bg_notes: Vec<BgNote>,
    /// Current position in bg_notes.
    pointer: usize,
    /// BGM volume (0.0 - 1.0).
    volume: f32,
}

impl KeySoundProcessor {
    /// Create a new processor from a BMS model.
    pub fn new(model: &BmsModel, volume: f32) -> Self {
        let mut bg_notes = model.bg_notes.clone();
        bg_notes.sort_by_key(|n| n.time_us);
        Self {
            bg_notes,
            pointer: 0,
            volume,
        }
    }

    /// Set the starting time, advancing the pointer past already-elapsed notes.
    pub fn seek(&mut self, start_time_us: i64) {
        self.pointer = 0;
        while self.pointer < self.bg_notes.len()
            && self.bg_notes[self.pointer].time_us < start_time_us
        {
            self.pointer += 1;
        }
    }

    /// Update with current time, playing any BG notes that are due.
    ///
    /// Returns the number of notes played.
    pub fn update(&mut self, current_time_us: i64, driver: &mut dyn AudioDriver) -> usize {
        let mut played = 0;
        while self.pointer < self.bg_notes.len()
            && self.bg_notes[self.pointer].time_us <= current_time_us
        {
            driver.play_bg_note(&self.bg_notes[self.pointer], self.volume);
            self.pointer += 1;
            played += 1;
        }
        played
    }

    /// Set BGM volume.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    /// Check if all BG notes have been processed.
    pub fn is_finished(&self) -> bool {
        self.pointer >= self.bg_notes.len()
    }

    /// Get the time of the next BG note, or None if finished.
    pub fn next_time_us(&self) -> Option<i64> {
        self.bg_notes.get(self.pointer).map(|n| n.time_us)
    }

    /// Total number of BG notes.
    pub fn total_notes(&self) -> usize {
        self.bg_notes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::BgNote;

    fn make_model_with_bg_notes(times: &[i64]) -> BmsModel {
        let mut model = BmsModel::default();
        for &t in times {
            model.bg_notes.push(BgNote {
                wav_id: 1,
                time_us: t,
                micro_starttime: 0,
                micro_duration: 0,
            });
        }
        model
    }

    #[test]
    fn test_new_empty() {
        let model = BmsModel::default();
        let proc = KeySoundProcessor::new(&model, 1.0);
        assert_eq!(proc.total_notes(), 0);
        assert!(proc.is_finished());
    }

    #[test]
    fn test_seek() {
        let model = make_model_with_bg_notes(&[100_000, 200_000, 300_000, 400_000]);
        let mut proc = KeySoundProcessor::new(&model, 1.0);

        proc.seek(250_000);
        assert_eq!(proc.pointer, 2); // past 100k and 200k
        assert!(!proc.is_finished());
    }

    #[test]
    fn test_next_time() {
        let model = make_model_with_bg_notes(&[100_000, 200_000]);
        let proc = KeySoundProcessor::new(&model, 1.0);
        assert_eq!(proc.next_time_us(), Some(100_000));
    }
}
