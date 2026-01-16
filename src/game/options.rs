use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::bms::{Chart, NoteChannel, PlayMode};

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
    HRandom,
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
            Self::HRandom => "H-RANDOM",
        }
    }
}

/// Lane mapping for random options (BMS 7-key mode)
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
            RandomOption::SRandom | RandomOption::HRandom => {
                // S-RANDOM and H-RANDOM are not handled by LaneMapping
                // They should be handled by apply_s_random/apply_h_random directly
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

/// Lane mapping for PMS 9-key mode
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LaneMapping9Key {
    /// Mapping from original key lane (0-8) to new lane (0-8)
    map: [usize; 9],
}

impl LaneMapping9Key {
    /// Create identity mapping (no change)
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            map: [0, 1, 2, 3, 4, 5, 6, 7, 8],
        }
    }

    /// Create mirror mapping (reverse keys)
    #[allow(dead_code)]
    pub fn mirror() -> Self {
        // Key1 <-> Key9, Key2 <-> Key8, Key3 <-> Key7, Key4 <-> Key6, Key5 stays
        Self {
            map: [8, 7, 6, 5, 4, 3, 2, 1, 0],
        }
    }

    /// Create random mapping using Fisher-Yates shuffle
    #[allow(dead_code)]
    pub fn random(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut lanes: Vec<usize> = (0..9).collect();

        // Fisher-Yates shuffle
        for i in (1..lanes.len()).rev() {
            let j = rng.random_range(0..=i);
            lanes.swap(i, j);
        }

        Self {
            map: [
                lanes[0], lanes[1], lanes[2], lanes[3], lanes[4], lanes[5], lanes[6], lanes[7],
                lanes[8],
            ],
        }
    }

    /// Create rotate-random mapping
    #[allow(dead_code)]
    pub fn rotate_random(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let offset = rng.random_range(0..9);

        let mut map = [0usize; 9];
        for (i, item) in map.iter_mut().enumerate() {
            *item = (i + offset) % 9;
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
            RandomOption::SRandom | RandomOption::HRandom => {
                // S-RANDOM and H-RANDOM are not handled by LaneMapping9Key
                Self::identity()
            }
        }
    }

    /// Transform a PMS lane (0-8) to the new lane
    #[allow(dead_code)]
    pub fn transform(&self, lane: usize) -> usize {
        if lane < 9 { self.map[lane] } else { lane }
    }
}

/// Apply random option to a chart
#[allow(dead_code)]
pub fn apply_random_option(chart: &mut Chart, option: RandomOption, seed: u64) {
    if option == RandomOption::Off {
        return;
    }

    let play_mode = chart.metadata.play_mode;

    if option == RandomOption::SRandom {
        apply_s_random(chart, seed, play_mode);
        return;
    }

    if option == RandomOption::HRandom {
        apply_h_random(chart, seed, play_mode);
        return;
    }

    match play_mode {
        PlayMode::Bms7Key => {
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
        }
        PlayMode::Pms9Key => {
            let mapping = LaneMapping9Key::for_option(option, seed);
            for note in &mut chart.notes {
                if note.channel.is_key() {
                    let original_lane = note.channel.lane_index_for_mode(PlayMode::Pms9Key);
                    let new_lane = mapping.transform(original_lane);
                    if let Some(new_channel) = NoteChannel::from_pms_lane(new_lane) {
                        note.channel = new_channel;
                    }
                }
            }
        }
        PlayMode::Dp14Key => {
            // For DP mode, apply MIRROR to both sides independently
            if option == RandomOption::Mirror {
                apply_mirror_dp(chart);
            }
            // RANDOM for DP is not yet fully implemented
            // For now, just use MIRROR for other options too
        }
    }

    // Rebuild lane index after modifying notes
    // Note: This is handled by the caller if needed
}

/// Apply S-RANDOM option to a chart
/// Each note gets an independent random lane, but LN start/end pairs stay together
fn apply_s_random(chart: &mut Chart, seed: u64, play_mode: PlayMode) {
    use crate::bms::NoteType;
    use std::collections::HashMap;

    let mut rng = StdRng::seed_from_u64(seed);

    // First pass: identify LN pairs by index
    // Key: index of LongStart note, Value: index of corresponding LongEnd note
    let ln_pairs = find_ln_pairs(chart, play_mode);

    // Map from LongStart index to new lane
    let mut ln_start_lanes: HashMap<usize, usize> = HashMap::new();

    #[allow(clippy::type_complexity)]
    let (lane_range, lane_converter): (
        std::ops::RangeInclusive<usize>,
        fn(usize) -> Option<NoteChannel>,
    ) = match play_mode {
        PlayMode::Bms7Key => (1..=7, NoteChannel::from_key_lane),
        PlayMode::Pms9Key => (0..=8, NoteChannel::from_pms_lane),
        PlayMode::Dp14Key => (1..=14, NoteChannel::from_dp_lane),
    };

    for (i, note) in chart.notes.iter_mut().enumerate() {
        if !note.channel.is_key() {
            continue;
        }

        let new_lane = match note.note_type {
            NoteType::LongStart => {
                // Generate new random lane for LN start
                let new_lane = rng.random_range(lane_range.clone());
                // Store mapping for the corresponding LN end
                ln_start_lanes.insert(i, new_lane);
                new_lane
            }
            NoteType::LongEnd => {
                // Find the corresponding LN start's lane
                if let Some(&start_idx) = ln_pairs.get(&i) {
                    // Use the same lane as the LN start
                    ln_start_lanes.get(&start_idx).copied().unwrap_or_else(|| {
                        // Fallback if start wasn't processed (shouldn't happen)
                        rng.random_range(lane_range.clone())
                    })
                } else {
                    // Orphan LN end: generate random lane
                    rng.random_range(lane_range.clone())
                }
            }
            NoteType::Normal | NoteType::Invisible | NoteType::Landmine => {
                // Normal notes get independent random lanes
                rng.random_range(lane_range.clone())
            }
        };

        if let Some(new_channel) = lane_converter(new_lane) {
            note.channel = new_channel;
        }
    }
}

/// Find LN pairs by matching LongStart and LongEnd notes on the same lane
/// Returns a map from LongEnd index to LongStart index
fn find_ln_pairs(chart: &Chart, play_mode: PlayMode) -> HashMap<usize, usize> {
    use crate::bms::NoteType;
    use std::collections::HashMap;

    // Track active LN starts per lane: lane -> stack of (start_index, time_ms)
    let mut active_ln_starts: HashMap<usize, Vec<(usize, f64)>> = HashMap::new();

    // Result: LongEnd index -> LongStart index
    let mut pairs: HashMap<usize, usize> = HashMap::new();

    for (i, note) in chart.notes.iter().enumerate() {
        if !note.channel.is_key() {
            continue;
        }

        let lane = note.channel.lane_index_for_mode(play_mode);

        match note.note_type {
            NoteType::LongStart => {
                // Push to stack for this lane
                active_ln_starts
                    .entry(lane)
                    .or_default()
                    .push((i, note.time_ms));
            }
            NoteType::LongEnd => {
                // Find the earliest LN start on this lane that started before this end
                if let Some(starts) = active_ln_starts.get_mut(&lane) {
                    // Find the first start that is before this end (FIFO)
                    if let Some(pos) = starts.iter().position(|(_, t)| *t < note.time_ms) {
                        let (start_idx, _) = starts.remove(pos);
                        pairs.insert(i, start_idx);
                    }
                }
            }
            _ => {}
        }
    }

    pairs
}

/// Apply H-RANDOM option to a chart
/// Similar to S-RANDOM but avoids placing notes on the same lane as the previous note
/// to reduce consecutive same-lane patterns (縦連打)
fn apply_h_random(chart: &mut Chart, seed: u64, play_mode: PlayMode) {
    use crate::bms::NoteType;
    use std::collections::HashMap;

    let mut rng = StdRng::seed_from_u64(seed);

    // First pass: identify LN pairs by index
    let ln_pairs = find_ln_pairs(chart, play_mode);

    // Map from LongStart index to new lane
    let mut ln_start_lanes: HashMap<usize, usize> = HashMap::new();

    // Track the last assigned lane to avoid consecutive same-lane notes
    let mut last_lane: Option<usize> = None;

    let lane_converter: fn(usize) -> Option<NoteChannel> = match play_mode {
        PlayMode::Bms7Key => NoteChannel::from_key_lane,
        PlayMode::Pms9Key => NoteChannel::from_pms_lane,
        PlayMode::Dp14Key => NoteChannel::from_dp_lane,
    };

    for (i, note) in chart.notes.iter_mut().enumerate() {
        if !note.channel.is_key() {
            continue;
        }

        let new_lane = match note.note_type {
            NoteType::LongStart => {
                // Generate new random lane for LN start, avoiding last lane if possible
                let new_lane = pick_random_lane_avoiding(&mut rng, last_lane, play_mode);
                ln_start_lanes.insert(i, new_lane);
                last_lane = Some(new_lane);
                new_lane
            }
            NoteType::LongEnd => {
                // Find the corresponding LN start's lane
                if let Some(&start_idx) = ln_pairs.get(&i) {
                    // Use the same lane as the LN start
                    // Don't update last_lane for LN end since it's paired with start
                    ln_start_lanes.get(&start_idx).copied().unwrap_or_else(|| {
                        let new_lane = pick_random_lane_avoiding(&mut rng, last_lane, play_mode);
                        last_lane = Some(new_lane);
                        new_lane
                    })
                } else {
                    // Orphan LN end: generate random lane
                    let new_lane = pick_random_lane_avoiding(&mut rng, last_lane, play_mode);
                    last_lane = Some(new_lane);
                    new_lane
                }
            }
            NoteType::Normal | NoteType::Invisible | NoteType::Landmine => {
                // Normal notes get random lanes avoiding the last lane
                let new_lane = pick_random_lane_avoiding(&mut rng, last_lane, play_mode);
                last_lane = Some(new_lane);
                new_lane
            }
        };

        if let Some(new_channel) = lane_converter(new_lane) {
            note.channel = new_channel;
        }
    }
}

/// Pick a random lane avoiding the specified lane if possible
fn pick_random_lane_avoiding(rng: &mut StdRng, avoid: Option<usize>, play_mode: PlayMode) -> usize {
    let (lane_range, lane_count) = match play_mode {
        PlayMode::Bms7Key => (1..=7usize, 7),
        PlayMode::Pms9Key => (0..=8usize, 9),
        PlayMode::Dp14Key => (1..=14usize, 14),
    };

    match avoid {
        Some(avoid_lane) if lane_range.contains(&avoid_lane) => {
            // Pick from available lanes excluding avoid_lane
            let available: Vec<usize> = lane_range.filter(|&l| l != avoid_lane).collect();
            available[rng.random_range(0..available.len())]
        }
        _ => {
            // No lane to avoid, pick any lane
            match play_mode {
                PlayMode::Bms7Key => rng.random_range(1..=lane_count),
                PlayMode::Pms9Key => rng.random_range(0..lane_count),
                PlayMode::Dp14Key => rng.random_range(1..=lane_count),
            }
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

/// Apply legacy note option to a chart (convert LN to normal notes)
/// LongStart notes are converted to Normal notes, LongEnd notes are removed
#[allow(dead_code)]
pub fn apply_legacy_note(chart: &mut Chart) {
    use crate::bms::NoteType;

    // Convert LongStart to Normal and mark LongEnd for removal
    for note in &mut chart.notes {
        if note.note_type == NoteType::LongStart {
            note.note_type = NoteType::Normal;
            note.long_end_time_ms = None;
        }
    }

    // Remove LongEnd notes
    chart
        .notes
        .retain(|note| note.note_type != NoteType::LongEnd);
}

/// Apply battle option to a chart (flip to 2P side layout)
/// In SP, this mirrors the key lanes and moves scratch to the right side
/// Layout change: S 1 2 3 4 5 6 7 -> 7 6 5 4 3 2 1 S
#[allow(dead_code)]
pub fn apply_battle(chart: &mut Chart) {
    for note in &mut chart.notes {
        let new_channel = match note.channel {
            // Scratch moves to lane index 8 (rendered on right side)
            NoteChannel::Scratch => NoteChannel::Scratch, // Scratch stays as Scratch but position changes
            // Keys are mirrored: 1<->7, 2<->6, 3<->5, 4 stays
            NoteChannel::Key1 => NoteChannel::Key7,
            NoteChannel::Key2 => NoteChannel::Key6,
            NoteChannel::Key3 => NoteChannel::Key5,
            NoteChannel::Key4 => NoteChannel::Key4,
            NoteChannel::Key5 => NoteChannel::Key3,
            NoteChannel::Key6 => NoteChannel::Key2,
            NoteChannel::Key7 => NoteChannel::Key1,
            // DP P2 side and PMS channels keep as-is for this SP option
            other => other,
        };
        note.channel = new_channel;
    }
}

/// Apply MIRROR option for DP mode
/// Mirrors P1 and P2 sides independently
/// P1: S 1 2 3 4 5 6 7 -> S 7 6 5 4 3 2 1
/// P2: 1 2 3 4 5 6 7 S -> 7 6 5 4 3 2 1 S
#[allow(dead_code)]
pub fn apply_mirror_dp(chart: &mut Chart) {
    for note in &mut chart.notes {
        let new_channel = match note.channel {
            // P1 side: Scratch stays, keys mirror
            NoteChannel::Scratch => NoteChannel::Scratch,
            NoteChannel::Key1 => NoteChannel::Key7,
            NoteChannel::Key2 => NoteChannel::Key6,
            NoteChannel::Key3 => NoteChannel::Key5,
            NoteChannel::Key4 => NoteChannel::Key4,
            NoteChannel::Key5 => NoteChannel::Key3,
            NoteChannel::Key6 => NoteChannel::Key2,
            NoteChannel::Key7 => NoteChannel::Key1,
            // P2 side: Keys mirror, scratch stays
            NoteChannel::Key8 => NoteChannel::Key14, // P2 Key1 <-> P2 Key7
            NoteChannel::Key9 => NoteChannel::Key13, // P2 Key2 <-> P2 Key6
            NoteChannel::Key10 => NoteChannel::Key12, // P2 Key3 <-> P2 Key5
            NoteChannel::Key11 => NoteChannel::Key11, // P2 Key4 stays
            NoteChannel::Key12 => NoteChannel::Key10, // P2 Key5 <-> P2 Key3
            NoteChannel::Key13 => NoteChannel::Key9, // P2 Key6 <-> P2 Key2
            NoteChannel::Key14 => NoteChannel::Key8, // P2 Key7 <-> P2 Key1
            NoteChannel::Scratch2 => NoteChannel::Scratch2,
        };
        note.channel = new_channel;
    }
}

/// Apply FLIP option for DP mode
/// Swaps P1 and P2 sides entirely
/// P1 (S 1 2 3 4 5 6 7) <-> P2 (1 2 3 4 5 6 7 S)
#[allow(dead_code)]
pub fn apply_flip(chart: &mut Chart) {
    for note in &mut chart.notes {
        let new_channel = match note.channel {
            // P1 -> P2
            NoteChannel::Scratch => NoteChannel::Scratch2,
            NoteChannel::Key1 => NoteChannel::Key8,
            NoteChannel::Key2 => NoteChannel::Key9,
            NoteChannel::Key3 => NoteChannel::Key10,
            NoteChannel::Key4 => NoteChannel::Key11,
            NoteChannel::Key5 => NoteChannel::Key12,
            NoteChannel::Key6 => NoteChannel::Key13,
            NoteChannel::Key7 => NoteChannel::Key14,
            // P2 -> P1
            NoteChannel::Key8 => NoteChannel::Key1,
            NoteChannel::Key9 => NoteChannel::Key2,
            NoteChannel::Key10 => NoteChannel::Key3,
            NoteChannel::Key11 => NoteChannel::Key4,
            NoteChannel::Key12 => NoteChannel::Key5,
            NoteChannel::Key13 => NoteChannel::Key6,
            NoteChannel::Key14 => NoteChannel::Key7,
            NoteChannel::Scratch2 => NoteChannel::Scratch,
        };
        note.channel = new_channel;
    }
}

/// Lane mapping for DP 14-key mode
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LaneMappingDp {
    /// Mapping for P1 keys (indices 1-7 map Key1-Key7, 0 is Scratch)
    p1_map: [usize; 8],
    /// Mapping for P2 keys (indices 0-6 map Key8-Key14, 7 is Scratch2)
    p2_map: [usize; 8],
}

impl LaneMappingDp {
    /// Create identity mapping (no change)
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            p1_map: [0, 1, 2, 3, 4, 5, 6, 7],
            p2_map: [0, 1, 2, 3, 4, 5, 6, 7],
        }
    }

    /// Create mirror mapping (P1 and P2 mirrored independently)
    #[allow(dead_code)]
    pub fn mirror() -> Self {
        Self {
            p1_map: [0, 7, 6, 5, 4, 3, 2, 1], // Scratch stays, keys mirror
            p2_map: [6, 5, 4, 3, 2, 1, 0, 7], // Keys mirror, scratch stays
        }
    }

    /// Create random mapping for both sides
    #[allow(dead_code)]
    pub fn random(seed: u64) -> Self {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(seed);

        // P1 side: shuffle keys 1-7
        let mut p1_keys: Vec<usize> = (1..=7).collect();
        for i in (1..p1_keys.len()).rev() {
            let j = rng.random_range(0..=i);
            p1_keys.swap(i, j);
        }

        // P2 side: shuffle keys 0-6
        let mut p2_keys: Vec<usize> = (0..=6).collect();
        for i in (1..p2_keys.len()).rev() {
            let j = rng.random_range(0..=i);
            p2_keys.swap(i, j);
        }

        Self {
            p1_map: [
                0, p1_keys[0], p1_keys[1], p1_keys[2], p1_keys[3], p1_keys[4], p1_keys[5],
                p1_keys[6],
            ],
            p2_map: [
                p2_keys[0], p2_keys[1], p2_keys[2], p2_keys[3], p2_keys[4], p2_keys[5], p2_keys[6],
                7,
            ],
        }
    }
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

    #[test]
    fn test_h_random_avoids_consecutive_same_lane() {
        // Create many consecutive notes on the same lane
        let notes: Vec<Note> = (0..20)
            .map(|i| create_test_note(NoteChannel::Key1, NoteType::Normal, i as f64 * 100.0))
            .collect();
        let mut chart = create_test_chart(notes);

        apply_random_option(&mut chart, RandomOption::HRandom, 12345);

        // Check that no two consecutive notes are on the same lane
        for i in 1..chart.notes.len() {
            assert_ne!(
                chart.notes[i - 1].channel,
                chart.notes[i].channel,
                "Notes {} and {} should not be on the same lane",
                i - 1,
                i
            );
        }
    }

    #[test]
    fn test_h_random_is_deterministic() {
        let notes = vec![
            create_test_note(NoteChannel::Key1, NoteType::Normal, 0.0),
            create_test_note(NoteChannel::Key2, NoteType::Normal, 100.0),
            create_test_note(NoteChannel::Key3, NoteType::Normal, 200.0),
        ];

        let mut chart1 = create_test_chart(notes.clone());
        let mut chart2 = create_test_chart(notes);

        apply_random_option(&mut chart1, RandomOption::HRandom, 12345);
        apply_random_option(&mut chart2, RandomOption::HRandom, 12345);

        for (n1, n2) in chart1.notes.iter().zip(chart2.notes.iter()) {
            assert_eq!(n1.channel, n2.channel);
        }
    }

    #[test]
    fn test_h_random_ln_consistency() {
        let notes = vec![
            create_test_note(NoteChannel::Key1, NoteType::LongStart, 0.0),
            create_test_note(NoteChannel::Key1, NoteType::LongEnd, 500.0),
        ];
        let mut chart = create_test_chart(notes);

        apply_random_option(&mut chart, RandomOption::HRandom, 12345);

        // LN start and end should be on the same lane
        assert_eq!(
            chart.notes[0].channel, chart.notes[1].channel,
            "LN start and end should be on the same lane"
        );
    }

    #[test]
    fn test_h_random_scratch_unchanged() {
        let notes = vec![
            create_test_note(NoteChannel::Scratch, NoteType::Normal, 0.0),
            create_test_note(NoteChannel::Key1, NoteType::Normal, 100.0),
        ];
        let mut chart = create_test_chart(notes);

        apply_random_option(&mut chart, RandomOption::HRandom, 12345);

        assert_eq!(
            chart.notes[0].channel,
            NoteChannel::Scratch,
            "Scratch should not be randomized"
        );
    }

    #[test]
    fn test_h_random_display_name() {
        assert_eq!(RandomOption::HRandom.display_name(), "H-RANDOM");
    }
}
