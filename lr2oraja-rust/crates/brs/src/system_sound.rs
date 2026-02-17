// System sound playback queue manager.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// System sound types for state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemSound {
    Decide,
    ResultClear,
    ResultFail,
    Select,
    #[allow(dead_code)] // Parsed for completeness (Java SystemSound enum)
    Scratch,
    Folder,
    #[allow(dead_code)] // Parsed for completeness (Java SystemSound enum)
    OptionChange,
}

/// Manages system sound playback queue.
#[derive(Default)]
pub struct SystemSoundManager {
    /// Queue of sounds to play this frame.
    queue: Vec<SystemSound>,
    /// Paths to system sound files, loaded from config.
    #[allow(dead_code)] // TODO: integrate with audio system
    sound_paths: HashMap<SystemSound, PathBuf>,
}

impl SystemSoundManager {
    #[allow(dead_code)] // Used in tests
    pub fn new() -> Self {
        Self::default()
    }

    /// Load system sound file paths from the given base directory.
    #[allow(dead_code)] // TODO: integrate with audio system
    pub fn load_sounds(&mut self, base_dir: &Path) {
        let sound_files = [
            (SystemSound::Decide, "decide.wav"),
            (SystemSound::Select, "select.wav"),
            (SystemSound::Folder, "folder.wav"),
            (SystemSound::ResultClear, "clear.wav"),
            (SystemSound::ResultFail, "fail.wav"),
            (SystemSound::Scratch, "scratch.wav"),
            (SystemSound::OptionChange, "option.wav"),
        ];
        for (sound, filename) in &sound_files {
            let path = base_dir.join(filename);
            if path.exists() {
                self.sound_paths.insert(*sound, path);
            }
        }
    }

    /// Get the file path for a sound type (if loaded).
    #[allow(dead_code)] // TODO: integrate with audio system
    pub fn sound_path(&self, sound: SystemSound) -> Option<&Path> {
        self.sound_paths.get(&sound).map(|p| p.as_path())
    }

    /// Queue a sound for playback.
    pub fn play(&mut self, sound: SystemSound) {
        self.queue.push(sound);
    }

    /// Drain the queue (consumed by audio system each frame).
    #[allow(dead_code)] // Used in tests
    pub fn drain(&mut self) -> Vec<SystemSound> {
        std::mem::take(&mut self.queue)
    }

    /// Check if queue is empty.
    #[allow(dead_code)] // Used in tests
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let mgr = SystemSoundManager::new();
        assert!(mgr.is_empty());
    }

    #[test]
    fn play_adds_to_queue() {
        let mut mgr = SystemSoundManager::new();
        mgr.play(SystemSound::Decide);
        mgr.play(SystemSound::Select);
        assert!(!mgr.is_empty());
    }

    #[test]
    fn drain_returns_and_clears_queue() {
        let mut mgr = SystemSoundManager::new();
        mgr.play(SystemSound::ResultClear);
        mgr.play(SystemSound::Folder);
        let drained = mgr.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0], SystemSound::ResultClear);
        assert_eq!(drained[1], SystemSound::Folder);
        assert!(mgr.is_empty());
    }
}
