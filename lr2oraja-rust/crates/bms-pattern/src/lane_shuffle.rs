// Lane shuffle modifiers (7 types)
//
// Ported from Java: LaneShuffleModifier.java
//
// In the Java codebase, each modifier produces a `random[]` mapping array
// where `random[i]` = source lane for destination lane i, then iterates
// timelines to move notes. In Rust, notes live in a flat `Vec<Note>` with
// `lane: usize`, so we rewrite lane indices directly.

use bms_model::{BmsModel, NoteType};

use crate::java_random::JavaRandom;
use crate::modifier::{AssistLevel, PatternModifier, get_keys};

// ---------------------------------------------------------------------------
// Common helpers
// ---------------------------------------------------------------------------

/// Build an identity mapping `[0, 1, 2, ..., key_count-1]`.
fn make_identity_mapping(key_count: usize) -> Vec<usize> {
    (0..key_count).collect()
}

/// Apply a bijective lane mapping to all notes in the model.
///
/// `mapping[old_lane] = new_lane` for lanes in range; unmapped lanes are
/// left unchanged.  `pair_index` values are indices into the notes vec
/// and remain valid because the vec itself is not reordered.
fn apply_lane_mapping(model: &mut BmsModel, mapping: &[usize]) {
    for note in &mut model.notes {
        if note.lane < mapping.len() {
            note.lane = mapping[note.lane];
        }
    }
}

// ---------------------------------------------------------------------------
// 1. Mirror
// ---------------------------------------------------------------------------

/// Reverses the key order (excluding or including scratch depending on mode).
///
/// Java: `LaneMirrorShuffleModifier`
pub struct LaneMirrorShuffle {
    player: usize,
    contains_scratch: bool,
}

impl LaneMirrorShuffle {
    pub fn new(player: usize, contains_scratch: bool) -> Self {
        Self {
            player,
            contains_scratch,
        }
    }

    /// Generate the mirror mapping array.
    pub fn make_random(&self, keys: &[usize], key_count: usize) -> Vec<usize> {
        let mut result = make_identity_mapping(key_count);
        for (i, &k) in keys.iter().enumerate() {
            result[k] = keys[keys.len() - 1 - i];
        }
        result
    }
}

impl PatternModifier for LaneMirrorShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        let keys = get_keys(model.mode, self.player, self.contains_scratch);
        if keys.is_empty() {
            return;
        }
        let mapping = self.make_random(&keys, model.mode.key_count());
        apply_lane_mapping(model, &mapping);
    }

    fn assist_level(&self) -> AssistLevel {
        if self.contains_scratch {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Rotate
// ---------------------------------------------------------------------------

/// Random circular rotation of key lanes.
///
/// Java: `LaneRotateShuffleModifier`
pub struct LaneRotateShuffle {
    player: usize,
    contains_scratch: bool,
    seed: i64,
}

impl LaneRotateShuffle {
    pub fn new(player: usize, contains_scratch: bool, seed: i64) -> Self {
        Self {
            player,
            contains_scratch,
            seed,
        }
    }

    /// Generate the rotate mapping array.
    pub fn make_random(&self, keys: &[usize], key_count: usize) -> Vec<usize> {
        let mut rng = JavaRandom::new(self.seed);
        let inc = rng.next_int(2) == 1;
        let start = rng.next_int(keys.len() as i32 - 1) as usize + if inc { 1 } else { 0 };

        let mut result = make_identity_mapping(key_count);
        let mut rlane = start;
        for &k in keys {
            result[k] = keys[rlane];
            rlane = if inc {
                (rlane + 1) % keys.len()
            } else {
                (rlane + keys.len() - 1) % keys.len()
            };
        }
        result
    }
}

impl PatternModifier for LaneRotateShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        let keys = get_keys(model.mode, self.player, self.contains_scratch);
        if keys.is_empty() {
            return;
        }
        let mapping = self.make_random(&keys, model.mode.key_count());
        apply_lane_mapping(model, &mapping);
    }

    fn assist_level(&self) -> AssistLevel {
        if self.contains_scratch {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Random
// ---------------------------------------------------------------------------

/// Fisher–Yates-like random permutation of key lanes.
///
/// Java: `LaneRandomShuffleModifier` (uses `IntArray.removeIndex`)
pub struct LaneRandomShuffle {
    player: usize,
    contains_scratch: bool,
    seed: i64,
}

impl LaneRandomShuffle {
    pub fn new(player: usize, contains_scratch: bool, seed: i64) -> Self {
        Self {
            player,
            contains_scratch,
            seed,
        }
    }

    /// Generate the random mapping array.
    pub fn make_random(&self, keys: &[usize], key_count: usize) -> Vec<usize> {
        let mut rng = JavaRandom::new(self.seed);
        let mut remaining = keys.to_vec();
        let mut result = make_identity_mapping(key_count);
        for &k in keys {
            let r = rng.next_int(remaining.len() as i32) as usize;
            result[k] = remaining.remove(r);
        }
        result
    }
}

impl PatternModifier for LaneRandomShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        let keys = get_keys(model.mode, self.player, self.contains_scratch);
        if keys.is_empty() {
            return;
        }
        let mapping = self.make_random(&keys, model.mode.key_count());
        apply_lane_mapping(model, &mapping);
    }

    fn assist_level(&self) -> AssistLevel {
        if self.contains_scratch {
            AssistLevel::LightAssist
        } else {
            AssistLevel::None
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Cross
// ---------------------------------------------------------------------------

/// Swaps adjacent pairs from both ends moving inward.
///
/// Java: `LaneCrossShuffleModifier`
pub struct LaneCrossShuffle {
    player: usize,
    contains_scratch: bool,
}

impl LaneCrossShuffle {
    pub fn new(player: usize, contains_scratch: bool) -> Self {
        Self {
            player,
            contains_scratch,
        }
    }

    /// Generate the cross mapping array.
    pub fn make_random(&self, keys: &[usize], key_count: usize) -> Vec<usize> {
        let mut result = make_identity_mapping(key_count);
        let len = keys.len();
        let mut i = 0;
        while i < len / 2 - 1 {
            // Front pair swap
            result[keys[i]] = keys[i + 1];
            result[keys[i + 1]] = keys[i];
            // Back pair swap
            result[keys[len - i - 1]] = keys[len - i - 2];
            result[keys[len - i - 2]] = keys[len - i - 1];
            i += 2;
        }
        result
    }
}

impl PatternModifier for LaneCrossShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        let keys = get_keys(model.mode, self.player, self.contains_scratch);
        if keys.is_empty() {
            return;
        }
        let mapping = self.make_random(&keys, model.mode.key_count());
        apply_lane_mapping(model, &mapping);
    }

    fn assist_level(&self) -> AssistLevel {
        AssistLevel::LightAssist
    }
}

// ---------------------------------------------------------------------------
// 5. Flip (Player Flip)
// ---------------------------------------------------------------------------

/// Swaps 1P and 2P sides in double-play modes.
///
/// Java: `PlayerFlipModifier`
pub struct PlayerFlipShuffle;

impl PlayerFlipShuffle {
    pub fn new() -> Self {
        Self
    }

    /// Generate the flip mapping array.
    pub fn make_random(&self, key_count: usize, player_count: usize) -> Vec<usize> {
        let mut result = make_identity_mapping(key_count);
        if player_count == 2 {
            for (i, item) in result.iter_mut().enumerate().take(key_count) {
                *item = (i + key_count / 2) % key_count;
            }
        }
        result
    }
}

impl Default for PlayerFlipShuffle {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternModifier for PlayerFlipShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        let mapping = self.make_random(model.mode.key_count(), model.mode.player_count());
        apply_lane_mapping(model, &mapping);
    }

    fn assist_level(&self) -> AssistLevel {
        AssistLevel::None
    }
}

// ---------------------------------------------------------------------------
// 6. Battle
// ---------------------------------------------------------------------------

/// Copies 1P notes to 2P in double-play modes.
///
/// Java: `PlayerBattleModifier`
///
/// In the Java implementation, `makeRandom` returns `[keys, keys]` (doubled),
/// and the `modify()` loop's clone logic overwrites 2P lanes with 1P clones.
///
/// In Rust, we:
/// 1. Remove all 2P notes (lane >= half).
/// 2. Clone each 1P note with `lane += half`.
/// 3. Append clones, fixing `pair_index` for LN pairs.
pub struct PlayerBattleShuffle;

impl PlayerBattleShuffle {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlayerBattleShuffle {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternModifier for PlayerBattleShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        if model.mode.player_count() != 2 {
            return;
        }

        let half = model.mode.key_count() / 2;

        // Remove 2P notes
        model.notes.retain(|n| n.lane < half);

        // Build index mapping: old_index -> new_index after retain
        // (retain preserves order, so we just need a contiguous re-index
        //  for pair_index references.)
        // Actually, retain doesn't change indices within the kept slice,
        // but pair_index values may point to removed notes. We need to
        // rebuild pair_index for 1P notes first.

        // Rebuild pair_index for remaining 1P notes.
        // LN start notes reference their end note by pair_index.
        // After removing 2P notes, the indices shifted.
        // We create old_to_new mapping.
        // But since we don't have the old indices after retain, let's
        // rebuild pairs from scratch.
        rebuild_ln_pairs(&mut model.notes);

        let original_len = model.notes.len();

        // Clone 1P notes to 2P
        //
        // Java's Battle clone logic for LN end notes uses the paired
        // start note's wav_id (via pair traversal). We replicate this:
        // LN end clones get wav_id from the corresponding start note.
        let clones: Vec<_> = model.notes[..original_len]
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let mut c = n.clone();
                c.lane += half;
                c.pair_index = usize::MAX; // will be fixed below
                // Java compatibility: LN end clones inherit start note's wav_id
                if n.is_long_note() && n.end_time_us == 0 {
                    // This is an LN end note — find its paired start note
                    let pair_idx = model.notes[i].pair_index;
                    if pair_idx != usize::MAX && pair_idx < original_len {
                        c.wav_id = model.notes[pair_idx].wav_id;
                    }
                }
                c
            })
            .collect();
        model.notes.extend(clones);

        // Fix pair_index for cloned 2P notes:
        // If 1P note at index `i` has pair_index `j` (pointing to another 1P note),
        // then the cloned note at index `i + original_len` should point to `j + original_len`.
        for i in 0..original_len {
            let pair = model.notes[i].pair_index;
            if pair != usize::MAX && pair < original_len {
                model.notes[i + original_len].pair_index = pair + original_len;
            }
        }
    }

    fn assist_level(&self) -> AssistLevel {
        AssistLevel::Assist
    }
}

/// Rebuild LN pair indices from scratch by matching start/end notes on the
/// same lane by time order.
fn rebuild_ln_pairs(notes: &mut [bms_model::Note]) {
    // Reset all pair indices
    for note in notes.iter_mut() {
        note.pair_index = usize::MAX;
    }

    // Group LN notes by lane, find start/end pairs
    // LN start: is_long_note() && end_time_us > 0
    // LN end: is_long_note() && end_time_us == 0 && pair_index should link to start
    //
    // Actually in bms-model, LN notes are stored as a single note with
    // start time, end time, and pair_index pointing to the "end" note.
    // But looking at the Note struct, each LN has time_us (start) and
    // end_time_us (end), plus pair_index.
    //
    // After checking the model: LN notes in bms-model are stored as
    // individual notes. The pair_index on a start note points to its
    // corresponding end note in the vec. Let's rebuild by matching
    // notes that share lane + LN type + timing.

    // Collect LN start notes (those with end_time_us > 0)
    let starts: Vec<usize> = notes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.is_long_note() && n.end_time_us > 0)
        .map(|(i, _)| i)
        .collect();

    // For each start, find its end note (same lane, same note_type,
    // time_us == start.end_time_us, end_time_us == 0)
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

// ---------------------------------------------------------------------------
// 7. Playable Random (PMS / PopN9K only)
// ---------------------------------------------------------------------------

/// Exhaustive 9! permutation search to avoid murioshi (impossible) patterns.
///
/// Java: `LanePlayableRandomShuffleModifier`
///
/// Algorithm:
/// 1. Scan the chart for 3+ note chords (bitmask).
/// 2. If any 7+ note chord exists, fall back to normal or mirror.
/// 3. Generate all 9! permutations via Heap's algorithm.
/// 4. For each permutation, check if any chord maps to a murioshi pattern.
/// 5. Remove the mirror permutation `[8,7,6,5,4,3,2,1,0]`.
/// 6. Pick a random permutation from valid candidates.
///
/// Note: Java uses `Math.random()` (unseeded). We use `JavaRandom` with a
/// seed for reproducibility.
pub struct LanePlayableRandomShuffle {
    player: usize,
    contains_scratch: bool,
    seed: i64,
}

/// Murioshi (impossible to press) chord patterns for PopN 9-button.
///
/// Values are 1-indexed button numbers matching the Java definition.
/// Java: `murioshiChords` in `LanePlayableRandomShuffleModifier`
const MURIOSHI_CHORDS: [[u8; 3]; 10] = [
    [1, 4, 7],
    [1, 4, 8],
    [1, 4, 9],
    [1, 5, 8],
    [1, 5, 9],
    [1, 6, 9],
    [2, 5, 8],
    [2, 5, 9],
    [2, 6, 9],
    [3, 6, 9],
];

impl LanePlayableRandomShuffle {
    pub fn new(player: usize, contains_scratch: bool, seed: i64) -> Self {
        Self {
            player,
            contains_scratch,
            seed,
        }
    }

    /// Generate the playable-random mapping array.
    pub fn make_random(&self, keys: &[usize], model: &BmsModel) -> Vec<usize> {
        let key_count = model.mode.key_count();

        // Collect 3+ note chord bitmasks from the chart
        let (is_impossible, original_patterns) = self.collect_chord_patterns(model, keys);

        // Search for valid permutations
        let candidates = if is_impossible {
            Vec::new()
        } else {
            search_no_murioshi_combinations(&original_patterns)
        };

        let mut result = make_identity_mapping(key_count);
        if !candidates.is_empty() {
            let mut rng = JavaRandom::new(self.seed);
            let r = rng.next_int(candidates.len() as i32) as usize;
            let perm = &candidates[r];
            // perm[j] = destination lane number for original lane j
            // Java: result[perm.get(r).get(i)] = i
            for (i, &dest) in perm.iter().enumerate() {
                result[dest] = i;
            }
        } else {
            // Fallback: 50/50 normal or mirror
            let mut rng = JavaRandom::new(self.seed);
            let mirror = rng.next_int(2);
            for (i, item) in result.iter_mut().enumerate().take(9.min(key_count)) {
                *item = if mirror == 0 { i } else { 8 - i };
            }
        }
        result
    }

    /// Scan the chart to collect chord patterns (3+ simultaneous notes) as bitmasks.
    /// Returns `(is_impossible, patterns)` where `is_impossible` is true if any
    /// chord has 7+ notes.
    fn collect_chord_patterns(&self, model: &BmsModel, keys: &[usize]) -> (bool, Vec<u32>) {
        let key_count = model.mode.key_count();
        let mut ln_active = vec![-1i64; key_count];
        let mut end_ln_time = vec![-1i64; key_count];
        let mut patterns = std::collections::HashSet::new();

        // Group notes by timeline (time_us)
        let mut time_groups: std::collections::BTreeMap<i64, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, note) in model.notes.iter().enumerate() {
            time_groups.entry(note.time_us).or_default().push(idx);
        }

        let key_set: std::collections::HashSet<usize> = keys.iter().copied().collect();

        for indices in time_groups.values() {
            // Update LN state
            for &idx in indices {
                let note = &model.notes[idx];
                if note.is_long_note() {
                    if note.end_time_us == 0 && note.time_us == end_ln_time[note.lane] {
                        // LN end
                        ln_active[note.lane] = -1;
                        end_ln_time[note.lane] = -1;
                    } else {
                        ln_active[note.lane] = note.lane as i64;
                        if note.end_time_us > 0 {
                            end_ln_time[note.lane] = note.end_time_us;
                        }
                    }
                }
            }

            // Count active note lanes
            let mut note_lanes = Vec::new();
            for &idx in indices {
                let note = &model.notes[idx];
                if key_set.contains(&note.lane)
                    && note.note_type == NoteType::Normal
                    && !note_lanes.contains(&note.lane)
                {
                    note_lanes.push(note.lane);
                }
            }
            // Also count active LN lanes
            for (lane, &active) in ln_active.iter().enumerate().take(key_count) {
                if key_set.contains(&lane) && active != -1 && !note_lanes.contains(&lane) {
                    note_lanes.push(lane);
                }
            }

            if note_lanes.len() >= 7 {
                return (true, Vec::new());
            }
            if note_lanes.len() >= 3 {
                let mut pattern: u32 = 0;
                for &lane in &note_lanes {
                    pattern += 1 << lane;
                }
                patterns.insert(pattern);
            }
        }

        (false, patterns.into_iter().collect())
    }
}

impl PatternModifier for LanePlayableRandomShuffle {
    fn modify(&mut self, model: &mut BmsModel) {
        let keys = get_keys(model.mode, self.player, self.contains_scratch);
        if keys.is_empty() {
            return;
        }
        let mapping = self.make_random(&keys, model);
        apply_lane_mapping(model, &mapping);
    }

    fn assist_level(&self) -> AssistLevel {
        AssistLevel::LightAssist
    }
}

/// Search all 9! permutations for ones that avoid murioshi patterns.
///
/// Uses Heap's algorithm (iterative) matching the Java implementation.
pub fn search_no_murioshi_combinations(original_patterns: &[u32]) -> Vec<Vec<usize>> {
    let mut results = Vec::new();
    let mut indexes = [0usize; 9];
    let mut lane_numbers: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    // Java's Heap's algorithm does NOT check the initial identity permutation.
    // The loop starts from the first swap, so identity is excluded unless it
    // also appears as a swapped result (it never does in Heap's algorithm).

    let mut i = 0;
    while i < 9 {
        if indexes[i] < i {
            let swap_idx = if i % 2 == 0 { 0 } else { indexes[i] };
            lane_numbers.swap(swap_idx, i);

            if !has_murioshi(&lane_numbers, original_patterns) {
                results.push(lane_numbers.to_vec());
            }

            indexes[i] += 1;
            i = 0;
        } else {
            indexes[i] = 0;
            i += 1;
        }
    }

    // Remove mirror permutation [8,7,6,5,4,3,2,1,0]
    let mirror: Vec<usize> = vec![8, 7, 6, 5, 4, 3, 2, 1, 0];
    results.retain(|p| *p != mirror);

    results
}

/// Check if the given permutation causes any murioshi chord.
///
/// For each original pattern bitmask, extracts the lane indices, maps them
/// through the permutation, and checks against the murioshi table.
fn has_murioshi(lane_numbers: &[usize; 9], original_patterns: &[u32]) -> bool {
    for &pattern in original_patterns {
        // Extract 1-indexed button numbers after permutation
        let mut mapped: Vec<u8> = Vec::new();
        for (j, &lane_num) in lane_numbers.iter().enumerate().take(9) {
            if (pattern >> j) & 1 == 1 {
                mapped.push(lane_num as u8 + 1);
            }
        }

        // Check against murioshi table
        for chord in &MURIOSHI_CHORDS {
            if chord.iter().all(|c| mapped.contains(c)) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::{LnType, Note, PlayMode};

    /// Helper to create a minimal BmsModel with the given mode and notes.
    fn make_model(mode: PlayMode, notes: Vec<Note>) -> BmsModel {
        BmsModel {
            title: String::new(),
            subtitle: String::new(),
            artist: String::new(),
            sub_artist: String::new(),
            genre: String::new(),
            banner: String::new(),
            stage_file: String::new(),
            back_bmp: String::new(),
            preview: String::new(),
            play_level: 0,
            judge_rank: 100,
            judge_rank_raw: 100,
            judge_rank_type: bms_model::JudgeRankType::BmsonJudgeRank,
            total: 300.0,
            difficulty: 0,
            mode,
            ln_type: LnType::LongNote,
            player: 1,
            initial_bpm: 130.0,
            bpm_changes: Vec::new(),
            stop_events: Vec::new(),
            timelines: Vec::new(),
            notes,
            bg_notes: Vec::new(),
            bga_events: Vec::new(),
            wav_defs: Default::default(),
            bmp_defs: Default::default(),
            md5: String::new(),
            sha256: String::new(),
            total_measures: 4,
            total_time_us: 0,
            has_random: false,
        }
    }

    // -----------------------------------------------------------------------
    // Mirror
    // -----------------------------------------------------------------------

    #[test]
    fn test_mirror_mapping_beat7k_no_scratch() {
        let shuffle = LaneMirrorShuffle::new(0, false);
        // Beat7K: keys = [0,1,2,3,4,5,6] (excluding scratch lane 7)
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let mapping = shuffle.make_random(&keys, 8);
        // Mirror: 0<->6, 1<->5, 2<->4, 3<->3, scratch 7 unchanged
        assert_eq!(mapping, vec![6, 5, 4, 3, 2, 1, 0, 7]);
    }

    #[test]
    fn test_mirror_mapping_beat7k_with_scratch() {
        let shuffle = LaneMirrorShuffle::new(0, true);
        let keys = get_keys(PlayMode::Beat7K, 0, true);
        let mapping = shuffle.make_random(&keys, 8);
        // Mirror with scratch: 0<->7, 1<->6, 2<->5, 3<->4
        assert_eq!(mapping, vec![7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_mirror_mapping_popn9k() {
        let shuffle = LaneMirrorShuffle::new(0, false);
        let keys = get_keys(PlayMode::PopN9K, 0, false);
        let mapping = shuffle.make_random(&keys, 9);
        // Mirror: 0<->8, 1<->7, 2<->6, 3<->5, 4<->4
        assert_eq!(mapping, vec![8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_mirror_modify_notes() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(3, 2000, 2),
            Note::normal(6, 3000, 3),
            Note::normal(7, 4000, 4), // scratch - untouched
        ];
        let mut model = make_model(PlayMode::Beat7K, notes);
        let mut shuffle = LaneMirrorShuffle::new(0, false);
        shuffle.modify(&mut model);

        assert_eq!(model.notes[0].lane, 6); // 0 -> 6
        assert_eq!(model.notes[1].lane, 3); // 3 -> 3 (center)
        assert_eq!(model.notes[2].lane, 0); // 6 -> 0
        assert_eq!(model.notes[3].lane, 7); // scratch unchanged
    }

    #[test]
    fn test_mirror_assist_level() {
        let no_scratch = LaneMirrorShuffle::new(0, false);
        assert_eq!(no_scratch.assist_level(), AssistLevel::None);

        let with_scratch = LaneMirrorShuffle::new(0, true);
        assert_eq!(with_scratch.assist_level(), AssistLevel::LightAssist);
    }

    // -----------------------------------------------------------------------
    // Rotate
    // -----------------------------------------------------------------------

    #[test]
    fn test_rotate_mapping_is_permutation() {
        let shuffle = LaneRotateShuffle::new(0, false, 42);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let mapping = shuffle.make_random(&keys, 8);

        // All key lanes should appear exactly once in the mapping output
        let mut mapped_keys: Vec<usize> = keys.iter().map(|&k| mapping[k]).collect();
        mapped_keys.sort();
        assert_eq!(mapped_keys, keys);
        // Scratch lane should be unchanged
        assert_eq!(mapping[7], 7);
    }

    #[test]
    fn test_rotate_different_seeds_differ() {
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let s1 = LaneRotateShuffle::new(0, false, 1);
        let s2 = LaneRotateShuffle::new(0, false, 999);
        let m1 = s1.make_random(&keys, 8);
        let m2 = s2.make_random(&keys, 8);
        // Very unlikely to be the same with different seeds
        assert_ne!(m1, m2);
    }

    #[test]
    fn test_rotate_seed_0() {
        // Deterministic test with seed 0
        // Java: new Random(0), nextInt(2)==1(inc=true), nextInt(6)==4(start_offset=4)
        // Direction: increasing. start=5
        // keys=[0,1,2,3,4,5,6], rlane starts at 5
        // lane=0: result[keys[0]]=result[0]=keys[5]=5, rlane=(5+1)%7=6
        // lane=1: result[keys[1]]=result[1]=keys[6]=6, rlane=(6+1)%7=0
        // lane=2: result[keys[2]]=result[2]=keys[0]=0, rlane=(0+1)%7=1
        // lane=3: result[keys[3]]=result[3]=keys[1]=1, rlane=(1+1)%7=2
        // lane=4: result[keys[4]]=result[4]=keys[2]=2, rlane=(2+1)%7=3
        // lane=5: result[keys[5]]=result[5]=keys[3]=3, rlane=(3+1)%7=4
        // lane=6: result[keys[6]]=result[6]=keys[4]=4, rlane=(4+1)%7=5
        let shuffle = LaneRotateShuffle::new(0, false, 0);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let mapping = shuffle.make_random(&keys, 8);
        assert_eq!(mapping, vec![5, 6, 0, 1, 2, 3, 4, 7]);
    }

    // -----------------------------------------------------------------------
    // Random
    // -----------------------------------------------------------------------

    #[test]
    fn test_random_mapping_is_permutation() {
        let shuffle = LaneRandomShuffle::new(0, false, 12345);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let mapping = shuffle.make_random(&keys, 8);

        let mut mapped_keys: Vec<usize> = keys.iter().map(|&k| mapping[k]).collect();
        mapped_keys.sort();
        assert_eq!(mapped_keys, keys);
        assert_eq!(mapping[7], 7); // scratch unchanged
    }

    #[test]
    fn test_random_seed_deterministic() {
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let s1 = LaneRandomShuffle::new(0, false, 42);
        let s2 = LaneRandomShuffle::new(0, false, 42);
        assert_eq!(s1.make_random(&keys, 8), s2.make_random(&keys, 8));
    }

    #[test]
    fn test_random_seed_0() {
        // Deterministic test with seed 0
        // JavaRandom(0): next_int(7)=5, next_int(6)=4, next_int(5)=4,
        //                next_int(4)=2, next_int(3)=2, next_int(2)=0, next_int(1)=0
        // remaining starts as [0,1,2,3,4,5,6], keys=[0,1,2,3,4,5,6]
        // lane 0: r=5, result[keys[0]]=result[0]=remaining[5]=5, remaining=[0,1,2,3,4,6]
        // lane 1: r=4, result[keys[1]]=result[1]=remaining[4]=4, remaining=[0,1,2,3,6]
        // lane 2: r=4, result[keys[2]]=result[2]=remaining[4]=6, remaining=[0,1,2,3]
        // lane 3: r=2, result[keys[3]]=result[3]=remaining[2]=2, remaining=[0,1,3]
        // lane 4: r=2, result[keys[4]]=result[4]=remaining[2]=3, remaining=[0,1]
        // lane 5: r=0, result[keys[5]]=result[5]=remaining[0]=0, remaining=[1]
        // lane 6: r=0, result[keys[6]]=result[6]=remaining[0]=1, remaining=[]
        let shuffle = LaneRandomShuffle::new(0, false, 0);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        let mapping = shuffle.make_random(&keys, 8);
        assert_eq!(mapping, vec![5, 4, 6, 2, 3, 0, 1, 7]);
    }

    // -----------------------------------------------------------------------
    // Cross
    // -----------------------------------------------------------------------

    #[test]
    fn test_cross_mapping_beat7k() {
        let shuffle = LaneCrossShuffle::new(0, false);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        // keys = [0,1,2,3,4,5,6], len=7, loop: i=0,2 (i < 7/2-1 = 2)
        // Actually i < 7/2 - 1 = 2, so only i=0
        // i=0: result[0]=keys[1]=1, result[1]=keys[0]=0
        //      result[6]=keys[5]=5, result[5]=keys[6]=6
        let mapping = shuffle.make_random(&keys, 8);
        assert_eq!(mapping, vec![1, 0, 2, 3, 4, 6, 5, 7]);
    }

    #[test]
    fn test_cross_mapping_popn9k() {
        let shuffle = LaneCrossShuffle::new(0, false);
        let keys = get_keys(PlayMode::PopN9K, 0, false);
        // keys = [0,1,2,3,4,5,6,7,8], len=9, loop: i=0,2 (i < 9/2-1 = 3)
        // i=0: result[0]=1, result[1]=0, result[8]=7, result[7]=8
        // i=2: result[2]=3, result[3]=2, result[6]=5, result[5]=6
        let mapping = shuffle.make_random(&keys, 9);
        assert_eq!(mapping, vec![1, 0, 3, 2, 4, 6, 5, 8, 7]);
    }

    #[test]
    fn test_cross_assist_level() {
        let shuffle = LaneCrossShuffle::new(0, false);
        assert_eq!(shuffle.assist_level(), AssistLevel::LightAssist);
    }

    // -----------------------------------------------------------------------
    // Flip
    // -----------------------------------------------------------------------

    #[test]
    fn test_flip_dp() {
        let shuffle = PlayerFlipShuffle::new();
        // Beat14K: 16 keys, 2 players
        let mapping = shuffle.make_random(16, 2);
        // Each lane i -> (i + 8) % 16
        for i in 0..16 {
            assert_eq!(mapping[i], (i + 8) % 16);
        }
    }

    #[test]
    fn test_flip_sp_noop() {
        let shuffle = PlayerFlipShuffle::new();
        // Beat7K: 8 keys, 1 player -> no change
        let mapping = shuffle.make_random(8, 1);
        assert_eq!(mapping, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_flip_modify_dp_model() {
        let notes = vec![
            Note::normal(0, 1000, 1),  // 1P lane 0 -> 2P lane 8
            Note::normal(8, 2000, 2),  // 2P lane 8 -> 1P lane 0
            Note::normal(15, 3000, 3), // 2P lane 15 -> 1P lane 7
        ];
        let mut model = make_model(PlayMode::Beat14K, notes);
        let mut shuffle = PlayerFlipShuffle::new();
        shuffle.modify(&mut model);

        assert_eq!(model.notes[0].lane, 8);
        assert_eq!(model.notes[1].lane, 0);
        assert_eq!(model.notes[2].lane, 7);
    }

    #[test]
    fn test_flip_assist_level() {
        let shuffle = PlayerFlipShuffle::new();
        assert_eq!(shuffle.assist_level(), AssistLevel::None);
    }

    // -----------------------------------------------------------------------
    // Battle
    // -----------------------------------------------------------------------

    #[test]
    fn test_battle_dp_copies_1p_to_2p() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(3, 2000, 2),
            Note::normal(7, 3000, 3), // scratch
        ];
        let mut model = make_model(PlayMode::Beat14K, notes);
        let mut shuffle = PlayerBattleShuffle::new();
        shuffle.modify(&mut model);

        // Should have 6 notes: 3 original 1P + 3 cloned 2P
        assert_eq!(model.notes.len(), 6);

        // 1P notes unchanged
        assert_eq!(model.notes[0].lane, 0);
        assert_eq!(model.notes[1].lane, 3);
        assert_eq!(model.notes[2].lane, 7);

        // 2P clones with lane + 8
        assert_eq!(model.notes[3].lane, 8);
        assert_eq!(model.notes[4].lane, 11);
        assert_eq!(model.notes[5].lane, 15);

        // Same wav_ids
        assert_eq!(model.notes[3].wav_id, 1);
        assert_eq!(model.notes[4].wav_id, 2);
        assert_eq!(model.notes[5].wav_id, 3);
    }

    #[test]
    fn test_battle_sp_noop() {
        let notes = vec![Note::normal(0, 1000, 1)];
        let mut model = make_model(PlayMode::Beat7K, notes);
        let mut shuffle = PlayerBattleShuffle::new();
        shuffle.modify(&mut model);

        // SP: no change
        assert_eq!(model.notes.len(), 1);
        assert_eq!(model.notes[0].lane, 0);
    }

    #[test]
    fn test_battle_removes_existing_2p_notes() {
        let notes = vec![
            Note::normal(0, 1000, 1),  // 1P
            Note::normal(8, 2000, 2),  // 2P (should be removed and replaced)
            Note::normal(12, 3000, 3), // 2P (should be removed and replaced)
        ];
        let mut model = make_model(PlayMode::Beat14K, notes);
        let mut shuffle = PlayerBattleShuffle::new();
        shuffle.modify(&mut model);

        // Only 1P note (lane 0) + its clone (lane 8)
        assert_eq!(model.notes.len(), 2);
        assert_eq!(model.notes[0].lane, 0);
        assert_eq!(model.notes[1].lane, 8);
    }

    #[test]
    fn test_battle_ln_pair_integrity() {
        // Create a DP model with LN on 1P side
        let mut notes = vec![
            Note::long_note(0, 1000, 2000, 1, 2, LnType::LongNote), // LN start
            Note::long_note(0, 2000, 0, 2, 0, LnType::LongNote),    // LN end
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;

        let mut model = make_model(PlayMode::Beat14K, notes);
        let mut shuffle = PlayerBattleShuffle::new();
        shuffle.modify(&mut model);

        assert_eq!(model.notes.len(), 4);

        // 1P pair: 0 <-> 1
        assert_eq!(model.notes[0].pair_index, 1);
        assert_eq!(model.notes[1].pair_index, 0);

        // 2P pair: 2 <-> 3
        assert_eq!(model.notes[2].pair_index, 3);
        assert_eq!(model.notes[3].pair_index, 2);

        // Lanes
        assert_eq!(model.notes[2].lane, 8);
        assert_eq!(model.notes[3].lane, 8);
    }

    #[test]
    fn test_battle_assist_level() {
        let shuffle = PlayerBattleShuffle::new();
        assert_eq!(shuffle.assist_level(), AssistLevel::Assist);
    }

    // -----------------------------------------------------------------------
    // Playable Random
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_murioshi_basic() {
        // Pattern with buttons 1,4,7 (0-indexed: lanes 0,3,6) -> bitmask = 1+8+64 = 73
        let patterns = vec![73u32]; // bits 0,3,6
        let identity = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        // After permutation: lane_numbers[0]+1=1, lane_numbers[3]+1=4, lane_numbers[6]+1=7
        // -> [1,4,7] is a murioshi chord
        assert!(has_murioshi(&identity, &patterns));
    }

    #[test]
    fn test_has_murioshi_no_match() {
        // Pattern with buttons 1,2,3 (0-indexed: lanes 0,1,2) -> bitmask = 1+2+4 = 7
        let patterns = vec![7u32]; // bits 0,1,2
        let identity = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        // After permutation: [1,2,3] is NOT a murioshi chord
        assert!(!has_murioshi(&identity, &patterns));
    }

    #[test]
    fn test_search_no_murioshi_empty_patterns() {
        // No chord patterns -> all permutations valid minus identity and mirror
        // Java Heap's algorithm skips identity (loop starts after first swap),
        // then removes mirror. Total: 9! - 2 = 362878
        let results = search_no_murioshi_combinations(&[]);
        assert_eq!(results.len(), 362878);
    }

    #[test]
    fn test_search_no_murioshi_excludes_identity_and_mirror() {
        let results = search_no_murioshi_combinations(&[]);
        let identity = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let mirror = vec![8, 7, 6, 5, 4, 3, 2, 1, 0];
        assert!(!results.contains(&identity));
        assert!(!results.contains(&mirror));
    }

    #[test]
    fn test_playable_random_deterministic() {
        let notes = vec![
            Note::normal(0, 1000, 1),
            Note::normal(4, 1000, 2),
            Note::normal(8, 1000, 3),
        ];
        let model = make_model(PlayMode::PopN9K, notes);
        let keys = get_keys(PlayMode::PopN9K, 0, false);
        let s1 = LanePlayableRandomShuffle::new(0, false, 42);
        let s2 = LanePlayableRandomShuffle::new(0, false, 42);
        assert_eq!(s1.make_random(&keys, &model), s2.make_random(&keys, &model));
    }

    // -----------------------------------------------------------------------
    // LN pair integrity after lane mapping
    // -----------------------------------------------------------------------

    #[test]
    fn test_ln_pair_integrity_after_mirror() {
        let mut notes = vec![
            Note::long_note(0, 1000, 2000, 1, 2, LnType::LongNote), // LN start on lane 0
            Note::long_note(0, 2000, 0, 2, 0, LnType::LongNote),    // LN end on lane 0
            Note::long_note(3, 3000, 4000, 3, 4, LnType::LongNote), // LN start on lane 3
            Note::long_note(3, 4000, 0, 4, 0, LnType::LongNote),    // LN end on lane 3
        ];
        notes[0].pair_index = 1;
        notes[1].pair_index = 0;
        notes[2].pair_index = 3;
        notes[3].pair_index = 2;

        let mut model = make_model(PlayMode::Beat7K, notes);
        let mut shuffle = LaneMirrorShuffle::new(0, false);
        shuffle.modify(&mut model);

        // After mirror: lane 0->6, lane 3->3
        assert_eq!(model.notes[0].lane, 6);
        assert_eq!(model.notes[1].lane, 6);
        assert_eq!(model.notes[2].lane, 3);
        assert_eq!(model.notes[3].lane, 3);

        // pair_index should still be valid (unchanged since we only modify lanes)
        assert_eq!(model.notes[0].pair_index, 1);
        assert_eq!(model.notes[1].pair_index, 0);
        assert_eq!(model.notes[2].pair_index, 3);
        assert_eq!(model.notes[3].pair_index, 2);

        // Start and end notes should be on the same lane
        let start0 = &model.notes[0];
        let end0 = &model.notes[start0.pair_index];
        assert_eq!(start0.lane, end0.lane);

        let start1 = &model.notes[2];
        let end1 = &model.notes[start1.pair_index];
        assert_eq!(start1.lane, end1.lane);
    }

    // -----------------------------------------------------------------------
    // apply_lane_mapping edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_apply_lane_mapping_out_of_range() {
        // Notes with lane >= mapping.len() should be unchanged
        let notes = vec![Note::normal(10, 1000, 1)];
        let mut model = make_model(PlayMode::Beat7K, notes);
        let mapping = vec![7, 6, 5, 4, 3, 2, 1, 0]; // 8 entries
        apply_lane_mapping(&mut model, &mapping);
        assert_eq!(model.notes[0].lane, 10); // unchanged
    }

    #[test]
    fn test_make_identity_mapping() {
        assert_eq!(make_identity_mapping(0), Vec::<usize>::new());
        assert_eq!(make_identity_mapping(4), vec![0, 1, 2, 3]);
        assert_eq!(make_identity_mapping(9), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
