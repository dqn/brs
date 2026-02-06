use std::collections::HashMap;

use anyhow::Result;

use crate::traits::audio::{AudioBackend, SoundId};

/// Manages keysound playback during gameplay.
/// Handles both BGM autoplay (notes on BGM lane) and player-triggered keysounds.
pub struct KeysoundProcessor {
    /// Mapping from WAV ID (BMS #WAV index) to loaded SoundId.
    wav_map: HashMap<u32, SoundId>,
    /// Volume for keysounds (0.0 - 1.0).
    keysound_volume: f32,
    /// Volume for BGM (0.0 - 1.0).
    bgm_volume: f32,
}

impl KeysoundProcessor {
    /// Create a new keysound processor.
    pub fn new() -> Self {
        Self {
            wav_map: HashMap::new(),
            keysound_volume: 1.0,
            bgm_volume: 1.0,
        }
    }

    /// Register a WAV ID to SoundId mapping.
    pub fn register_wav(&mut self, wav_id: u32, sound_id: SoundId) {
        self.wav_map.insert(wav_id, sound_id);
    }

    /// Get the SoundId for a WAV ID.
    pub fn get_sound(&self, wav_id: u32) -> Option<SoundId> {
        self.wav_map.get(&wav_id).copied()
    }

    /// Play a keysound triggered by player input.
    pub fn play_keysound<A: AudioBackend>(&self, backend: &mut A, wav_id: u32) -> Result<()> {
        if let Some(sound_id) = self.wav_map.get(&wav_id) {
            backend.set_volume(*sound_id, self.keysound_volume)?;
            backend.play(*sound_id)?;
        }
        Ok(())
    }

    /// Play a BGM note (autoplay lane).
    pub fn play_bgm<A: AudioBackend>(&self, backend: &mut A, wav_id: u32) -> Result<()> {
        if let Some(sound_id) = self.wav_map.get(&wav_id) {
            backend.set_volume(*sound_id, self.bgm_volume)?;
            backend.play(*sound_id)?;
        }
        Ok(())
    }

    /// Stop a sound by WAV ID.
    pub fn stop_sound<A: AudioBackend>(&self, backend: &mut A, wav_id: u32) -> Result<()> {
        if let Some(sound_id) = self.wav_map.get(&wav_id) {
            backend.stop(*sound_id)?;
        }
        Ok(())
    }

    /// Set keysound volume.
    pub fn set_keysound_volume(&mut self, volume: f32) {
        self.keysound_volume = volume.clamp(0.0, 1.0);
    }

    /// Set BGM volume.
    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);
    }

    /// Number of registered WAV mappings.
    pub fn wav_count(&self) -> usize {
        self.wav_map.len()
    }

    /// Clear all WAV mappings.
    pub fn clear(&mut self) {
        self.wav_map.clear();
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
    use std::path::Path;

    struct MockAudio {
        played: Vec<u64>,
        stopped: Vec<u64>,
        volumes: HashMap<u64, f32>,
    }

    impl MockAudio {
        fn new() -> Self {
            Self {
                played: Vec::new(),
                stopped: Vec::new(),
                volumes: HashMap::new(),
            }
        }
    }

    impl AudioBackend for MockAudio {
        fn load_sound(&mut self, _path: &Path) -> Result<SoundId> {
            Ok(SoundId(1))
        }
        fn load_sound_from_memory(&mut self, _data: &[u8], _ext: &str) -> Result<SoundId> {
            Ok(SoundId(1))
        }
        fn play(&mut self, id: SoundId) -> Result<()> {
            self.played.push(id.0);
            Ok(())
        }
        fn stop(&mut self, id: SoundId) -> Result<()> {
            self.stopped.push(id.0);
            Ok(())
        }
        fn set_pitch(&mut self, _id: SoundId, _semitones: f32) -> Result<()> {
            Ok(())
        }
        fn set_volume(&mut self, id: SoundId, volume: f32) -> Result<()> {
            self.volumes.insert(id.0, volume);
            Ok(())
        }
        fn dispose(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn register_and_play() {
        let mut proc = KeysoundProcessor::new();
        proc.register_wav(1, SoundId(10));
        proc.register_wav(2, SoundId(20));

        let mut audio = MockAudio::new();
        proc.play_keysound(&mut audio, 1).unwrap();
        assert_eq!(audio.played, vec![10]);
    }

    #[test]
    fn play_bgm() {
        let mut proc = KeysoundProcessor::new();
        proc.register_wav(5, SoundId(50));

        let mut audio = MockAudio::new();
        proc.play_bgm(&mut audio, 5).unwrap();
        assert_eq!(audio.played, vec![50]);
    }

    #[test]
    fn stop_sound() {
        let mut proc = KeysoundProcessor::new();
        proc.register_wav(1, SoundId(10));

        let mut audio = MockAudio::new();
        proc.stop_sound(&mut audio, 1).unwrap();
        assert_eq!(audio.stopped, vec![10]);
    }

    #[test]
    fn missing_wav_is_noop() {
        let proc = KeysoundProcessor::new();
        let mut audio = MockAudio::new();
        proc.play_keysound(&mut audio, 999).unwrap();
        assert!(audio.played.is_empty());
    }

    #[test]
    fn volume_settings() {
        let mut proc = KeysoundProcessor::new();
        proc.set_keysound_volume(0.5);
        proc.set_bgm_volume(0.8);
        proc.register_wav(1, SoundId(10));

        let mut audio = MockAudio::new();
        proc.play_keysound(&mut audio, 1).unwrap();
        assert!((audio.volumes[&10] - 0.5).abs() < f32::EPSILON);

        proc.play_bgm(&mut audio, 1).unwrap();
        assert!((audio.volumes[&10] - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn clear() {
        let mut proc = KeysoundProcessor::new();
        proc.register_wav(1, SoundId(10));
        assert_eq!(proc.wav_count(), 1);
        proc.clear();
        assert_eq!(proc.wav_count(), 0);
    }
}
