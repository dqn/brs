/// Kira-based audio driver for real-time BMS playback.
///
/// Loads PCM data into Kira StaticSoundData and plays key sounds and BGM.
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use kira::manager::backend::cpal::{CpalBackend, CpalBackendSettings};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::tween::Tween;
use tracing::{info, warn};

use bms_model::{BgNote, BmsModel, Note};

use crate::decode;
use crate::driver::{AudioDriver, channel_id, pitch_from_shift};
use crate::pcm::Pcm;

/// Convert Pcm (f32 interleaved) to WAV bytes in memory.
fn pcm_to_wav_bytes(pcm: &Pcm) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: pcm.channels,
        sample_rate: pcm.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::new(&mut cursor, spec).expect("WAV writer creation");
    for &sample in &pcm.samples {
        writer.write_sample(sample).expect("WAV sample write");
    }
    writer.finalize().expect("WAV finalize");
    cursor.into_inner()
}

/// Kira-based audio driver for real-time BMS key sound playback.
pub struct KiraAudioDriver {
    manager: AudioManager<CpalBackend>,
    /// wav_id -> StaticSoundData (full, un-sliced)
    sounds: HashMap<u16, StaticSoundData>,
    /// Active sound handles keyed by channel_id
    active_handles: HashMap<i32, StaticSoundHandle>,
    /// Global pitch multiplier
    global_pitch: f32,
    /// Loading progress tracking
    loaded_count: usize,
    total_count: usize,
    /// Consecutive playback error count for recovery detection.
    consecutive_errors: u32,
    /// Flag indicating the driver needs recovery.
    recovery_pending: bool,
    /// M7: Additional key sounds indexed by [judge_level][early=0/late=1].
    additional_key_sounds: [[Option<StaticSoundData>; 2]; 6],
    /// Selected device name for recovery (None = system default).
    device_name: Option<String>,
}

/// Find a cpal output device by name.
fn find_device_by_name(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    host.output_devices()
        .ok()?
        .find(|d| d.name().ok().as_deref() == Some(name))
}

/// Build CpalBackendSettings from an optional device name.
fn backend_settings(device_name: Option<&str>) -> CpalBackendSettings {
    let device = device_name.and_then(|name| {
        let dev = find_device_by_name(name);
        if dev.is_none() {
            warn!(name, "Audio device not found, falling back to default");
        }
        dev
    });
    CpalBackendSettings {
        device,
        ..Default::default()
    }
}

impl KiraAudioDriver {
    /// Create a new KiraAudioDriver with the system default output device.
    pub fn new() -> Result<Self> {
        Self::with_device(None)
    }

    /// Create a new KiraAudioDriver with a specific output device.
    ///
    /// Pass `None` to use the system default output device.
    pub fn with_device(device_name: Option<String>) -> Result<Self> {
        let settings = backend_settings(device_name.as_deref());
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings {
            backend_settings: settings,
            ..Default::default()
        })
        .map_err(|e| anyhow::anyhow!("Failed to create audio manager: {e}"))?;
        Ok(Self {
            manager,
            sounds: HashMap::new(),
            active_handles: HashMap::new(),
            global_pitch: 1.0,
            loaded_count: 0,
            total_count: 0,
            consecutive_errors: 0,
            recovery_pending: false,
            additional_key_sounds: Default::default(),
            device_name,
        })
    }
}

impl AudioDriver for KiraAudioDriver {
    fn set_model(&mut self, model: &BmsModel, base_path: &Path) -> Result<()> {
        self.sounds.clear();
        self.active_handles.clear();
        self.loaded_count = 0;

        // Collect unique wav_ids needed
        let mut wav_ids: Vec<u16> = Vec::new();
        for note in &model.notes {
            if !wav_ids.contains(&note.wav_id) {
                wav_ids.push(note.wav_id);
            }
        }
        for bg in &model.bg_notes {
            if !wav_ids.contains(&bg.wav_id) {
                wav_ids.push(bg.wav_id);
            }
        }
        self.total_count = wav_ids.len();

        for wav_id in wav_ids {
            let wav_name = match model.wav_defs.get(&wav_id) {
                Some(path) => path,
                None => {
                    self.loaded_count += 1;
                    continue;
                }
            };

            let resolved = decode::resolve_audio_path(base_path, &wav_name.to_string_lossy());
            let audio_path = match resolved {
                Some(p) => p,
                None => {
                    self.loaded_count += 1;
                    continue;
                }
            };

            let pcm = match decode::load_audio(&audio_path) {
                Ok(p) => p,
                Err(e) => {
                    warn!(wav_id, path = %audio_path.display(), "Failed to load audio: {e}");
                    self.loaded_count += 1;
                    continue;
                }
            };

            let wav_bytes = pcm_to_wav_bytes(&pcm);
            match StaticSoundData::from_cursor(Cursor::new(wav_bytes)) {
                Ok(sound_data) => {
                    self.sounds.insert(wav_id, sound_data);
                }
                Err(e) => {
                    warn!(wav_id, "Failed to create StaticSoundData: {e}");
                }
            }
            self.loaded_count += 1;
        }

        info!(
            loaded = self.sounds.len(),
            total = self.total_count,
            "KiraAudioDriver: loaded sounds"
        );
        Ok(())
    }

    fn play_note(&mut self, note: &Note, volume: f32, pitch_shift: i32) {
        let ch_id = channel_id(note.wav_id, pitch_shift);

        // Stop existing sound on this channel
        if let Some(mut handle) = self.active_handles.remove(&ch_id) {
            handle.stop(Tween::default());
        }

        if let Some(sound_data) = self.sounds.get(&note.wav_id) {
            let pitch = pitch_from_shift(pitch_shift) * self.global_pitch;
            let data = sound_data
                .clone()
                .volume(volume as f64)
                .playback_rate(pitch as f64);
            match self.manager.play(data) {
                Ok(handle) => {
                    self.active_handles.insert(ch_id, handle);
                    self.consecutive_errors = 0;
                }
                Err(e) => {
                    warn!(wav_id = note.wav_id, "Failed to play note: {e}");
                    self.consecutive_errors += 1;
                    if self.consecutive_errors >= 5 {
                        self.recovery_pending = true;
                    }
                }
            }
        }
    }

    fn play_bg_note(&mut self, bg_note: &BgNote, volume: f32) {
        if let Some(sound_data) = self.sounds.get(&bg_note.wav_id) {
            let data = sound_data
                .clone()
                .volume(volume as f64)
                .playback_rate(self.global_pitch as f64);
            match self.manager.play(data) {
                Ok(_handle) => {
                    // BG notes don't need channel tracking
                    self.consecutive_errors = 0;
                }
                Err(e) => {
                    warn!(wav_id = bg_note.wav_id, "Failed to play bg note: {e}");
                    self.consecutive_errors += 1;
                    if self.consecutive_errors >= 5 {
                        self.recovery_pending = true;
                    }
                }
            }
        }
    }

    fn stop_note(&mut self, note: &Note) {
        let ch_id = channel_id(note.wav_id, 0);
        if let Some(mut handle) = self.active_handles.remove(&ch_id) {
            handle.stop(Tween::default());
        }
    }

    fn stop_all(&mut self) {
        for (_, mut handle) in self.active_handles.drain() {
            handle.stop(Tween::default());
        }
    }

    fn set_note_volume(&mut self, note: &Note, volume: f32) {
        let ch_id = channel_id(note.wav_id, 0);
        if let Some(handle) = self.active_handles.get_mut(&ch_id) {
            handle.set_volume(volume as f64, Tween::default());
        }
    }

    fn set_global_pitch(&mut self, pitch: f32) {
        self.global_pitch = pitch;
    }

    fn global_pitch(&self) -> f32 {
        self.global_pitch
    }

    fn progress(&self) -> f32 {
        if self.total_count == 0 {
            1.0
        } else {
            self.loaded_count as f32 / self.total_count as f32
        }
    }

    fn needs_recovery(&self) -> bool {
        self.recovery_pending
    }

    fn try_recover(&mut self) -> Result<()> {
        warn!(
            "Attempting audio driver recovery after {} consecutive errors",
            self.consecutive_errors
        );

        // Drop active handles before recreating the manager
        self.active_handles.clear();

        // Recreate the audio manager with the same device
        let settings = backend_settings(self.device_name.as_deref());
        self.manager = AudioManager::<CpalBackend>::new(AudioManagerSettings {
            backend_settings: settings,
            ..Default::default()
        })
        .map_err(|e| anyhow::anyhow!("Audio recovery failed: {e}"))?;

        // Reset error tracking
        self.consecutive_errors = 0;
        self.recovery_pending = false;

        warn!("Audio driver recovery successful, sounds remain loaded");
        Ok(())
    }

    fn is_playing(&self, wav_id: u16) -> bool {
        let ch_id = channel_id(wav_id, 0);
        self.active_handles.contains_key(&ch_id)
    }

    fn set_additional_key_sound(&mut self, judge: usize, early: bool, path: &Path) -> Result<()> {
        if judge >= 6 {
            return Ok(());
        }
        let pcm = crate::decode::load_audio(path)?;
        let wav_bytes = pcm_to_wav_bytes(&pcm);
        let sound_data = StaticSoundData::from_cursor(Cursor::new(wav_bytes))
            .map_err(|e| anyhow::anyhow!("Failed to create additional key sound: {e}"))?;
        let idx = if early { 0 } else { 1 };
        self.additional_key_sounds[judge][idx] = Some(sound_data);
        Ok(())
    }

    fn play_additional_key_sound(&mut self, judge: usize, early: bool) {
        if judge >= 6 {
            return;
        }
        let idx = if early { 0 } else { 1 };
        if let Some(sound_data) = &self.additional_key_sounds[judge][idx]
            && let Err(e) = self.manager.play(sound_data.clone())
        {
            warn!(judge, early, "Failed to play additional key sound: {e}");
        }
    }
}
