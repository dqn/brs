use crate::traits::input::KeyEvent;

/// Records key input events during play for replay storage.
/// Events are collected in chronological order.
#[derive(Debug, Clone, Default)]
pub struct KeyInputLog {
    events: Vec<KeyEvent>,
}

impl KeyInputLog {
    /// Create an empty log.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record a key event.
    pub fn record(&mut self, event: KeyEvent) {
        self.events.push(event);
    }

    /// Record multiple events.
    pub fn record_all(&mut self, events: &[KeyEvent]) {
        self.events.extend_from_slice(events);
    }

    /// Get all recorded events.
    pub fn events(&self) -> &[KeyEvent] {
        &self.events
    }

    /// Take ownership of all events, clearing the log.
    pub fn take(&mut self) -> Vec<KeyEvent> {
        std::mem::take(&mut self.events)
    }

    /// Number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_retrieve() {
        let mut log = KeyInputLog::new();
        assert!(log.is_empty());

        log.record(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 1000,
        });
        log.record(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 2000,
        });

        assert_eq!(log.len(), 2);
        assert_eq!(log.events()[0].time_us, 1000);
        assert_eq!(log.events()[1].time_us, 2000);
    }

    #[test]
    fn record_all() {
        let mut log = KeyInputLog::new();
        let events = vec![
            KeyEvent {
                key: 0,
                pressed: true,
                time_us: 100,
            },
            KeyEvent {
                key: 1,
                pressed: true,
                time_us: 200,
            },
        ];
        log.record_all(&events);
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn take_clears_log() {
        let mut log = KeyInputLog::new();
        log.record(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 100,
        });

        let events = log.take();
        assert_eq!(events.len(), 1);
        assert!(log.is_empty());
    }

    #[test]
    fn clear() {
        let mut log = KeyInputLog::new();
        log.record(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 100,
        });
        log.clear();
        assert!(log.is_empty());
    }
}
