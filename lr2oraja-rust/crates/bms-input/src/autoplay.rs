/// Autoplay log generation.
///
/// Ported from Java `KeyInputLog.createAutoplayLog()`.
/// Generates a sequence of key press/release events that perfectly play a chart.
use std::collections::BTreeMap;

use bms_model::{LnType, Note, NoteType, PlayMode};
use bms_replay::key_input_log::KeyInputLog;

/// Create autoplay key input log from a BMS model's notes.
///
/// The algorithm iterates through all timelines (grouped by time) and for each lane:
/// - LongNote start → press event
/// - LongNote end → release event (+ BSS handling for scratch lanes if ln_type != LongNote)
/// - NormalNote → press event
/// - No note and no active LN → release event (+ scratch lane+1 release)
///
/// This faithfully ports Java `KeyInputLog.createAutoplayLog()`.
pub fn create_autoplay_log(notes: &[Note], mode: PlayMode, ln_type: LnType) -> Vec<KeyInputLog> {
    let keys = mode.key_count();
    let scratch_keys = mode.scratch_keys();

    // Group notes by time_us, preserving order within each time.
    // Also collect LN end times as separate entries.
    let mut timeline: BTreeMap<i64, Vec<TimelineEntry>> = BTreeMap::new();

    for note in notes {
        match note.note_type {
            NoteType::Normal => {
                timeline
                    .entry(note.time_us)
                    .or_default()
                    .push(TimelineEntry::Normal { lane: note.lane });
            }
            NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote => {
                // LN start
                timeline
                    .entry(note.time_us)
                    .or_default()
                    .push(TimelineEntry::LnStart { lane: note.lane });
                // LN end
                if note.end_time_us > note.time_us {
                    timeline
                        .entry(note.end_time_us)
                        .or_default()
                        .push(TimelineEntry::LnEnd { lane: note.lane });
                }
            }
            NoteType::Mine | NoteType::Invisible => {
                // Mines and invisible notes are ignored for autoplay.
            }
        }
    }

    let mut keylog = Vec::new();
    let mut ln_active = vec![false; keys];

    for (&time_us, entries) in &timeline {
        // Track which lanes have entries at this time.
        let mut has_entry = vec![false; keys];
        for entry in entries {
            let lane = entry.lane();
            if lane < keys {
                has_entry[lane] = true;
            }
        }

        // Process entries in order.
        for entry in entries {
            let lane = entry.lane();
            if lane >= keys {
                continue;
            }

            match entry {
                TimelineEntry::LnStart { .. } => {
                    keylog.push(KeyInputLog::new(time_us, lane as i32, true));
                    ln_active[lane] = true;
                }
                TimelineEntry::LnEnd { .. } => {
                    keylog.push(KeyInputLog::new(time_us, lane as i32, false));
                    // BSS handling: for scratch lanes with non-default LN type,
                    // press lane+1 to complete the backspin scratch.
                    if ln_type != LnType::LongNote && scratch_keys.contains(&lane) {
                        keylog.push(KeyInputLog::new(time_us, lane as i32 + 1, true));
                    }
                    ln_active[lane] = false;
                }
                TimelineEntry::Normal { .. } => {
                    keylog.push(KeyInputLog::new(time_us, lane as i32, true));
                }
            }
        }

        // For lanes without entries and without active LN, emit release.
        for lane in 0..keys {
            if !has_entry[lane] && !ln_active[lane] {
                keylog.push(KeyInputLog::new(time_us, lane as i32, false));
                if scratch_keys.contains(&lane) {
                    keylog.push(KeyInputLog::new(time_us, lane as i32 + 1, false));
                }
            }
        }
    }

    keylog
}

#[derive(Debug)]
enum TimelineEntry {
    Normal { lane: usize },
    LnStart { lane: usize },
    LnEnd { lane: usize },
}

impl TimelineEntry {
    fn lane(&self) -> usize {
        match self {
            TimelineEntry::Normal { lane }
            | TimelineEntry::LnStart { lane }
            | TimelineEntry::LnEnd { lane } => *lane,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_notes_produces_empty_log() {
        let log = create_autoplay_log(&[], PlayMode::Beat7K, LnType::LongNote);
        assert!(log.is_empty());
    }

    #[test]
    fn single_normal_note() {
        let notes = vec![Note::normal(0, 1_000_000, 1)];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::LongNote);

        // Should have: press lane 0, and releases for all other lanes
        let press_events: Vec<_> = log.iter().filter(|e| e.keycode == 0 && e.pressed).collect();
        assert_eq!(press_events.len(), 1);
        assert_eq!(press_events[0].presstime, 1_000_000);
    }

    #[test]
    fn long_note_press_and_release() {
        let notes = vec![Note::long_note(
            0,
            1_000_000,
            2_000_000,
            1,
            0,
            LnType::LongNote,
        )];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::LongNote);

        // Should have press at 1_000_000 and release at 2_000_000
        let press: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 0 && e.pressed && e.presstime == 1_000_000)
            .collect();
        assert_eq!(press.len(), 1);

        let release: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 0 && !e.pressed && e.presstime == 2_000_000)
            .collect();
        assert_eq!(release.len(), 1);
    }

    #[test]
    fn bss_handling_on_scratch_lane() {
        // Beat7K scratch lane is 7
        let notes = vec![Note::long_note(
            7,
            1_000_000,
            2_000_000,
            1,
            0,
            LnType::ChargeNote,
        )];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::ChargeNote);

        // At LN end (2_000_000), should have release for lane 7 AND press for lane 8
        let ln_end_events: Vec<_> = log.iter().filter(|e| e.presstime == 2_000_000).collect();

        let release_7: Vec<_> = ln_end_events
            .iter()
            .filter(|e| e.keycode == 7 && !e.pressed)
            .collect();
        assert_eq!(release_7.len(), 1, "Should release lane 7 at LN end");

        let press_8: Vec<_> = ln_end_events
            .iter()
            .filter(|e| e.keycode == 8 && e.pressed)
            .collect();
        assert_eq!(
            press_8.len(),
            1,
            "Should press lane 8 (BSS) at scratch LN end"
        );
    }

    #[test]
    fn no_bss_for_default_ln_type() {
        // With LnType::LongNote, no BSS handling
        let notes = vec![Note::long_note(
            7,
            1_000_000,
            2_000_000,
            1,
            0,
            LnType::LongNote,
        )];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::LongNote);

        // At LN end, no press for lane 8
        let press_8: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 8 && e.pressed && e.presstime == 2_000_000)
            .collect();
        assert!(
            press_8.is_empty(),
            "Should not have BSS press for LnType::LongNote"
        );
    }

    #[test]
    fn no_bss_for_non_scratch_lane() {
        // Lane 0 is not a scratch lane in Beat7K
        let notes = vec![Note::long_note(
            0,
            1_000_000,
            2_000_000,
            1,
            0,
            LnType::ChargeNote,
        )];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::ChargeNote);

        // At LN end, no press for lane 1 (BSS is only for scratch lanes)
        let press_1_at_end: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 1 && e.pressed && e.presstime == 2_000_000)
            .collect();
        assert!(
            press_1_at_end.is_empty(),
            "Should not have BSS press for non-scratch lane"
        );
    }

    #[test]
    fn mines_and_invisible_ignored() {
        let notes = vec![
            Note::mine(0, 1_000_000, 1, 50),
            Note::invisible(1, 1_000_000, 1),
        ];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::LongNote);

        // No press events should exist (mines and invisible are skipped)
        let press_events: Vec<_> = log.iter().filter(|e| e.pressed).collect();
        assert!(
            press_events.is_empty(),
            "Mines and invisible notes should not generate press events"
        );
    }

    #[test]
    fn release_events_for_empty_lanes() {
        let notes = vec![Note::normal(0, 1_000_000, 1)];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::LongNote);

        // Lane 1 should have a release event at t=1_000_000
        let lane1_release: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 1 && !e.pressed && e.presstime == 1_000_000)
            .collect();
        assert_eq!(
            lane1_release.len(),
            1,
            "Empty lanes should get release events"
        );
    }

    #[test]
    fn ln_active_suppresses_release() {
        // Two notes at different times: LN from t=1M to t=3M on lane 0, normal at t=2M on lane 1
        let notes = vec![
            Note::long_note(0, 1_000_000, 3_000_000, 1, 0, LnType::LongNote),
            Note::normal(1, 2_000_000, 2),
        ];
        let log = create_autoplay_log(&notes, PlayMode::Beat7K, LnType::LongNote);

        // At t=2_000_000, lane 0 has active LN so should NOT get a release
        let lane0_release_at_2m: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 0 && !e.pressed && e.presstime == 2_000_000)
            .collect();
        assert!(
            lane0_release_at_2m.is_empty(),
            "Active LN should suppress release events"
        );
    }
}
