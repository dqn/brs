//! Replay data structures.

use serde::{Deserialize, Serialize};

use crate::input::KeyInputLog;

/// Current replay format version.
pub const REPLAY_VERSION: u32 = 1;

/// Replay metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMetadata {
    /// Replay format version.
    pub version: u32,
    /// SHA256 hash of the chart.
    pub sha256: String,
    /// Player name.
    pub player_name: String,
    /// Timestamp when the replay was recorded (Unix seconds).
    pub recorded_at: i64,
    /// Gauge type used (0=Normal, 1=Easy, etc.).
    pub gauge_type: i32,
    /// Hi-speed setting.
    pub hi_speed: f32,
    /// Play mode (5/7/10/14/25/29).
    #[serde(default)]
    pub play_mode: i32,
    /// Long note mode (0=normal, 1=LN, 2=CN, 3=HCN).
    #[serde(default)]
    pub long_note_mode: i32,
    /// Judge rank value.
    #[serde(default)]
    pub judge_rank: i32,
    /// Judge rank type (0=BMS, 1=DEFEXRANK, 2=BMSON).
    #[serde(default)]
    pub judge_rank_type: i32,
    /// TOTAL value used for gauge scaling.
    #[serde(default)]
    pub total: f64,
    /// Source format (0=BMS, 1=BMSON).
    #[serde(default)]
    pub source_format: i32,
}

/// Score data stored in replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayScore {
    /// EX-SCORE (PG*2 + GR).
    pub ex_score: u32,
    /// Maximum combo achieved.
    pub max_combo: u32,
    /// Perfect Great count.
    pub pg_count: u32,
    /// Great count.
    pub gr_count: u32,
    /// Good count.
    pub gd_count: u32,
    /// Bad count.
    pub bd_count: u32,
    /// Poor count.
    pub pr_count: u32,
    /// Miss count.
    pub ms_count: u32,
    /// Clear type (0=NoPlay, 1=Failed, etc.).
    pub clear_type: i32,
    /// Fast timing count.
    pub fast_count: u32,
    /// Slow timing count.
    pub slow_count: u32,
}

/// Complete replay data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayData {
    /// Replay metadata.
    pub metadata: ReplayMetadata,
    /// Score data.
    pub score: ReplayScore,
    /// Input logs.
    pub input_logs: Vec<KeyInputLog>,
}

impl ReplayData {
    /// Create a new replay data with metadata.
    pub fn new(sha256: String, player_name: String, gauge_type: i32, hi_speed: f32) -> Self {
        Self {
            metadata: ReplayMetadata {
                version: REPLAY_VERSION,
                sha256,
                player_name,
                recorded_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
                gauge_type,
                hi_speed,
                play_mode: 0,
                long_note_mode: 0,
                judge_rank: 0,
                judge_rank_type: 0,
                total: 0.0,
                source_format: 0,
            },
            score: ReplayScore {
                ex_score: 0,
                max_combo: 0,
                pg_count: 0,
                gr_count: 0,
                gd_count: 0,
                bd_count: 0,
                pr_count: 0,
                ms_count: 0,
                clear_type: 0,
                fast_count: 0,
                slow_count: 0,
            },
            input_logs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_data_serialization() {
        let data = ReplayData::new("abc123".to_string(), "player".to_string(), 0, 1.0);
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ReplayData = serde_json::from_str(&json).unwrap();
        assert_eq!(data.metadata.sha256, deserialized.metadata.sha256);
        assert_eq!(data.metadata.player_name, deserialized.metadata.player_name);
    }

    #[test]
    fn test_replay_metadata() {
        let data = ReplayData::new("sha256".to_string(), "test".to_string(), 2, 1.5);
        assert_eq!(data.metadata.version, REPLAY_VERSION);
        assert_eq!(data.metadata.gauge_type, 2);
        assert_eq!(data.metadata.hi_speed, 1.5);
        assert!(data.metadata.recorded_at > 0);
    }
}
