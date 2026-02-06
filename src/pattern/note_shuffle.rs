//! Note-level shuffle modifiers.
//!
//! These modifiers assign lanes per-timeslice (group of notes at the same time),
//! tracking active LN state to prevent lane changes during holds.
//!
//! Ported from beatoraja `Randomizer.java` and `NoteShuffleModifier.java`.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use rand::Rng;

use crate::model::note::{Note, PlayMode};

use super::{get_keys, is_playable};

/// Parameters for time-based shuffle operations.
struct ShuffleParams<'a> {
    notes: &'a [Note],
    range: std::ops::Range<usize>,
    time: i64,
    changeable: Vec<usize>,
    assignable: Vec<usize>,
    threshold_us: i64,
    last_note_time: &'a HashMap<usize, i64>,
}

/// S-Random: shuffle note lanes per timeslice with a 40ms threshold to avoid
/// rapid repeated hits on the same lane.
///
/// Corresponds to `SRandomizer` with `SRAN_THRESHOLD` (40ms = 40_000us) in beatoraja.
pub fn s_random(
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
    time_based_shuffle(notes, &keys, 40_000, rng);
}

/// S-Random with no threshold (PMS variant).
///
/// Corresponds to `S_RANDOM_NO_THRESHOLD` in beatoraja.
pub fn s_random_no_threshold(
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
    time_based_shuffle(notes, &keys, 0, rng);
}

/// H-Random: shuffle note lanes per timeslice with a BPM-based threshold.
///
/// `threshold_us` is typically `ceil(15000 / BPM) * 1000` in microseconds.
/// Corresponds to `SRandomizer` with H-Random threshold in beatoraja.
pub fn h_random(
    notes: &mut [Note],
    mode: PlayMode,
    player: usize,
    include_scratch: bool,
    threshold_us: i64,
    rng: &mut impl Rng,
) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }
    time_based_shuffle(notes, &keys, threshold_us, rng);
}

/// Spiral: rotates note assignment by a fixed increment per timeslice.
///
/// Corresponds to `SpiralRandomizer` in beatoraja.
pub fn spiral(
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

    let cycle = keys.len();
    let increment = rng.random_range(1..cycle);
    let mut head: usize = 0;

    let groups = group_notes_by_time(notes);
    let mut ln_active: HashMap<usize, usize> = HashMap::new();

    for (_, range) in &groups {
        let changeable_count = keys.iter().filter(|k| !ln_active.contains_key(k)).count();

        if changeable_count == cycle {
            head = (head + increment) % cycle;
        }

        let mut rotation_map: HashMap<usize, usize> = HashMap::new();
        for (i, &key) in keys.iter().enumerate() {
            if !ln_active.contains_key(&key) {
                rotation_map.insert(key, keys[(i + head) % cycle]);
            }
        }

        for (&src, &dst) in &ln_active {
            rotation_map.insert(src, dst);
        }

        apply_timeslice_map(notes, range, &rotation_map, &mut ln_active);
    }
}

/// All-SCR: assigns notes to scratch lane(s) preferentially, with S-Random
/// for remaining notes.
///
/// Corresponds to `AllScratchRandomizer` in beatoraja.
pub fn all_scratch(
    notes: &mut [Note],
    mode: PlayMode,
    player: usize,
    h_threshold_us: i64,
    rng: &mut impl Rng,
) {
    let keys = get_keys(mode, player, true);
    if keys.is_empty() {
        return;
    }

    let scratch_lanes: Vec<usize> = keys
        .iter()
        .copied()
        .filter(|&k| mode.is_scratch(k))
        .collect();
    if scratch_lanes.is_empty() {
        time_based_shuffle(notes, &keys, 40_000, rng);
        return;
    }

    let groups = group_notes_by_time(notes);

    let mut ln_active: HashMap<usize, usize> = HashMap::new();
    let mut last_note_time: HashMap<usize, i64> = HashMap::new();
    for &k in &keys {
        last_note_time.insert(k, -10_000_000);
    }
    let mut scratch_index: usize = 0;

    for (time, range) in &groups {
        let time = *time;

        let changeable: Vec<usize> = keys
            .iter()
            .copied()
            .filter(|k| !ln_active.contains_key(k))
            .collect();
        let assignable: Vec<usize> = changeable.clone();

        let mut scratch_assigned = false;
        let scratch_lane = scratch_lanes[scratch_index];
        if assignable.contains(&scratch_lane) {
            let elapsed = time
                - last_note_time
                    .get(&scratch_lane)
                    .copied()
                    .unwrap_or(-10_000_000);
            if elapsed > 40_000 {
                let note_src = changeable.iter().copied().find(|&cl| {
                    notes[range.clone()]
                        .iter()
                        .any(|n| n.lane == cl && is_playable(n.note_type))
                });
                if let Some(src) = note_src {
                    let mut map: HashMap<usize, usize> = HashMap::new();
                    map.insert(src, scratch_lane);
                    let remaining_changeable: Vec<usize> =
                        changeable.iter().copied().filter(|&c| c != src).collect();
                    let remaining_assignable: Vec<usize> = assignable
                        .iter()
                        .copied()
                        .filter(|&a| a != scratch_lane)
                        .collect();

                    scratch_index = (scratch_index + 1) % scratch_lanes.len();
                    scratch_assigned = true;

                    let params = ShuffleParams {
                        notes,
                        range: range.clone(),
                        time,
                        changeable: remaining_changeable,
                        assignable: remaining_assignable,
                        threshold_us: h_threshold_us,
                        last_note_time: &last_note_time,
                    };
                    let remaining_map = build_time_based_map(&params, rng);
                    map.extend(remaining_map);

                    for (&src, &dst) in &ln_active {
                        map.insert(src, dst);
                    }

                    update_note_time(notes, range, &map, &mut last_note_time);
                    apply_timeslice_map(notes, range, &map, &mut ln_active);
                }
            }
        }

        if !scratch_assigned {
            let params = ShuffleParams {
                notes,
                range: range.clone(),
                time,
                changeable,
                assignable,
                threshold_us: h_threshold_us,
                last_note_time: &last_note_time,
            };
            let mut map = build_time_based_map(&params, rng);

            for (&src, &dst) in &ln_active {
                map.insert(src, dst);
            }

            update_note_time(notes, range, &map, &mut last_note_time);
            apply_timeslice_map(notes, range, &map, &mut ln_active);
        }
    }
}

/// Converge: tries to create long runs of repeated notes on the same lane.
///
/// Corresponds to `ConvergeRandomizer` in beatoraja.
pub fn converge(
    notes: &mut [Note],
    mode: PlayMode,
    player: usize,
    include_scratch: bool,
    threshold_us: i64,
    rng: &mut impl Rng,
) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }

    let threshold2 = threshold_us * 2;
    let groups = group_notes_by_time(notes);

    let mut ln_active: HashMap<usize, usize> = HashMap::new();
    let mut last_note_time: HashMap<usize, i64> = HashMap::new();
    let mut renda_count: HashMap<usize, usize> = HashMap::new();
    for &k in &keys {
        last_note_time.insert(k, -10_000_000);
        renda_count.insert(k, 0);
    }

    for (time, range) in &groups {
        let time = *time;

        // Reset renda count for lanes where time gap exceeds threshold2
        for (&lane, &last_time) in last_note_time.iter() {
            if time - last_time > threshold2 {
                renda_count.insert(lane, 0);
            }
        }

        let changeable: Vec<usize> = keys
            .iter()
            .copied()
            .filter(|k| !ln_active.contains_key(k))
            .collect();
        let assignable: Vec<usize> = changeable.clone();

        let mut map = build_time_based_map_converge(
            notes,
            range,
            time,
            &changeable,
            &assignable,
            threshold_us,
            &last_note_time,
            &mut renda_count,
            rng,
        );

        for (&src, &dst) in &ln_active {
            map.insert(src, dst);
        }

        update_note_time(notes, range, &map, &mut last_note_time);
        apply_timeslice_map(notes, range, &map, &mut ln_active);
    }
}

/// S-Random Playable (PMS variant): S-Random with murioshi prevention.
///
/// Corresponds to `NoMurioshiRandomizer` in beatoraja.
pub fn s_random_playable(
    notes: &mut [Note],
    mode: PlayMode,
    player: usize,
    include_scratch: bool,
    threshold_us: i64,
    rng: &mut impl Rng,
) {
    let keys = get_keys(mode, player, include_scratch);
    if keys.is_empty() {
        return;
    }

    // No-murioshi 6-button combination table (0-indexed)
    let button_combinations: Vec<Vec<usize>> = vec![
        vec![0, 1, 2, 3, 4, 5],
        vec![0, 1, 2, 4, 5, 6],
        vec![0, 1, 2, 5, 6, 7],
        vec![0, 1, 2, 6, 7, 8],
        vec![1, 2, 3, 4, 5, 6],
        vec![1, 2, 3, 5, 6, 7],
        vec![1, 2, 3, 6, 7, 8],
        vec![2, 3, 4, 5, 6, 7],
        vec![2, 3, 4, 6, 7, 8],
        vec![3, 4, 5, 6, 7, 8],
    ];

    let groups = group_notes_by_time(notes);

    let mut ln_active: HashMap<usize, usize> = HashMap::new();
    let mut last_note_time: HashMap<usize, i64> = HashMap::new();
    for &k in &keys {
        last_note_time.insert(k, -10_000_000);
    }

    for (time, range) in &groups {
        let time = *time;

        let changeable: Vec<usize> = keys
            .iter()
            .copied()
            .filter(|k| !ln_active.contains_key(k))
            .collect();
        let assignable: Vec<usize> = changeable.clone();

        let note_count = count_playable_notes(notes, range, &keys) + ln_active.len();

        let mut map = if note_count > 2 && note_count < 7 {
            let ln_lanes: Vec<usize> = ln_active.values().copied().collect();
            let candidates: Vec<&Vec<usize>> = if ln_lanes.is_empty() {
                button_combinations.iter().collect()
            } else {
                button_combinations
                    .iter()
                    .filter(|combo| ln_lanes.iter().all(|l| combo.contains(l)))
                    .collect()
            };

            if !candidates.is_empty() {
                let renda_lanes: Vec<usize> = last_note_time
                    .iter()
                    .filter(|&(_, lt)| time - *lt < threshold_us)
                    .map(|(&lane, _)| lane)
                    .collect();
                let candidates2: Vec<Vec<usize>> = candidates
                    .iter()
                    .map(|combo| {
                        combo
                            .iter()
                            .copied()
                            .filter(|l| !renda_lanes.contains(l))
                            .collect::<Vec<usize>>()
                    })
                    .filter(|filtered| filtered.len() >= note_count)
                    .collect();

                if !candidates2.is_empty() {
                    let chosen = &candidates2[rng.random_range(0..candidates2.len())];
                    build_time_based_map_with_preference(
                        notes,
                        range,
                        &changeable,
                        &assignable,
                        rng,
                        chosen,
                    )
                } else {
                    let params = ShuffleParams {
                        notes,
                        range: range.clone(),
                        time,
                        changeable: changeable.clone(),
                        assignable: assignable.clone(),
                        threshold_us,
                        last_note_time: &last_note_time,
                    };
                    build_time_based_map(&params, rng)
                }
            } else {
                let params = ShuffleParams {
                    notes,
                    range: range.clone(),
                    time,
                    changeable: changeable.clone(),
                    assignable: assignable.clone(),
                    threshold_us,
                    last_note_time: &last_note_time,
                };
                build_time_based_map(&params, rng)
            }
        } else {
            let params = ShuffleParams {
                notes,
                range: range.clone(),
                time,
                changeable: changeable.clone(),
                assignable: assignable.clone(),
                threshold_us,
                last_note_time: &last_note_time,
            };
            build_time_based_map(&params, rng)
        };

        for (&src, &dst) in &ln_active {
            map.insert(src, dst);
        }

        update_note_time(notes, range, &map, &mut last_note_time);
        apply_timeslice_map(notes, range, &map, &mut ln_active);
    }
}

// ===== Internal helpers =====

/// Groups note indices by their time_us value.
/// Returns (time_us, Range<usize>) pairs in time order.
/// Assumes notes are sorted by time_us.
fn group_notes_by_time(notes: &[Note]) -> Vec<(i64, std::ops::Range<usize>)> {
    if notes.is_empty() {
        return Vec::new();
    }
    let mut groups = Vec::new();
    let mut start = 0;
    let mut current_time = notes[0].time_us;
    for (i, note) in notes.iter().enumerate() {
        if note.time_us != current_time {
            groups.push((current_time, start..i));
            start = i;
            current_time = note.time_us;
        }
    }
    groups.push((current_time, start..notes.len()));
    groups
}

/// Core time-based shuffle logic.
///
/// For each timeslice, assigns notes with playable content to lanes that haven't
/// had a note within `threshold_us`, falling back to the least-recently-used lane.
///
/// Corresponds to `TimeBasedRandomizer.timeBasedShuffle()` in beatoraja.
fn time_based_shuffle(notes: &mut [Note], keys: &[usize], threshold_us: i64, rng: &mut impl Rng) {
    let groups = group_notes_by_time(notes);

    let mut ln_active: HashMap<usize, usize> = HashMap::new();
    let mut last_note_time: HashMap<usize, i64> = HashMap::new();
    for &k in keys {
        last_note_time.insert(k, -10_000_000);
    }

    for (time, range) in &groups {
        let time = *time;
        let changeable: Vec<usize> = keys
            .iter()
            .copied()
            .filter(|k| !ln_active.contains_key(k))
            .collect();
        let assignable: Vec<usize> = changeable.clone();

        let params = ShuffleParams {
            notes,
            range: range.clone(),
            time,
            changeable,
            assignable,
            threshold_us,
            last_note_time: &last_note_time,
        };
        let mut map = build_time_based_map(&params, rng);

        for (&src, &dst) in &ln_active {
            map.insert(src, dst);
        }

        update_note_time(notes, range, &map, &mut last_note_time);
        apply_timeslice_map(notes, range, &map, &mut ln_active);
    }
}

/// Build a lane mapping for one timeslice using time-based logic.
fn build_time_based_map(params: &ShuffleParams<'_>, rng: &mut impl Rng) -> HashMap<usize, usize> {
    let mut map: HashMap<usize, usize> = HashMap::new();

    let mut note_lanes: Vec<usize> = Vec::new();
    let mut empty_lanes: Vec<usize> = Vec::new();
    for &cl in &params.changeable {
        let has_note = params.notes[params.range.clone()]
            .iter()
            .any(|n| n.lane == cl && is_playable(n.note_type));
        if has_note {
            note_lanes.push(cl);
        } else {
            empty_lanes.push(cl);
        }
    }

    let mut primary: Vec<usize> = Vec::new();
    let mut inferior: Vec<usize> = Vec::new();
    for &al in &params.assignable {
        let last = params
            .last_note_time
            .get(&al)
            .copied()
            .unwrap_or(-10_000_000);
        if params.time - last > params.threshold_us {
            primary.push(al);
        } else {
            inferior.push(al);
        }
    }

    // Assign note lanes to primary (beyond-threshold) lanes first
    while !note_lanes.is_empty() && !primary.is_empty() {
        let r = rng.random_range(0..primary.len());
        let src = note_lanes.remove(0);
        let dst = primary.remove(r);
        map.insert(src, dst);
    }

    // Remaining note lanes: assign to inferior lane with smallest last_note_time
    while !note_lanes.is_empty() && !inferior.is_empty() {
        let min_time = inferior
            .iter()
            .map(|l| params.last_note_time.get(l).copied().unwrap_or(-10_000_000))
            .min()
            .unwrap();
        let min_lanes: Vec<usize> = inferior
            .iter()
            .copied()
            .filter(|l| params.last_note_time.get(l).copied().unwrap_or(-10_000_000) == min_time)
            .collect();
        let chosen = min_lanes[rng.random_range(0..min_lanes.len())];
        let src = note_lanes.remove(0);
        map.insert(src, chosen);
        inferior.retain(|&l| l != chosen);
    }

    // Remaining empty lanes: assign randomly to leftover assignable lanes
    primary.extend(inferior);
    while !empty_lanes.is_empty() && !primary.is_empty() {
        let r = rng.random_range(0..primary.len());
        let src = empty_lanes.remove(0);
        let dst = primary.remove(r);
        map.insert(src, dst);
    }

    map
}

/// Build a time-based map with preference for certain lanes (for murioshi prevention).
#[allow(clippy::too_many_arguments)]
fn build_time_based_map_with_preference(
    notes: &[Note],
    range: &std::ops::Range<usize>,
    changeable: &[usize],
    assignable: &[usize],
    rng: &mut impl Rng,
    preferred: &[usize],
) -> HashMap<usize, usize> {
    let mut map: HashMap<usize, usize> = HashMap::new();

    let mut note_lanes: Vec<usize> = Vec::new();
    let mut empty_lanes: Vec<usize> = Vec::new();
    for &cl in changeable {
        let has_note = notes[range.clone()]
            .iter()
            .any(|n| n.lane == cl && is_playable(n.note_type));
        if has_note {
            note_lanes.push(cl);
        } else {
            empty_lanes.push(cl);
        }
    }

    let mut primary: Vec<usize> = assignable
        .iter()
        .copied()
        .filter(|a| preferred.contains(a))
        .collect();
    let mut other: Vec<usize> = assignable
        .iter()
        .copied()
        .filter(|a| !preferred.contains(a))
        .collect();

    while !note_lanes.is_empty() && !primary.is_empty() {
        let r = rng.random_range(0..primary.len());
        let src = note_lanes.remove(0);
        let dst = primary.remove(r);
        map.insert(src, dst);
    }

    while !note_lanes.is_empty() && !other.is_empty() {
        let r = rng.random_range(0..other.len());
        let src = note_lanes.remove(0);
        let dst = other.remove(r);
        map.insert(src, dst);
    }

    primary.extend(other);
    while !empty_lanes.is_empty() && !primary.is_empty() {
        let r = rng.random_range(0..primary.len());
        let src = empty_lanes.remove(0);
        let dst = primary.remove(r);
        map.insert(src, dst);
    }

    map
}

/// Build a time-based map for Converge randomizer.
#[allow(clippy::too_many_arguments)]
fn build_time_based_map_converge(
    notes: &[Note],
    range: &std::ops::Range<usize>,
    time: i64,
    changeable: &[usize],
    assignable: &[usize],
    threshold_us: i64,
    last_note_time: &HashMap<usize, i64>,
    renda_count: &mut HashMap<usize, usize>,
    rng: &mut impl Rng,
) -> HashMap<usize, usize> {
    let mut map: HashMap<usize, usize> = HashMap::new();

    let mut note_lanes: Vec<usize> = Vec::new();
    let mut empty_lanes: Vec<usize> = Vec::new();
    for &cl in changeable {
        let has_note = notes[range.clone()]
            .iter()
            .any(|n| n.lane == cl && is_playable(n.note_type));
        if has_note {
            note_lanes.push(cl);
        } else {
            empty_lanes.push(cl);
        }
    }

    let mut primary: Vec<usize> = Vec::new();
    let mut inferior: Vec<usize> = Vec::new();
    for &al in assignable {
        let last = last_note_time.get(&al).copied().unwrap_or(-10_000_000);
        if time - last > threshold_us {
            primary.push(al);
        } else {
            inferior.push(al);
        }
    }

    // For Converge: prefer lanes with highest renda count
    while !note_lanes.is_empty() && !primary.is_empty() {
        let max_renda = primary
            .iter()
            .map(|l| renda_count.get(l).copied().unwrap_or(0))
            .max()
            .unwrap();
        let max_lanes: Vec<usize> = primary
            .iter()
            .copied()
            .filter(|l| renda_count.get(l).copied().unwrap_or(0) == max_renda)
            .collect();
        let chosen = max_lanes[rng.random_range(0..max_lanes.len())];
        *renda_count.entry(chosen).or_insert(0) += 1;
        let src = note_lanes.remove(0);
        primary.retain(|&l| l != chosen);
        map.insert(src, chosen);
    }

    while !note_lanes.is_empty() && !inferior.is_empty() {
        let min_time = inferior
            .iter()
            .map(|l| last_note_time.get(l).copied().unwrap_or(-10_000_000))
            .min()
            .unwrap();
        let min_lanes: Vec<usize> = inferior
            .iter()
            .copied()
            .filter(|l| last_note_time.get(l).copied().unwrap_or(-10_000_000) == min_time)
            .collect();
        let chosen = min_lanes[rng.random_range(0..min_lanes.len())];
        let src = note_lanes.remove(0);
        map.insert(src, chosen);
        inferior.retain(|&l| l != chosen);
    }

    primary.extend(inferior);
    while !empty_lanes.is_empty() && !primary.is_empty() {
        let r = rng.random_range(0..primary.len());
        let src = empty_lanes.remove(0);
        let dst = primary.remove(r);
        map.insert(src, dst);
    }

    map
}

/// Apply a lane mapping to notes in a timeslice range and update LN tracking.
fn apply_timeslice_map(
    notes: &mut [Note],
    range: &std::ops::Range<usize>,
    map: &HashMap<usize, usize>,
    ln_active: &mut HashMap<usize, usize>,
) {
    for note in &mut notes[range.clone()] {
        if let Some(&new_lane) = map.get(&note.lane) {
            let old_lane = note.lane;
            note.lane = new_lane;

            if note.is_long_note() {
                if let Entry::Vacant(e) = ln_active.entry(old_lane) {
                    // LN starting
                    e.insert(new_lane);
                } else {
                    // LN ending
                    ln_active.remove(&old_lane);
                }
            }
        }
    }
}

/// Update last_note_time for lanes that received a playable note.
fn update_note_time(
    notes: &[Note],
    range: &std::ops::Range<usize>,
    map: &HashMap<usize, usize>,
    last_note_time: &mut HashMap<usize, i64>,
) {
    for note in &notes[range.clone()] {
        if is_playable(note.note_type)
            && let Some(&dst) = map.get(&note.lane)
        {
            last_note_time.insert(dst, note.time_us);
        }
    }
}

/// Count playable notes in a timeslice that are on modify lanes.
fn count_playable_notes(notes: &[Note], range: &std::ops::Range<usize>, keys: &[usize]) -> usize {
    notes[range.clone()]
        .iter()
        .filter(|n| keys.contains(&n.lane) && is_playable(n.note_type))
        .count()
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

    #[test]
    fn group_by_time_basic() {
        let notes = vec![
            normal_note(0, 0),
            normal_note(1, 0),
            normal_note(2, 1000),
            normal_note(3, 2000),
        ];
        let groups = group_notes_by_time(&notes);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], (0, 0..2));
        assert_eq!(groups[1], (1000, 2..3));
        assert_eq!(groups[2], (2000, 3..4));
    }

    #[test]
    fn group_by_time_empty() {
        let notes: Vec<Note> = Vec::new();
        let groups = group_notes_by_time(&notes);
        assert!(groups.is_empty());
    }

    #[test]
    fn s_random_assigns_within_keys() {
        let mut notes: Vec<Note> = (0..7).map(|i| normal_note(i, i as i64 * 100_000)).collect();
        let mut rng = SmallRng::seed_from_u64(42);
        s_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        let keys = get_keys(PlayMode::Beat7K, 0, false);
        for n in &notes {
            assert!(
                keys.contains(&n.lane),
                "Lane {} not in keys {:?}",
                n.lane,
                keys
            );
        }
    }

    #[test]
    fn s_random_ln_tracking() {
        let mut notes = vec![
            long_note(0, 0, 2_000_000),
            normal_note(1, 1_000_000),
            normal_note(2, 1_000_000),
        ];
        let mut rng = SmallRng::seed_from_u64(42);
        s_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);

        let ln_lane = notes[0].lane;
        for n in &notes[1..] {
            assert_ne!(
                n.lane, ln_lane,
                "Note at time {} should not share LN lane {}",
                n.time_us, ln_lane
            );
        }
    }

    #[test]
    fn h_random_with_threshold() {
        let mut notes: Vec<Note> = (0..14)
            .map(|i| normal_note(i % 7, i as i64 * 50_000))
            .collect();
        let mut rng = SmallRng::seed_from_u64(42);
        h_random(&mut notes, PlayMode::Beat7K, 0, false, 100_000, &mut rng);
        assert_eq!(notes.len(), 14);
    }

    #[test]
    fn spiral_rotates_lanes() {
        let mut notes: Vec<Note> = (0..7).map(|i| normal_note(i, i as i64 * 100_000)).collect();
        let original_lanes: Vec<usize> = notes.iter().map(|n| n.lane).collect();
        let mut rng = SmallRng::seed_from_u64(42);
        spiral(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        let new_lanes: Vec<usize> = notes.iter().map(|n| n.lane).collect();
        assert_ne!(
            original_lanes, new_lanes,
            "Spiral should change lane assignment"
        );
    }

    #[test]
    fn all_scratch_assigns_to_scratch() {
        let mut notes: Vec<Note> = (0..10)
            .map(|i| normal_note(i % 7, i as i64 * 200_000))
            .collect();
        let mut rng = SmallRng::seed_from_u64(42);
        all_scratch(&mut notes, PlayMode::Beat7K, 0, 100_000, &mut rng);
        let scratch_count = notes.iter().filter(|n| n.lane == 7).count();
        assert!(
            scratch_count > 0,
            "All-SCR should assign some notes to scratch lane"
        );
    }

    #[test]
    fn mine_notes_not_shuffled_to_note_lanes() {
        let mut notes = vec![mine_note(0, 0), normal_note(1, 0), normal_note(2, 0)];
        let mut rng = SmallRng::seed_from_u64(42);
        s_random(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(notes.len(), 3);
        assert_eq!(
            notes
                .iter()
                .filter(|n| n.note_type == NoteType::Mine)
                .count(),
            1
        );
    }

    #[test]
    fn s_random_no_threshold_works() {
        let mut notes: Vec<Note> = (0..10)
            .map(|i| normal_note(i % 7, i as i64 * 10_000))
            .collect();
        let mut rng = SmallRng::seed_from_u64(42);
        s_random_no_threshold(&mut notes, PlayMode::Beat7K, 0, false, &mut rng);
        assert_eq!(notes.len(), 10);
    }

    #[test]
    fn converge_basic() {
        let mut notes: Vec<Note> = (0..10)
            .map(|i| normal_note(i % 7, i as i64 * 50_000))
            .collect();
        let mut rng = SmallRng::seed_from_u64(42);
        converge(&mut notes, PlayMode::Beat7K, 0, false, 100_000, &mut rng);
        assert_eq!(notes.len(), 10);
    }

    #[test]
    fn s_random_playable_basic() {
        let mut notes: Vec<Note> = (0..9).map(|i| normal_note(i, i as i64 * 100_000)).collect();
        let mut rng = SmallRng::seed_from_u64(42);
        s_random_playable(&mut notes, PlayMode::PopN9K, 0, false, 100_000, &mut rng);
        assert_eq!(notes.len(), 9);
    }
}
