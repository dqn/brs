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
}
