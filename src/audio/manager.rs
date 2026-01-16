use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Supported audio file extensions for fallback
const AUDIO_EXTENSIONS: &[&str] = &["wav", "ogg", "mp3", "flac"];

/// Try different extensions and case variations to find an existing audio file
fn find_audio_file(base_path: &Path, filename: &str) -> Option<PathBuf> {
    // Try original filename first
    let file_path = base_path.join(filename);
    if file_path.exists() {
        return Some(file_path);
    }

    // Try lowercase filename
    let lower = filename.to_lowercase();
    let lower_path = base_path.join(&lower);
    if lower_path.exists() {
        return Some(lower_path);
    }

    // Try different extensions
    let stem = Path::new(filename).file_stem()?.to_str()?;
    for ext in AUDIO_EXTENSIONS {
        // Original case with different extension
        let alt_filename = format!("{}.{}", stem, ext);
        let alt_path = base_path.join(&alt_filename);
        if alt_path.exists() {
            return Some(alt_path);
        }

        // Lowercase with different extension
        let alt_lower = alt_filename.to_lowercase();
        let alt_lower_path = base_path.join(&alt_lower);
        if alt_lower_path.exists() {
            return Some(alt_lower_path);
        }
    }

    None
}
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
            if let Some(file_path) = find_audio_file(base_path, filename) {
                match self.load_sound(&file_path) {
                    Ok(sound) => {
                        self.sounds.insert(id, sound);
                        result.loaded += 1;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to decode: {}", e);
                        eprintln!(
                            "Warning: Failed to load keysound #{:02X} '{}': {}",
                            id, filename, error_msg
                        );
                        result.failed.push((id, filename.clone(), error_msg));
                    }
                }
            } else {
                let error_msg = "File not found".to_string();
                eprintln!("Warning: Keysound #{:02X} '{}' not found", id, filename);
                result.failed.push((id, filename.clone(), error_msg));
            }
        }

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
