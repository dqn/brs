//! Audio subsystem using kira.
//!
//! This module provides:
//! - [`AudioDriver`]: Low-level audio management with kira
//! - [`SoundPool`]: Caches loaded audio data
//! - [`KeysoundProcessor`]: Handles BGM and player keysound playback
//! - [`AudioConfig`]: Configuration for audio settings
//! - [`LatencyMeasurement`]: Audio latency tracking

mod audio_config;
mod audio_driver;
mod keysound_processor;
mod latency;
mod preview_player;
mod sound_pool;

pub use audio_config::AudioConfig;
pub use audio_driver::{AudioDriver, LoadProgress};
pub use keysound_processor::{BgmEvent, KeysoundProcessor};
pub use latency::LatencyMeasurement;
pub use preview_player::PreviewPlayer;
pub use sound_pool::SoundPool;
