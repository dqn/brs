//! Replay file storage and slot management.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use super::replay_data::ReplayData;

/// Replay slot (0-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaySlot(pub u8);

impl ReplaySlot {
    /// Default slot (slot 0).
    pub const SLOT_0: Self = Self(0);
    /// Slot 1.
    pub const SLOT_1: Self = Self(1);
    /// Slot 2.
    pub const SLOT_2: Self = Self(2);
    /// Slot 3.
    pub const SLOT_3: Self = Self(3);

    /// Get all available slots.
    pub fn all() -> [Self; 4] {
        [Self::SLOT_0, Self::SLOT_1, Self::SLOT_2, Self::SLOT_3]
    }
}

/// Get the replay directory path.
pub fn replay_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".brs")
        .join("replay")
}

/// Get the replay file path for a chart and slot.
pub fn replay_path(sha256: &str, slot: ReplaySlot) -> PathBuf {
    let filename = match slot.0 {
        0 => format!("{}.json.gz", sha256),
        n => format!("{}_{}.json.gz", sha256, n),
    };
    replay_dir().join(filename)
}

/// Ensure the replay directory exists.
pub fn ensure_replay_dir() -> Result<()> {
    let dir = replay_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create replay directory: {}", dir.display()))?;
    }
    Ok(())
}

/// Save replay data to a slot.
pub fn save_replay(data: &ReplayData, slot: ReplaySlot) -> Result<PathBuf> {
    ensure_replay_dir()?;

    let path = replay_path(&data.metadata.sha256, slot);

    // Serialize to JSON.
    let json = serde_json::to_string(data).context("Failed to serialize replay data")?;

    // Compress with GZIP.
    let file = File::create(&path)
        .with_context(|| format!("Failed to create replay file: {}", path.display()))?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder
        .write_all(json.as_bytes())
        .context("Failed to write compressed data")?;
    encoder.finish().context("Failed to finish compression")?;

    Ok(path)
}

/// Load replay data from a slot.
pub fn load_replay(sha256: &str, slot: ReplaySlot) -> Result<Option<ReplayData>> {
    let path = replay_path(sha256, slot);
    if !path.exists() {
        return Ok(None);
    }

    // Read and decompress.
    let file = File::open(&path)
        .with_context(|| format!("Failed to open replay file: {}", path.display()))?;
    let mut decoder = GzDecoder::new(file);
    let mut json = String::new();
    decoder
        .read_to_string(&mut json)
        .context("Failed to decompress replay data")?;

    // Deserialize.
    let data: ReplayData =
        serde_json::from_str(&json).context("Failed to deserialize replay data")?;

    Ok(Some(data))
}

/// List available replay slots for a chart.
pub fn list_replays(sha256: &str) -> Vec<(ReplaySlot, ReplayData)> {
    let mut replays = Vec::new();

    for slot in ReplaySlot::all() {
        if let Ok(Some(data)) = load_replay(sha256, slot) {
            replays.push((slot, data));
        }
    }

    replays
}

/// Find the next available slot for a chart.
pub fn find_empty_slot(sha256: &str) -> Option<ReplaySlot> {
    for slot in ReplaySlot::all() {
        let path = replay_path(sha256, slot);
        if !path.exists() {
            return Some(slot);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_slot_all() {
        let slots = ReplaySlot::all();
        assert_eq!(slots.len(), 4);
        assert_eq!(slots[0], ReplaySlot::SLOT_0);
        assert_eq!(slots[3], ReplaySlot::SLOT_3);
    }

    #[test]
    fn test_replay_path() {
        let path0 = replay_path("abc123", ReplaySlot::SLOT_0);
        assert!(path0.to_string_lossy().ends_with("abc123.json.gz"));

        let path1 = replay_path("abc123", ReplaySlot::SLOT_1);
        assert!(path1.to_string_lossy().ends_with("abc123_1.json.gz"));
    }
}
