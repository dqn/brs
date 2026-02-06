use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::traits::input::KeyEvent;

/// Replay data for a play session.
/// Compatible with beatoraja's ReplayData format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayData {
    /// Player name.
    #[serde(default)]
    pub player: String,
    /// Chart SHA-256 hash.
    #[serde(default)]
    pub sha256: String,
    /// Play mode (lane count).
    #[serde(default)]
    pub mode: u32,
    /// Compressed key input log (base64 + gzip).
    #[serde(default)]
    pub keyinput: Option<String>,
    /// Gauge type index.
    #[serde(default)]
    pub gauge: usize,
    /// Random option for 1P.
    #[serde(default)]
    pub randomoption: i32,
    /// Random option seed for 1P.
    #[serde(default)]
    pub randomoptionseed: i64,
    /// Random option for 2P.
    #[serde(default)]
    pub randomoption2: i32,
    /// Random option seed for 2P.
    #[serde(default)]
    pub randomoption2seed: i64,
    /// Double option.
    #[serde(default)]
    pub doubleoption: i32,
    /// Play date (unix timestamp).
    #[serde(default)]
    pub date: i64,

    /// Deserialized key log (not serialized directly).
    #[serde(skip)]
    pub keylog: Vec<KeyEvent>,
}

impl Default for ReplayData {
    fn default() -> Self {
        Self {
            player: String::new(),
            sha256: String::new(),
            mode: 0,
            keyinput: None,
            gauge: 0,
            randomoption: 0,
            randomoptionseed: -1,
            randomoption2: 0,
            randomoption2seed: -1,
            doubleoption: 0,
            date: 0,
            keylog: Vec::new(),
        }
    }
}

impl ReplayData {
    /// Compress keylog into the keyinput field (base64 + gzip).
    /// Follows beatoraja's shrink() format:
    /// Each entry is 10 bytes: 2 bytes keycode (i16 LE) + 8 bytes time (little-endian).
    /// Keycode = (key + 1) * (pressed ? 1 : -1).
    pub fn shrink(&mut self) -> Result<()> {
        if self.keylog.is_empty() {
            return Ok(());
        }

        let mut raw = Vec::with_capacity(self.keylog.len() * 10);
        for event in &self.keylog {
            let keycode: i16 = (event.key as i16 + 1) * if event.pressed { 1 } else { -1 };
            raw.extend_from_slice(&keycode.to_le_bytes());
            raw.extend_from_slice(&event.time_us.to_le_bytes());
        }

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw)?;
        let compressed = encoder.finish()?;

        self.keyinput = Some(URL_SAFE.encode(compressed));
        self.keylog.clear();
        Ok(())
    }

    /// Expand keyinput back into keylog.
    /// Follows beatoraja's validate() format.
    pub fn expand(&mut self) -> Result<()> {
        let keyinput = match &self.keyinput {
            Some(s) if !s.is_empty() => s.clone(),
            _ => return Ok(()),
        };

        let compressed = URL_SAFE
            .decode(keyinput.as_bytes())
            .map_err(|e| anyhow!("base64 decode error: {e}"))?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut raw = Vec::new();
        decoder.read_to_end(&mut raw)?;

        let mut keylog = Vec::with_capacity(raw.len() / 10);
        let mut pos = 0;
        while pos + 10 <= raw.len() {
            let keycode_bytes: [u8; 2] = raw[pos..pos + 2]
                .try_into()
                .map_err(|_| anyhow!("invalid keyinput data"))?;
            let keycode = i16::from_le_bytes(keycode_bytes);
            let time_bytes: [u8; 8] = raw[pos + 2..pos + 10]
                .try_into()
                .map_err(|_| anyhow!("invalid keyinput data"))?;
            let time_us = i64::from_le_bytes(time_bytes);

            let key = (keycode.unsigned_abs() - 1) as usize;
            let pressed = keycode >= 0;

            keylog.push(KeyEvent {
                key,
                pressed,
                time_us,
            });
            pos += 10;
        }

        self.keylog = keylog;
        self.keyinput = None;
        Ok(())
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| anyhow!("JSON serialization error: {e}"))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| anyhow!("JSON deserialization error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_events() -> Vec<KeyEvent> {
        vec![
            KeyEvent {
                key: 0,
                pressed: true,
                time_us: 1_000_000,
            },
            KeyEvent {
                key: 1,
                pressed: true,
                time_us: 1_500_000,
            },
            KeyEvent {
                key: 0,
                pressed: false,
                time_us: 2_000_000,
            },
            KeyEvent {
                key: 1,
                pressed: false,
                time_us: 2_500_000,
            },
        ]
    }

    #[test]
    fn shrink_and_expand_roundtrip() {
        let events = make_events();
        let mut data = ReplayData {
            keylog: events.clone(),
            ..Default::default()
        };

        data.shrink().unwrap();
        assert!(data.keyinput.is_some());
        assert!(data.keylog.is_empty());

        data.expand().unwrap();
        assert!(data.keyinput.is_none());
        assert_eq!(data.keylog.len(), events.len());

        for (a, b) in data.keylog.iter().zip(events.iter()) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.pressed, b.pressed);
            assert_eq!(a.time_us, b.time_us);
        }
    }

    #[test]
    fn shrink_empty_keylog() {
        let mut data = ReplayData::default();
        data.shrink().unwrap();
        assert!(data.keyinput.is_none());
    }

    #[test]
    fn expand_no_keyinput() {
        let mut data = ReplayData::default();
        data.expand().unwrap();
        assert!(data.keylog.is_empty());
    }

    #[test]
    fn json_roundtrip() {
        let mut data = ReplayData {
            player: "testplayer".to_string(),
            sha256: "abc123".to_string(),
            mode: 7,
            gauge: 2,
            date: 1234567890,
            keylog: make_events(),
            ..Default::default()
        };
        data.shrink().unwrap();

        let json = data.to_json().unwrap();
        let mut restored = ReplayData::from_json(&json).unwrap();
        assert_eq!(restored.player, "testplayer");
        assert_eq!(restored.sha256, "abc123");
        assert_eq!(restored.mode, 7);
        assert_eq!(restored.gauge, 2);

        restored.expand().unwrap();
        assert_eq!(restored.keylog.len(), 4);
    }

    #[test]
    fn keycode_encoding_matches_beatoraja() {
        // beatoraja format: keycode = (key + 1) * (pressed ? 1 : -1)
        // key=0, pressed=true -> +1
        // key=0, pressed=false -> -1
        // key=3, pressed=true -> +4
        // key=3, pressed=false -> -4
        let events = vec![
            KeyEvent {
                key: 0,
                pressed: true,
                time_us: 100,
            },
            KeyEvent {
                key: 0,
                pressed: false,
                time_us: 200,
            },
            KeyEvent {
                key: 3,
                pressed: true,
                time_us: 300,
            },
        ];
        let mut data = ReplayData {
            keylog: events,
            ..Default::default()
        };
        data.shrink().unwrap();
        data.expand().unwrap();
        assert_eq!(data.keylog[0].key, 0);
        assert!(data.keylog[0].pressed);
        assert_eq!(data.keylog[1].key, 0);
        assert!(!data.keylog[1].pressed);
        assert_eq!(data.keylog[2].key, 3);
        assert!(data.keylog[2].pressed);
    }

    #[test]
    fn default_values() {
        let data = ReplayData::default();
        assert!(data.player.is_empty());
        assert!(data.sha256.is_empty());
        assert_eq!(data.mode, 0);
        assert_eq!(data.gauge, 0);
        assert_eq!(data.randomoptionseed, -1);
        assert_eq!(data.date, 0);
    }
}
