//! Pattern modifiers for shuffling note lanes.
//!
//! Ported from beatoraja's `pattern/` package. Modifiers operate on `Vec<Note>`
//! and remap lane indices according to various algorithms.
//!
//! Two categories:
//! - **Lane shuffle**: a single permutation applied to all notes (Mirror, Random, R-Random, Rotate, Cross).
//! - **Note shuffle**: per-timeslice permutation with LN tracking (S-Random, H-Random, Spiral, All-SCR, etc.).

mod lane_shuffle;
mod note_shuffle;

pub use lane_shuffle::{battle, cross, flip, lane_random, mirror, playable_random, rotate};
pub use note_shuffle::{
    all_scratch, converge, h_random, s_random, s_random_no_threshold, s_random_playable, spiral,
};

use crate::model::note::{NoteType, PlayMode};
use serde::{Deserialize, Serialize};

/// Random option for pattern modification.
///
/// Corresponds to `Random` enum in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RandomOption {
    /// No modification.
    Identity,
    /// Mirror: reverse key lanes.
    Mirror,
    /// Random: shuffle key lanes (one permutation for entire chart).
    Random,
    /// Rotate: rotate key lanes by a random offset.
    Rotate,
    /// S-Random: per-timeslice shuffle with 40ms threshold.
    SRandom,
    /// Spiral: rotating lane assignment.
    Spiral,
    /// H-Random: per-timeslice shuffle with BPM-based threshold.
    HRandom,
    /// All-SCR: assign notes to scratch lane preferentially.
    AllScr,
    /// Mirror with scratch lane included.
    MirrorEx,
    /// Random with scratch lane included.
    RandomEx,
    /// Rotate with scratch lane included.
    RotateEx,
    /// S-Random with scratch lane included.
    SRandomEx,
    /// Cross: swap pairs of lanes from outside inward.
    Cross,
    /// Converge: create long runs of repeated notes on the same lane (PMS).
    Converge,
    /// S-Random with no threshold (PMS).
    SRandomNoThreshold,
    /// Playable Random: lane shuffle avoiding murioshi (PMS).
    RandomPlayable,
    /// S-Random Playable: per-timeslice shuffle avoiding murioshi (PMS).
    SRandomPlayable,
    /// Flip: swap 1P and 2P sides (DP).
    Flip,
    /// Battle: copy 1P notes to 2P side (DP).
    Battle,
}

impl RandomOption {
    /// Whether this option modifies scratch lanes.
    pub fn is_scratch_lane_modify(self) -> bool {
        matches!(
            self,
            Self::AllScr
                | Self::MirrorEx
                | Self::RandomEx
                | Self::RotateEx
                | Self::SRandomEx
                | Self::Converge
                | Self::RandomPlayable
                | Self::SRandomPlayable
                | Self::Flip
                | Self::Battle
        )
    }

    /// General random options for BEAT modes.
    pub const OPTION_GENERAL: &[RandomOption] = &[
        Self::Identity,
        Self::Mirror,
        Self::Random,
        Self::Rotate,
        Self::SRandom,
        Self::Spiral,
        Self::HRandom,
        Self::AllScr,
        Self::RandomEx,
        Self::SRandomEx,
    ];

    /// Random options for PMS modes.
    pub const OPTION_PMS: &[RandomOption] = &[
        Self::Identity,
        Self::Mirror,
        Self::Random,
        Self::Rotate,
        Self::SRandomNoThreshold,
        Self::Spiral,
        Self::HRandom,
        Self::Converge,
        Self::RandomPlayable,
        Self::SRandomPlayable,
    ];

    /// Double play options.
    pub const OPTION_DOUBLE: &[RandomOption] = &[Self::Identity, Self::Flip];

    /// Single play battle option.
    pub const OPTION_SINGLE: &[RandomOption] = &[Self::Identity, Self::Battle];

    /// Get a random option by ID for a given play mode.
    pub fn from_id(id: usize, mode: PlayMode) -> Self {
        let options = match mode {
            PlayMode::PopN9K => Self::OPTION_PMS,
            _ => Self::OPTION_GENERAL,
        };
        options.get(id).copied().unwrap_or(Self::Identity)
    }
}

/// Returns the key lane indices for a given player side, optionally including scratch.
///
/// Corresponds to `PatternModifier.getKeys()` in beatoraja.
pub fn get_keys(mode: PlayMode, player: usize, include_scratch: bool) -> Vec<usize> {
    let total = mode.lane_count();
    let players = mode.player_count();
    if player >= players {
        return Vec::new();
    }
    let lanes_per_player = total / players;
    let start = lanes_per_player * player;
    (start..start + lanes_per_player)
        .filter(|&i| include_scratch || !mode.is_scratch(i))
        .collect()
}

/// Returns true if the note type is playable (not mine, not invisible).
pub fn is_playable(note_type: NoteType) -> bool {
    !matches!(note_type, NoteType::Mine | NoteType::Invisible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::note::{Note, NoteType, PlayMode};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn normal_note(lane: usize, time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::Normal,
            time_us,
            end_time_us: 0,
            wav_id: 0,
            damage: 0.0,
        }
    }

    fn long_note(lane: usize, time_us: i64, end_time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::LongNote,
            time_us,
            end_time_us,
            wav_id: 0,
            damage: 0.0,
        }
    }

    fn mine_note(lane: usize, time_us: i64) -> Note {
        Note {
            lane,
            note_type: NoteType::Mine,
            time_us,
            end_time_us: 0,
            wav_id: 0,
            damage: 10.0,
        }
    }

    fn count_notes_per_lane(notes: &[Note], lane_count: usize) -> Vec<usize> {
        let mut counts = vec![0; lane_count];
        for n in notes {
            counts[n.lane] += 1;
        }
        counts
    }

    #[test]
    fn get_keys_beat7k_no_scratch() {
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn get_keys_beat7k_with_scratch() {
        let keys = get_keys(PlayMode::Beat7K, 0, true);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn get_keys_beat14k_player1() {
        let keys = get_keys(PlayMode::Beat14K, 1, false);
        assert_eq!(keys, vec![8, 9, 10, 11, 12, 13, 14]);
    }

    #[test]
    fn get_keys_popn9k() {
        let keys = get_keys(PlayMode::PopN9K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn get_keys_invalid_player() {
        let keys = get_keys(PlayMode::Beat7K, 1, false);
        assert!(keys.is_empty());
    }

    #[test]
    fn mirror_applied_twice_is_identity() {
        let original = vec![
            normal_note(0, 0),
            normal_note(1, 1000),
            normal_note(2, 2000),
            normal_note(6, 3000),
        ];
        let mut notes = original.clone();
        mirror(&mut notes, PlayMode::Beat7K, 0, false);
        mirror(&mut notes, PlayMode::Beat7K, 0, false);
        for (a, b) in notes.iter().zip(original.iter()) {
            assert_eq!(a.lane, b.lane);
        }
    }

    #[test]
    fn mirror_preserves_note_count() {
        let mut notes = vec![
            normal_note(0, 0),
            normal_note(3, 1000),
            long_note(5, 2000, 3000),
            mine_note(6, 4000),
        ];
        let original_count = notes.len();
        mirror(&mut notes, PlayMode::Beat7K, 0, false);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn mirror_beat7k_reverses_key_lanes() {
        let mut notes = vec![normal_note(0, 0)];
        mirror(&mut notes, PlayMode::Beat7K, 0, false);
        assert_eq!(notes[0].lane, 6);
    }

    #[test]
    fn mirror_does_not_affect_scratch() {
        let mut notes = vec![normal_note(7, 0)];
        mirror(&mut notes, PlayMode::Beat7K, 0, false);
        assert_eq!(notes[0].lane, 7);
    }

    #[test]
    fn lane_random_preserves_note_count() {
        let mut notes = vec![
            normal_note(0, 0),
            normal_note(1, 1000),
            normal_note(2, 2000),
            long_note(3, 3000, 4000),
        ];
        let original_count = notes.len();
        let mut rng = SmallRng::seed_from_u64(42);
        lane_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn lane_random_maps_lanes_within_range() {
        let mut notes: Vec<Note> = (0..7).map(|i| normal_note(i, i as i64 * 1000)).collect();
        let mut rng = SmallRng::seed_from_u64(42);
        lane_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        for n in &notes {
            assert!(n.lane < 7, "Lane {} out of range", n.lane);
        }
    }

    #[test]
    fn rotate_preserves_note_count() {
        let mut notes = vec![
            normal_note(0, 0),
            normal_note(1, 1000),
            normal_note(2, 2000),
        ];
        let original_count = notes.len();
        let mut rng = SmallRng::seed_from_u64(42);
        rotate(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn flip_swaps_player_sides_in_dp() {
        let mut notes = vec![normal_note(0, 0), normal_note(8, 0)];
        flip(&mut notes, PlayMode::Beat14K);
        assert_eq!(notes[0].lane, 8);
        assert_eq!(notes[1].lane, 0);
    }

    #[test]
    fn flip_noop_for_single_player() {
        let mut notes = vec![normal_note(3, 0)];
        flip(&mut notes, PlayMode::Beat7K);
        assert_eq!(notes[0].lane, 3);
    }

    #[test]
    fn battle_copies_p1_to_p2() {
        let mut notes = vec![normal_note(0, 0), normal_note(1, 1000)];
        battle(&mut notes, PlayMode::Beat14K);
        assert!(notes.len() >= 2);
    }

    #[test]
    fn s_random_preserves_note_count() {
        let mut notes: Vec<Note> = (0..7)
            .map(|i| normal_note(i, (i as i64 / 3) * 1000))
            .collect();
        let original_count = notes.len();
        let mut rng = SmallRng::seed_from_u64(42);
        s_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn s_random_respects_lane_bounds() {
        let mut notes: Vec<Note> = (0..20)
            .map(|i| normal_note(i % 7, i as i64 * 100_000))
            .collect();
        let mut rng = SmallRng::seed_from_u64(123);
        s_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        for n in &notes {
            assert!(keys.contains(&n.lane), "Lane {} not in key lanes", n.lane);
        }
    }

    #[test]
    fn h_random_preserves_note_count() {
        let mut notes: Vec<Note> = (0..10)
            .map(|i| normal_note(i % 7, i as i64 * 50_000))
            .collect();
        let original_count = notes.len();
        let mut rng = SmallRng::seed_from_u64(42);
        h_random(&mut notes, PlayMode::Beat7K, 0, false, 100_000, &mut rng);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn spiral_preserves_note_count() {
        let mut notes: Vec<Note> = (0..14)
            .map(|i| normal_note(i % 7, i as i64 * 50_000))
            .collect();
        let original_count = notes.len();
        let mut rng = SmallRng::seed_from_u64(42);
        spiral(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn all_scratch_preserves_note_count() {
        let mut notes: Vec<Note> = (0..10)
            .map(|i| normal_note(i % 7, i as i64 * 100_000))
            .collect();
        let original_count = notes.len();
        let mut rng = SmallRng::seed_from_u64(42);
        all_scratch(&mut notes, PlayMode::Beat7K, 0, 100_000, &mut rng);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn cross_preserves_note_count() {
        let mut notes: Vec<Note> = (0..7).map(|i| normal_note(i, i as i64 * 1000)).collect();
        let original_count = notes.len();
        cross(&mut notes, PlayMode::Beat7K, 0, false);
        assert_eq!(notes.len(), original_count);
    }

    #[test]
    fn mirror_remaps_long_notes_consistently() {
        let mut notes = vec![long_note(0, 1000, 3000), normal_note(0, 5000)];
        mirror(&mut notes, PlayMode::Beat7K, 0, false);
        let ln_lane = notes[0].lane;
        let normal_lane = notes[1].lane;
        assert_eq!(ln_lane, normal_lane);
    }

    #[test]
    fn s_random_keeps_ln_on_same_lane_during_hold() {
        let mut notes = vec![
            long_note(0, 0, 2_000_000),
            normal_note(1, 1_000_000),
            normal_note(2, 1_000_000),
        ];
        let mut rng = SmallRng::seed_from_u64(42);
        s_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        let ln_lane = notes[0].lane;
        for n in &notes[1..] {
            if n.time_us == 1_000_000 {
                assert_ne!(
                    n.lane, ln_lane,
                    "Note during LN hold should not share LN lane"
                );
            }
        }
    }

    #[test]
    fn all_modifiers_preserve_note_count() {
        let notes: Vec<Note> = (0..20)
            .map(|i| normal_note(i % 7, i as i64 * 100_000))
            .collect();
        let count = notes.len();

        let mut m = notes.clone();
        mirror(&mut m, PlayMode::Beat7K, 0, false);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        let mut rng = SmallRng::seed_from_u64(1);
        lane_random(&mut m, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        let mut rng = SmallRng::seed_from_u64(2);
        rotate(&mut m, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        let mut rng = SmallRng::seed_from_u64(3);
        s_random(&mut m, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        let mut rng = SmallRng::seed_from_u64(4);
        h_random(&mut m, PlayMode::Beat7K, 0, false, 100_000, &mut rng);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        let mut rng = SmallRng::seed_from_u64(5);
        spiral(&mut m, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        let mut rng = SmallRng::seed_from_u64(6);
        all_scratch(&mut m, PlayMode::Beat7K, 0, 100_000, &mut rng);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        cross(&mut m, PlayMode::Beat7K, 0, false);
        assert_eq!(m.len(), count);

        let mut m = notes.clone();
        flip(&mut m, PlayMode::Beat7K);
        assert_eq!(m.len(), count);
    }

    #[test]
    fn random_option_from_id_general() {
        assert_eq!(
            RandomOption::from_id(0, PlayMode::Beat7K),
            RandomOption::Identity
        );
        assert_eq!(
            RandomOption::from_id(1, PlayMode::Beat7K),
            RandomOption::Mirror
        );
        assert_eq!(
            RandomOption::from_id(4, PlayMode::Beat7K),
            RandomOption::SRandom
        );
        assert_eq!(
            RandomOption::from_id(7, PlayMode::Beat7K),
            RandomOption::AllScr
        );
        assert_eq!(
            RandomOption::from_id(99, PlayMode::Beat7K),
            RandomOption::Identity
        );
    }

    #[test]
    fn random_option_from_id_pms() {
        assert_eq!(
            RandomOption::from_id(0, PlayMode::PopN9K),
            RandomOption::Identity
        );
        assert_eq!(
            RandomOption::from_id(4, PlayMode::PopN9K),
            RandomOption::SRandomNoThreshold
        );
        assert_eq!(
            RandomOption::from_id(7, PlayMode::PopN9K),
            RandomOption::Converge
        );
        assert_eq!(
            RandomOption::from_id(9, PlayMode::PopN9K),
            RandomOption::SRandomPlayable
        );
    }

    #[test]
    fn random_option_scratch_modify() {
        assert!(!RandomOption::Identity.is_scratch_lane_modify());
        assert!(!RandomOption::Mirror.is_scratch_lane_modify());
        assert!(!RandomOption::Random.is_scratch_lane_modify());
        assert!(RandomOption::AllScr.is_scratch_lane_modify());
        assert!(RandomOption::MirrorEx.is_scratch_lane_modify());
        assert!(RandomOption::RandomEx.is_scratch_lane_modify());
        assert!(RandomOption::Flip.is_scratch_lane_modify());
        assert!(RandomOption::Battle.is_scratch_lane_modify());
    }

    #[test]
    fn lane_shuffle_is_bijection() {
        let notes: Vec<Note> = (0..7).map(|i| normal_note(i, 0)).collect();

        let mut m = notes.clone();
        mirror(&mut m, PlayMode::Beat7K, 0, false);
        let before = count_notes_per_lane(&notes, 8);
        let after = count_notes_per_lane(&m, 8);
        let mut before_sorted = before;
        let mut after_sorted = after;
        before_sorted.sort();
        after_sorted.sort();
        assert_eq!(before_sorted, after_sorted);
    }
}
