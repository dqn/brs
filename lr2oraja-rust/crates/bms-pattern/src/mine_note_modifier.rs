// Mine note modifier — removes or adds mine notes.
//
// Ported from Java: MineNoteModifier.java

use std::collections::BTreeMap;

use bms_model::{BmsModel, Note, NoteType};

use crate::java_random::JavaRandom;
use crate::modifier::{AssistLevel, PatternModifier};

/// Mode for mine note modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineNoteMode {
    /// Remove all mine notes
    Remove,
    /// Add mines randomly on blank lanes (10% chance)
    AddRandom,
    /// Add mines on blank lanes adjacent to occupied lanes
    AddNear,
    /// Add mines on all blank lanes
    AddBlank,
}

/// Removes or adds mine notes.
///
/// Java: `MineNoteModifier`
pub struct MineNoteModifier {
    pub mode: MineNoteMode,
    pub damage: i32,
    /// Seed for random mine placement (AddRandom mode)
    seed: i64,
    /// Whether mine notes existed (for Remove mode)
    mine_note_exists: bool,
}

impl MineNoteModifier {
    pub fn new(mode: MineNoteMode) -> Self {
        Self {
            mode,
            damage: 10,
            seed: 0,
            mine_note_exists: false,
        }
    }

    pub fn with_damage(mut self, damage: i32) -> Self {
        self.damage = damage;
        self
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = seed;
        self
    }

    pub fn mine_note_exists(&self) -> bool {
        self.mine_note_exists
    }
}

impl PatternModifier for MineNoteModifier {
    fn modify(&mut self, model: &mut BmsModel) {
        match self.mode {
            MineNoteMode::Remove => self.remove_mines(model),
            _ => self.add_mines(model),
        }
    }

    fn assist_level(&self) -> AssistLevel {
        if self.mode == MineNoteMode::Remove && self.mine_note_exists {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        }
    }
}

impl MineNoteModifier {
    fn remove_mines(&mut self, model: &mut BmsModel) {
        let had_mines = model.notes.iter().any(|n| n.note_type == NoteType::Mine);
        if had_mines {
            self.mine_note_exists = true;
            model.notes.retain(|n| n.note_type != NoteType::Mine);
        }
    }

    fn add_mines(&mut self, model: &mut BmsModel) {
        let key_count = model.mode.key_count();
        let mut rng = JavaRandom::new(self.seed);

        // Group notes by time_us
        let mut time_groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (idx, note) in model.notes.iter().enumerate() {
            time_groups.entry(note.time_us).or_default().push(idx);
        }

        // Track LN active state per lane
        let mut ln_active = vec![false; key_count];
        let mut new_mines: Vec<Note> = Vec::new();

        for (&time_us, indices) in &time_groups {
            let mut blank = vec![false; key_count];

            // Update LN state and determine blank lanes
            for key in 0..key_count {
                let note_at_lane = indices.iter().find(|&&i| model.notes[i].lane == key);

                if let Some(&idx) = note_at_lane {
                    let note = &model.notes[idx];
                    if note.is_long_note() {
                        // LN end note has end_time_us == 0
                        ln_active[key] = note.end_time_us > 0;
                    }
                    blank[key] = false;
                } else {
                    blank[key] = !ln_active[key];
                }
            }

            // Add mines based on mode
            for key in 0..key_count {
                if blank[key] {
                    let should_add = match self.mode {
                        MineNoteMode::AddRandom => rng.next_double() > 0.9,
                        MineNoteMode::AddNear => {
                            (key > 0 && !blank[key - 1]) || (key < key_count - 1 && !blank[key + 1])
                        }
                        MineNoteMode::AddBlank => true,
                        MineNoteMode::Remove => false,
                    };

                    if should_add {
                        new_mines.push(Note::mine(key, time_us, 0, self.damage));
                    }
                }
            }
        }

        model.notes.extend(new_mines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::PlayMode;

    fn make_model(notes: Vec<Note>) -> BmsModel {
        BmsModel {
            mode: PlayMode::Beat7K,
            notes,
            ..Default::default()
        }
    }

    #[test]
    fn test_remove_mines() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::mine(1, 1000, 2, 10),
            Note::normal(2, 2000, 3),
            Note::mine(3, 2000, 4, 10),
        ];
        let mut model = make_model(notes);
        let mut modifier = MineNoteModifier::new(MineNoteMode::Remove);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert!(model.notes.iter().all(|n| n.note_type != NoteType::Mine));
        assert!(modifier.mine_note_exists());
        assert_eq!(modifier.assist_level(), AssistLevel::LightAssist);
    }

    #[test]
    fn test_remove_no_mines() {
        let notes = vec![Note::normal(0, 1000, 1), Note::normal(1, 2000, 2)];
        let mut model = make_model(notes);
        let mut modifier = MineNoteModifier::new(MineNoteMode::Remove);
        modifier.modify(&mut model);

        assert_eq!(model.notes.len(), 2);
        assert!(!modifier.mine_note_exists());
        assert_eq!(modifier.assist_level(), AssistLevel::None);
    }

    #[test]
    fn test_add_blank_adds_on_all_blank_lanes() {
        // One note on lane 0 at time 1000
        let notes = vec![Note::normal(0, 1000, 1)];
        let mut model = make_model(notes);
        let mut modifier = MineNoteModifier::new(MineNoteMode::AddBlank).with_damage(20);
        modifier.modify(&mut model);

        // Should have 1 normal + 7 mines (lanes 1-7 are blank)
        let mine_count = model
            .notes
            .iter()
            .filter(|n| n.note_type == NoteType::Mine)
            .count();
        assert_eq!(mine_count, 7);

        // All mines should have damage 20
        for note in model.notes.iter().filter(|n| n.note_type == NoteType::Mine) {
            assert_eq!(note.damage, 20);
        }
    }

    #[test]
    fn test_add_near_adds_adjacent_only() {
        // Note on lane 3 at time 1000
        let notes = vec![Note::normal(3, 1000, 1)];
        let mut model = make_model(notes);
        let mut modifier = MineNoteModifier::new(MineNoteMode::AddNear);
        modifier.modify(&mut model);

        let mines: Vec<&Note> = model
            .notes
            .iter()
            .filter(|n| n.note_type == NoteType::Mine)
            .collect();

        // Only lanes 2 and 4 should have mines (adjacent to lane 3)
        assert_eq!(mines.len(), 2);
        let mine_lanes: Vec<usize> = mines.iter().map(|n| n.lane).collect();
        assert!(mine_lanes.contains(&2));
        assert!(mine_lanes.contains(&4));
    }

    #[test]
    fn test_add_random_is_deterministic_with_seed() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let notes2 = notes.clone();

        let mut model1 = make_model(notes);
        let mut modifier1 = MineNoteModifier::new(MineNoteMode::AddRandom).with_seed(42);
        modifier1.modify(&mut model1);

        let mut model2 = make_model(notes2);
        let mut modifier2 = MineNoteModifier::new(MineNoteMode::AddRandom).with_seed(42);
        modifier2.modify(&mut model2);

        let mines1: Vec<usize> = model1
            .notes
            .iter()
            .filter(|n| n.note_type == NoteType::Mine)
            .map(|n| n.lane)
            .collect();
        let mines2: Vec<usize> = model2
            .notes
            .iter()
            .filter(|n| n.note_type == NoteType::Mine)
            .map(|n| n.lane)
            .collect();
        assert_eq!(mines1, mines2);
    }

    #[test]
    fn test_add_blank_respects_active_ln() {
        // LN active on lane 1 from time 1000 to 3000
        let mut notes = vec![
            Note::long_note(1, 1000, 3000, 1, 2, bms_model::LnType::LongNote),
            Note::long_note(1, 3000, 0, 2, 0, bms_model::LnType::LongNote),
            Note::normal(0, 2000, 3), // At time 2000, lane 1 has active LN
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;

        let mut model = make_model(notes);
        let mut modifier = MineNoteModifier::new(MineNoteMode::AddBlank);
        modifier.modify(&mut model);

        // At time 2000: lane 0 has note, lane 1 has active LN -> not blank
        // Lanes 2-7 should have mines
        let mines_at_2000: Vec<&Note> = model
            .notes
            .iter()
            .filter(|n| n.note_type == NoteType::Mine && n.time_us == 2000)
            .collect();
        assert_eq!(mines_at_2000.len(), 6); // lanes 2,3,4,5,6,7
        assert!(mines_at_2000.iter().all(|n| n.lane != 0 && n.lane != 1));
    }

    #[test]
    fn test_no_notes_no_mines_added() {
        let mut model = make_model(Vec::new());
        let mut modifier = MineNoteModifier::new(MineNoteMode::AddBlank);
        modifier.modify(&mut model);
        assert!(model.notes.is_empty());
    }
}
