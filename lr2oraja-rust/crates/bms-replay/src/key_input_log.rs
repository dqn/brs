// Key input log for replay recording.
//
// Matches Java `KeyInputLog.java`.

use serde::{Deserialize, Serialize};

/// A single key input event in a replay.
///
/// Stores the time (in microseconds), key code, and press/release state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyInputLog {
    /// Key input time in microseconds.
    #[serde(default)]
    pub presstime: i64,
    /// Key code (lane index).
    pub keycode: i32,
    /// Whether the key was pressed (true) or released (false).
    pub pressed: bool,
    /// Legacy field: key input time in milliseconds (for old data compatibility).
    #[serde(default)]
    pub time: i64,
}

impl KeyInputLog {
    pub fn new(presstime: i64, keycode: i32, pressed: bool) -> Self {
        Self {
            presstime,
            keycode,
            pressed,
            time: 0,
        }
    }

    /// Returns the input time in microseconds.
    ///
    /// Uses `presstime` if non-zero, otherwise converts legacy `time` (ms) to μs.
    pub fn get_time(&self) -> i64 {
        if self.presstime != 0 {
            self.presstime
        } else {
            self.time * 1000
        }
    }

    /// Validates and migrates legacy data.
    ///
    /// Returns `true` if the entry is valid.
    pub fn validate(&mut self) -> bool {
        if self.time > 0 {
            self.presstime = self.time * 1000;
            self.time = 0;
        }
        self.presstime >= 0 && self.keycode >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let log = KeyInputLog::new(1000000, 3, true);
        assert_eq!(log.presstime, 1000000);
        assert_eq!(log.keycode, 3);
        assert!(log.pressed);
        assert_eq!(log.time, 0);
    }

    #[test]
    fn test_get_time_presstime() {
        let log = KeyInputLog::new(500000, 0, true);
        assert_eq!(log.get_time(), 500000);
    }

    #[test]
    fn test_get_time_legacy_fallback() {
        let log = KeyInputLog {
            presstime: 0,
            keycode: 1,
            pressed: true,
            time: 500,
        };
        assert_eq!(log.get_time(), 500000);
    }

    #[test]
    fn test_validate_legacy_migration() {
        let mut log = KeyInputLog {
            presstime: 0,
            keycode: 2,
            pressed: false,
            time: 100,
        };
        assert!(log.validate());
        assert_eq!(log.presstime, 100000);
        assert_eq!(log.time, 0);
    }

    #[test]
    fn test_validate_invalid() {
        let mut log = KeyInputLog::new(-1, 0, true);
        assert!(!log.validate());

        let mut log2 = KeyInputLog::new(0, -1, true);
        assert!(!log2.validate());
    }

    #[test]
    fn test_serde_round_trip() {
        let log = KeyInputLog::new(123456, 5, true);
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: KeyInputLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log, deserialized);
    }
}
