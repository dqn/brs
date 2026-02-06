use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};
use kira::manager::backend::DefaultBackend;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::PlaybackState;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};

use crate::traits::audio::{AudioBackend, SoundId};

/// Audio driver backed by kira for low-latency playback.
pub struct AudioDriver {
    manager: AudioManager,
    /// Loaded sound data keyed by SoundId.
    sounds: HashMap<u64, StaticSoundData>,
    /// Active playback handles.
    handles: HashMap<u64, StaticSoundHandle>,
    /// Next sound ID to assign.
    next_id: u64,
}

impl AudioDriver {
    /// Create a new audio driver.
    pub fn new() -> Result<Self> {
        let settings = AudioManagerSettings::default();
        let manager = AudioManager::<DefaultBackend>::new(settings)
            .map_err(|e| anyhow!("Failed to create audio manager: {e}"))?;
        Ok(Self {
            manager,
            sounds: HashMap::new(),
            handles: HashMap::new(),
            next_id: 1,
        })
    }

    /// Check if a sound is currently playing.
    pub fn is_playing(&self, id: SoundId) -> bool {
        self.handles
            .get(&id.0)
            .is_some_and(|h| h.state() == PlaybackState::Playing)
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl AudioBackend for AudioDriver {
    fn load_sound(&mut self, path: &Path) -> Result<SoundId> {
        let data = StaticSoundData::from_file(path)
            .map_err(|e| anyhow!("Failed to load sound {}: {e}", path.display()))?;
        let id = self.alloc_id();
        self.sounds.insert(id, data);
        Ok(SoundId(id))
    }

    fn load_sound_from_memory(&mut self, data: &[u8], ext: &str) -> Result<SoundId> {
        let cursor = std::io::Cursor::new(data.to_vec());
        let sound_data = match ext.to_lowercase().as_str() {
            "wav" | "wave" => StaticSoundData::from_cursor(cursor),
            "ogg" => StaticSoundData::from_cursor(cursor),
            "mp3" => StaticSoundData::from_cursor(cursor),
            "flac" => StaticSoundData::from_cursor(cursor),
            _ => return Err(anyhow!("Unsupported audio format: {ext}")),
        }
        .map_err(|e| anyhow!("Failed to load sound from memory ({ext}): {e}"))?;

        let id = self.alloc_id();
        self.sounds.insert(id, sound_data);
        Ok(SoundId(id))
    }

    fn play(&mut self, id: SoundId) -> Result<()> {
        let data = self
            .sounds
            .get(&id.0)
            .ok_or_else(|| anyhow!("Sound not found: {:?}", id))?
            .clone();
        let handle = self
            .manager
            .play(data)
            .map_err(|e| anyhow!("Failed to play sound: {e}"))?;
        self.handles.insert(id.0, handle);
        Ok(())
    }

    fn stop(&mut self, id: SoundId) -> Result<()> {
        if let Some(mut handle) = self.handles.remove(&id.0) {
            handle.stop(Default::default());
        }
        Ok(())
    }

    fn set_pitch(&mut self, id: SoundId, semitones: f32) -> Result<()> {
        if let Some(handle) = self.handles.get_mut(&id.0) {
            // Convert semitones to playback rate: rate = 2^(semitones/12)
            let rate = 2.0_f64.powf(semitones as f64 / 12.0);
            handle.set_playback_rate(rate, Default::default());
        }
        Ok(())
    }

    fn set_volume(&mut self, id: SoundId, volume: f32) -> Result<()> {
        if let Some(handle) = self.handles.get_mut(&id.0) {
            handle.set_volume(volume as f64, Default::default());
        }
        Ok(())
    }

    fn dispose(&mut self) -> Result<()> {
        for (_, mut handle) in self.handles.drain() {
            handle.stop(Default::default());
        }
        self.sounds.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AudioDriver tests require audio hardware, so we test the trait interface
    // with basic checks.

    #[test]
    fn sound_id_equality() {
        assert_eq!(SoundId(1), SoundId(1));
        assert_ne!(SoundId(1), SoundId(2));
    }
}
