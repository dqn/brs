// Lane property: maps physical keys to lanes and vice versa.
//
// Ported from Java: LaneProperty.java
// Provides key→lane, lane→keys, and scratch lane mappings per PlayMode.

use crate::PlayMode;

/// Lane property configuration for a play mode.
///
/// Maps between physical input keys and logical BMS lanes, including scratch
/// lane identification. Physical key count >= lane count when scratch lanes
/// have two physical keys (e.g., Beat7K: 9 physical keys → 8 lanes).
#[derive(Debug, Clone)]
pub struct LaneProperty {
    /// Physical key index → lane index
    key_to_lane: Vec<usize>,
    /// Lane index → list of physical key indices
    lane_to_keys: Vec<Vec<usize>>,
    /// Lane index → scratch index (-1 if not scratch)
    lane_to_scratch: Vec<i32>,
    /// Per-scratch: [key_a, key_b] physical key pair
    scratch_to_keys: Vec<[usize; 2]>,
}

impl LaneProperty {
    /// Create a LaneProperty for the given play mode (matches Java LaneProperty constructor).
    pub fn new(mode: PlayMode) -> Self {
        match mode {
            PlayMode::Beat5K => Self {
                key_to_lane: vec![0, 1, 2, 3, 4, 5, 5],
                lane_to_keys: vec![vec![0], vec![1], vec![2], vec![3], vec![4], vec![5, 6]],
                lane_to_scratch: vec![-1, -1, -1, -1, -1, 0],
                scratch_to_keys: vec![[5, 6]],
            },
            PlayMode::Beat7K => Self {
                key_to_lane: vec![0, 1, 2, 3, 4, 5, 6, 7, 7],
                lane_to_keys: vec![
                    vec![0],
                    vec![1],
                    vec![2],
                    vec![3],
                    vec![4],
                    vec![5],
                    vec![6],
                    vec![7, 8],
                ],
                lane_to_scratch: vec![-1, -1, -1, -1, -1, -1, -1, 0],
                scratch_to_keys: vec![[7, 8]],
            },
            PlayMode::Beat10K => Self {
                key_to_lane: vec![0, 1, 2, 3, 4, 5, 5, 6, 7, 8, 9, 10, 11, 11],
                lane_to_keys: vec![
                    vec![0],
                    vec![1],
                    vec![2],
                    vec![3],
                    vec![4],
                    vec![5, 6],
                    vec![7],
                    vec![8],
                    vec![9],
                    vec![10],
                    vec![11],
                    vec![12, 13],
                ],
                lane_to_scratch: vec![-1, -1, -1, -1, -1, 0, -1, -1, -1, -1, -1, 1],
                scratch_to_keys: vec![[5, 6], [12, 13]],
            },
            PlayMode::Beat14K => Self {
                key_to_lane: vec![0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15],
                lane_to_keys: vec![
                    vec![0],
                    vec![1],
                    vec![2],
                    vec![3],
                    vec![4],
                    vec![5],
                    vec![6],
                    vec![7, 8],
                    vec![9],
                    vec![10],
                    vec![11],
                    vec![12],
                    vec![13],
                    vec![14],
                    vec![15],
                    vec![16, 17],
                ],
                lane_to_scratch: vec![-1, -1, -1, -1, -1, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, 1],
                scratch_to_keys: vec![[7, 8], [16, 17]],
            },
            PlayMode::PopN5K | PlayMode::PopN9K => {
                let key_count = mode.key_count();
                Self {
                    key_to_lane: (0..key_count).collect(),
                    lane_to_keys: (0..key_count).map(|i| vec![i]).collect(),
                    lane_to_scratch: vec![-1; key_count],
                    scratch_to_keys: vec![],
                }
            }
            PlayMode::Keyboard24K => Self {
                key_to_lane: (0..26).collect(),
                lane_to_keys: (0..26).map(|i| vec![i]).collect(),
                lane_to_scratch: vec![-1; 26],
                scratch_to_keys: vec![],
            },
            PlayMode::Keyboard24KDouble => Self {
                key_to_lane: (0..52).collect(),
                lane_to_keys: (0..52).map(|i| vec![i]).collect(),
                lane_to_scratch: vec![-1; 52],
                scratch_to_keys: vec![],
            },
        }
    }

    /// Number of physical input keys.
    pub fn physical_key_count(&self) -> usize {
        self.key_to_lane.len()
    }

    /// Number of logical lanes.
    pub fn lane_count(&self) -> usize {
        self.lane_to_keys.len()
    }

    /// Map a physical key index to a lane index.
    pub fn key_to_lane(&self, key_idx: usize) -> usize {
        self.key_to_lane[key_idx]
    }

    /// Get the physical key indices for a lane.
    pub fn lane_to_keys(&self, lane_idx: usize) -> &[usize] {
        &self.lane_to_keys[lane_idx]
    }

    /// Get the scratch index for a lane, or None if not a scratch lane.
    pub fn scratch_index(&self, lane_idx: usize) -> Option<usize> {
        let sc = self.lane_to_scratch[lane_idx];
        if sc >= 0 { Some(sc as usize) } else { None }
    }

    /// Number of scratch controllers.
    pub fn scratch_count(&self) -> usize {
        self.scratch_to_keys.len()
    }

    /// Get the two physical key indices for a scratch controller.
    pub fn scratch_keys(&self, scratch_idx: usize) -> [usize; 2] {
        self.scratch_to_keys[scratch_idx]
    }

    /// Convert per-lane boolean states to per-physical-key states.
    /// Non-scratch lanes: 1:1 copy. Scratch lanes: replicate to both physical keys.
    pub fn lane_to_key_states(&self, lane_states: &[bool]) -> Vec<bool> {
        let mut key_states = vec![false; self.physical_key_count()];
        for (lane_idx, &state) in lane_states.iter().enumerate() {
            for &key_idx in &self.lane_to_keys[lane_idx] {
                key_states[key_idx] = state;
            }
        }
        key_states
    }

    /// Convert per-lane timestamps to per-physical-key timestamps.
    /// Non-scratch lanes: 1:1 copy. Scratch lanes: replicate to both physical keys.
    pub fn lane_to_key_times(&self, lane_times: &[i64]) -> Vec<i64> {
        let mut key_times = vec![i64::MIN; self.physical_key_count()];
        for (lane_idx, &time) in lane_times.iter().enumerate() {
            for &key_idx in &self.lane_to_keys[lane_idx] {
                key_times[key_idx] = time;
            }
        }
        key_times
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat7k_physical_key_count() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        assert_eq!(lp.physical_key_count(), 9);
        assert_eq!(lp.lane_count(), 8);
    }

    #[test]
    fn beat7k_key_to_lane_mapping() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        // Keys 0-6 map to lanes 0-6
        for i in 0..7 {
            assert_eq!(lp.key_to_lane(i), i);
        }
        // Keys 7 and 8 both map to lane 7 (scratch)
        assert_eq!(lp.key_to_lane(7), 7);
        assert_eq!(lp.key_to_lane(8), 7);
    }

    #[test]
    fn beat7k_lane_to_keys_mapping() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        // Non-scratch lanes have 1 key
        for i in 0..7 {
            assert_eq!(lp.lane_to_keys(i), &[i]);
        }
        // Scratch lane has 2 keys
        assert_eq!(lp.lane_to_keys(7), &[7, 8]);
    }

    #[test]
    fn beat7k_scratch_index() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        for i in 0..7 {
            assert_eq!(lp.scratch_index(i), None);
        }
        assert_eq!(lp.scratch_index(7), Some(0));
    }

    #[test]
    fn beat7k_scratch_keys() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        assert_eq!(lp.scratch_count(), 1);
        assert_eq!(lp.scratch_keys(0), [7, 8]);
    }

    #[test]
    fn beat5k_mapping() {
        let lp = LaneProperty::new(PlayMode::Beat5K);
        assert_eq!(lp.physical_key_count(), 7);
        assert_eq!(lp.lane_count(), 6);
        assert_eq!(lp.key_to_lane(5), 5);
        assert_eq!(lp.key_to_lane(6), 5);
        assert_eq!(lp.lane_to_keys(5), &[5, 6]);
        assert_eq!(lp.scratch_index(5), Some(0));
    }

    #[test]
    fn beat14k_mapping() {
        let lp = LaneProperty::new(PlayMode::Beat14K);
        assert_eq!(lp.physical_key_count(), 18);
        assert_eq!(lp.lane_count(), 16);
        // 1P scratch: keys 7,8 → lane 7
        assert_eq!(lp.key_to_lane(7), 7);
        assert_eq!(lp.key_to_lane(8), 7);
        assert_eq!(lp.scratch_index(7), Some(0));
        // 2P scratch: keys 16,17 → lane 15
        assert_eq!(lp.key_to_lane(16), 15);
        assert_eq!(lp.key_to_lane(17), 15);
        assert_eq!(lp.scratch_index(15), Some(1));
        assert_eq!(lp.scratch_count(), 2);
    }

    #[test]
    fn beat10k_mapping() {
        let lp = LaneProperty::new(PlayMode::Beat10K);
        assert_eq!(lp.physical_key_count(), 14);
        assert_eq!(lp.lane_count(), 12);
        // 1P scratch: keys 5,6 → lane 5
        assert_eq!(lp.key_to_lane(5), 5);
        assert_eq!(lp.key_to_lane(6), 5);
        assert_eq!(lp.scratch_index(5), Some(0));
        // 2P scratch: keys 12,13 → lane 11
        assert_eq!(lp.key_to_lane(12), 11);
        assert_eq!(lp.key_to_lane(13), 11);
        assert_eq!(lp.scratch_index(11), Some(1));
    }

    #[test]
    fn popn9k_no_scratch() {
        let lp = LaneProperty::new(PlayMode::PopN9K);
        assert_eq!(lp.physical_key_count(), 9);
        assert_eq!(lp.lane_count(), 9);
        assert_eq!(lp.scratch_count(), 0);
        for i in 0..9 {
            assert_eq!(lp.key_to_lane(i), i);
            assert_eq!(lp.scratch_index(i), None);
        }
    }

    #[test]
    fn keyboard24k_no_scratch() {
        let lp = LaneProperty::new(PlayMode::Keyboard24K);
        assert_eq!(lp.physical_key_count(), 26);
        assert_eq!(lp.lane_count(), 26);
        assert_eq!(lp.scratch_count(), 0);
    }

    #[test]
    fn lane_to_key_states_conversion() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        let lane_states = vec![true, false, true, false, false, false, false, true];
        let key_states = lp.lane_to_key_states(&lane_states);
        assert_eq!(key_states.len(), 9);
        assert!(key_states[0]); // lane 0
        assert!(!key_states[1]); // lane 1
        assert!(key_states[2]); // lane 2
        assert!(key_states[7]); // scratch key A
        assert!(key_states[8]); // scratch key B (replicated)
    }

    #[test]
    fn lane_to_key_times_conversion() {
        let lp = LaneProperty::new(PlayMode::Beat7K);
        let lane_times = vec![
            100,
            i64::MIN,
            200,
            i64::MIN,
            i64::MIN,
            i64::MIN,
            i64::MIN,
            300,
        ];
        let key_times = lp.lane_to_key_times(&lane_times);
        assert_eq!(key_times.len(), 9);
        assert_eq!(key_times[0], 100);
        assert_eq!(key_times[1], i64::MIN);
        assert_eq!(key_times[7], 300);
        assert_eq!(key_times[8], 300); // scratch key B gets same time
    }
}
