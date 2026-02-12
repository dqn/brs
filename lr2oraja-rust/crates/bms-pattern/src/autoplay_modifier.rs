// Autoplay modifier — moves specified lanes to background (auto-play).
//
// Ported from Java: AutoplayModifier.java

use std::collections::BTreeMap;

use bms_model::{BgNote, BmsModel, NoteType};

use crate::modifier::{AssistLevel, PatternModifier};

/// Moves notes on specified lanes to background for auto-play.
///
/// When `margin_ms > 0`, only moves notes when other (non-autoplay) lanes
/// have notes within the margin window.
///
/// Java: `AutoplayModifier`
pub struct AutoplayModifier {
    /// Lane indices to auto-play
    pub lanes: Vec<usize>,
    /// Margin in milliseconds for conditional auto-play
    pub margin_ms: i64,
    /// Track whether assist was applied
    assist: AssistLevel,
}

impl AutoplayModifier {
    pub fn new(lanes: Vec<usize>) -> Self {
        Self {
            lanes,
            margin_ms: 0,
            assist: AssistLevel::None,
        }
    }

    pub fn with_margin(mut self, margin_ms: i64) -> Self {
        self.margin_ms = margin_ms;
        self
    }
}

impl PatternModifier for AutoplayModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        let key_count = model.mode.key_count();
        let lane_set: Vec<usize> = self.lanes.clone();

        if self.margin_ms <= 0 {
            // Simple: move all notes on specified lanes to bg
            self.move_all_to_bg(model, &lane_set);
        } else {
            // Conditional: only move when other lanes have notes nearby
            self.move_with_margin(model, &lane_set, key_count);
        }
    }

    fn assist_level(&self) -> AssistLevel {
        self.assist
    }
}

impl AutoplayModifier {
    fn move_all_to_bg(&mut self, model: &mut BmsModel, lanes: &[usize]) {
        let mut indices_to_remove: Vec<usize> = Vec::new();

        for (i, note) in model.notes.iter().enumerate() {
            if lanes.contains(&note.lane) {
                // Track assist level
                if note.note_type != NoteType::Mine {
                    self.assist = AssistLevel::Assist;
                }
                indices_to_remove.push(i);
                // Also remove LN pair
                if note.is_long_note()
                    && note.pair_index != usize::MAX
                    && !indices_to_remove.contains(&note.pair_index)
                {
                    indices_to_remove.push(note.pair_index);
                }
            }
        }

        self.remove_and_move_to_bg(model, &mut indices_to_remove);
    }

    fn move_with_margin(&mut self, model: &mut BmsModel, lanes: &[usize], key_count: usize) {
        let margin_us = self.margin_ms * 1000;

        // Group notes by time
        let mut time_groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (idx, note) in model.notes.iter().enumerate() {
            time_groups.entry(note.time_us).or_default().push(idx);
        }

        let times: Vec<i64> = time_groups.keys().copied().collect();

        // Track LN active state per lane
        let mut ln_active = vec![false; key_count];
        let mut indices_to_remove: Vec<usize> = Vec::new();

        // For each timeline, check if we should move autoplay lanes
        let mut pos = 0usize;
        for (t_idx, &time_us) in times.iter().enumerate() {
            // Update LN state from earlier timelines up to margin before current
            while pos < t_idx && times[pos] < time_us - margin_us {
                if let Some(indices) = time_groups.get(&times[pos]) {
                    for &idx in indices {
                        let note = &model.notes[idx];
                        if note.is_long_note() && note.lane < key_count {
                            ln_active[note.lane] = note.end_time_us > 0;
                        }
                    }
                }
                pos += 1;
            }

            // Determine end time window (considering LN ends on autoplay lanes)
            let mut end_time_us = time_us + margin_us;
            if let Some(indices) = time_groups.get(&time_us) {
                for &idx in indices {
                    let note = &model.notes[idx];
                    if lanes.contains(&note.lane) && note.is_long_note() && note.end_time_us > 0 {
                        end_time_us = end_time_us.max(note.end_time_us + margin_us);
                    }
                }
            }

            // Check if any non-autoplay lane has notes in the window
            let mut should_remove = false;
            for &check_time in &times {
                if check_time < times.get(pos).copied().unwrap_or(time_us) {
                    continue;
                }
                if check_time >= end_time_us {
                    break;
                }
                if let Some(indices) = time_groups.get(&check_time) {
                    for &idx in indices {
                        let note = &model.notes[idx];
                        if !lanes.contains(&note.lane) && note.lane < key_count {
                            should_remove = true;
                            break;
                        }
                    }
                }
                // Also check LN active state
                if !should_remove {
                    for (lane, &active) in ln_active.iter().enumerate().take(key_count) {
                        if !lanes.contains(&lane) && active {
                            should_remove = true;
                            break;
                        }
                    }
                }
                if should_remove {
                    break;
                }
            }

            if should_remove && let Some(indices) = time_groups.get(&time_us) {
                for &idx in indices {
                    let note = &model.notes[idx];
                    if lanes.contains(&note.lane) {
                        if note.note_type != NoteType::Mine && note.note_type != NoteType::Invisible
                        {
                            self.assist = AssistLevel::Assist;
                        }
                        if !indices_to_remove.contains(&idx) {
                            indices_to_remove.push(idx);
                        }
                        // Also remove LN pair
                        if note.is_long_note()
                            && note.pair_index != usize::MAX
                            && !indices_to_remove.contains(&note.pair_index)
                        {
                            indices_to_remove.push(note.pair_index);
                        }
                    }
                }
            }
        }

        self.remove_and_move_to_bg(model, &mut indices_to_remove);
    }

    fn remove_and_move_to_bg(&self, model: &mut BmsModel, indices: &mut Vec<usize>) {
        indices.sort_unstable();
        indices.dedup();

        // Move non-mine notes to bg
        for &i in indices.iter() {
            let note = &model.notes[i];
            if note.note_type != NoteType::Mine {
                model.bg_notes.push(BgNote {
                    wav_id: note.wav_id,
                    time_us: note.time_us,
                    micro_starttime: note.micro_starttime,
                    micro_duration: note.micro_duration,
                });
            }
        }

        // Remove in reverse order
        for &i in indices.iter().rev() {
            model.notes.remove(i);
        }

        // Rebuild pair indices
        rebuild_pair_indices(&mut model.notes);
    }
}

/// Rebuild LN pair indices.
fn rebuild_pair_indices(notes: &mut [bms_model::Note]) {
    for note in notes.iter_mut() {
        note.pair_index = usize::MAX;
    }

    let starts: Vec<usize> = notes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.is_long_note() && n.end_time_us > 0)
        .map(|(i, _)| i)
        .collect();

    for &si in &starts {
        let lane = notes[si].lane;
        let note_type = notes[si].note_type;
        let end_time = notes[si].end_time_us;

        if let Some(ei) = notes.iter().enumerate().position(|(i, n)| {
            i != si
                && n.lane == lane
                && n.note_type == note_type
                && n.time_us == end_time
                && n.end_time_us == 0
        }) {
            notes[si].pair_index = ei;
            notes[ei].pair_index = si;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{LnType, Note, PlayMode};

    fn make_model(notes: Vec<Note>) -> BmsModel {
        BmsModel {
            mode: PlayMode::Beat7K,
            notes,
            ..Default::default()
        }
    }

    #[test]
    fn test_autoplay_moves_to_bg() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(1, 1000, 2),
            Note::normal(7, 2000, 3), // scratch
        ];
        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![7]); // autoplay scratch
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert_eq!(model.bg_notes.len(), 1);
        assert_eq!(model.bg_notes[0].wav_id, 3);
        assert_eq!(modifier.assist_level(), AssistLevel::Assist);
    }

    #[test]
    fn test_autoplay_multiple_lanes() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(1, 1000, 2),
            Note::normal(2, 1000, 3),
            Note::normal(3, 1000, 4),
        ];
        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![0, 1]);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert_eq!(model.notes[0].lane, 2);
        assert_eq!(model.notes[1].lane, 3);
    }

    #[test]
    fn test_autoplay_ln_pair_moved_together() {
        let mut notes = vec![
            Note::long_note(7, 1000, 2000, 1, 2, LnType::LongNote),
            Note::long_note(7, 2000, 0, 2, 0, LnType::LongNote),
            Note::normal(0, 1500, 3),
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;

        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![7]);
        modifier.modify(&mut model);

        // Only the normal note remains
        assert_eq!(model.notes.len(), 1);
        assert_eq!(model.notes[0].lane, 0);
        // Both LN notes moved to bg
        assert_eq!(model.bg_notes.len(), 2);
    }

    #[test]
    fn test_autoplay_no_lanes_noop() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![]);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 1);
        assert_eq!(modifier.assist_level(), AssistLevel::None);
    }

    #[test]
    fn test_autoplay_with_margin_removes_when_other_lanes_active() {
        // Margin mode: only remove when other lanes have notes within margin
        let notes = vec![
            Note::normal(7, 1_000_000, 1), // scratch at 1000ms
            Note::normal(0, 1_000_000, 2), // other lane at 1000ms
        ];
        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![7]).with_margin(500);
        modifier.modify(&mut model);

        // Scratch should be moved because lane 0 has note at same time (within margin)
        assert_eq!(model.notes.len(), 1);
        assert_eq!(model.notes[0].lane, 0);
        assert_eq!(modifier.assist_level(), AssistLevel::Assist);
    }

    #[test]
    fn test_autoplay_with_margin_keeps_when_no_other_lanes() {
        // Only scratch at 1000ms, no other lanes within 500ms margin
        let notes = vec![
            Note::normal(7, 1_000_000, 1), // 1000ms
            Note::normal(0, 5_000_000, 2), // 5000ms (far away)
        ];
        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![7]).with_margin(500);
        modifier.modify(&mut model);

        // Scratch should remain (no other notes within 500ms)
        assert_eq!(model.notes.len(), 2);
    }

    #[test]
    fn test_autoplay_mine_not_added_to_bg() {
        let notes = vec![Note::mine(7, 1000, 1, 10)];
        let mut model = make_model(notes);
        let mut modifier = AutoplayModifier::new(vec![7]);
        modifier.modify(&mut model);

        assert!(model.notes.is_empty());
        assert!(model.bg_notes.is_empty()); // mine not added to bg
    }
}
