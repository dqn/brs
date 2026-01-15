use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::bms::{Chart, NoteChannel};

/// Random option for lane assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RandomOption {
    #[default]
    Off,
    Mirror,
    Random,
    #[allow(dead_code)]
    RRandom,
    SRandom,
}

impl RandomOption {
    /// Get display name for the option
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Mirror => "MIRROR",
            Self::Random => "RANDOM",
            Self::RRandom => "R-RANDOM",
            Self::SRandom => "S-RANDOM",
        }
    }
}

/// Lane mapping for random options
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LaneMapping {
    /// Mapping from original key lane (1-7) to new lane (1-7)
    /// Index 0 is unused, indices 1-7 map Key1-Key7
    map: [usize; 8],
}

impl LaneMapping {
    /// Create identity mapping (no change)
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            map: [0, 1, 2, 3, 4, 5, 6, 7],
        }
    }

    /// Create mirror mapping (reverse keys)
    #[allow(dead_code)]
    pub fn mirror() -> Self {
        // Key1 <-> Key7, Key2 <-> Key6, Key3 <-> Key5, Key4 stays
        Self {
            map: [0, 7, 6, 5, 4, 3, 2, 1],
        }
    }

    /// Create random mapping using Fisher-Yates shuffle
    #[allow(dead_code)]
    pub fn random(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut lanes: Vec<usize> = (1..=7).collect();

        // Fisher-Yates shuffle
        for i in (1..lanes.len()).rev() {
            let j = rng.random_range(0..=i);
            lanes.swap(i, j);
        }

        Self {
            map: [
                0, lanes[0], lanes[1], lanes[2], lanes[3], lanes[4], lanes[5], lanes[6],
            ],
        }
    }

    /// Create rotate-random mapping
    #[allow(dead_code)]
    pub fn rotate_random(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let offset = rng.random_range(0..7);

        let mut map = [0usize; 8];
        for (i, item) in map.iter_mut().enumerate().skip(1) {
            *item = ((i - 1 + offset) % 7) + 1;
        }

        Self { map }
    }

    /// Create mapping for the given option
    #[allow(dead_code)]
    pub fn for_option(option: RandomOption, seed: u64) -> Self {
        match option {
            RandomOption::Off => Self::identity(),
            RandomOption::Mirror => Self::mirror(),
            RandomOption::Random => Self::random(seed),
            RandomOption::RRandom => Self::rotate_random(seed),
            RandomOption::SRandom => {
                // S-RANDOM is not handled by LaneMapping
                // It should be handled by apply_s_random directly
                Self::identity()
            }
        }
    }

    /// Transform a key lane (1-7) to the new lane
    #[allow(dead_code)]
    pub fn transform(&self, lane: usize) -> usize {
        if (1..=7).contains(&lane) {
            self.map[lane]
        } else {
            lane // Scratch (0) and invalid lanes unchanged
        }
    }
}

/// Apply random option to a chart
#[allow(dead_code)]
pub fn apply_random_option(chart: &mut Chart, option: RandomOption, seed: u64) {
    if option == RandomOption::Off {
        return;
    }

    if option == RandomOption::SRandom {
        apply_s_random(chart, seed);
        return;
    }

    let mapping = LaneMapping::for_option(option, seed);

    for note in &mut chart.notes {
        if note.channel.is_key() {
            let original_lane = note.channel.lane_index();
            let new_lane = mapping.transform(original_lane);
            if let Some(new_channel) = NoteChannel::from_key_lane(new_lane) {
                note.channel = new_channel;
            }
        }
    }

    // Rebuild lane index after modifying notes
    // Note: This is handled by the caller if needed
}

/// Apply S-RANDOM option to a chart
/// Each note gets an independent random lane, but LN start/end pairs stay together
fn apply_s_random(chart: &mut Chart, seed: u64) {
    use crate::bms::NoteType;
    use std::collections::HashMap;

    let mut rng = StdRng::seed_from_u64(seed);

    // Track LN start positions to maintain LN start/end pair consistency
    // Key: (original_lane, time_ms of LongStart), Value: new_lane
    let mut ln_mappings: HashMap<(usize, i64), usize> = HashMap::new();

    for note in &mut chart.notes {
        if !note.channel.is_key() {
            continue;
        }

        let original_lane = note.channel.lane_index();

        let new_lane = match note.note_type {
            NoteType::LongStart => {
                // Generate new random lane for LN start
                let new_lane = rng.random_range(1..=7);
                // Store mapping for the corresponding LN end
                let time_key = (note.time_ms * 1000.0) as i64; // Convert to integer for HashMap key
                ln_mappings.insert((original_lane, time_key), new_lane);
                new_lane
            }
            NoteType::LongEnd => {
                // Find the corresponding LN start mapping
                // LN end should match with a previous LN start on the same original lane
                let time_key = (note.time_ms * 1000.0) as i64;
                // Search for the nearest LN start on this lane
                let mapping_key = ln_mappings
                    .keys()
                    .filter(|(lane, _)| *lane == original_lane)
                    .min_by_key(|(_, t)| (t - time_key).abs());

                if let Some(&key) = mapping_key {
                    ln_mappings.remove(&key).unwrap_or(original_lane)
                } else {
                    // Fallback: generate random lane if no matching LN start found
                    rng.random_range(1..=7)
                }
            }
            NoteType::Normal | NoteType::Invisible | NoteType::Landmine => {
                // Normal notes get independent random lanes
                rng.random_range(1..=7)
            }
        };

        if let Some(new_channel) = NoteChannel::from_key_lane(new_lane) {
            note.channel = new_channel;
        }
    }
}

/// Generate a random seed for replay reproducibility
#[allow(dead_code)]
pub fn generate_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bms::{Metadata, Note, NoteType, TimingData};
    use fraction::Fraction;

    #[test]
    fn test_identity_mapping() {
        let mapping = LaneMapping::identity();
        for i in 1..=7 {
            assert_eq!(mapping.transform(i), i);
        }
    }

    #[test]
    fn test_mirror_mapping() {
        let mapping = LaneMapping::mirror();
        assert_eq!(mapping.transform(1), 7);
        assert_eq!(mapping.transform(2), 6);
        assert_eq!(mapping.transform(3), 5);
        assert_eq!(mapping.transform(4), 4);
        assert_eq!(mapping.transform(5), 3);
        assert_eq!(mapping.transform(6), 2);
        assert_eq!(mapping.transform(7), 1);
    }

    #[test]
    fn test_random_mapping_is_permutation() {
        let mapping = LaneMapping::random(12345);
        let mut seen = [false; 8];
        for i in 1..=7 {
            let new_lane = mapping.transform(i);
            assert!(new_lane >= 1 && new_lane <= 7);
            assert!(!seen[new_lane], "Lane {} appears twice", new_lane);
            seen[new_lane] = true;
        }
    }

    #[test]
    fn test_random_is_deterministic() {
        let mapping1 = LaneMapping::random(12345);
        let mapping2 = LaneMapping::random(12345);
        for i in 1..=7 {
            assert_eq!(mapping1.transform(i), mapping2.transform(i));
        }
    }

    #[test]
    fn test_scratch_unchanged() {
        let mapping = LaneMapping::mirror();
        assert_eq!(mapping.transform(0), 0);
    }

    fn create_test_note(channel: NoteChannel, note_type: NoteType, time_ms: f64) -> Note {
        Note {
            measure: 0,
            position: Fraction::from(0),
            time_ms,
            channel,
            keysound_id: 0,
            note_type,
            long_end_time_ms: None,
        }
    }

    fn create_test_chart(notes: Vec<Note>) -> Chart {
        Chart {
            metadata: Metadata::default(),
            timing_data: TimingData::default(),
            notes,
            bgm_events: vec![],
            bga_events: vec![],
        }
    }

    #[test]
    fn test_s_random_normal_notes() {
        let notes = vec![
            create_test_note(NoteChannel::Key1, NoteType::Normal, 0.0),
            create_test_note(NoteChannel::Key2, NoteType::Normal, 100.0),
            create_test_note(NoteChannel::Key3, NoteType::Normal, 200.0),
        ];
        let mut chart = create_test_chart(notes);

        apply_random_option(&mut chart, RandomOption::SRandom, 12345);

        // All notes should have valid key lanes (1-7)
        for note in &chart.notes {
            let lane = note.channel.lane_index();
            assert!(lane >= 1 && lane <= 7, "Lane {} is out of range", lane);
        }
    }

    #[test]
    fn test_s_random_is_deterministic() {
        let notes = vec![
            create_test_note(NoteChannel::Key1, NoteType::Normal, 0.0),
            create_test_note(NoteChannel::Key2, NoteType::Normal, 100.0),
            create_test_note(NoteChannel::Key3, NoteType::Normal, 200.0),
        ];

        let mut chart1 = create_test_chart(notes.clone());
        let mut chart2 = create_test_chart(notes);

        apply_random_option(&mut chart1, RandomOption::SRandom, 12345);
        apply_random_option(&mut chart2, RandomOption::SRandom, 12345);

        for (n1, n2) in chart1.notes.iter().zip(chart2.notes.iter()) {
            assert_eq!(n1.channel, n2.channel);
        }
    }

    #[test]
    fn test_s_random_ln_consistency() {
        // Create LN start and end on the same lane
        let notes = vec![
            create_test_note(NoteChannel::Key1, NoteType::LongStart, 0.0),
            create_test_note(NoteChannel::Key1, NoteType::LongEnd, 500.0),
        ];
        let mut chart = create_test_chart(notes);

        apply_random_option(&mut chart, RandomOption::SRandom, 12345);

        // LN start and end should be on the same lane
        assert_eq!(
            chart.notes[0].channel, chart.notes[1].channel,
            "LN start and end should be on the same lane"
        );
    }

    #[test]
    fn test_s_random_scratch_unchanged() {
        let notes = vec![
            create_test_note(NoteChannel::Scratch, NoteType::Normal, 0.0),
            create_test_note(NoteChannel::Key1, NoteType::Normal, 100.0),
        ];
        let mut chart = create_test_chart(notes);

        apply_random_option(&mut chart, RandomOption::SRandom, 12345);

        // Scratch should remain unchanged
        assert_eq!(
            chart.notes[0].channel,
            NoteChannel::Scratch,
            "Scratch should not be randomized"
        );
    }

    #[test]
    fn test_s_random_display_name() {
        assert_eq!(RandomOption::SRandom.display_name(), "S-RANDOM");
    }
}
