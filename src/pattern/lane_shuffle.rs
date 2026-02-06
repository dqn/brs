//! Lane-level shuffle modifiers.
//!
//! These apply a single permutation mapping to all notes. The mapping is
//! computed once and applied uniformly, meaning every note on lane X moves
//! to lane Y regardless of timing.
//!
//! Ported from beatoraja `LaneShuffleModifier.java`.

use rand::Rng;

use crate::model::note::{Note, PlayMode};

use super::get_keys;

/// Apply a lane permutation to all notes.
///
/// `mapping[original_lane] = new_lane`. Lanes not in the mapping are unchanged.
fn apply_lane_mapping(notes: &mut [Note], mapping: &[usize]) {
    for note in notes.iter_mut() {
        if note.lane < mapping.len() {
            note.lane = mapping[note.lane];
        }
    }
}

/// Mirror modifier: reverses the order of key lanes.
///
/// Corresponds to `LaneMirrorShuffleModifier` in beatoraja.
/// Scratch lanes are only included when `include_scratch` is true.
pub fn mirror(notes: &mut [Note], mode: PlayMode, player: usize, include_scratch: bool) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }
    let total = mode.lane_count();
    let mut mapping: Vec<usize> = (0..total).collect();
    for (i, &key) in keys.iter().enumerate() {
        mapping[key] = keys[keys.len() - 1 - i];
    }
    apply_lane_mapping(notes, &mapping);
}

/// Random modifier: randomly shuffles key lanes (one permutation for entire chart).
///
/// Corresponds to `LaneRandomShuffleModifier` in beatoraja.
pub fn lane_random(
    notes: &mut [Note],
    mode: PlayMode,
    player: usize,
    include_scratch: bool,
    rng: &mut impl Rng,
) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }
    let total = mode.lane_count();
    let mut mapping: Vec<usize> = (0..total).collect();

    // Fisher-Yates shuffle matching beatoraja's approach:
    // Pick from remaining pool for each position.
    let mut pool = keys.clone();
    for &key in &keys {
        let r = rng.random_range(0..pool.len());
        mapping[key] = pool[r];
        pool.swap_remove(r);
    }
    apply_lane_mapping(notes, &mapping);
}

/// Rotate modifier: rotates key lanes by a random offset in a random direction.
///
/// Corresponds to `LaneRotateShuffleModifier` in beatoraja.
pub fn rotate(
    notes: &mut [Note],
    mode: PlayMode,
    player: usize,
    include_scratch: bool,
    rng: &mut impl Rng,
) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }
    let total = mode.lane_count();
    let mut mapping: Vec<usize> = (0..total).collect();

    let inc: bool = rng.random_range(0..2) == 1;
    let start = rng.random_range(0..keys.len() - 1) + if inc { 1 } else { 0 };
    let len = keys.len();
    let mut rlane = start;
    for &key in &keys {
        mapping[key] = keys[rlane];
        if inc {
            rlane = (rlane + 1) % len;
        } else {
            rlane = (rlane + len - 1) % len;
        }
    }
    apply_lane_mapping(notes, &mapping);
}

/// Cross modifier: swaps pairs of lanes from outside inward.
///
/// Corresponds to `LaneCrossShuffleModifier` in beatoraja.
pub fn cross(notes: &mut [Note], mode: PlayMode, player: usize, include_scratch: bool) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }
    let total = mode.lane_count();
    let mut mapping: Vec<usize> = (0..total).collect();

    let mut i = 0;
    while i < keys.len() / 2 - 1 {
        mapping[keys[i]] = keys[i + 1];
        mapping[keys[i + 1]] = keys[i];
        mapping[keys[keys.len() - i - 1]] = keys[keys.len() - i - 2];
        mapping[keys[keys.len() - i - 2]] = keys[keys.len() - i - 1];
        i += 2;
    }
    apply_lane_mapping(notes, &mapping);
}

/// Flip modifier: swaps 1P and 2P sides in double play modes.
///
/// Corresponds to `PlayerFlipModifier` in beatoraja.
/// No-op for single-player modes.
pub fn flip(notes: &mut [Note], mode: PlayMode) {
    if mode.player_count() != 2 {
        return;
    }
    let total = mode.lane_count();
    let half = total / 2;
    let mapping: Vec<usize> = (0..total).map(|i| (i + half) % total).collect();
    apply_lane_mapping(notes, &mapping);
}

/// Battle modifier: copies 1P notes to 2P side in double play modes.
///
/// Corresponds to `PlayerBattleModifier` in beatoraja.
/// No-op for single-player modes.
pub fn battle(notes: &mut Vec<Note>, mode: PlayMode) {
    if mode.player_count() != 2 {
        return;
    }
    let total = mode.lane_count();
    let half = total / 2;

    // Remove all P2 notes (lanes >= half), then duplicate P1 notes to P2.
    notes.retain(|n| n.lane < half);
    let p1_notes: Vec<Note> = notes.to_vec();
    for n in &p1_notes {
        let mut p2_note: Note = n.clone();
        p2_note.lane += half;
        notes.push(p2_note);
    }
    // Re-sort by time, then lane for determinism.
    notes.sort_by(|a, b| a.time_us.cmp(&b.time_us).then(a.lane.cmp(&b.lane)));
}

/// Playable Random modifier (PMS): shuffles lanes avoiding impossible chords.
///
/// Corresponds to `LanePlayableRandomShuffleModifier` in beatoraja.
/// This is specifically designed for PopN 9K mode to avoid "murioshi" (impossible)
/// button combinations.
pub fn playable_random(notes: &mut [Note], mode: PlayMode, player: usize, rng: &mut impl Rng) {
    let keys = get_keys(mode, player, false);
    if keys.is_empty() || keys.len() != 9 {
        // Only applicable to 9-key modes (PopN)
        lane_random(notes, mode, player, false, rng);
        return;
    }

    let total = mode.lane_count();

    // Collect timeslices and their chord patterns
    let mut ln_active: Vec<bool> = vec![false; total];
    let mut ln_end_time: Vec<i64> = vec![-1; total];
    let mut patterns: Vec<u32> = Vec::new();
    let mut is_impossible = false;

    // Group notes by time
    let mut time_groups: Vec<(i64, Vec<&Note>)> = Vec::new();
    for n in notes.iter() {
        if let Some(last) = time_groups.last_mut()
            && last.0 == n.time_us
        {
            last.1.push(n);
            continue;
        }
        time_groups.push((n.time_us, vec![n]));
    }

    for (time, group) in &time_groups {
        // Update LN tracking
        for n in group {
            if n.is_long_note() {
                if ln_active[n.lane] && *time == ln_end_time[n.lane] {
                    ln_active[n.lane] = false;
                    ln_end_time[n.lane] = -1;
                } else {
                    ln_active[n.lane] = true;
                    ln_end_time[n.lane] = n.end_time_us;
                }
            }
        }

        // Count active notes (normal + LN active)
        let mut note_lanes: Vec<usize> = Vec::new();
        for (i, &is_ln_active) in ln_active.iter().enumerate().take(total) {
            let has_note = group
                .iter()
                .any(|n| n.lane == i && super::is_playable(n.note_type));
            if has_note || is_ln_active {
                note_lanes.push(i);
            }
        }

        if note_lanes.len() >= 7 {
            is_impossible = true;
            break;
        } else if note_lanes.len() >= 3 {
            let mut pattern: u32 = 0;
            for &l in &note_lanes {
                pattern |= 1 << l;
            }
            patterns.push(pattern);
        }
    }

    let patterns_set: std::collections::HashSet<u32> = patterns.into_iter().collect();

    if !is_impossible {
        // Search for lane combinations that avoid murioshi
        let candidates = search_no_murioshi_combinations(&patterns_set, &keys);
        if !candidates.is_empty() {
            let idx = rng.random_range(0..candidates.len());
            let mut mapping: Vec<usize> = (0..total).collect();
            for (i, &dest) in candidates[idx].iter().enumerate() {
                mapping[dest] = i;
            }
            for note in notes.iter_mut() {
                if note.lane < mapping.len() {
                    note.lane = mapping[note.lane];
                }
            }
            return;
        }
    }

    // Fallback: normal or mirror randomly
    let use_mirror: bool = rng.random_range(0..2) == 0;
    if use_mirror {
        mirror(notes, mode, player, false);
    }
}

/// Murioshi (impossible chord) patterns for PMS 9-key.
/// These are 3-button combinations that require impossible finger spans.
const MURIOSHI_CHORDS: &[[usize; 3]] = &[
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

/// Search for permutations of 9 lanes that avoid murioshi chords.
/// Uses Heap's algorithm for permutation generation matching beatoraja.
fn search_no_murioshi_combinations(
    original_patterns: &std::collections::HashSet<u32>,
    _keys: &[usize],
) -> Vec<Vec<usize>> {
    let mut results: Vec<Vec<usize>> = Vec::new();
    let mut lane_numbers: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    let mut indexes = [0usize; 9];

    // Check initial permutation (identity)
    if !has_murioshi(original_patterns, &lane_numbers) {
        results.push(lane_numbers.to_vec());
    }

    // Heap's algorithm
    let mut i = 0;
    while i < 9 {
        if indexes[i] < i {
            let swap_idx = if i % 2 == 0 { 0 } else { indexes[i] };
            lane_numbers.swap(swap_idx, i);

            if !has_murioshi(original_patterns, &lane_numbers) {
                results.push(lane_numbers.to_vec());
            }

            indexes[i] += 1;
            i = 0;
        } else {
            indexes[i] = 0;
            i += 1;
        }
    }

    // Remove mirror (reverse) from candidates
    let mirror_perm = vec![8, 7, 6, 5, 4, 3, 2, 1, 0];
    results.retain(|r| *r != mirror_perm);

    results
}

/// Check if any original chord pattern, when remapped through the given
/// lane permutation, creates a murioshi chord.
fn has_murioshi(
    original_patterns: &std::collections::HashSet<u32>,
    lane_numbers: &[usize; 9],
) -> bool {
    for &pattern in original_patterns {
        let mut remapped: Vec<usize> = Vec::new();
        for (j, &lane_num) in lane_numbers.iter().enumerate() {
            if (pattern >> j) & 1 == 1 {
                // 1-indexed for murioshi check (matching beatoraja)
                remapped.push(lane_num + 1);
            }
        }
        for chord in MURIOSHI_CHORDS {
            if chord.iter().all(|c| remapped.contains(c)) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::note::NoteType;
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

    #[test]
    fn mirror_beat5k() {
        let mut notes = vec![normal_note(0, 0), normal_note(4, 0)];
        mirror(&mut notes, PlayMode::Beat5K, 0, false);
        assert_eq!(notes[0].lane, 4);
        assert_eq!(notes[1].lane, 0);
    }

    #[test]
    fn mirror_beat14k_player0() {
        let mut notes = vec![normal_note(0, 0)];
        mirror(&mut notes, PlayMode::Beat14K, 0, false);
        assert_eq!(notes[0].lane, 6);
    }

    #[test]
    fn mirror_beat14k_player1() {
        let mut notes = vec![normal_note(8, 0)];
        mirror(&mut notes, PlayMode::Beat14K, 1, false);
        assert_eq!(notes[0].lane, 14);
    }

    #[test]
    fn lane_random_is_permutation() {
        let mut notes: Vec<Note> = (0..7).map(|i| normal_note(i, 0)).collect();
        let mut rng = SmallRng::seed_from_u64(42);
        lane_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);

        let mut lanes: Vec<usize> = notes.iter().map(|n| n.lane).collect();
        lanes.sort();
        lanes.dedup();
        assert_eq!(lanes.len(), 7);
        assert!(lanes.iter().all(|&l| l < 7));
    }

    #[test]
    fn rotate_produces_rotation() {
        let notes: Vec<Note> = (0..7).map(|i| normal_note(i, 0)).collect();
        let mut seen_different = false;
        for seed in 0..100 {
            let mut m = notes.clone();
            let mut rng = SmallRng::seed_from_u64(seed);
            rotate(&mut m, PlayMode::Beat7K, 0, false, &mut rng);
            if m.iter().map(|n| n.lane).collect::<Vec<_>>()
                != notes.iter().map(|n| n.lane).collect::<Vec<_>>()
            {
                seen_different = true;
                break;
            }
        }
        assert!(
            seen_different,
            "Rotate should produce at least one non-identity result"
        );
    }

    #[test]
    fn cross_swaps_pairs() {
        let mut notes: Vec<Note> = (0..6).map(|i| normal_note(i, 0)).collect();
        cross(&mut notes, PlayMode::Beat7K, 0, false);
        assert_eq!(notes[0].lane, 1);
        assert_eq!(notes[1].lane, 0);
        assert_eq!(notes[5].lane, 6);
    }

    #[test]
    fn flip_beat14k() {
        let mut notes = vec![
            normal_note(0, 0),
            normal_note(7, 0),
            normal_note(8, 0),
            normal_note(15, 0),
        ];
        flip(&mut notes, PlayMode::Beat14K);
        assert_eq!(notes[0].lane, 8);
        assert_eq!(notes[1].lane, 15);
        assert_eq!(notes[2].lane, 0);
        assert_eq!(notes[3].lane, 7);
    }

    #[test]
    fn battle_duplicates_to_p2() {
        let mut notes = vec![normal_note(0, 0), normal_note(3, 1000)];
        battle(&mut notes, PlayMode::Beat14K);
        assert_eq!(notes.len(), 4);
        let p2_notes: Vec<&Note> = notes.iter().filter(|n| n.lane >= 8).collect();
        assert_eq!(p2_notes.len(), 2);
    }
}
