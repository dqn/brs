// ReplayData — replay recording with compressed key input log.
//
// Matches Java `ReplayData.java`.

use std::io::{Read, Write};
use std::path::Path;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};

use bms_config::PlayConfig;
use bms_pattern::PatternModifyLog;

use crate::key_input_log::KeyInputLog;

/// Complete replay data for a play session.
///
/// Contains key input logs, pattern modification info, gauge type, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayData {
    /// Player name.
    #[serde(default)]
    pub player: String,
    /// Song SHA-256 hash.
    #[serde(default)]
    pub sha256: String,
    /// Play mode (int representation).
    #[serde(default)]
    pub mode: i32,
    /// Key input log entries (populated after `validate()`).
    #[serde(default)]
    pub keylog: Vec<KeyInputLog>,
    /// Compressed key input data (Base64 URL-safe encoded GZIP).
    /// Populated after `shrink()`, cleared after `validate()`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyinput: Option<String>,
    /// Gauge type.
    #[serde(default)]
    pub gauge: i32,
    /// Pattern modification log (legacy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<Vec<PatternModifyLog>>,
    /// Lane shuffle pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "laneShufflePattern")]
    pub lane_shuffle_pattern: Option<Vec<Vec<i32>>>,
    /// Random sequence numbers (for #RANDOM in BMS).
    #[serde(default)]
    pub rand: Vec<i32>,
    /// Play date (unix timestamp).
    #[serde(default)]
    pub date: i64,
    /// 7-to-9 arrangement pattern.
    #[serde(default, rename = "sevenToNinePattern")]
    pub seven_to_nine_pattern: i32,
    /// Pattern option (1P).
    #[serde(default)]
    pub randomoption: i32,
    /// Pattern option seed (1P).
    #[serde(default)]
    pub randomoptionseed: i64,
    /// Pattern option (2P).
    #[serde(default)]
    pub randomoption2: i32,
    /// Pattern option seed (2P).
    #[serde(default)]
    pub randomoption2seed: i64,
    /// DP option.
    #[serde(default)]
    pub doubleoption: i32,
    /// Play config snapshot at time of play.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<PlayConfig>,
}

impl Default for ReplayData {
    fn default() -> Self {
        Self {
            player: String::new(),
            sha256: String::new(),
            mode: 0,
            keylog: Vec::new(),
            keyinput: None,
            gauge: 0,
            pattern: None,
            lane_shuffle_pattern: None,
            rand: Vec::new(),
            date: 0,
            seven_to_nine_pattern: 0,
            randomoption: 0,
            randomoptionseed: -1,
            randomoption2: 0,
            randomoption2seed: -1,
            doubleoption: 0,
            config: None,
        }
    }
}

impl ReplayData {
    /// Compress keylog into Base64 URL-safe encoded GZIP data.
    ///
    /// Each KeyInputLog entry is encoded as 9 bytes:
    /// - 1 byte: `(keycode + 1) * sign` (positive if pressed, negative if released)
    /// - 8 bytes: time in little-endian i64
    ///
    /// After compression, `keylog` is cleared and `keyinput` is populated.
    pub fn shrink(&mut self) {
        if self.keylog.is_empty() {
            return;
        }

        let mut raw = Vec::with_capacity(self.keylog.len() * 9);
        for log in &self.keylog {
            let sign: i8 = if log.pressed { 1 } else { -1 };
            let keycode_byte = ((log.keycode + 1) * sign as i32) as i8;
            raw.push(keycode_byte as u8);
            raw.extend_from_slice(&log.get_time().to_le_bytes());
        }

        let mut gzip_buf = Vec::new();
        let mut encoder = GzEncoder::new(&mut gzip_buf, Compression::default());
        encoder.write_all(&raw).unwrap();
        encoder.finish().unwrap();

        self.keyinput = Some(URL_SAFE.encode(&gzip_buf));
        self.keylog.clear();
    }

    /// Decompress keyinput and populate keylog.
    ///
    /// Decodes Base64 → GZIP decompress → parse 9-byte records.
    /// After validation, `keyinput` is cleared.
    /// Returns `true` if keylog is non-empty after validation.
    pub fn validate(&mut self) -> bool {
        if let Some(ref input) = self.keyinput {
            if let Ok(gzip_data) = URL_SAFE.decode(input) {
                let mut decoder = GzDecoder::new(&gzip_data[..]);
                let mut raw = Vec::new();
                if decoder.read_to_end(&mut raw).is_ok() {
                    let mut logs = Vec::with_capacity(raw.len() / 9);
                    let mut pos = 0;
                    while pos + 9 <= raw.len() {
                        let keycode_byte = raw[pos] as i8;
                        let time_bytes: [u8; 8] = raw[pos + 1..pos + 9].try_into().unwrap();
                        let time = i64::from_le_bytes(time_bytes);
                        let keycode = keycode_byte.unsigned_abs() as i32 - 1;
                        let pressed = keycode_byte >= 0;
                        logs.push(KeyInputLog::new(time, keycode, pressed));
                        pos += 9;
                    }
                    self.keylog = logs;
                }
            }
            self.keyinput = None;
        }

        // Remove invalid entries
        self.keylog.retain_mut(|log| log.validate());
        !self.keylog.is_empty()
    }
}

/// Read a .brd replay file (GZIP-compressed JSON).
pub fn read_brd(path: &Path) -> Result<ReplayData> {
    let file = std::fs::File::open(path)?;
    let decoder = GzDecoder::new(file);
    let mut replay: ReplayData = serde_json::from_reader(decoder)?;
    replay.validate();
    Ok(replay)
}

/// Write a .brd replay file (GZIP-compressed JSON).
pub fn write_brd(replay: &mut ReplayData, path: &Path) -> Result<()> {
    replay.shrink();
    let file = std::fs::File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    serde_json::to_writer(encoder, replay)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_keylog() -> Vec<KeyInputLog> {
        vec![
            KeyInputLog::new(0, 0, true),
            KeyInputLog::new(100000, 3, true),
            KeyInputLog::new(200000, 0, false),
            KeyInputLog::new(300000, 3, false),
            KeyInputLog::new(500000, 6, true),
            KeyInputLog::new(600000, 6, false),
        ]
    }

    #[test]
    fn test_shrink_validate_round_trip() {
        let original = sample_keylog();
        let mut replay = ReplayData {
            keylog: original.clone(),
            ..Default::default()
        };

        replay.shrink();
        assert!(replay.keylog.is_empty());
        assert!(replay.keyinput.is_some());

        replay.validate();
        assert!(replay.keyinput.is_none());
        assert_eq!(replay.keylog.len(), original.len());

        for (a, b) in replay.keylog.iter().zip(original.iter()) {
            assert_eq!(a.presstime, b.presstime);
            assert_eq!(a.keycode, b.keycode);
            assert_eq!(a.pressed, b.pressed);
        }
    }

    #[test]
    fn test_shrink_empty_keylog() {
        let mut replay = ReplayData::default();
        replay.shrink();
        assert!(replay.keyinput.is_none());
    }

    #[test]
    fn test_validate_no_keyinput() {
        let mut replay = ReplayData::default();
        assert!(!replay.validate());
    }

    #[test]
    fn test_keycode_encoding_all_keys() {
        // Test keycodes 0..=25 (covers all BMS keys including scratch)
        let mut logs: Vec<KeyInputLog> = Vec::new();
        for kc in 0..=25 {
            logs.push(KeyInputLog::new(kc as i64 * 10000, kc, true));
            logs.push(KeyInputLog::new(kc as i64 * 10000 + 5000, kc, false));
        }
        let original = logs.clone();
        let mut replay = ReplayData {
            keylog: logs,
            ..Default::default()
        };
        replay.shrink();
        replay.validate();
        assert_eq!(replay.keylog.len(), original.len());
        for (a, b) in replay.keylog.iter().zip(original.iter()) {
            assert_eq!(a.keycode, b.keycode);
            assert_eq!(a.pressed, b.pressed);
            assert_eq!(a.presstime, b.presstime);
        }
    }

    #[test]
    fn test_brd_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.brd");

        let mut replay = ReplayData {
            player: "test_player".into(),
            sha256: "abc123".into(),
            mode: 7,
            keylog: sample_keylog(),
            gauge: 2,
            randomoption: 1,
            randomoptionseed: 42,
            ..Default::default()
        };

        write_brd(&mut replay, &path).unwrap();
        let loaded = read_brd(&path).unwrap();

        assert_eq!(loaded.player, "test_player");
        assert_eq!(loaded.sha256, "abc123");
        assert_eq!(loaded.mode, 7);
        assert_eq!(loaded.gauge, 2);
        assert_eq!(loaded.randomoption, 1);
        assert_eq!(loaded.randomoptionseed, 42);
        assert_eq!(loaded.keylog.len(), 6);
    }

    #[test]
    fn test_serde_json_round_trip() {
        let mut replay = ReplayData {
            player: "player1".into(),
            keylog: sample_keylog(),
            ..Default::default()
        };
        replay.shrink();

        let json = serde_json::to_string(&replay).unwrap();
        let mut deserialized: ReplayData = serde_json::from_str(&json).unwrap();
        deserialized.validate();

        assert_eq!(deserialized.player, "player1");
        assert_eq!(deserialized.keylog.len(), 6);
    }

    #[test]
    fn test_validate_removes_invalid_entries() {
        let mut replay = ReplayData {
            keylog: vec![
                KeyInputLog::new(100, 0, true),
                KeyInputLog::new(-1, 1, true), // invalid: negative presstime
                KeyInputLog::new(200, -1, false), // invalid: negative keycode
                KeyInputLog::new(300, 2, false),
            ],
            ..Default::default()
        };
        assert!(replay.validate());
        assert_eq!(replay.keylog.len(), 2);
    }

    #[test]
    fn test_legacy_time_field_migration() {
        let mut replay = ReplayData {
            keylog: vec![KeyInputLog {
                presstime: 0,
                keycode: 1,
                pressed: true,
                time: 500,
            }],
            ..Default::default()
        };
        replay.validate();
        assert_eq!(replay.keylog[0].presstime, 500000);
        assert_eq!(replay.keylog[0].time, 0);
    }
}
