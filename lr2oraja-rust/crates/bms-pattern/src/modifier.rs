// Pattern modifier foundation types
//
// Defines RandomType, RandomUnit, AssistLevel, PatternModifyLog,
// the PatternModifier trait, and helper functions.

use bms_model::{BgNote, BmsModel, NoteType, PlayMode};
use serde::{Deserialize, Serialize};

/// Unit of randomization for each shuffle type.
///
/// Matches Java `RandomUnit` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomUnit {
    /// No modification
    None,
    /// Lane-level permutation (swap entire lanes)
    Lane,
    /// Note-level permutation (swap individual notes per timeline)
    Note,
    /// Player-level swap (flip/battle for DP)
    Player,
}

/// All pattern shuffle types.
///
/// Matches Java `Random` enum in the same ordinal order.
/// The index-based lookup arrays (`OPTION_GENERAL`, `OPTION_PMS`, etc.)
/// replicate the Java `getRandom(id, mode)` semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomType {
    Identity,
    Mirror,
    Random,
    Rotate,
    SRandom,
    Spiral,
    HRandom,
    AllScr,
    MirrorEx,
    RandomEx,
    RotateEx,
    SRandomEx,
    Cross,
    Converge,
    SRandomNoThreshold,
    RandomPlayable,
    SRandomPlayable,
    Flip,
    Battle,
}

impl RandomType {
    /// The randomization unit for this type.
    pub fn unit(self) -> RandomUnit {
        match self {
            Self::Identity => RandomUnit::None,
            Self::Mirror
            | Self::Random
            | Self::Rotate
            | Self::MirrorEx
            | Self::RandomEx
            | Self::RotateEx
            | Self::Cross
            | Self::RandomPlayable => RandomUnit::Lane,
            Self::SRandom
            | Self::Spiral
            | Self::HRandom
            | Self::AllScr
            | Self::SRandomEx
            | Self::SRandomNoThreshold
            | Self::SRandomPlayable
            | Self::Converge => RandomUnit::Note,
            Self::Flip | Self::Battle => RandomUnit::Player,
        }
    }

    /// Whether this type modifies scratch lanes too.
    pub fn is_scratch_lane_modify(self) -> bool {
        match self {
            Self::Identity
            | Self::Mirror
            | Self::Random
            | Self::Rotate
            | Self::SRandom
            | Self::Spiral
            | Self::HRandom
            | Self::Cross
            | Self::SRandomNoThreshold => false,
            Self::MirrorEx
            | Self::RandomEx
            | Self::RotateEx
            | Self::AllScr
            | Self::SRandomEx
            | Self::Converge
            | Self::RandomPlayable
            | Self::SRandomPlayable
            | Self::Flip
            | Self::Battle => true,
        }
    }
}

/// General mode options (Beat5K/7K/10K/14K/Keyboard).
///
/// Java: `Random.OPTION_GENERAL`
const OPTION_GENERAL: &[RandomType] = &[
    RandomType::Identity,
    RandomType::Mirror,
    RandomType::Random,
    RandomType::Rotate,
    RandomType::SRandom,
    RandomType::Spiral,
    RandomType::HRandom,
    RandomType::AllScr,
    RandomType::RandomEx,
    RandomType::SRandomEx,
];

/// PMS mode options (PopN5K/9K).
///
/// Java: `Random.OPTION_PMS`
const OPTION_PMS: &[RandomType] = &[
    RandomType::Identity,
    RandomType::Mirror,
    RandomType::Random,
    RandomType::Rotate,
    RandomType::SRandomNoThreshold,
    RandomType::Spiral,
    RandomType::HRandom,
    RandomType::Converge,
    RandomType::RandomPlayable,
    RandomType::SRandomPlayable,
];

/// Look up a RandomType by numeric ID and mode.
///
/// Matches Java `Random.getRandom(int id, Mode mode)`.
pub fn get_random(id: usize, mode: PlayMode) -> RandomType {
    let options = match mode {
        PlayMode::PopN5K | PlayMode::PopN9K => OPTION_PMS,
        _ => OPTION_GENERAL,
    };
    options.get(id).copied().unwrap_or(RandomType::Identity)
}

/// Assist level for the pattern modifier.
///
/// Matches Java `PatternModifier.AssistLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AssistLevel {
    #[default]
    None,
    LightAssist,
    Assist,
}

/// Log of a pattern modification (for replay/export).
///
/// Matches Java `PatternModifyLog`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternModifyLog {
    /// Measure section number
    pub section: f64,
    /// Lane mapping: modify[i] = source lane for destination lane i
    pub modify: Vec<usize>,
}

/// Trait for pattern modifiers.
///
/// Matches Java `PatternModifier.modify(BMSModel)`.
pub trait PatternModifier {
    /// Apply this pattern modification to the model (in-place).
    fn modify(&mut self, model: &mut BmsModel);

    /// The assist level of this modifier.
    fn assist_level(&self) -> AssistLevel {
        AssistLevel::None
    }
}

/// Returns the lane indices that should be modified for the given mode,
/// player number, and whether scratch lanes are included.
///
/// Matches Java `PatternModifier.getKeys(Mode, int, boolean)`:
/// ```java
/// final int startkey = mode.key * player / mode.player;
/// return IntStream.range(startkey, startkey + mode.key / mode.player)
///     .filter(i -> containsScratch || !mode.isScratchKey(i))
///     .toArray();
/// ```
pub fn get_keys(mode: PlayMode, player: usize, contains_scratch: bool) -> Vec<usize> {
    let total_keys = mode.key_count();
    let player_count = mode.player_count();

    if player >= player_count {
        return Vec::new();
    }

    let start_key = total_keys * player / player_count;
    let keys_per_player = total_keys / player_count;

    (start_key..start_key + keys_per_player)
        .filter(|&i| contains_scratch || !mode.is_scratch_key(i))
        .collect()
}

/// Move a note on the given lane at the given time to background.
///
/// For normal/invisible notes: moves from `model.notes` to `model.bg_notes`.
/// For LN notes: moves both start and end notes.
/// For mine notes: removes without adding to background.
///
/// Matches Java `PatternModifier.moveToBackground(TimeLine[], TimeLine, int)`.
pub fn move_to_background(model: &mut BmsModel, lane: usize, time_us: i64) {
    let mut indices_to_remove: Vec<usize> = Vec::new();

    // Find the note at the given lane and time
    for (i, note) in model.notes.iter().enumerate() {
        if note.lane == lane && note.time_us == time_us {
            indices_to_remove.push(i);

            // If LN, also remove the paired note
            if note.is_long_note() && note.pair_index != usize::MAX {
                let pair = note.pair_index;
                if !indices_to_remove.contains(&pair) {
                    indices_to_remove.push(pair);
                }
            }
            break;
        }
    }

    indices_to_remove.sort_unstable();
    indices_to_remove.dedup();

    // Move non-mine notes to bg
    for &i in &indices_to_remove {
        let note = &model.notes[i];
        if note.note_type != NoteType::Mine {
            model.bg_notes.push(BgNote {
                wav_id: note.wav_id,
                time_us: note.time_us,
                micro_starttime: note.micro_starttime,
                micro_duration: note.micro_duration,
            });
        }
    }

    // Remove in reverse order
    for &i in indices_to_remove.iter().rev() {
        model.notes.remove(i);
    }
}

/// Rebuild LN pair indices after notes have been added/removed.
pub fn rebuild_pair_indices(notes: &mut [bms_model::Note]) {
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

    #[test]
    fn test_random_type_unit() {
        assert_eq!(RandomType::Identity.unit(), RandomUnit::None);
        assert_eq!(RandomType::Mirror.unit(), RandomUnit::Lane);
        assert_eq!(RandomType::SRandom.unit(), RandomUnit::Note);
        assert_eq!(RandomType::Flip.unit(), RandomUnit::Player);
        assert_eq!(RandomType::Battle.unit(), RandomUnit::Player);
        assert_eq!(RandomType::Cross.unit(), RandomUnit::Lane);
        assert_eq!(RandomType::Converge.unit(), RandomUnit::Note);
        assert_eq!(RandomType::RandomPlayable.unit(), RandomUnit::Lane);
        assert_eq!(RandomType::SRandomPlayable.unit(), RandomUnit::Note);
    }

    #[test]
    fn test_random_type_scratch_modify() {
        assert!(!RandomType::Identity.is_scratch_lane_modify());
        assert!(!RandomType::Mirror.is_scratch_lane_modify());
        assert!(!RandomType::Random.is_scratch_lane_modify());
        assert!(!RandomType::Rotate.is_scratch_lane_modify());
        assert!(!RandomType::SRandom.is_scratch_lane_modify());
        assert!(!RandomType::Cross.is_scratch_lane_modify());

        assert!(RandomType::MirrorEx.is_scratch_lane_modify());
        assert!(RandomType::RandomEx.is_scratch_lane_modify());
        assert!(RandomType::RotateEx.is_scratch_lane_modify());
        assert!(RandomType::AllScr.is_scratch_lane_modify());
        assert!(RandomType::SRandomEx.is_scratch_lane_modify());
        assert!(RandomType::Flip.is_scratch_lane_modify());
        assert!(RandomType::Battle.is_scratch_lane_modify());
    }

    #[test]
    fn test_get_random_general() {
        assert_eq!(get_random(0, PlayMode::Beat7K), RandomType::Identity);
        assert_eq!(get_random(1, PlayMode::Beat7K), RandomType::Mirror);
        assert_eq!(get_random(2, PlayMode::Beat7K), RandomType::Random);
        assert_eq!(get_random(3, PlayMode::Beat7K), RandomType::Rotate);
        assert_eq!(get_random(4, PlayMode::Beat7K), RandomType::SRandom);
        assert_eq!(get_random(5, PlayMode::Beat7K), RandomType::Spiral);
        assert_eq!(get_random(6, PlayMode::Beat7K), RandomType::HRandom);
        assert_eq!(get_random(7, PlayMode::Beat7K), RandomType::AllScr);
        assert_eq!(get_random(8, PlayMode::Beat7K), RandomType::RandomEx);
        assert_eq!(get_random(9, PlayMode::Beat7K), RandomType::SRandomEx);
        // Out of bounds -> Identity
        assert_eq!(get_random(10, PlayMode::Beat7K), RandomType::Identity);
        assert_eq!(get_random(999, PlayMode::Beat7K), RandomType::Identity);
    }

    #[test]
    fn test_get_random_pms() {
        assert_eq!(get_random(0, PlayMode::PopN9K), RandomType::Identity);
        assert_eq!(get_random(1, PlayMode::PopN9K), RandomType::Mirror);
        assert_eq!(get_random(2, PlayMode::PopN9K), RandomType::Random);
        assert_eq!(get_random(3, PlayMode::PopN9K), RandomType::Rotate);
        assert_eq!(
            get_random(4, PlayMode::PopN9K),
            RandomType::SRandomNoThreshold
        );
        assert_eq!(get_random(5, PlayMode::PopN9K), RandomType::Spiral);
        assert_eq!(get_random(6, PlayMode::PopN9K), RandomType::HRandom);
        assert_eq!(get_random(7, PlayMode::PopN9K), RandomType::Converge);
        assert_eq!(get_random(8, PlayMode::PopN9K), RandomType::RandomPlayable);
        assert_eq!(get_random(9, PlayMode::PopN9K), RandomType::SRandomPlayable);
    }

    #[test]
    fn test_get_keys_beat7k_no_scratch() {
        // Beat7K: 8 keys (0-7), scratch at 7, player 0
        // Should return [0,1,2,3,4,5,6] (without scratch lane 7)
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_get_keys_beat7k_with_scratch() {
        // Beat7K: 8 keys (0-7), scratch at 7, player 0
        // Should return [0,1,2,3,4,5,6,7] (with scratch lane 7)
        let keys = get_keys(PlayMode::Beat7K, 0, true);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_get_keys_beat5k_no_scratch() {
        // Beat5K: 6 keys (0-5), scratch at 5
        let keys = get_keys(PlayMode::Beat5K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_get_keys_beat5k_with_scratch() {
        let keys = get_keys(PlayMode::Beat5K, 0, true);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_get_keys_beat14k_player0_no_scratch() {
        // Beat14K: 16 keys (0-15), scratch at 7,15, 2 players
        // Player 0: keys 0..8, without scratch (lane 7)
        let keys = get_keys(PlayMode::Beat14K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_get_keys_beat14k_player1_no_scratch() {
        // Player 1: keys 8..16, without scratch (lane 15)
        let keys = get_keys(PlayMode::Beat14K, 1, false);
        assert_eq!(keys, vec![8, 9, 10, 11, 12, 13, 14]);
    }

    #[test]
    fn test_get_keys_beat14k_player1_with_scratch() {
        let keys = get_keys(PlayMode::Beat14K, 1, true);
        assert_eq!(keys, vec![8, 9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_get_keys_popn9k() {
        // PopN9K: 9 keys (0-8), no scratch
        let keys = get_keys(PlayMode::PopN9K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        // With scratch flag has no effect (no scratch keys in PMS)
        let keys_with = get_keys(PlayMode::PopN9K, 0, true);
        assert_eq!(keys_with, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_get_keys_invalid_player() {
        // Player >= player_count should return empty
        let keys = get_keys(PlayMode::Beat7K, 1, false);
        assert!(keys.is_empty());

        let keys = get_keys(PlayMode::Beat7K, 2, true);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_get_keys_beat10k() {
        // Beat10K: 12 keys (0-11), scratch at 5,11, 2 players
        // Player 0: keys 0..6, without scratch (5)
        let keys = get_keys(PlayMode::Beat10K, 0, false);
        assert_eq!(keys, vec![0, 1, 2, 3, 4]);

        // Player 1: keys 6..12, without scratch (11)
        let keys = get_keys(PlayMode::Beat10K, 1, false);
        assert_eq!(keys, vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_assist_level_default() {
        assert_eq!(AssistLevel::default(), AssistLevel::None);
    }
}
