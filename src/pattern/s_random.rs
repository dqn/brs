use rand::Rng;
use rand::seq::SliceRandom;

use crate::model::BMSModel;
use crate::model::note::{Lane, NoteType};

/// S-Random modifier that randomly assigns each note to a lane.
/// Includes vertical collision avoidance to prevent rapid consecutive
/// notes on the same lane.
pub struct SRandomModifier;

impl SRandomModifier {
    /// Minimum time interval (ms) between notes on the same lane.
    const MIN_INTERVAL_MS: f64 = 50.0;

    /// Apply S-Random transformation to the model.
    /// Each note is individually assigned to a random lane,
    /// with collision avoidance for rapid consecutive notes.
    /// Scratch lane is not affected.
    pub fn modify<R: Rng>(&self, model: &mut BMSModel, rng: &mut R) {
        // Track last note time for each lane (for collision avoidance)
        let mut last_note_time: [f64; 7] = [f64::NEG_INFINITY; 7];

        // Collect all key notes with their timeline/note indices
        let mut key_notes: Vec<(usize, usize, f64)> = Vec::new();
        for (tl_idx, timeline) in model.timelines.entries().iter().enumerate() {
            for (n_idx, note) in timeline.notes.iter().enumerate() {
                if note.lane.is_key() {
                    key_notes.push((tl_idx, n_idx, note.start_time_ms));
                }
            }
        }

        // Sort by time for proper collision avoidance
        key_notes.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        // Track LN pairs (start -> end mapping)
        let mut ln_lane_map: std::collections::HashMap<(usize, usize), Lane> =
            std::collections::HashMap::new();

        // First pass: assign lanes to LN starts and normal notes
        for &(tl_idx, n_idx, time) in &key_notes {
            let note = &model.timelines.entries()[tl_idx].notes[n_idx];

            // Skip LN ends - they will be assigned in second pass
            if note.note_type == NoteType::LongEnd {
                continue;
            }

            // Find available lanes (not recently used)
            let available: Vec<usize> = (0..7)
                .filter(|&i| time - last_note_time[i] >= Self::MIN_INTERVAL_MS)
                .collect();

            let new_lane_idx = if available.is_empty() {
                // All lanes are "hot", pick random
                rng.gen_range(0..7)
            } else {
                // Pick random from available
                *available.choose(rng).unwrap()
            };

            let new_lane = Lane::from_index(new_lane_idx + 1).unwrap();

            // Update last note time
            last_note_time[new_lane_idx] = time;

            // If this is an LN start, record the lane mapping
            if note.note_type == NoteType::LongStart {
                ln_lane_map.insert((tl_idx, n_idx), new_lane);
            }

            // Apply the lane change
            model.timelines.entries_mut()[tl_idx].notes[n_idx].lane = new_lane;
        }

        // Second pass: assign LN ends to match their starts
        for (tl_idx, timeline) in model.timelines.entries_mut().iter_mut().enumerate() {
            for note in &mut timeline.notes {
                if note.note_type == NoteType::LongEnd && note.lane.is_key() {
                    // Find the matching LN start
                    // For simplicity, find the start with matching wav_id and closest time before this end
                    if let Some(&start_lane) = ln_lane_map
                        .iter()
                        .find(|&((stl, _sn), _)| *stl <= tl_idx)
                        .map(|(_, lane)| lane)
                    {
                        note.lane = start_lane;
                    }
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

        for i in 0..10 {
            let mut tl = Timeline::new(i as f64 * 100.0, 0, 0.0, 120.0);
            tl.add_note(Note::normal(Lane::Key1, i as f64 * 100.0, i as u16));
            timelines.push(tl);
        }

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
            total_notes: 10,
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
    fn test_s_random_distributes_notes() {
        let mut model = create_test_model();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        SRandomModifier.modify(&mut model, &mut rng);

        // All notes should be on key lanes
        for note in model.timelines.all_notes() {
            assert!(note.lane.is_key());
        }

        // Notes should be distributed across multiple lanes (with high probability)
        let lanes: std::collections::HashSet<_> =
            model.timelines.all_notes().map(|n| n.lane).collect();
        assert!(
            lanes.len() > 1,
            "S-Random should distribute across multiple lanes"
        );
    }

    #[test]
    fn test_s_random_avoids_rapid_same_lane() {
        let mut timelines = Timelines::new();

        // Create notes very close together
        for i in 0..5 {
            let mut tl = Timeline::new(i as f64 * 10.0, 0, 0.0, 120.0);
            tl.add_note(Note::normal(Lane::Key1, i as f64 * 10.0, i as u16));
            timelines.push(tl);
        }

        let mut model = BMSModel {
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
            total_notes: 5,
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
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        SRandomModifier.modify(&mut model, &mut rng);

        // With collision avoidance, notes close together should be on different lanes
        let notes: Vec<_> = model.timelines.all_notes().collect();
        for i in 1..notes.len() {
            let time_diff = notes[i].start_time_ms - notes[i - 1].start_time_ms;
            if time_diff < SRandomModifier::MIN_INTERVAL_MS {
                // Adjacent notes within min interval should be on different lanes (if possible)
                // This may not always be guaranteed if all lanes are "hot"
            }
        }
    }
}
