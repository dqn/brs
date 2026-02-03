use serde::{Deserialize, Serialize};

/// A single input event for replay recording.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeyInputLog {
    /// Timestamp in microseconds from game start.
    pub time_us: u64,
    /// Lane index (0 = Scratch, 1-14 = Key1-Key14, 8 = Scratch2).
    pub lane: u8,
    /// true = pressed, false = released.
    pub pressed: bool,
}

/// Logger for recording input events.
#[derive(Debug)]
pub struct InputLogger {
    logs: Vec<KeyInputLog>,
}

impl InputLogger {
    /// Create a new input logger with pre-allocated capacity.
    pub fn new() -> Self {
        Self {
            logs: Vec::with_capacity(10000),
        }
    }

    /// Record an input event.
    pub fn record(&mut self, time_us: u64, lane: u8, pressed: bool) {
        self.logs.push(KeyInputLog {
            time_us,
            lane,
            pressed,
        });
    }

    /// Get all recorded logs.
    pub fn logs(&self) -> &[KeyInputLog] {
        &self.logs
    }

    /// Take ownership of logs (consumes logger).
    pub fn into_logs(self) -> Vec<KeyInputLog> {
        self.logs
    }

    /// Clear all logs.
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// Get the number of recorded events.
    pub fn len(&self) -> usize {
        self.logs.len()
    }

    /// Check if no events are recorded.
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }
}

impl Default for InputLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_logger_new() {
        let logger = InputLogger::new();
        assert!(logger.is_empty());
        assert_eq!(logger.len(), 0);
    }

    #[test]
    fn test_input_logger_record() {
        let mut logger = InputLogger::new();
        logger.record(1000, 0, true);
        logger.record(2000, 0, false);

        assert_eq!(logger.len(), 2);
        assert_eq!(logger.logs()[0].time_us, 1000);
        assert!(logger.logs()[0].pressed);
        assert_eq!(logger.logs()[1].time_us, 2000);
        assert!(!logger.logs()[1].pressed);
    }

    #[test]
    fn test_input_logger_clear() {
        let mut logger = InputLogger::new();
        logger.record(1000, 0, true);
        assert!(!logger.is_empty());

        logger.clear();
        assert!(logger.is_empty());
    }

    #[test]
    fn test_input_logger_into_logs() {
        let mut logger = InputLogger::new();
        logger.record(1000, 1, true);
        logger.record(2000, 1, false);

        let logs = logger.into_logs();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_key_input_log_serialization() {
        let log = KeyInputLog {
            time_us: 1000,
            lane: 1,
            pressed: true,
        };
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: KeyInputLog = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.time_us, log.time_us);
        assert_eq!(deserialized.lane, log.lane);
        assert_eq!(deserialized.pressed, log.pressed);
    }
}
