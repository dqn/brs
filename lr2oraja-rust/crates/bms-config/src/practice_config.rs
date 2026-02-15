// Practice mode configuration — per-song settings persisted as JSON.
//
// Ported from Java: PracticeConfiguration.PracticeProperty

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Practice mode settings for a single chart.
///
/// Persisted to `practice/{sha256}.json`. Each field corresponds to a
/// Java `PracticeProperty` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PracticeProperty {
    /// Start time in milliseconds.
    pub starttime: i32,
    /// End time in milliseconds.
    pub endtime: i32,
    /// Gauge type index (0-8).
    pub gaugetype: i32,
    /// Starting gauge value (1-max).
    pub startgauge: i32,
    /// 1P random option index (0-9).
    pub random: i32,
    /// 2P random option index (0-9).
    pub random2: i32,
    /// DP option (0-1).
    pub doubleop: i32,
    /// Judge rank (1-400, 100 = default).
    pub judgerank: i32,
    /// Playback frequency percentage (50-200).
    pub freq: i32,
    /// Total value (20.0-5000.0).
    pub total: f64,
    /// Graph type (0-2).
    pub graphtype: i32,
}

impl Default for PracticeProperty {
    fn default() -> Self {
        Self {
            starttime: 0,
            endtime: 0,
            gaugetype: 2, // Normal gauge
            startgauge: 20,
            random: 0,
            random2: 0,
            doubleop: 0,
            judgerank: 100,
            freq: 100,
            total: 300.0,
            graphtype: 0,
        }
    }
}

impl PracticeProperty {
    /// Load practice property from `{dir}/practice/{sha256}.json`.
    /// Returns default if the file does not exist or cannot be parsed.
    pub fn load(dir: &Path, sha256: &str) -> Self {
        let path = dir.join("practice").join(format!("{sha256}.json"));
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save practice property to `{dir}/practice/{sha256}.json`.
    pub fn save(&self, dir: &Path, sha256: &str) -> Result<()> {
        let practice_dir = dir.join("practice");
        std::fs::create_dir_all(&practice_dir)?;
        let path = practice_dir.join(format!("{sha256}.json"));
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let p = PracticeProperty::default();
        assert_eq!(p.starttime, 0);
        assert_eq!(p.endtime, 0);
        assert_eq!(p.gaugetype, 2);
        assert_eq!(p.startgauge, 20);
        assert_eq!(p.random, 0);
        assert_eq!(p.random2, 0);
        assert_eq!(p.doubleop, 0);
        assert_eq!(p.judgerank, 100);
        assert_eq!(p.freq, 100);
        assert!((p.total - 300.0).abs() < f64::EPSILON);
        assert_eq!(p.graphtype, 0);
    }

    #[test]
    fn serde_round_trip() {
        let p = PracticeProperty {
            starttime: 5000,
            endtime: 30000,
            gaugetype: 3,
            startgauge: 50,
            random: 2,
            random2: 1,
            doubleop: 1,
            judgerank: 200,
            freq: 75,
            total: 500.0,
            graphtype: 1,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: PracticeProperty = serde_json::from_str(&json).unwrap();
        assert_eq!(back.starttime, 5000);
        assert_eq!(back.endtime, 30000);
        assert_eq!(back.gaugetype, 3);
        assert_eq!(back.startgauge, 50);
        assert_eq!(back.random, 2);
        assert_eq!(back.random2, 1);
        assert_eq!(back.doubleop, 1);
        assert_eq!(back.judgerank, 200);
        assert_eq!(back.freq, 75);
        assert!((back.total - 500.0).abs() < f64::EPSILON);
        assert_eq!(back.graphtype, 1);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let p = PracticeProperty::load(dir.path(), "nonexistent_sha");
        assert_eq!(p.judgerank, 100);
        assert_eq!(p.freq, 100);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let sha = "abc123def456";
        let original = PracticeProperty {
            starttime: 1000,
            endtime: 20000,
            freq: 80,
            total: 450.0,
            ..Default::default()
        };
        original.save(dir.path(), sha).unwrap();
        let loaded = PracticeProperty::load(dir.path(), sha);
        assert_eq!(loaded.starttime, 1000);
        assert_eq!(loaded.endtime, 20000);
        assert_eq!(loaded.freq, 80);
        assert!((loaded.total - 450.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deserialize_partial_json() {
        let json = r#"{"starttime": 500, "freq": 150}"#;
        let p: PracticeProperty = serde_json::from_str(json).unwrap();
        assert_eq!(p.starttime, 500);
        assert_eq!(p.freq, 150);
        // Other fields should be defaults
        assert_eq!(p.gaugetype, 2);
        assert_eq!(p.judgerank, 100);
    }
}
