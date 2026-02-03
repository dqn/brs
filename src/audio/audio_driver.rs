use crate::audio::audio_config::AudioConfig;
use crate::audio::sound_pool::SoundPool;
use crate::model::BMSModel;
use anyhow::{Result, anyhow};
use kira::sound::PlaybackState;
use kira::sound::static_sound::{StaticSoundHandle, StaticSoundSettings};
use kira::track::{TrackBuilder, TrackHandle};
use kira::{AudioManager, AudioManagerSettings, Capacities, Decibels, DefaultBackend, Tween};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::warn;

/// Audio driver using kira for low-latency audio playback.
pub struct AudioDriver {
    #[allow(dead_code)] // Must be kept alive for audio to work
    manager: AudioManager,
    keysound_track: TrackHandle,
    bgm_track: TrackHandle,
    sound_pool: SoundPool,
    active_sounds: Vec<StaticSoundHandle>,
    config: AudioConfig,
}

/// Convert linear amplitude (0.0-1.0) to decibels.
fn amplitude_to_db(amplitude: f64) -> Decibels {
    if amplitude <= 0.0 {
        return Decibels::SILENCE;
    }
    Decibels(20.0 * amplitude.log10() as f32)
}

impl std::fmt::Debug for AudioDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDriver")
            .field("active_sounds_count", &self.active_sounds.len())
            .field("loaded_sounds_count", &self.sound_pool.len())
            .field("config", &self.config)
            .finish()
    }
}

impl AudioDriver {
    /// Create a new AudioDriver with the given configuration.
    pub fn new(config: AudioConfig) -> Result<Self> {
        let settings = AudioManagerSettings {
            capacities: Capacities {
                sub_track_capacity: 16,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut manager = AudioManager::<DefaultBackend>::new(settings)
            .map_err(|e| anyhow!("Failed to create AudioManager: {}", e))?;

        let keysound_track = manager
            .add_sub_track(TrackBuilder::new())
            .map_err(|e| anyhow!("Failed to create keysound track: {}", e))?;

        let bgm_track = manager
            .add_sub_track(TrackBuilder::new())
            .map_err(|e| anyhow!("Failed to create BGM track: {}", e))?;

        Ok(Self {
            manager,
            keysound_track,
            bgm_track,
            sound_pool: SoundPool::new(std::path::PathBuf::new(), config.max_memory_mb),
            active_sounds: Vec::with_capacity(256),
            config,
        })
    }

    /// Load all sounds from a BMS model.
    pub fn load_sounds(&mut self, model: &BMSModel, bms_dir: &Path) -> Result<LoadProgress> {
        self.sound_pool = SoundPool::new(bms_dir.to_path_buf(), self.config.max_memory_mb);

        let total = model.wav_files.len();
        let loaded = Arc::new(AtomicUsize::new(0));

        for (wav_id, filename) in &model.wav_files {
            if let Err(e) = self.sound_pool.load(*wav_id, filename) {
                warn!("Failed to load WAV {}: {}", wav_id, e);
            }
            loaded.fetch_add(1, Ordering::SeqCst);
        }

        Ok(LoadProgress { loaded, total })
    }

    /// Play a keysound with the given WAV ID and volume.
    pub fn play_keysound(&mut self, wav_id: u16, volume: f64) -> Result<()> {
        if let Some(sound_data) = self.sound_pool.get(wav_id) {
            let effective_volume = volume * self.config.keysound_volume * self.config.master_volume;
            let settings = StaticSoundSettings::new().volume(amplitude_to_db(effective_volume));

            let handle = self
                .keysound_track
                .play(sound_data.with_settings(settings))
                .map_err(|e| anyhow!("Failed to play keysound: {}", e))?;

            self.active_sounds.push(handle);
            self.cleanup_finished_sounds();
        }
        Ok(())
    }

    /// Play a keysound with panning.
    pub fn play_keysound_with_pan(&mut self, wav_id: u16, volume: f64, pan: f64) -> Result<()> {
        if let Some(sound_data) = self.sound_pool.get(wav_id) {
            let effective_volume = volume * self.config.keysound_volume * self.config.master_volume;
            let settings = StaticSoundSettings::new()
                .volume(amplitude_to_db(effective_volume))
                .panning(pan as f32);

            let handle = self
                .keysound_track
                .play(sound_data.with_settings(settings))
                .map_err(|e| anyhow!("Failed to play keysound: {}", e))?;

            self.active_sounds.push(handle);
            self.cleanup_finished_sounds();
        }
        Ok(())
    }

    /// Set the keysound volume.
    pub fn set_keysound_volume(&mut self, volume: f64) {
        self.config.keysound_volume = volume.clamp(0.0, 1.0);
        self.keysound_track.set_volume(
            amplitude_to_db(self.config.keysound_volume * self.config.master_volume),
            Tween::default(),
        );
    }

    /// Set the BGM volume.
    pub fn set_bgm_volume(&mut self, volume: f64) {
        self.config.bgm_volume = volume.clamp(0.0, 1.0);
        self.bgm_track.set_volume(
            amplitude_to_db(self.config.bgm_volume * self.config.master_volume),
            Tween::default(),
        );
    }

    /// Clean up finished sounds to free resources.
    fn cleanup_finished_sounds(&mut self) {
        self.active_sounds
            .retain(|handle| handle.state() != PlaybackState::Stopped);

        // Limit active sounds to prevent resource exhaustion
        if self.active_sounds.len() > 256 {
            let to_remove = self.active_sounds.len() - 256;
            for mut handle in self.active_sounds.drain(0..to_remove) {
                handle.stop(Tween::default());
            }
        }
    }

    /// Get the number of currently active sounds.
    pub fn active_sound_count(&self) -> usize {
        self.active_sounds.len()
    }

    /// Stop all currently playing sounds without clearing the cache.
    /// 再生中の音をすべて停止し、キャッシュは保持する。
    pub fn stop_all(&mut self) {
        for mut handle in self.active_sounds.drain(..) {
            handle.stop(Tween::default());
        }
    }

    /// Stop all sounds and clear the cache.
    pub fn clear(&mut self) {
        for mut handle in self.active_sounds.drain(..) {
            handle.stop(Tween::default());
        }
        self.sound_pool.clear();
    }

    /// Get the number of loaded sounds.
    pub fn loaded_sound_count(&self) -> usize {
        self.sound_pool.len()
    }
}

/// Progress tracker for sound loading.
pub struct LoadProgress {
    loaded: Arc<AtomicUsize>,
    total: usize,
}

impl LoadProgress {
    /// Get the loading progress as a fraction (0.0 - 1.0).
    pub fn progress(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.loaded.load(Ordering::SeqCst) as f32 / self.total as f32
    }

    /// Check if loading is complete.
    pub fn is_complete(&self) -> bool {
        self.loaded.load(Ordering::SeqCst) >= self.total
    }

    /// Get the number of loaded sounds.
    pub fn loaded(&self) -> usize {
        self.loaded.load(Ordering::SeqCst)
    }

    /// Get the total number of sounds to load.
    pub fn total(&self) -> usize {
        self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_driver_init() {
        let config = AudioConfig::default();
        let driver = AudioDriver::new(config);
        assert!(driver.is_ok());
    }

    #[test]
    fn test_load_progress() {
        let progress = LoadProgress {
            loaded: Arc::new(AtomicUsize::new(5)),
            total: 10,
        };
        assert!((progress.progress() - 0.5).abs() < f32::EPSILON);
        assert!(!progress.is_complete());
    }
}
