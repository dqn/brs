// Long note modifier — removes or adds long notes.
//
// Ported from Java: LongNoteModifier.java

use std::collections::BTreeMap;

use bms_model::{BmsModel, Note, NoteType};

use crate::java_random::JavaRandom;
use crate::modifier::{AssistLevel, PatternModifier};

/// Mode for long note modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongNoteMode {
    /// Remove LNs randomly (rate probability), replace with normal
    Remove,
    /// Add LongNote type between consecutive normal notes
    AddLn,
    /// Add ChargeNote type
    AddCn,
    /// Add HellChargeNote type
    AddHcn,
    /// Add random LN type
    AddAll,
}

/// Removes or adds long notes.
///
/// Java: `LongNoteModifier`
pub struct LongNoteModifier {
    pub mode: LongNoteMode,
    /// Probability of removing/adding (0.0..=1.0)
    pub rate: f64,
    /// Seed for random operations
    seed: i64,
    /// Track assist level
    assist: AssistLevel,
}

impl LongNoteModifier {
    pub fn new(mode: LongNoteMode, rate: f64) -> Self {
        Self {
            mode,
            rate,
            seed: 0,
            assist: AssistLevel::None,
        }
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = seed;
        self
    }
}

impl PatternModifier for LongNoteModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        let mut rng = JavaRandom::new(self.seed);

        match self.mode {
            LongNoteMode::Remove => self.remove_lns(model, &mut rng),
            _ => self.add_lns(model, &mut rng),
        }
    }

    fn assist_level(&self) -> AssistLevel {
        self.assist
    }
}

impl LongNoteModifier {
    /// Remove LN notes with probability `rate`, replacing LN start with Normal
    /// and removing LN end.
    fn remove_lns(&mut self, model: &mut BmsModel, rng: &mut JavaRandom) {
        let mut indices_to_remove: Vec<usize> = Vec::new();

        for i in 0..model.notes.len() {
            let note = &model.notes[i];
            if note.is_long_note() && rng.next_double() < self.rate {
                if note.end_time_us > 0 {
                    // LN start -> convert to Normal
                    let pair = note.pair_index;
                    if pair != usize::MAX && !indices_to_remove.contains(&pair) {
                        indices_to_remove.push(pair);
                    }
                    // Will be converted to Normal below
                } else {
                    // LN end -> mark for removal
                    if !indices_to_remove.contains(&i) {
                        indices_to_remove.push(i);
                    }
                }
                self.assist = AssistLevel::Assist;
            }
        }

        // Convert LN start notes to Normal
        for i in 0..model.notes.len() {
            let note = &model.notes[i];
            if note.is_long_note() && note.end_time_us > 0 {
                let pair = note.pair_index;
                if pair != usize::MAX && indices_to_remove.contains(&pair) {
                    model.notes[i].note_type = NoteType::Normal;
                    model.notes[i].end_time_us = 0;
                    model.notes[i].pair_index = usize::MAX;
                }
            }
        }

        // Remove LN end notes in reverse order
        indices_to_remove.sort_unstable();
        indices_to_remove.dedup();
        for &i in indices_to_remove.iter().rev() {
            model.notes.remove(i);
        }

        // Rebuild pair indices for remaining LN notes
        rebuild_pair_indices(&mut model.notes);
    }

    /// Add LN between consecutive normal notes on the same lane.
    fn add_lns(&mut self, model: &mut BmsModel, rng: &mut JavaRandom) {
        let key_count = model.mode.key_count();

        // Group note indices by time, sorted by time
        let mut time_groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (idx, note) in model.notes.iter().enumerate() {
            time_groups.entry(note.time_us).or_default().push(idx);
        }

        let time_keys: Vec<i64> = time_groups.keys().copied().collect();

        // For each pair of consecutive timelines, check if we can convert
        let mut conversions: Vec<(usize, i64, NoteType)> = Vec::new();

        for t in 0..time_keys.len().saturating_sub(1) {
            let current_time = time_keys[t];
            let next_time = time_keys[t + 1];
            let current_indices = &time_groups[&current_time];
            let next_indices = &time_groups[&next_time];

            for lane in 0..key_count {
                let current_note_idx = current_indices
                    .iter()
                    .find(|&&i| model.notes[i].lane == lane);
                let next_has_note = next_indices.iter().any(|&i| model.notes[i].lane == lane);

                if let Some(&idx) = current_note_idx {
                    let note = &model.notes[idx];
                    if note.note_type == NoteType::Normal
                        && !next_has_note
                        && rng.next_double() < self.rate
                    {
                        let ln_note_type = match self.mode {
                            LongNoteMode::AddLn => NoteType::LongNote,
                            LongNoteMode::AddCn => NoteType::ChargeNote,
                            LongNoteMode::AddHcn => NoteType::HellChargeNote,
                            LongNoteMode::AddAll => {
                                let r = (rng.next_double() * 3.0) as i32;
                                match r {
                                    0 => NoteType::LongNote,
                                    1 => NoteType::ChargeNote,
                                    _ => NoteType::HellChargeNote,
                                }
                            }
                            LongNoteMode::Remove => unreachable!(),
                        };

                        if ln_note_type != NoteType::LongNote {
                            self.assist = AssistLevel::Assist;
                        }

                        conversions.push((idx, next_time, ln_note_type));
                    }
                }
            }
        }

        // Apply conversions: convert start note to LN and add end note
        let mut new_end_notes: Vec<Note> = Vec::new();

        for (idx, end_time, ln_note_type) in &conversions {
            model.notes[*idx].note_type = *ln_note_type;
            model.notes[*idx].end_time_us = *end_time;

            new_end_notes.push(Note {
                lane: model.notes[*idx].lane,
                note_type: *ln_note_type,
                time_us: *end_time,
                end_time_us: 0,
                wav_id: 0,
                end_wav_id: 0,
                damage: 0,
                pair_index: usize::MAX,
                micro_starttime: 0,
                micro_duration: 0,
            });
        }

        model.notes.extend(new_end_notes);

        // Rebuild pair indices
        rebuild_pair_indices(&mut model.notes);
    }
}

/// Rebuild LN pair indices.
fn rebuild_pair_indices(notes: &mut [Note]) {
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
    use bms_model::{LnType, PlayMode};

    fn make_model(notes: Vec<Note>) -> BmsModel {
        BmsModel {
            mode: PlayMode::Beat7K,
            notes,
            ..Default::default()
        }
    }

    #[test]
    fn test_remove_ln_replaces_with_normal() {
        let mut notes = vec![
            Note::long_note(0, 1000, 2000, 1, 2, LnType::LongNote),
            Note::long_note(0, 2000, 0, 2, 0, LnType::LongNote),
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;

        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::Remove, 1.0).with_seed(0);
        modifier.modify(&mut model);

        // LN should be converted to normal, end removed
        assert_eq!(model.notes.len(), 1);
        assert_eq!(model.notes[0].note_type, NoteType::Normal);
        assert_eq!(model.notes[0].time_us, 1000);
        assert_eq!(modifier.assist_level(), AssistLevel::Assist);
    }

    #[test]
    fn test_remove_ln_rate_zero_keeps_all() {
        let mut notes = vec![
            Note::long_note(0, 1000, 2000, 1, 2, LnType::LongNote),
            Note::long_note(0, 2000, 0, 2, 0, LnType::LongNote),
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;

        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::Remove, 0.0).with_seed(0);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert!(model.notes[0].is_long_note());
    }

    #[test]
    fn test_add_ln_between_consecutive_normals() {
        // Lane 0 at time 1000 and time 2000 — but next timeline has note on
        // same lane, so cannot convert. Use different notes: lane 0 at 1000,
        // lane 1 at 2000 (no note on lane 0 at 2000).
        let notes = vec![Note::normal(0, 1000, 1), Note::normal(1, 2000, 2)];
        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::AddLn, 1.0).with_seed(0);
        modifier.modify(&mut model);

        // First note should be converted to LN with end at 2000
        let ln_starts: Vec<&Note> = model
            .notes
            .iter()
            .filter(|n| n.is_long_note() && n.end_time_us > 0)
            .collect();
        assert_eq!(ln_starts.len(), 1);
        assert_eq!(ln_starts[0].time_us, 1000);
        assert_eq!(ln_starts[0].end_time_us, 2000);
        assert_eq!(ln_starts[0].note_type, NoteType::LongNote);
    }

    #[test]
    fn test_add_cn_type() {
        let notes = vec![Note::normal(0, 1000, 1), Note::normal(1, 2000, 2)];
        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::AddCn, 1.0).with_seed(0);
        modifier.modify(&mut model);

        let ln_starts: Vec<&Note> = model
            .notes
            .iter()
            .filter(|n| n.is_long_note() && n.end_time_us > 0)
            .collect();
        assert_eq!(ln_starts.len(), 1);
        assert_eq!(ln_starts[0].note_type, NoteType::ChargeNote);
        assert_eq!(modifier.assist_level(), AssistLevel::Assist);
    }

    #[test]
    fn test_add_ln_does_not_add_if_next_lane_occupied() {
        // Both timelines have notes on lane 0 — cannot add LN
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(0, 2000, 2),
            Note::normal(1, 1000, 3), // Lane 1 has note at both times
            Note::normal(1, 2000, 4),
        ];
        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::AddLn, 1.0).with_seed(0);
        modifier.modify(&mut model);

        // Lane 1 cannot be converted (next timeline has a note on lane 1)
        // Lane 0 cannot be converted (next timeline has a note on lane 0)
        let ln_count = model.notes.iter().filter(|n| n.is_long_note()).count();
        assert_eq!(ln_count, 0);
    }

    #[test]
    fn test_add_ln_rate_zero_adds_none() {
        let notes = vec![Note::normal(0, 1000, 1), Note::normal(0, 2000, 2)];
        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::AddLn, 0.0).with_seed(0);
        modifier.modify(&mut model);

        let ln_count = model.notes.iter().filter(|n| n.is_long_note()).count();
        assert_eq!(ln_count, 0);
    }

    #[test]
    fn test_add_ln_pair_index_valid() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(1, 2000, 2), // Different lane at next time
        ];
        let mut model = make_model(notes);
        let mut modifier = LongNoteModifier::new(LongNoteMode::AddLn, 1.0).with_seed(0);
        modifier.modify(&mut model);

        // Check that any LN start/end pairs are correctly linked
        for (i, note) in model.notes.iter().enumerate() {
            if note.is_long_note() && note.pair_index != usize::MAX {
                let pair = &model.notes[note.pair_index];
                assert_eq!(pair.pair_index, i, "pair_index not bidirectional at {i}");
                assert_eq!(pair.lane, note.lane);
            }
        }
    }
}
