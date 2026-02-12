// Practice modifier — moves notes outside the selected time range to background.
//
// Ported from Java: PracticeModifier.java

use bms_model::BmsModel;

use crate::modifier::{AssistLevel, PatternModifier};

/// Moves visible notes outside `[start_ms, end_ms)` to background.
///
/// Adjusts `model.total` proportionally to remaining notes.
///
/// Java: `PracticeModifier`
pub struct PracticeModifier {
    /// Start time in milliseconds (inclusive)
    pub start_ms: i64,
    /// End time in milliseconds (exclusive)
    pub end_ms: i64,
}

impl PracticeModifier {
    pub fn new(start_ms: i64, end_ms: i64) -> Self {
        Self { start_ms, end_ms }
    }
}

impl PatternModifier for PracticeModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        let total_notes_before = model.total_notes();

        move_notes_outside_range(model, self.start_ms, self.end_ms);

        let total_notes_after = model.total_notes();
        if total_notes_before > 0 {
            model.total = model.total * total_notes_after as f64 / total_notes_before as f64;
        }
    }

    fn assist_level(&self) -> AssistLevel {
        AssistLevel::Assist
    }
}

/// Move notes outside `[start_ms, end_ms)` to background.
///
/// For LN notes, if the start note is outside the range, both start and end
/// (via pair_index) are moved to background.
/// Mine notes outside range are simply removed (not added to bg).
fn move_notes_outside_range(model: &mut BmsModel, start_ms: i64, end_ms: i64) {
    // Collect indices of notes to move/remove
    let mut indices_to_remove: Vec<usize> = Vec::new();

    for (i, note) in model.notes.iter().enumerate() {
        let time_ms = note.time_us / 1000;
        if time_ms < start_ms || time_ms >= end_ms {
            indices_to_remove.push(i);
            // If this is an LN start, also mark the paired end note
            if note.is_long_note()
                && note.pair_index != usize::MAX
                && !indices_to_remove.contains(&note.pair_index)
            {
                indices_to_remove.push(note.pair_index);
            }
        }
    }

    indices_to_remove.sort_unstable();
    indices_to_remove.dedup();

    // Move non-mine notes to bg_notes, remove all marked notes
    for &i in &indices_to_remove {
        let note = &model.notes[i];
        if note.note_type != bms_model::NoteType::Mine {
            model.bg_notes.push(bms_model::BgNote {
                wav_id: note.wav_id,
                time_us: note.time_us,
                micro_starttime: note.micro_starttime,
                micro_duration: note.micro_duration,
            });
        }
    }

    // Remove notes in reverse order to preserve indices
    for &i in indices_to_remove.iter().rev() {
        model.notes.remove(i);
    }

    // Rebuild pair_index for remaining LN notes
    rebuild_pair_indices(&mut model.notes);
}

/// Rebuild LN pair indices after notes have been removed.
fn rebuild_pair_indices(notes: &mut [bms_model::Note]) {
    // Reset all pair indices
    for note in notes.iter_mut() {
        note.pair_index = usize::MAX;
    }

    // Find LN start notes (end_time_us > 0) and match to end notes
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
            total: 300.0,
            notes,
            ..Default::default()
        }
    }

    #[test]
    fn test_practice_removes_notes_outside_range() {
        let notes = vec![
            Note::normal(0, 500_000, 1),  // 500ms - outside [1000, 3000)
            Note::normal(1, 1500_000, 2), // 1500ms - inside
            Note::normal(2, 2500_000, 3), // 2500ms - inside
            Note::normal(3, 3500_000, 4), // 3500ms - outside
        ];
        let mut model = make_model(notes);
        let mut modifier = PracticeModifier::new(1000, 3000);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert_eq!(model.notes[0].wav_id, 2);
        assert_eq!(model.notes[1].wav_id, 3);
    }

    #[test]
    fn test_practice_adjusts_total() {
        let notes = vec![
            Note::normal(0, 500_000, 1),
            Note::normal(1, 1500_000, 2),
            Note::normal(2, 2500_000, 3),
            Note::normal(3, 3500_000, 4),
        ];
        let mut model = make_model(notes);
        model.total = 400.0;
        let mut modifier = PracticeModifier::new(1000, 3000);
        modifier.modify(&mut model);

        // 2/4 notes remain -> total = 400 * 2/4 = 200
        assert!((model.total - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_practice_moves_to_bg_notes() {
        let notes = vec![Note::normal(0, 500_000, 1), Note::normal(1, 1500_000, 2)];
        let mut model = make_model(notes);
        let mut modifier = PracticeModifier::new(1000, 2000);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 1);
        assert_eq!(model.bg_notes.len(), 1);
        assert_eq!(model.bg_notes[0].wav_id, 1);
    }

    #[test]
    fn test_practice_mine_notes_not_added_to_bg() {
        let notes = vec![Note::mine(0, 500_000, 1, 10), Note::normal(1, 1500_000, 2)];
        let mut model = make_model(notes);
        let mut modifier = PracticeModifier::new(1000, 2000);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 1);
        assert_eq!(model.bg_notes.len(), 0);
    }

    #[test]
    fn test_practice_empty_range_removes_all() {
        let notes = vec![Note::normal(0, 1000_000, 1), Note::normal(1, 2000_000, 2)];
        let mut model = make_model(notes);
        model.total = 300.0;
        let mut modifier = PracticeModifier::new(5000, 6000);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 0);
        assert!((model.total - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_practice_all_notes_in_range() {
        let notes = vec![Note::normal(0, 1000_000, 1), Note::normal(1, 2000_000, 2)];
        let mut model = make_model(notes);
        model.total = 300.0;
        let mut modifier = PracticeModifier::new(0, 5000);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert!((model.total - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_practice_ln_pair_moved_together() {
        let mut notes = vec![
            Note::long_note(0, 1000_000, 2000_000, 1, 2, LnType::LongNote), // LN start inside
            Note::long_note(0, 2000_000, 0, 2, 0, LnType::LongNote),        // LN end inside
            Note::long_note(1, 500_000, 800_000, 3, 4, LnType::LongNote),   // LN start outside
            Note::long_note(1, 800_000, 0, 4, 0, LnType::LongNote),         // LN end outside
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;
        notes[2].pair_index = 3;
        notes[3].pair_index = 2;

        let mut model = make_model(notes);
        let mut modifier = PracticeModifier::new(900, 3000);
        modifier.modify(&mut model);

        // Only the inside LN pair should remain
        assert_eq!(model.notes.len(), 2);
        assert_eq!(model.notes[0].pair_index, 1);
        assert_eq!(model.notes[1].pair_index, 0);
    }

    #[test]
    fn test_practice_assist_level() {
        let modifier = PracticeModifier::new(0, 1000);
        assert_eq!(modifier.assist_level(), AssistLevel::Assist);
    }
}
