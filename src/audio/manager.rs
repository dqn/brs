use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rayon::prelude::*;

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
use kira::Decibels;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};

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
    master_volume: f64,
    keysound_volume: f64,
    bgm_volume: f64,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let manager = KiraAudioManager::new(AudioManagerSettings::default())
            .context("Failed to create audio manager")?;

        Ok(Self {
            manager,
            sounds: HashMap::new(),
            master_volume: 1.0,
            keysound_volume: 1.0,
            bgm_volume: 1.0,
        })
    }

    /// Set master volume (0.0 - 1.0)
    pub fn set_master_volume(&mut self, volume: f64) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set keysound volume (0.0 - 1.0)
    pub fn set_keysound_volume(&mut self, volume: f64) {
        self.keysound_volume = volume.clamp(0.0, 1.0);
    }

    /// Set BGM volume (0.0 - 1.0)
    pub fn set_bgm_volume(&mut self, volume: f64) {
        self.bgm_volume = volume.clamp(0.0, 1.0);
    }

    /// Convert amplitude (0.0-1.0) to Decibels
    fn amplitude_to_decibels(amplitude: f64) -> Decibels {
        if amplitude <= 0.0 {
            Decibels::SILENCE
        } else if amplitude >= 1.0 {
            Decibels::IDENTITY
        } else {
            Decibels(20.0 * (amplitude as f32).log10())
        }
    }

    pub fn load_keysounds<P: AsRef<Path>>(
        &mut self,
        base_path: P,
        wav_files: &HashMap<u32, String>,
    ) -> KeysoundLoadResult {
        let base_path = base_path.as_ref();

        // Load and decode audio files in parallel
        let results: Vec<_> = wav_files
            .par_iter()
            .map(
                |(&id, filename)| match find_audio_file(base_path, filename) {
                    Some(file_path) => match StaticSoundData::from_file(&file_path) {
                        Ok(sound) => Ok((id, sound)),
                        Err(e) => Err((id, filename.clone(), format!("Failed to decode: {}", e))),
                    },
                    None => Err((id, filename.clone(), "File not found".to_string())),
                },
            )
            .collect();

        // Aggregate results
        let mut result = KeysoundLoadResult::default();
        for r in results {
            match r {
                Ok((id, sound)) => {
                    self.sounds.insert(id, sound);
                    result.loaded += 1;
                }
                Err((id, filename, msg)) => {
                    eprintln!("Warning: Keysound #{:02X} '{}': {}", id, filename, msg);
                    result.failed.push((id, filename, msg));
                }
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

    /// Play keysound with current volume settings
    pub fn play(&mut self, keysound_id: u32) -> Option<StaticSoundHandle> {
        if let Some(sound_data) = self.sounds.get(&keysound_id) {
            let effective_volume = self.master_volume * self.keysound_volume;
            let decibels = Self::amplitude_to_decibels(effective_volume);
            let settings = StaticSoundSettings::new().volume(decibels);
            self.manager
                .play(sound_data.clone().with_settings(settings))
                .ok()
        } else {
            None
        }
    }

    /// Play BGM with current volume settings
    pub fn play_bgm(&mut self, keysound_id: u32) -> Option<StaticSoundHandle> {
        if let Some(sound_data) = self.sounds.get(&keysound_id) {
            let effective_volume = self.master_volume * self.bgm_volume;
            let decibels = Self::amplitude_to_decibels(effective_volume);
            let settings = StaticSoundSettings::new().volume(decibels);
            self.manager
                .play(sound_data.clone().with_settings(settings))
                .ok()
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
