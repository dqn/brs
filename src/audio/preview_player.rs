use std::path::Path;

use anyhow::Result;

use crate::traits::audio::{AudioBackend, SoundId};

/// Plays preview sounds on the song select screen.
/// Handles loading, fading, and switching between preview tracks.
pub struct PreviewPlayer {
    /// Currently playing preview sound.
    current: Option<SoundId>,
    /// Volume for preview playback.
    volume: f32,
}

impl PreviewPlayer {
    /// Create a new preview player.
    pub fn new() -> Self {
        Self {
            current: None,
            volume: 0.7,
        }
    }

    /// Start playing a preview file.
    /// Stops any currently playing preview first.
    pub fn play<A: AudioBackend>(&mut self, backend: &mut A, path: &Path) -> Result<()> {
        self.stop(backend)?;
        let id = backend.load_sound(path)?;
        backend.set_volume(id, self.volume)?;
        backend.play(id)?;
        self.current = Some(id);
        Ok(())
    }

    /// Stop the current preview.
    pub fn stop<A: AudioBackend>(&mut self, backend: &mut A) -> Result<()> {
        if let Some(id) = self.current.take() {
            backend.stop(id)?;
        }
        Ok(())
    }

    /// Set preview volume.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Whether a preview is currently loaded.
    pub fn is_active(&self) -> bool {
        self.current.is_some()
    }
}

impl Default for PreviewPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockAudio {
        next_id: u64,
        playing: Vec<u64>,
        stopped: Vec<u64>,
        volumes: HashMap<u64, f32>,
    }

    impl MockAudio {
        fn new() -> Self {
            Self {
                next_id: 1,
                playing: Vec::new(),
                stopped: Vec::new(),
                volumes: HashMap::new(),
            }
        }
    }

    impl AudioBackend for MockAudio {
        fn load_sound(&mut self, _path: &Path) -> Result<SoundId> {
            let id = self.next_id;
            self.next_id += 1;
            Ok(SoundId(id))
        }
        fn load_sound_from_memory(&mut self, _data: &[u8], _ext: &str) -> Result<SoundId> {
            let id = self.next_id;
            self.next_id += 1;
            Ok(SoundId(id))
        }
        fn play(&mut self, id: SoundId) -> Result<()> {
            self.playing.push(id.0);
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
    fn play_and_stop() {
        let mut player = PreviewPlayer::new();
        let mut audio = MockAudio::new();

        player.play(&mut audio, Path::new("/test.ogg")).unwrap();
        assert!(player.is_active());
        assert_eq!(audio.playing.len(), 1);

        player.stop(&mut audio).unwrap();
        assert!(!player.is_active());
        assert_eq!(audio.stopped.len(), 1);
    }

    #[test]
    fn switching_preview_stops_previous() {
        let mut player = PreviewPlayer::new();
        let mut audio = MockAudio::new();

        player.play(&mut audio, Path::new("/a.ogg")).unwrap();
        player.play(&mut audio, Path::new("/b.ogg")).unwrap();

        // First sound should have been stopped before playing second
        assert_eq!(audio.stopped.len(), 1);
        assert_eq!(audio.playing.len(), 2);
    }

    #[test]
    fn volume() {
        let mut player = PreviewPlayer::new();
        player.set_volume(0.5);
        let mut audio = MockAudio::new();
        player.play(&mut audio, Path::new("/test.ogg")).unwrap();
        assert!((audio.volumes[&1] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stop_when_nothing_playing() {
        let mut player = PreviewPlayer::new();
        let mut audio = MockAudio::new();
        player.stop(&mut audio).unwrap(); // Should not panic
        assert!(!player.is_active());
    }
}
