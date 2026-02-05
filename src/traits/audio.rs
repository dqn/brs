use anyhow::Result;

/// Handle for referencing loaded sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub u64);

/// Abstraction over audio backends.
/// Implementations: AudioDriver (kira), MockAudio (testing).
pub trait AudioBackend {
    fn load_sound(&mut self, path: &std::path::Path) -> Result<SoundId>;
    fn load_sound_from_memory(&mut self, data: &[u8], ext: &str) -> Result<SoundId>;

    fn play(&mut self, id: SoundId) -> Result<()>;
    fn stop(&mut self, id: SoundId) -> Result<()>;

    /// Set playback pitch in semitones (-12..=+12).
    fn set_pitch(&mut self, id: SoundId, semitones: f32) -> Result<()>;

    /// Set volume (0.0..=1.0).
    fn set_volume(&mut self, id: SoundId, volume: f32) -> Result<()>;

    fn dispose(&mut self) -> Result<()>;
}
