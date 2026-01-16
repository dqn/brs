use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use kira::AudioManager as KiraAudioManager;
use kira::AudioManagerSettings;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};

/// Result of loading keysounds
#[derive(Debug, Default)]
pub struct KeysoundLoadResult {
    /// Number of successfully loaded keysounds
    pub loaded: usize,
    /// List of failed keysounds: (id, filename, error message)
    pub failed: Vec<(u32, String, String)>,
}

impl KeysoundLoadResult {
    /// Total number of keysounds attempted
    pub fn total(&self) -> usize {
        self.loaded + self.failed.len()
    }

    /// Whether all keysounds loaded successfully
    #[allow(dead_code)]
    pub fn all_loaded(&self) -> bool {
        self.failed.is_empty()
    }
}

pub struct AudioManager {
    manager: KiraAudioManager,
    sounds: HashMap<u32, StaticSoundData>,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let manager = KiraAudioManager::new(AudioManagerSettings::default())
            .context("Failed to create audio manager")?;

        Ok(Self {
            manager,
            sounds: HashMap::new(),
        })
    }

    pub fn load_keysounds<P: AsRef<Path>>(
        &mut self,
        base_path: P,
        wav_files: &HashMap<u32, String>,
    ) -> KeysoundLoadResult {
        let base_path = base_path.as_ref();
        let mut result = KeysoundLoadResult::default();

        for (&id, filename) in wav_files {
            let file_path = base_path.join(filename);

            // Try original filename first
            if file_path.exists() {
                match self.load_sound(&file_path) {
                    Ok(sound) => {
                        self.sounds.insert(id, sound);
                        result.loaded += 1;
                        continue;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to decode: {}", e);
                        eprintln!(
                            "Warning: Failed to load keysound #{:02X} '{}': {}",
                            id, filename, error_msg
                        );
                        result.failed.push((id, filename.clone(), error_msg));
                        continue;
                    }
                }
            }

            // Try lowercase filename as fallback (case-insensitive filesystems)
            let lower = filename.to_lowercase();
            let lower_path = base_path.join(&lower);
            if lower_path.exists() {
                match self.load_sound(&lower_path) {
                    Ok(sound) => {
                        self.sounds.insert(id, sound);
                        result.loaded += 1;
                        continue;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to decode (lowercase): {}", e);
                        eprintln!(
                            "Warning: Failed to load keysound #{:02X} '{}': {}",
                            id, filename, error_msg
                        );
                        result.failed.push((id, filename.clone(), error_msg));
                        continue;
                    }
                }
            }

            // File not found
            let error_msg = "File not found".to_string();
            eprintln!(
                "Warning: Keysound #{:02X} '{}' not found at {}",
                id,
                filename,
                file_path.display()
            );
            result.failed.push((id, filename.clone(), error_msg));
        }

        // Log summary
        if !result.failed.is_empty() {
            eprintln!(
                "Keysound loading: {}/{} loaded, {} failed",
                result.loaded,
                result.total(),
                result.failed.len()
            );
        }

        result
    }

    fn load_sound<P: AsRef<Path>>(&self, path: P) -> Result<StaticSoundData> {
        let path = path.as_ref();
        StaticSoundData::from_file(path)
            .with_context(|| format!("Failed to load sound: {}", path.display()))
    }

    pub fn play(&mut self, keysound_id: u32) -> Option<StaticSoundHandle> {
        if let Some(sound_data) = self.sounds.get(&keysound_id) {
            self.manager.play(sound_data.clone()).ok()
        } else {
            None
        }
    }

    // Public API for querying loaded keysound count
    #[allow(dead_code)]
    pub fn keysound_count(&self) -> usize {
        self.sounds.len()
    }
}
