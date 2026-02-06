use crate::model::note::{Note, NoteType, PlayMode};
use crate::traits::input::KeyEvent;

/// Autoplay mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoplayMode {
    /// Full autoplay: all lanes.
    Full,
    /// Assist scratch: only scratch lanes.
    AssistScratch,
    /// Assist keys: only non-scratch lanes.
    AssistKeys,
    /// No autoplay.
    Off,
}

/// Generate autoplay key events from chart notes.
///
/// Returns sorted KeyEvent list for notes that match the autoplay mode.
/// For normal notes: press at note time.
/// For LN/CN: press at start, release at end.
/// For HCN: press at start, release at end.
/// Mine notes are skipped (not pressed).
pub fn generate_autoplay_events(
    notes: &[Note],
    mode: PlayMode,
    autoplay_mode: AutoplayMode,
) -> Vec<KeyEvent> {
    if autoplay_mode == AutoplayMode::Off {
        return Vec::new();
    }

    let mut events = Vec::new();

    for note in notes {
        let is_scratch = mode.is_scratch(note.lane);
        let should_autoplay = match autoplay_mode {
            AutoplayMode::Full => true,
            AutoplayMode::AssistScratch => is_scratch,
            AutoplayMode::AssistKeys => !is_scratch,
            AutoplayMode::Off => false,
        };

        if !should_autoplay {
            continue;
        }

        match note.note_type {
            NoteType::Normal => {
                events.push(KeyEvent {
                    key: note.lane,
                    pressed: true,
                    time_us: note.time_us,
                });
            }
            NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote => {
                // Press at start, release at end
                events.push(KeyEvent {
                    key: note.lane,
                    pressed: true,
                    time_us: note.time_us,
                });
                events.push(KeyEvent {
                    key: note.lane,
                    pressed: false,
                    time_us: note.end_time_us,
                });
            }
            NoteType::Mine | NoteType::Invisible => {
                // Do not press mines or invisible notes
            }
        }
    }

    // Sort by time, then press before release at the same time
    events.sort_by(|a, b| {
        a.time_us
            .cmp(&b.time_us)
            .then_with(|| b.pressed.cmp(&a.pressed))
    });

    events
}

/// Scripted input that replays a sequence of key events.
/// Implements InputProvider for testing and autoplay.
pub struct ScriptedInput {
    events: Vec<KeyEvent>,
    index: usize,
    key_states: Vec<bool>,
}

impl ScriptedInput {
    pub fn new(events: Vec<KeyEvent>, key_count: usize) -> Self {
        Self {
            events,
            index: 0,
            key_states: vec![false; key_count],
        }
    }

    /// Poll events up to and including the given time.
    pub fn poll_up_to(&mut self, time_us: i64) -> Vec<KeyEvent> {
        let mut result = Vec::new();
        while self.index < self.events.len() && self.events[self.index].time_us <= time_us {
            let event = self.events[self.index];
            if event.key < self.key_states.len() {
                self.key_states[event.key] = event.pressed;
            }
            result.push(event);
            self.index += 1;
        }
        result
    }

    /// Check if a specific key is currently held down.
    pub fn is_pressed(&self, key: usize) -> bool {
        self.key_states.get(key).copied().unwrap_or(false)
    }

    /// Number of available keys.
    pub fn key_count(&self) -> usize {
        self.key_states.len()
    }

    /// Whether all events have been consumed.
    pub fn is_finished(&self) -> bool {
        self.index >= self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_note(lane: usize, note_type: NoteType, time_us: i64, end_time_us: i64) -> Note {
        Note {
            lane,
            note_type,
            time_us,
            end_time_us,
            wav_id: 0,
            damage: 0.0,
        }
    }

    #[test]
    fn autoplay_full_normal_notes() {
        let notes = vec![
            make_note(0, NoteType::Normal, 1_000_000, 0),
            make_note(1, NoteType::Normal, 2_000_000, 0),
        ];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Full);
        assert_eq!(events.len(), 2);
        assert!(events[0].pressed);
        assert_eq!(events[0].key, 0);
        assert_eq!(events[0].time_us, 1_000_000);
        assert!(events[1].pressed);
        assert_eq!(events[1].key, 1);
        assert_eq!(events[1].time_us, 2_000_000);
    }

    #[test]
    fn autoplay_full_long_note() {
        let notes = vec![make_note(0, NoteType::LongNote, 1_000_000, 2_000_000)];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Full);
        assert_eq!(events.len(), 2);
        assert!(events[0].pressed); // press at start
        assert_eq!(events[0].time_us, 1_000_000);
        assert!(!events[1].pressed); // release at end
        assert_eq!(events[1].time_us, 2_000_000);
    }

    #[test]
    fn autoplay_skips_mines() {
        let notes = vec![
            make_note(0, NoteType::Normal, 1_000_000, 0),
            make_note(1, NoteType::Mine, 1_500_000, 0),
            make_note(2, NoteType::Normal, 2_000_000, 0),
        ];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Full);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn autoplay_assist_scratch_only() {
        let notes = vec![
            make_note(0, NoteType::Normal, 1_000_000, 0), // key lane
            make_note(7, NoteType::Normal, 2_000_000, 0), // scratch lane for Beat7K
        ];
        let events =
            generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::AssistScratch);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, 7);
    }

    #[test]
    fn autoplay_assist_keys_only() {
        let notes = vec![
            make_note(0, NoteType::Normal, 1_000_000, 0),
            make_note(7, NoteType::Normal, 2_000_000, 0), // scratch
        ];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::AssistKeys);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, 0);
    }

    #[test]
    fn autoplay_off_returns_empty() {
        let notes = vec![make_note(0, NoteType::Normal, 1_000_000, 0)];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Off);
        assert!(events.is_empty());
    }

    #[test]
    fn autoplay_events_sorted_by_time() {
        let notes = vec![
            make_note(1, NoteType::Normal, 2_000_000, 0),
            make_note(0, NoteType::Normal, 1_000_000, 0),
        ];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Full);
        assert!(events[0].time_us <= events[1].time_us);
    }

    #[test]
    fn autoplay_charge_note() {
        let notes = vec![make_note(0, NoteType::ChargeNote, 1_000_000, 3_000_000)];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Full);
        assert_eq!(events.len(), 2);
        assert!(events[0].pressed);
        assert!(!events[1].pressed);
    }

    #[test]
    fn autoplay_hell_charge_note() {
        let notes = vec![make_note(0, NoteType::HellChargeNote, 1_000_000, 3_000_000)];
        let events = generate_autoplay_events(&notes, PlayMode::Beat7K, AutoplayMode::Full);
        assert_eq!(events.len(), 2);
    }

    // =========================================================================
    // ScriptedInput tests
    // =========================================================================

    #[test]
    fn scripted_input_poll() {
        let events = vec![
            KeyEvent {
                key: 0,
                pressed: true,
                time_us: 1_000_000,
            },
            KeyEvent {
                key: 1,
                pressed: true,
                time_us: 2_000_000,
            },
            KeyEvent {
                key: 0,
                pressed: false,
                time_us: 3_000_000,
            },
        ];
        let mut input = ScriptedInput::new(events, 8);

        let polled = input.poll_up_to(1_500_000);
        assert_eq!(polled.len(), 1);
        assert!(input.is_pressed(0));
        assert!(!input.is_pressed(1));

        let polled = input.poll_up_to(2_500_000);
        assert_eq!(polled.len(), 1);
        assert!(input.is_pressed(1));

        let polled = input.poll_up_to(3_500_000);
        assert_eq!(polled.len(), 1);
        assert!(!input.is_pressed(0));
        assert!(input.is_finished());
    }

    #[test]
    fn scripted_input_poll_all_at_once() {
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
        let mut input = ScriptedInput::new(events, 8);
        let polled = input.poll_up_to(1_000_000);
        assert_eq!(polled.len(), 2);
        assert!(input.is_finished());
    }

    #[test]
    fn scripted_input_empty() {
        let mut input = ScriptedInput::new(vec![], 8);
        let polled = input.poll_up_to(1_000_000);
        assert!(polled.is_empty());
        assert!(input.is_finished());
    }
}
