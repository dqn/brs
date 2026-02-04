//! Test utilities for building notes, charts, and simulating inputs.
//!
//! This module provides helpers for creating test fixtures in a fluent manner.

#[cfg(test)]
pub mod builders {
    use crate::model::note::{Lane, Note, NoteType};
    use crate::state::play::NoteWithIndex;

    /// Builder for creating test notes.
    #[derive(Debug, Clone)]
    pub struct NoteBuilder {
        lane: Lane,
        time_ms: f64,
        end_time_ms: Option<f64>,
        wav_id: u16,
        note_type: NoteType,
        mine_damage: Option<f64>,
    }

    impl NoteBuilder {
        /// Create a new normal note builder.
        pub fn normal(lane: Lane, time_ms: f64) -> Self {
            Self {
                lane,
                time_ms,
                end_time_ms: None,
                wav_id: 1,
                note_type: NoteType::Normal,
                mine_damage: None,
            }
        }

        /// Create a long note start builder.
        pub fn long_start(lane: Lane, start_ms: f64, end_ms: f64) -> Self {
            Self {
                lane,
                time_ms: start_ms,
                end_time_ms: Some(end_ms),
                wav_id: 1,
                note_type: NoteType::LongStart,
                mine_damage: None,
            }
        }

        /// Create a long note end builder.
        pub fn long_end(lane: Lane, time_ms: f64) -> Self {
            Self {
                lane,
                time_ms,
                end_time_ms: None,
                wav_id: 1,
                note_type: NoteType::LongEnd,
                mine_damage: None,
            }
        }

        /// Create an invisible note builder.
        pub fn invisible(lane: Lane, time_ms: f64) -> Self {
            Self {
                lane,
                time_ms,
                end_time_ms: None,
                wav_id: 1,
                note_type: NoteType::Invisible,
                mine_damage: None,
            }
        }

        /// Create a mine note builder.
        pub fn mine(lane: Lane, time_ms: f64, damage: f64) -> Self {
            Self {
                lane,
                time_ms,
                end_time_ms: None,
                wav_id: 0,
                note_type: NoteType::Mine,
                mine_damage: Some(damage),
            }
        }

        /// Set the wav_id.
        pub fn wav_id(mut self, id: u16) -> Self {
            self.wav_id = id;
            self
        }

        /// Build the Note.
        pub fn build(self) -> Note {
            Note {
                lane: self.lane,
                start_time_ms: self.time_ms,
                end_time_ms: self.end_time_ms,
                wav_id: self.wav_id,
                note_type: self.note_type,
                mine_damage: self.mine_damage,
            }
        }
    }

    /// Builder for creating a list of notes with indices.
    #[derive(Debug, Default)]
    pub struct ChartBuilder {
        notes: Vec<Note>,
    }

    impl ChartBuilder {
        pub fn new() -> Self {
            Self { notes: Vec::new() }
        }

        /// Add a note to the chart.
        pub fn add(mut self, note: Note) -> Self {
            self.notes.push(note);
            self
        }

        /// Add a normal note.
        pub fn normal(self, lane: Lane, time_ms: f64) -> Self {
            self.add(NoteBuilder::normal(lane, time_ms).build())
        }

        /// Add a long note (start + end pair).
        pub fn long_note(self, lane: Lane, start_ms: f64, end_ms: f64) -> Self {
            self.add(NoteBuilder::long_start(lane, start_ms, end_ms).build())
                .add(NoteBuilder::long_end(lane, end_ms).build())
        }

        /// Add an invisible note.
        pub fn invisible(self, lane: Lane, time_ms: f64) -> Self {
            self.add(NoteBuilder::invisible(lane, time_ms).build())
        }

        /// Add a mine note.
        pub fn mine(self, lane: Lane, time_ms: f64, damage: f64) -> Self {
            self.add(NoteBuilder::mine(lane, time_ms, damage).build())
        }

        /// Build to a list of NoteWithIndex.
        pub fn build(self) -> Vec<NoteWithIndex> {
            self.notes
                .into_iter()
                .enumerate()
                .map(|(index, note)| NoteWithIndex { index, note })
                .collect()
        }

        /// Build to a list of raw Notes.
        pub fn build_notes(self) -> Vec<Note> {
            self.notes
        }
    }

    /// Create a simple chart with normal notes for testing.
    pub fn create_simple_chart(
        note_count: usize,
        lane: Lane,
        interval_ms: f64,
    ) -> Vec<NoteWithIndex> {
        let mut builder = ChartBuilder::new();
        for i in 0..note_count {
            let time_ms = (i as f64) * interval_ms;
            builder = builder.normal(lane, time_ms);
        }
        builder.build()
    }

    /// Create a chart with a single long note for testing.
    pub fn create_ln_chart(lane: Lane, start_ms: f64, end_ms: f64) -> Vec<NoteWithIndex> {
        ChartBuilder::new()
            .long_note(lane, start_ms, end_ms)
            .build()
    }
}

#[cfg(test)]
pub mod input_sim {
    use crate::model::note::Lane;

    /// Input event for simulation.
    #[derive(Debug, Clone, Copy)]
    pub enum InputEvent {
        Press { lane: Lane, time_ms: f64 },
        Release { lane: Lane, time_ms: f64 },
    }

    impl InputEvent {
        pub fn press(lane: Lane, time_ms: f64) -> Self {
            Self::Press { lane, time_ms }
        }

        pub fn release(lane: Lane, time_ms: f64) -> Self {
            Self::Release { lane, time_ms }
        }
    }

    /// Simple input simulator for testing judgment logic.
    #[derive(Debug, Default)]
    pub struct InputSimulator {
        events: Vec<InputEvent>,
    }

    impl InputSimulator {
        pub fn new() -> Self {
            Self { events: Vec::new() }
        }

        /// Add a press event.
        pub fn press(mut self, lane: Lane, time_ms: f64) -> Self {
            self.events.push(InputEvent::press(lane, time_ms));
            self
        }

        /// Add a release event.
        pub fn release(mut self, lane: Lane, time_ms: f64) -> Self {
            self.events.push(InputEvent::release(lane, time_ms));
            self
        }

        /// Get the events sorted by time.
        pub fn events(self) -> Vec<InputEvent> {
            let mut events = self.events;
            events.sort_by(|a, b| {
                let time_a = match a {
                    InputEvent::Press { time_ms, .. } => *time_ms,
                    InputEvent::Release { time_ms, .. } => *time_ms,
                };
                let time_b = match b {
                    InputEvent::Press { time_ms, .. } => *time_ms,
                    InputEvent::Release { time_ms, .. } => *time_ms,
                };
                time_a.partial_cmp(&time_b).unwrap()
            });
            events
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builders::*;
    use super::input_sim::*;
    use crate::model::note::{Lane, NoteType};

    #[test]
    fn test_note_builder_normal() {
        let note = NoteBuilder::normal(Lane::Key1, 1000.0).wav_id(42).build();
        assert_eq!(note.lane, Lane::Key1);
        assert_eq!(note.start_time_ms, 1000.0);
        assert_eq!(note.wav_id, 42);
        assert_eq!(note.note_type, NoteType::Normal);
    }

    #[test]
    fn test_note_builder_long_note() {
        let note = NoteBuilder::long_start(Lane::Key2, 1000.0, 2000.0).build();
        assert_eq!(note.lane, Lane::Key2);
        assert_eq!(note.start_time_ms, 1000.0);
        assert_eq!(note.end_time_ms, Some(2000.0));
        assert_eq!(note.note_type, NoteType::LongStart);
    }

    #[test]
    fn test_chart_builder() {
        let notes = ChartBuilder::new()
            .normal(Lane::Key1, 0.0)
            .long_note(Lane::Key2, 100.0, 200.0)
            .mine(Lane::Key3, 300.0, 50.0)
            .build();

        assert_eq!(notes.len(), 4); // normal + ln_start + ln_end + mine
        assert_eq!(notes[0].note.note_type, NoteType::Normal);
        assert_eq!(notes[1].note.note_type, NoteType::LongStart);
        assert_eq!(notes[2].note.note_type, NoteType::LongEnd);
        assert_eq!(notes[3].note.note_type, NoteType::Mine);
    }

    #[test]
    fn test_create_simple_chart() {
        let notes = create_simple_chart(5, Lane::Key1, 100.0);
        assert_eq!(notes.len(), 5);
        for (i, nwi) in notes.iter().enumerate() {
            assert_eq!(nwi.index, i);
            assert_eq!(nwi.note.start_time_ms, i as f64 * 100.0);
        }
    }

    #[test]
    fn test_create_ln_chart() {
        let notes = create_ln_chart(Lane::Key1, 1000.0, 2000.0);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].note.note_type, NoteType::LongStart);
        assert_eq!(notes[0].note.end_time_ms, Some(2000.0));
        assert_eq!(notes[1].note.note_type, NoteType::LongEnd);
        assert_eq!(notes[1].note.start_time_ms, 2000.0);
    }

    #[test]
    fn test_input_simulator() {
        let events = InputSimulator::new()
            .press(Lane::Key1, 100.0)
            .release(Lane::Key1, 200.0)
            .press(Lane::Key2, 50.0)
            .events();

        assert_eq!(events.len(), 3);
        // Should be sorted by time
        match events[0] {
            InputEvent::Press { time_ms, .. } => assert_eq!(time_ms, 50.0),
            _ => panic!("Expected press event"),
        }
    }
}
