/// Autoplay log generation.
///
/// Ported from Java `KeyInputLog.createAutoplayLog()`.
/// Generates a sequence of key press/release events that perfectly play a chart.
use std::collections::BTreeMap;

use bms_model::{BmsModel, LnType, NoteType};
use bms_replay::key_input_log::KeyInputLog;

/// Create autoplay key input log from a BMS model.
///
/// Faithfully ports Java `KeyInputLog.createAutoplayLog()`.
/// The Java algorithm iterates through ALL timelines (including ones without playable notes,
/// such as empty measure boundaries, BPM changes, and mine/invisible note positions) and
/// for each timeline, walks lanes 0..keys in order:
/// - If note is LongNote end → release event (+ BSS for scratch lanes)
/// - If note is LongNote start → press event
/// - If note is NormalNote → press event
/// - If note is Mine/Invisible (not null in Java) → skip (no event)
/// - If no note and no active LN → release event (+ scratch lane+1 release)
pub fn create_autoplay_log(model: &BmsModel) -> Vec<KeyInputLog> {
    let keys = model.mode.key_count();
    let scratch_keys = model.mode.scratch_keys();
    let ln_type = model.ln_type;

    // Collect all timeline times from the model. In Java, getAllTimeLines() includes
    // all timelines (empty measure boundaries, BPM changes, note positions, etc.).
    // Collect unique, sorted times.
    let mut all_times: Vec<i64> = model.timelines.iter().map(|tl| tl.time_us).collect();
    all_times.sort();
    all_times.dedup();

    // Build a lane-indexed map: for each time_us → per-lane note type.
    let mut timeline: BTreeMap<i64, Vec<Option<LaneNote>>> = BTreeMap::new();

    for &t in &all_times {
        timeline.entry(t).or_insert_with(|| vec![None; keys]);
    }

    // Place notes into their nearest timeline slot (within ±2μs tolerance for
    // floating-point rounding differences in BPM→μs conversion).
    for note in &model.notes {
        let lane_note = match note.note_type {
            NoteType::Normal => Some(LaneNote::Normal),
            NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote => {
                Some(LaneNote::LnStart)
            }
            NoteType::Mine => Some(LaneNote::Ignored),
            // Java stores invisible notes via setHiddenNote(), so getNote(lane)
            // returns null for them → they don't suppress release events.
            NoteType::Invisible => None,
        };

        if let Some(ln) = lane_note {
            let matched_time = snap_to_timeline(note.time_us, &all_times);
            let slots = timeline
                .entry(matched_time)
                .or_insert_with(|| vec![None; keys]);
            if note.lane < keys {
                slots[note.lane] = Some(ln);
            }

            // LN end
            if matches!(
                note.note_type,
                NoteType::LongNote | NoteType::ChargeNote | NoteType::HellChargeNote
            ) && note.end_time_us > note.time_us
                && note.lane < keys
            {
                let end_matched = snap_to_timeline(note.end_time_us, &all_times);
                let end_slots = timeline
                    .entry(end_matched)
                    .or_insert_with(|| vec![None; keys]);
                end_slots[note.lane] = Some(LaneNote::LnEnd);
            }
        }
    }

    let mut keylog = Vec::new();
    let mut ln_active = vec![false; keys];

    // Java iterates timelines in order, then lanes 0..keys in order
    for (&time_us, slots) in &timeline {
        #[allow(clippy::needless_range_loop)]
        for lane in 0..keys {
            let note = slots.get(lane).and_then(|n| n.as_ref());
            if let Some(lane_note) = note {
                match lane_note {
                    LaneNote::LnEnd => {
                        keylog.push(KeyInputLog::new(time_us, lane as i32, false));
                        if ln_type != LnType::LongNote && scratch_keys.contains(&lane) {
                            // BSS handling
                            keylog.push(KeyInputLog::new(time_us, lane as i32 + 1, true));
                        }
                        ln_active[lane] = false;
                    }
                    LaneNote::LnStart => {
                        keylog.push(KeyInputLog::new(time_us, lane as i32, true));
                        ln_active[lane] = true;
                    }
                    LaneNote::Normal => {
                        keylog.push(KeyInputLog::new(time_us, lane as i32, true));
                    }
                    LaneNote::Ignored => {
                        // Mine/invisible: Java sees note != null, skips instanceof checks,
                        // so no event is emitted for this lane at this timeline.
                    }
                }
            } else if !ln_active[lane] {
                keylog.push(KeyInputLog::new(time_us, lane as i32, false));
                if scratch_keys.contains(&lane) {
                    keylog.push(KeyInputLog::new(time_us, lane as i32 + 1, false));
                }
            }
        }
    }

    keylog
}

/// Snap a time_us to the nearest timeline time within ±2μs tolerance.
/// If no match is found, returns the original time (which will create a new timeline entry).
fn snap_to_timeline(time_us: i64, sorted_times: &[i64]) -> i64 {
    match sorted_times.binary_search(&time_us) {
        Ok(_) => time_us,
        Err(idx) => {
            // Check neighbors
            let mut best = time_us;
            let mut best_diff = i64::MAX;
            if idx > 0 {
                let diff = (sorted_times[idx - 1] - time_us).abs();
                if diff <= 2 && diff < best_diff {
                    best = sorted_times[idx - 1];
                    best_diff = diff;
                }
            }
            if idx < sorted_times.len() {
                let diff = (sorted_times[idx] - time_us).abs();
                if diff <= 2 && diff < best_diff {
                    best = sorted_times[idx];
                }
            }
            best
        }
    }
}

/// Note type for a single lane at a single timeline.
#[derive(Debug, Clone)]
enum LaneNote {
    Normal,
    LnStart,
    LnEnd,
    /// Mine or invisible note — present in Java (not null) but no event emitted.
    Ignored,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{Note, PlayMode, TimeLine};

    /// Helper to build a minimal BmsModel for testing.
    fn make_model(
        notes: Vec<Note>,
        mode: PlayMode,
        ln_type: LnType,
        timeline_times: &[i64],
    ) -> BmsModel {
        let timelines = timeline_times
            .iter()
            .map(|&t| TimeLine {
                time_us: t,
                measure: 0,
                position: 0.0,
                bpm: 120.0,
            })
            .collect();
        BmsModel {
            mode,
            ln_type,
            notes,
            timelines,
            ..Default::default()
        }
    }

    #[test]
    fn single_normal_note() {
        let model = make_model(
            vec![Note::normal(0, 1_000_000, 1)],
            PlayMode::Beat7K,
            LnType::LongNote,
            &[0, 1_000_000],
        );
        let log = create_autoplay_log(&model);

        let press_events: Vec<_> = log.iter().filter(|e| e.keycode == 0 && e.pressed).collect();
        assert_eq!(press_events.len(), 1);
        assert_eq!(press_events[0].presstime, 1_000_000);
    }

    #[test]
    fn long_note_press_and_release() {
        let model = make_model(
            vec![Note::long_note(
                0,
                1_000_000,
                2_000_000,
                1,
                0,
                LnType::LongNote,
            )],
            PlayMode::Beat7K,
            LnType::LongNote,
            &[0, 1_000_000, 2_000_000],
        );
        let log = create_autoplay_log(&model);

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
        let model = make_model(
            vec![Note::long_note(
                7,
                1_000_000,
                2_000_000,
                1,
                0,
                LnType::ChargeNote,
            )],
            PlayMode::Beat7K,
            LnType::ChargeNote,
            &[0, 1_000_000, 2_000_000],
        );
        let log = create_autoplay_log(&model);

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
        let model = make_model(
            vec![Note::long_note(
                7,
                1_000_000,
                2_000_000,
                1,
                0,
                LnType::LongNote,
            )],
            PlayMode::Beat7K,
            LnType::LongNote,
            &[0, 1_000_000, 2_000_000],
        );
        let log = create_autoplay_log(&model);

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
        let model = make_model(
            vec![Note::long_note(
                0,
                1_000_000,
                2_000_000,
                1,
                0,
                LnType::ChargeNote,
            )],
            PlayMode::Beat7K,
            LnType::ChargeNote,
            &[0, 1_000_000, 2_000_000],
        );
        let log = create_autoplay_log(&model);

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
    fn mines_suppress_release_invisible_does_not() {
        let model = make_model(
            vec![
                Note::mine(0, 1_000_000, 1, 50),
                Note::invisible(1, 1_000_000, 1),
            ],
            PlayMode::Beat7K,
            LnType::LongNote,
            &[0, 1_000_000],
        );
        let log = create_autoplay_log(&model);

        // No press events should exist for mine/invisible lanes
        let press_events: Vec<_> = log.iter().filter(|e| e.pressed).collect();
        assert!(
            press_events.is_empty(),
            "Mines and invisible notes should not generate press events"
        );

        // Mine notes suppress release events (Java: note != null, no instanceof match)
        let mine_lane_release: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 0 && !e.pressed && e.presstime == 1_000_000)
            .collect();
        assert!(
            mine_lane_release.is_empty(),
            "Mine notes should suppress release events"
        );

        // Invisible notes do NOT suppress release events (Java: setHiddenNote, getNote returns null)
        let invisible_lane_release: Vec<_> = log
            .iter()
            .filter(|e| e.keycode == 1 && !e.pressed && e.presstime == 1_000_000)
            .collect();
        assert_eq!(
            invisible_lane_release.len(),
            1,
            "Invisible notes should allow release events (Java stores them in hiddenNote)"
        );
    }

    #[test]
    fn release_events_for_empty_lanes() {
        let model = make_model(
            vec![Note::normal(0, 1_000_000, 1)],
            PlayMode::Beat7K,
            LnType::LongNote,
            &[0, 1_000_000],
        );
        let log = create_autoplay_log(&model);

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
        let model = make_model(
            vec![
                Note::long_note(0, 1_000_000, 3_000_000, 1, 0, LnType::LongNote),
                Note::normal(1, 2_000_000, 2),
            ],
            PlayMode::Beat7K,
            LnType::LongNote,
            &[0, 1_000_000, 2_000_000, 3_000_000],
        );
        let log = create_autoplay_log(&model);

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
