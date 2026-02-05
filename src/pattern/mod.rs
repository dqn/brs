mod mirror;
mod random;
mod s_random;

pub use mirror::MirrorModifier;
use rand::Rng;
pub use random::{RRandomModifier, RandomModifier};
pub use s_random::SRandomModifier;

use crate::model::BMSModel;

/// Pattern modification option for gameplay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RandomOption {
    #[default]
    Off,
    Mirror,
    Random,
    RRandom,
    SRandom,
}

impl RandomOption {
    /// Apply this pattern option to a BMS model.
    pub fn apply<R: Rng>(&self, model: &mut BMSModel, rng: &mut R) {
        match self {
            RandomOption::Off => {}
            RandomOption::Mirror => MirrorModifier.modify(model),
            RandomOption::Random => RandomModifier.modify(model, rng),
            RandomOption::RRandom => RRandomModifier.modify(model, rng),
            RandomOption::SRandom => SRandomModifier.modify(model, rng),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::model::note::{Lane, Note};
    use crate::model::timeline::{Timeline, Timelines};
    use crate::model::{ChartFormat, JudgeRankType, LongNoteMode, PlayMode, TotalType};

    fn create_test_model() -> BMSModel {
        let mut timelines = Timelines::new();

        let mut tl1 = Timeline::new(0.0, 0, 0.0, 120.0);
        tl1.add_note(Note::normal(Lane::Key1, 0.0, 1));
        tl1.add_note(Note::normal(Lane::Key3, 0.0, 2));
        timelines.push(tl1);

        let mut tl2 = Timeline::new(500.0, 0, 0.5, 120.0);
        tl2.add_note(Note::normal(Lane::Key2, 500.0, 3));
        tl2.add_note(Note::normal(Lane::Key7, 500.0, 4));
        timelines.push(tl2);

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
    fn test_mirror_changes_lanes() {
        let mut model = create_test_model();
        RandomOption::Mirror.apply(&mut model, &mut ChaCha8Rng::seed_from_u64(0));

        let notes: Vec<_> = model.timelines.all_notes().collect();
        assert_eq!(notes[0].lane, Lane::Key7); // Key1 -> Key7
        assert_eq!(notes[1].lane, Lane::Key5); // Key3 -> Key5
        assert_eq!(notes[2].lane, Lane::Key6); // Key2 -> Key6
        assert_eq!(notes[3].lane, Lane::Key1); // Key7 -> Key1
    }

    #[test]
    fn test_random_shuffles_lanes() {
        let mut model = create_test_model();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        RandomOption::Random.apply(&mut model, &mut rng);

        // Lanes should be changed but all notes should still exist
        assert_eq!(model.timelines.all_notes().count(), 4);

        // All notes should still be on key lanes (not scratch)
        for note in model.timelines.all_notes() {
            assert!(note.lane.is_key());
        }
    }

    #[test]
    fn test_off_does_nothing() {
        let mut model = create_test_model();
        let original_lanes: Vec<_> = model.timelines.all_notes().map(|n| n.lane).collect();

        RandomOption::Off.apply(&mut model, &mut ChaCha8Rng::seed_from_u64(0));

        let new_lanes: Vec<_> = model.timelines.all_notes().map(|n| n.lane).collect();
        assert_eq!(original_lanes, new_lanes);
    }
}
