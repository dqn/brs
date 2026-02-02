//! Audio subsystem using kira.
//!
//! This module provides:
//! - [`AudioDriver`]: Low-level audio management with kira
//! - [`SoundPool`]: Caches loaded audio data
//! - [`KeysoundProcessor`]: Handles BGM and player keysound playback
//! - [`AudioConfig`]: Configuration for audio settings

mod audio_config;
mod audio_driver;
mod keysound_processor;
mod sound_pool;

pub use audio_config::AudioConfig;
pub use audio_driver::{AudioDriver, LoadProgress};
pub use keysound_processor::{BgmEvent, KeysoundProcessor};
pub use sound_pool::SoundPool;
