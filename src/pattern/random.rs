use rand::Rng;
use rand::seq::SliceRandom;

use crate::model::BMSModel;
use crate::model::note::Lane;

/// Random modifier that shuffles key lane assignments.
pub struct RandomModifier;

impl RandomModifier {
    /// Apply random transformation to the model.
    /// All key lanes are randomly shuffled.
    /// Scratch lane is not affected.
    pub fn modify<R: Rng>(&self, model: &mut BMSModel, rng: &mut R) {
        let lane_map = Self::create_shuffle_map(rng);

        for timeline in model.timelines.entries_mut() {
            for note in &mut timeline.notes {
                if note.lane.is_key() {
                    let old_index = note.lane.index() - 1; // Key1 = index 1, so subtract 1
                    let new_index = lane_map[old_index];
                    note.lane = Lane::from_index(new_index + 1).unwrap_or(note.lane);
                }
            }
        }
    }

    fn create_shuffle_map<R: Rng>(rng: &mut R) -> [usize; 7] {
        let mut indices: Vec<usize> = (0..7).collect();
        indices.shuffle(rng);
        let mut map = [0; 7];
        for (i, &v) in indices.iter().enumerate() {
            map[i] = v;
        }
        map
    }
}

/// R-Random modifier that rotates key lanes by a random offset.
pub struct RRandomModifier;

impl RRandomModifier {
    /// Apply R-Random transformation to the model.
    /// Key lanes are rotated by a random offset (1-6).
    /// Scratch lane is not affected.
    pub fn modify<R: Rng>(&self, model: &mut BMSModel, rng: &mut R) {
        let offset = rng.gen_range(1..7); // 1 to 6 rotation

        for timeline in model.timelines.entries_mut() {
            for note in &mut timeline.notes {
                if note.lane.is_key() {
                    let old_index = note.lane.index() - 1; // 0-based key index
                    let new_index = (old_index + offset) % 7;
                    note.lane = Lane::from_index(new_index + 1).unwrap_or(note.lane);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::model::note::Note;
    use crate::model::timeline::{Timeline, Timelines};
    use crate::model::{ChartFormat, JudgeRankType, LongNoteMode, PlayMode, TotalType};

    fn create_test_model() -> BMSModel {
        let mut timelines = Timelines::new();

        let mut tl = Timeline::new(0.0, 0, 0.0, 120.0);
        tl.add_note(Note::normal(Lane::Key1, 0.0, 1));
        tl.add_note(Note::normal(Lane::Key2, 0.0, 2));
        tl.add_note(Note::normal(Lane::Key3, 0.0, 3));
        tl.add_note(Note::normal(Lane::Scratch, 0.0, 4)); // Scratch should not change
        timelines.push(tl);

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
            total_notes: 4,
            total: 200.0,
            total_type: TotalType::Bms,
            judge_rank: 2,
            judge_rank_type: JudgeRankType::BmsRank,
            long_note_mode: LongNoteMode::Ln,
            play_mode: PlayMode::Beat7K,
            source_format: ChartFormat::Bms,
            has_long_note: false,
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
    fn test_random_preserves_scratch() {
        let mut model = create_test_model();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        RandomModifier.modify(&mut model, &mut rng);

        let notes: Vec<_> = model.timelines.all_notes().collect();
        // Find the scratch note (wav_id = 4)
        let scratch_note = notes.iter().find(|n| n.wav_id == 4).unwrap();
        assert_eq!(scratch_note.lane, Lane::Scratch);
    }

    #[test]
    fn test_random_changes_key_lanes() {
        let mut model = create_test_model();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        RandomModifier.modify(&mut model, &mut rng);

        // All key notes should still be on key lanes
        for note in model.timelines.all_notes() {
            if note.wav_id != 4 {
                // Not scratch
                assert!(note.lane.is_key());
            }
        }
    }

    #[test]
    fn test_r_random_rotation() {
        let mut model = create_test_model();
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        RRandomModifier.modify(&mut model, &mut rng);

        // All key notes should still be on key lanes
        for note in model.timelines.all_notes() {
            if note.wav_id != 4 {
                assert!(note.lane.is_key());
            }
        }

        // Scratch should remain unchanged
        let notes: Vec<_> = model.timelines.all_notes().collect();
        let scratch_note = notes.iter().find(|n| n.wav_id == 4).unwrap();
        assert_eq!(scratch_note.lane, Lane::Scratch);
    }
}
