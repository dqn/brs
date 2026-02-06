use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::traits::audio::{AudioBackend, SoundId};

/// Caches loaded sounds by their file path to avoid duplicate loading.
/// Wraps an AudioBackend and provides deduplication.
pub struct SoundPool<A: AudioBackend> {
    backend: A,
    /// Cache from canonical path string to SoundId.
    cache: HashMap<String, SoundId>,
}

impl<A: AudioBackend> SoundPool<A> {
    /// Create a new SoundPool wrapping the given backend.
    pub fn new(backend: A) -> Self {
        Self {
            backend,
            cache: HashMap::new(),
        }
    }

    /// Load a sound, returning a cached ID if already loaded.
    pub fn load(&mut self, path: &Path) -> Result<SoundId> {
        let key = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string();
        if let Some(&id) = self.cache.get(&key) {
            return Ok(id);
        }
        let id = self.backend.load_sound(path)?;
        self.cache.insert(key, id);
        Ok(id)
    }

    /// Load a sound from memory with a cache key.
    pub fn load_from_memory(&mut self, key: &str, data: &[u8], ext: &str) -> Result<SoundId> {
        if let Some(&id) = self.cache.get(key) {
            return Ok(id);
        }
        let id = self.backend.load_sound_from_memory(data, ext)?;
        self.cache.insert(key.to_string(), id);
        Ok(id)
    }

    /// Play a previously loaded sound.
    pub fn play(&mut self, id: SoundId) -> Result<()> {
        self.backend.play(id)
    }

    /// Stop a playing sound.
    pub fn stop(&mut self, id: SoundId) -> Result<()> {
        self.backend.stop(id)
    }

    /// Set volume for a sound.
    pub fn set_volume(&mut self, id: SoundId, volume: f32) -> Result<()> {
        self.backend.set_volume(id, volume)
    }

    /// Number of cached sounds.
    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }

    /// Clear all cached sounds and dispose the backend.
    pub fn dispose(&mut self) -> Result<()> {
        self.cache.clear();
        self.backend.dispose()
    }

    /// Get a reference to the underlying backend.
    pub fn backend(&self) -> &A {
        &self.backend
    }

    /// Get a mutable reference to the underlying backend.
    pub fn backend_mut(&mut self) -> &mut A {
        &mut self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock audio backend for testing.
    struct MockAudio {
        next_id: u64,
        played: Vec<u64>,
        stopped: Vec<u64>,
    }

    impl MockAudio {
        fn new() -> Self {
            Self {
                next_id: 1,
                played: Vec::new(),
                stopped: Vec::new(),
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

        fn set_volume(&mut self, _id: SoundId, _volume: f32) -> Result<()> {
            Ok(())
        }

        fn dispose(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn caches_same_path() {
        let mut pool = SoundPool::new(MockAudio::new());
        let path = Path::new("/test/sound.wav");
        let id1 = pool.load(path).unwrap();
        let id2 = pool.load(path).unwrap();
        assert_eq!(id1, id2);
        assert_eq!(pool.cached_count(), 1);
    }

    #[test]
    fn different_paths_get_different_ids() {
        let mut pool = SoundPool::new(MockAudio::new());
        let id1 = pool.load(Path::new("/test/a.wav")).unwrap();
        let id2 = pool.load(Path::new("/test/b.wav")).unwrap();
        assert_ne!(id1, id2);
        assert_eq!(pool.cached_count(), 2);
    }

    #[test]
    fn play_and_stop() {
        let mut pool = SoundPool::new(MockAudio::new());
        let id = pool.load(Path::new("/test/sound.wav")).unwrap();
        pool.play(id).unwrap();
        pool.stop(id).unwrap();
        assert_eq!(pool.backend().played, vec![id.0]);
        assert_eq!(pool.backend().stopped, vec![id.0]);
    }

    #[test]
    fn load_from_memory_caching() {
        let mut pool = SoundPool::new(MockAudio::new());
        let data = b"fake wav data";
        let id1 = pool.load_from_memory("key1", data, "wav").unwrap();
        let id2 = pool.load_from_memory("key1", data, "wav").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(pool.cached_count(), 1);
    }

    #[test]
    fn dispose_clears_cache() {
        let mut pool = SoundPool::new(MockAudio::new());
        pool.load(Path::new("/test/a.wav")).unwrap();
        pool.dispose().unwrap();
        assert_eq!(pool.cached_count(), 0);
    }
}
