use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::game::{ClearLamp, PlayResult};

/// Compute SHA256 hash of a file
pub fn compute_file_hash(path: &Path) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    let hash = Sha256::digest(&content);
    Ok(format!("{:x}", hash))
}

/// Saved score data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedScore {
    /// Chart hash (SHA256 of chart file)
    pub hash: String,
    /// Best clear lamp achieved
    pub clear_lamp: u8,
    /// Best EX score
    pub ex_score: u32,
    /// Best max combo
    pub max_combo: u32,
    /// Total play count
    pub play_count: u32,
    /// Clear count
    pub clear_count: u32,
    /// Last played timestamp (Unix epoch seconds)
    pub last_played: u64,
    /// Best judgment counts
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
}

impl SavedScore {
    /// Create a new score record with initial values
    pub fn new(hash: String) -> Self {
        Self {
            hash,
            clear_lamp: ClearLamp::NoPlay as u8,
            ex_score: 0,
            max_combo: 0,
            play_count: 0,
            clear_count: 0,
            last_played: 0,
            pgreat_count: 0,
            great_count: 0,
            good_count: 0,
            bad_count: 0,
            poor_count: 0,
        }
    }

    /// Create a score from play result
    pub fn from_play_result(hash: String, result: &PlayResult) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            hash,
            clear_lamp: result.clear_lamp.as_u8(),
            ex_score: result.ex_score,
            max_combo: result.max_combo,
            play_count: 0, // Will be incremented by update()
            clear_count: 0,
            last_played: timestamp,
            pgreat_count: result.pgreat_count,
            great_count: result.great_count,
            good_count: result.good_count,
            bad_count: result.bad_count,
            poor_count: result.poor_count,
        }
    }

    /// Get clear lamp enum
    #[allow(dead_code)]
    pub fn clear_lamp(&self) -> ClearLamp {
        ClearLamp::from_u8(self.clear_lamp)
    }

    /// Check if this score beats the existing one
    pub fn is_better_than(&self, other: &SavedScore) -> bool {
        // Clear lamp takes priority
        if self.clear_lamp > other.clear_lamp {
            return true;
        }
        if self.clear_lamp < other.clear_lamp {
            return false;
        }
        // Then EX score
        self.ex_score > other.ex_score
    }

    /// Update with new play result, returns true if it was a new best
    pub fn update(&mut self, new_score: &SavedScore) -> bool {
        self.play_count += 1;
        self.last_played = new_score.last_played;

        // Track clear count
        if new_score.clear_lamp > ClearLamp::Failed as u8 {
            self.clear_count += 1;
        }

        // Update best records
        let is_better = new_score.is_better_than(self);

        if new_score.clear_lamp > self.clear_lamp {
            self.clear_lamp = new_score.clear_lamp;
        }

        if new_score.ex_score > self.ex_score {
            self.ex_score = new_score.ex_score;
            // Update judgment counts when EX score improves
            self.pgreat_count = new_score.pgreat_count;
            self.great_count = new_score.great_count;
            self.good_count = new_score.good_count;
            self.bad_count = new_score.bad_count;
            self.poor_count = new_score.poor_count;
        }

        if new_score.max_combo > self.max_combo {
            self.max_combo = new_score.max_combo;
        }

        is_better
    }
}

impl ClearLamp {
    /// Convert from u8 value
    #[allow(dead_code)]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ClearLamp::NoPlay,
            1 => ClearLamp::Failed,
            2 => ClearLamp::AssistEasy,
            3 => ClearLamp::Easy,
            4 => ClearLamp::Normal,
            5 => ClearLamp::Hard,
            6 => ClearLamp::ExHard,
            7 => ClearLamp::FullCombo,
            _ => ClearLamp::NoPlay,
        }
    }

    /// Convert to u8 value for storage
    pub fn as_u8(&self) -> u8 {
        match self {
            ClearLamp::NoPlay => 0,
            ClearLamp::Failed => 1,
            ClearLamp::AssistEasy => 2,
            ClearLamp::Easy => 3,
            ClearLamp::Normal => 4,
            ClearLamp::Hard => 5,
            ClearLamp::ExHard => 6,
            ClearLamp::FullCombo => 7,
        }
    }
}
