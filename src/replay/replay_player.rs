use crate::replay::replay_data::ReplayData;
use crate::state::play::autoplay::ScriptedInput;
use crate::traits::input::KeyEvent;

/// Replay player that replays recorded key events.
/// Uses ScriptedInput internally for event delivery.
pub struct ReplayPlayer {
    input: ScriptedInput,
}

impl ReplayPlayer {
    /// Create a replay player from replay data.
    /// The replay data's keylog must be expanded (call expand() first if needed).
    pub fn new(replay: &ReplayData, key_count: usize) -> Self {
        Self {
            input: ScriptedInput::new(replay.keylog.clone(), key_count),
        }
    }

    /// Create a replay player from a list of key events.
    pub fn from_events(events: Vec<KeyEvent>, key_count: usize) -> Self {
        Self {
            input: ScriptedInput::new(events, key_count),
        }
    }

    /// Poll events up to the given time.
    pub fn poll_up_to(&mut self, time_us: i64) -> Vec<KeyEvent> {
        self.input.poll_up_to(time_us)
    }

    /// Check if a key is currently held.
    pub fn is_pressed(&self, key: usize) -> bool {
        self.input.is_pressed(key)
    }

    /// Whether all events have been replayed.
    pub fn is_finished(&self) -> bool {
        self.input.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_from_events() {
        let events = vec![
            KeyEvent {
                key: 0,
                pressed: true,
                time_us: 1_000_000,
            },
            KeyEvent {
                key: 0,
                pressed: false,
                time_us: 2_000_000,
            },
        ];
        let mut player = ReplayPlayer::from_events(events, 8);

        let polled = player.poll_up_to(1_500_000);
        assert_eq!(polled.len(), 1);
        assert!(player.is_pressed(0));

        let polled = player.poll_up_to(2_500_000);
        assert_eq!(polled.len(), 1);
        assert!(!player.is_pressed(0));
        assert!(player.is_finished());
    }

    #[test]
    fn replay_from_replay_data() {
        let data = ReplayData {
            keylog: vec![
                KeyEvent {
                    key: 1,
                    pressed: true,
                    time_us: 500_000,
                },
                KeyEvent {
                    key: 1,
                    pressed: false,
                    time_us: 1_500_000,
                },
            ],
            ..Default::default()
        };
        let mut player = ReplayPlayer::new(&data, 8);

        let polled = player.poll_up_to(1_000_000);
        assert_eq!(polled.len(), 1);
        assert!(player.is_pressed(1));
    }

    #[test]
    fn replay_determinism() {
        // Playing the same events twice should produce identical results
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
            KeyEvent {
                key: 0,
                pressed: false,
                time_us: 300,
            },
        ];

        let mut player1 = ReplayPlayer::from_events(events.clone(), 8);
        let mut player2 = ReplayPlayer::from_events(events, 8);

        let polled1 = player1.poll_up_to(1_000_000);
        let polled2 = player2.poll_up_to(1_000_000);

        assert_eq!(polled1.len(), polled2.len());
        for (a, b) in polled1.iter().zip(polled2.iter()) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.pressed, b.pressed);
            assert_eq!(a.time_us, b.time_us);
        }
    }
}
