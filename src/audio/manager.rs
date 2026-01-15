use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use kira::AudioManager as KiraAudioManager;
use kira::AudioManagerSettings;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};

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
    ) -> Result<usize> {
        let base_path = base_path.as_ref();
        let mut loaded = 0;

        for (&id, filename) in wav_files {
            let file_path = base_path.join(filename);

            if !file_path.exists() {
                let lower = filename.to_lowercase();
                let lower_path = base_path.join(&lower);
                if lower_path.exists() {
                    if let Ok(sound) = self.load_sound(&lower_path) {
                        self.sounds.insert(id, sound);
                        loaded += 1;
                    }
                }
                continue;
            }

            if let Ok(sound) = self.load_sound(&file_path) {
                self.sounds.insert(id, sound);
                loaded += 1;
            }
        }

        Ok(loaded)
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
