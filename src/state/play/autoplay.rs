//! Autoplay processor for automatic note handling.

use crate::model::BMSModel;
use crate::model::note::{LANE_COUNT, Lane, NoteType};

/// Autoplay mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoplayMode {
    /// Autoplay disabled.
    #[default]
    Off,
    /// Full autoplay - all lanes are automated.
    Full,
    /// Only scratch lane is automated.
    AssistScratch,
    /// Only key lanes are automated (scratch is manual).
    AssistKeys,
}

impl AutoplayMode {
    /// Check if this mode handles the given lane.
    pub fn handles_lane(&self, lane: Lane) -> bool {
        match self {
            AutoplayMode::Off => false,
            AutoplayMode::Full => true,
            AutoplayMode::AssistScratch => lane == Lane::Scratch,
            AutoplayMode::AssistKeys => lane != Lane::Scratch,
        }
    }
}

/// Pre-computed autoplay event.
#[derive(Debug, Clone)]
struct AutoplayEvent {
    time_ms: f64,
    lane: Lane,
    is_press: bool,
}

/// Autoplay processor that automatically triggers inputs at note times.
pub struct AutoplayProcessor {
    mode: AutoplayMode,
    events: Vec<AutoplayEvent>,
    current_index: usize,
    /// Simulated key states for 8 lanes.
    lane_states: [bool; LANE_COUNT],
    /// Just pressed this frame.
    just_pressed: [bool; LANE_COUNT],
    /// Just released this frame.
    just_released: [bool; LANE_COUNT],
    /// Press timestamps in microseconds.
    press_times: [u64; LANE_COUNT],
    /// Release timestamps in microseconds.
    release_times: [u64; LANE_COUNT],
}

impl AutoplayProcessor {
    /// Default press duration for normal notes (ms).
    const PRESS_DURATION_MS: f64 = 50.0;

    /// Create a new autoplay processor.
    pub fn new(mode: AutoplayMode, model: &BMSModel) -> Self {
        let events = Self::build_events(mode, model);

        Self {
            mode,
            events,
            current_index: 0,
            lane_states: [false; LANE_COUNT],
            just_pressed: [false; LANE_COUNT],
            just_released: [false; LANE_COUNT],
            press_times: [0; LANE_COUNT],
            release_times: [0; LANE_COUNT],
        }
    }

    /// Build autoplay events from the BMS model.
    fn build_events(mode: AutoplayMode, model: &BMSModel) -> Vec<AutoplayEvent> {
        let mut events = Vec::new();

        for timeline in model.timelines.entries() {
            for note in &timeline.notes {
                // Skip lanes not handled by this mode
                if !mode.handles_lane(note.lane) {
                    continue;
                }

                // Skip invisible and mine notes
                if matches!(note.note_type, NoteType::Invisible | NoteType::Mine) {
                    continue;
                }

                // Skip LN ends (handled with the start)
                if note.note_type == NoteType::LongEnd {
                    continue;
                }

                // Press event at note start time
                events.push(AutoplayEvent {
                    time_ms: note.start_time_ms,
                    lane: note.lane,
                    is_press: true,
                });

                // Release event
                let release_time = if let Some(end_time) = note.end_time_ms {
                    // Long note: release at end time
                    end_time
                } else {
                    // Normal note: release after short duration
                    note.start_time_ms + Self::PRESS_DURATION_MS
                };

                events.push(AutoplayEvent {
                    time_ms: release_time,
                    lane: note.lane,
                    is_press: false,
                });
            }
        }

        // Sort by time
        events.sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap());

        events
    }

    /// Get the autoplay mode.
    pub fn mode(&self) -> AutoplayMode {
        self.mode
    }

    /// Check if autoplay is enabled.
    pub fn is_enabled(&self) -> bool {
        self.mode != AutoplayMode::Off
    }

    /// Update the autoplay state for the given time.
    /// Processes all events up to current_time_ms.
    pub fn update(&mut self, current_time_ms: f64) {
        // Reset frame state
        self.just_pressed = [false; LANE_COUNT];
        self.just_released = [false; LANE_COUNT];

        let current_time_us = (current_time_ms * 1000.0) as u64;

        // Process all events up to current time
        while self.current_index < self.events.len() {
            let event = &self.events[self.current_index];
            if event.time_ms > current_time_ms {
                break;
            }

            let lane_idx = event.lane.index();

            if event.is_press {
                if !self.lane_states[lane_idx] {
                    self.just_pressed[lane_idx] = true;
                    self.press_times[lane_idx] = current_time_us;
                }
                self.lane_states[lane_idx] = true;
            } else {
                if self.lane_states[lane_idx] {
                    self.just_released[lane_idx] = true;
                    self.release_times[lane_idx] = current_time_us;
                }
                self.lane_states[lane_idx] = false;
            }

            self.current_index += 1;
        }
    }

    /// Seek to a specific time position.
    /// 指定した時刻にシークする。
    pub fn seek(&mut self, time_ms: f64) {
        self.reset();
        self.update(time_ms);
        self.just_pressed = [false; LANE_COUNT];
        self.just_released = [false; LANE_COUNT];
    }

    /// Check if a lane is currently pressed.
    pub fn is_pressed(&self, lane: Lane) -> bool {
        self.lane_states[lane.index()]
    }

    /// Check if a lane was just pressed this frame.
    pub fn just_pressed(&self, lane: Lane) -> bool {
        self.just_pressed[lane.index()]
    }

    /// Check if a lane was just released this frame.
    pub fn just_released(&self, lane: Lane) -> bool {
        self.just_released[lane.index()]
    }

    /// Get the press timestamp for a lane in microseconds.
    pub fn press_time_us(&self, lane: Lane) -> u64 {
        self.press_times[lane.index()]
    }

    /// Get the release timestamp for a lane in microseconds.
    pub fn release_time_us(&self, lane: Lane) -> u64 {
        self.release_times[lane.index()]
    }

    /// Reset the processor to the beginning.
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.lane_states = [false; LANE_COUNT];
        self.just_pressed = [false; LANE_COUNT];
        self.just_released = [false; LANE_COUNT];
        self.press_times = [0; LANE_COUNT];
        self.release_times = [0; LANE_COUNT];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::note::Note;
    use crate::model::timeline::{Timeline, Timelines};
    use crate::model::{ChartFormat, JudgeRankType, LongNoteMode, PlayMode, TotalType};

    fn create_test_model() -> BMSModel {
        let mut timelines = Timelines::new();

        // Note at 1000ms on Key1
        let mut tl1 = Timeline::new(1000.0, 0, 0.0, 120.0);
        tl1.add_note(Note::normal(Lane::Key1, 1000.0, 1));
        timelines.push(tl1);

        // Note at 2000ms on Scratch
        let mut tl2 = Timeline::new(2000.0, 0, 0.0, 120.0);
        tl2.add_note(Note::normal(Lane::Scratch, 2000.0, 2));
        timelines.push(tl2);

        // Long note from 3000ms to 3500ms on Key2
        let mut tl3 = Timeline::new(3000.0, 0, 0.0, 120.0);
        tl3.add_note(Note::long_start(Lane::Key2, 3000.0, 3500.0, 3));
        timelines.push(tl3);

        BMSModel {
            title: "Test".to_string(),
            subtitle: String::new(),
            artist: "Test".to_string(),
            subartist: String::new(),
            genre: "Test".to_string(),
            preview: None,
            stage_file: None,
            back_bmp: None,
            banner: None,
            initial_bpm: 120.0,
            min_bpm: 120.0,
            max_bpm: 120.0,
            total_notes: 3,
            total: 200.0,
            total_type: TotalType::Bms,
            judge_rank: 2,
            judge_rank_type: JudgeRankType::BmsRank,
            long_note_mode: LongNoteMode::Ln,
            play_mode: PlayMode::Beat7K,
            source_format: ChartFormat::Bms,
            has_long_note: true,
            has_mine: false,
            has_invisible: false,
            has_stop: false,
            play_level: None,
            difficulty: None,
            folder: String::new(),
            timelines,
            wav_files: std::collections::BTreeMap::new(),
            bga_files: std::collections::BTreeMap::new(),
            bga_events: Vec::new(),
            poor_bga_file: None,
            bgm_events: Vec::new(),
        }
    }

    #[test]
    fn test_autoplay_off_generates_no_events() {
        let model = create_test_model();
        let processor = AutoplayProcessor::new(AutoplayMode::Off, &model);
        assert!(!processor.is_enabled());
        assert_eq!(processor.events.len(), 0);
    }

    #[test]
    fn test_autoplay_full() {
        let model = create_test_model();
        let mut processor = AutoplayProcessor::new(AutoplayMode::Full, &model);
        assert!(processor.is_enabled());

        // Before any notes
        processor.update(500.0);
        assert!(!processor.is_pressed(Lane::Key1));
        assert!(!processor.just_pressed(Lane::Key1));

        // After Key1 note
        processor.update(1000.0);
        assert!(processor.is_pressed(Lane::Key1));
        assert!(processor.just_pressed(Lane::Key1));

        // After Key1 release (50ms default duration)
        processor.update(1100.0);
        assert!(!processor.is_pressed(Lane::Key1));
        assert!(processor.just_released(Lane::Key1));
    }

    #[test]
    fn test_autoplay_assist_scratch_only_handles_scratch() {
        let model = create_test_model();
        let processor = AutoplayProcessor::new(AutoplayMode::AssistScratch, &model);

        // Should have events for Scratch but not for keys
        // 1 Scratch note = 2 events (press + release)
        assert_eq!(processor.events.len(), 2);

        // Mode check
        assert!(processor.mode().handles_lane(Lane::Scratch));
        assert!(!processor.mode().handles_lane(Lane::Key1));
    }

    #[test]
    fn test_autoplay_assist_keys_only_handles_keys() {
        let model = create_test_model();
        let processor = AutoplayProcessor::new(AutoplayMode::AssistKeys, &model);

        // Should have events for Key1 and Key2 (LN) but not Scratch
        // Key1 normal = 2 events, Key2 LN = 2 events
        assert_eq!(processor.events.len(), 4);

        // Mode check
        assert!(!processor.mode().handles_lane(Lane::Scratch));
        assert!(processor.mode().handles_lane(Lane::Key1));
    }

    #[test]
    fn test_autoplay_long_note_timing() {
        let model = create_test_model();
        let mut processor = AutoplayProcessor::new(AutoplayMode::Full, &model);

        // LN starts at 3000ms
        processor.update(3000.0);
        assert!(processor.is_pressed(Lane::Key2));

        // LN still held at 3200ms
        processor.update(3200.0);
        assert!(processor.is_pressed(Lane::Key2));
        assert!(!processor.just_released(Lane::Key2));

        // LN released at 3500ms
        processor.update(3500.0);
        assert!(!processor.is_pressed(Lane::Key2));
        assert!(processor.just_released(Lane::Key2));
    }

    #[test]
    fn test_autoplay_reset() {
        let model = create_test_model();
        let mut processor = AutoplayProcessor::new(AutoplayMode::Full, &model);

        processor.update(1000.0);
        assert!(processor.is_pressed(Lane::Key1));

        processor.reset();
        assert!(!processor.is_pressed(Lane::Key1));
    }
}
