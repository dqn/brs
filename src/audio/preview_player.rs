use std::path::Path;

use anyhow::Result;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::track::{TrackBuilder, TrackHandle};
use kira::{AudioManager, AudioManagerSettings, Capacities, Decibels, DefaultBackend, Tween};

/// Simple audio player for preview playback in the select screen.
pub struct PreviewPlayer {
    #[allow(dead_code)]
    manager: AudioManager<DefaultBackend>,
    track: TrackHandle,
    current: Option<StaticSoundHandle>,
    volume: f64,
}

/// Convert linear amplitude (0.0-1.0) to decibels.
fn amplitude_to_db(amplitude: f64) -> Decibels {
    if amplitude <= 0.0 {
        return Decibels::SILENCE;
    }
    Decibels(20.0 * amplitude.log10() as f32)
}

impl PreviewPlayer {
    /// Create a new preview player.
    pub fn new() -> Result<Self> {
        let settings = AudioManagerSettings {
            capacities: Capacities {
                sub_track_capacity: 4,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut manager = AudioManager::<DefaultBackend>::new(settings)
            .map_err(|e| anyhow::anyhow!("Failed to create AudioManager: {}", e))?;
        let track = manager
            .add_sub_track(TrackBuilder::new())
            .map_err(|e| anyhow::anyhow!("Failed to create preview track: {}", e))?;

        Ok(Self {
            manager,
            track,
            current: None,
            volume: 0.7,
        })
    }

    /// Set preview volume.
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Play a preview audio file in a loop.
    pub fn play(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            self.stop();
            return Ok(());
        }

        self.stop();
        let sound_data = StaticSoundData::from_file(path)
            .map_err(|e| anyhow::anyhow!("Failed to load preview audio: {}", e))?;
        let settings = StaticSoundSettings::new()
            .volume(amplitude_to_db(self.volume))
            .loop_region(..);
        let handle = self
            .track
            .play(sound_data.with_settings(settings))
            .map_err(|e| anyhow::anyhow!("Failed to play preview audio: {}", e))?;
        self.current = Some(handle);
        Ok(())
    }

    /// Stop the current preview playback.
    pub fn stop(&mut self) {
        if let Some(mut handle) = self.current.take() {
            handle.stop(Tween::default());
        }
    }
}
