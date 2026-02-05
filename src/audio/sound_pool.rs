use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use kira::sound::static_sound::StaticSoundData;

/// Cache for loaded audio data.
pub struct SoundPool {
    sounds: HashMap<u16, StaticSoundData>,
    base_path: PathBuf,
    memory_usage: usize,
    max_memory: usize,
}

impl SoundPool {
    /// Create a new SoundPool with the given base path and memory limit.
    pub fn new(base_path: PathBuf, max_memory_mb: usize) -> Self {
        Self {
            sounds: HashMap::new(),
            base_path,
            memory_usage: 0,
            max_memory: max_memory_mb * 1024 * 1024,
        }
    }

    /// Load an audio file and cache it with the given WAV ID.
    pub fn load(&mut self, wav_id: u16, filename: &str) -> Result<()> {
        let path = self.resolve_audio_path(filename);
        let sound_data = StaticSoundData::from_file(&path)
            .map_err(|e| anyhow!("Failed to load audio {}: {}", path.display(), e))?;

        // Estimate memory usage (frames * 2 channels * 4 bytes per sample)
        let estimated_size = sound_data.num_frames() * 2 * 4;
        self.memory_usage += estimated_size;

        self.sounds.insert(wav_id, sound_data);
        Ok(())
    }

    /// Get a clone of the sound data for the given WAV ID.
    /// Cloning StaticSoundData is cheap as it uses Arc internally.
    pub fn get(&self, wav_id: u16) -> Option<StaticSoundData> {
        self.sounds.get(&wav_id).cloned()
    }

    /// Clear all cached sounds.
    pub fn clear(&mut self) {
        self.sounds.clear();
        self.memory_usage = 0;
    }

    /// Resolve the audio file path, trying different extensions if needed.
    fn resolve_audio_path(&self, filename: &str) -> PathBuf {
        let base = self.base_path.join(filename);
        if base.exists() {
            return base;
        }

        // Try with different extensions
        let stem = Path::new(filename)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        for ext in ["wav", "ogg", "flac", "mp3"] {
            let path = self.base_path.join(format!("{}.{}", stem, ext));
            if path.exists() {
                return path;
            }
        }

        // Return original path even if it doesn't exist
        base
    }

    /// Get current memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// Get maximum allowed memory in bytes.
    pub fn max_memory(&self) -> usize {
        self.max_memory
    }

    /// Get the number of cached sounds.
    pub fn len(&self) -> usize {
        self.sounds.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.sounds.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sound_pool() {
        let pool = SoundPool::new(PathBuf::from("."), 512);
        assert!(pool.is_empty());
        assert_eq!(pool.max_memory(), 512 * 1024 * 1024);
    }

    #[test]
    fn test_resolve_path_with_extension() {
        let pool = SoundPool::new(PathBuf::from("bms/bms-001"), 512);
        let path = pool.resolve_audio_path("test.ogg");
        assert!(path.to_string_lossy().contains("test"));
    }
}
