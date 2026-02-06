use crate::replay::replay_data::ReplayData;
use crate::traits::input::KeyEvent;

/// Records key input events during play for replay saving.
#[derive(Debug, Clone)]
pub struct ReplayRecorder {
    events: Vec<KeyEvent>,
}

impl Default for ReplayRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayRecorder {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record a key event.
    pub fn record(&mut self, event: KeyEvent) {
        self.events.push(event);
    }

    /// Get all recorded events.
    pub fn events(&self) -> &[KeyEvent] {
        &self.events
    }

    /// Build a ReplayData from the recorded events.
    pub fn build_replay_data(
        &self,
        player: String,
        sha256: String,
        mode: u32,
        gauge: usize,
    ) -> ReplayData {
        ReplayData {
            player,
            sha256,
            mode,
            gauge,
            keylog: self.events.clone(),
            ..Default::default()
        }
    }

    /// Clear all recorded events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_retrieve() {
        let mut recorder = ReplayRecorder::new();
        assert!(recorder.events().is_empty());

        recorder.record(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 1_000_000,
        });
        recorder.record(KeyEvent {
            key: 1,
            pressed: true,
            time_us: 2_000_000,
        });
        recorder.record(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 3_000_000,
        });

        assert_eq!(recorder.events().len(), 3);
        assert_eq!(recorder.events()[0].key, 0);
        assert!(recorder.events()[0].pressed);
    }

    #[test]
    fn build_replay_data() {
        let mut recorder = ReplayRecorder::new();
        recorder.record(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 100,
        });
        recorder.record(KeyEvent {
            key: 0,
            pressed: false,
            time_us: 200,
        });

        let data =
            recorder.build_replay_data("player1".to_string(), "sha256hash".to_string(), 7, 2);
        assert_eq!(data.player, "player1");
        assert_eq!(data.sha256, "sha256hash");
        assert_eq!(data.mode, 7);
        assert_eq!(data.gauge, 2);
        assert_eq!(data.keylog.len(), 2);
    }

    #[test]
    fn clear_events() {
        let mut recorder = ReplayRecorder::new();
        recorder.record(KeyEvent {
            key: 0,
            pressed: true,
            time_us: 100,
        });
        assert_eq!(recorder.events().len(), 1);

        recorder.clear();
        assert!(recorder.events().is_empty());
    }

    #[test]
    fn default_recorder() {
        let recorder = ReplayRecorder::default();
        assert!(recorder.events().is_empty());
    }
}
