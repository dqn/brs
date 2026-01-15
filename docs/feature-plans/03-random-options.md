# Random Options Implementation

## Overview

Random options modify note lane assignments to provide variety and help players practice different patterns. These are essential features for any BMS player.

## Random Types

### Lane-based Options (applied once at song start)

| Option | Description | Scratch Affected |
|--------|-------------|:----------------:|
| **OFF** | Original chart | - |
| **MIRROR** | Flip all lanes left-to-right | No |
| **RANDOM** | Shuffle key lanes randomly | No |
| **R-RANDOM** | Random rotation (shift all lanes by random amount) | No |
| **S-RANDOM** | Per-note random lane assignment | No |
| **H-RANDOM** | S-RANDOM with consecutive lane limit | No |

### Special Options

| Option | Description |
|--------|-------------|
| **SPIRAL** | Notes spiral outward from center |
| **ALL-SCR** | All notes become scratch notes |
| **CONVERGE** | Notes converge toward center |
| **CROSS** | Alternating cross pattern |

### Double Play Options

| Option | Description |
|--------|-------------|
| **FLIP** | Swap left and right side charts |
| **BATTLE** | Play same chart on both sides |

## Algorithm Details

### MIRROR

Simply reverse the lane indices.

```rust
pub fn apply_mirror(lanes: &[usize]) -> Vec<usize> {
    // For 7-key: [0,1,2,3,4,5,6] → [6,5,4,3,2,1,0]
    let max_lane = lanes.len() - 1;
    lanes.iter().map(|&lane| max_lane - lane).collect()
}
```

### RANDOM

Fisher-Yates shuffle on lane mapping.

```rust
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub fn apply_random(key_count: usize, seed: u64) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut lanes: Vec<usize> = (0..key_count).collect();

    // Fisher-Yates shuffle
    for i in (1..lanes.len()).rev() {
        let j = rng.gen_range(0..=i);
        lanes.swap(i, j);
    }

    lanes
}
```

**Possible permutations for 7-key:** 7! = 5,040

### R-RANDOM (Rotate Random)

Random starting position with circular shift.

```rust
pub fn apply_rotate_random(key_count: usize, seed: u64) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(seed);
    let offset = rng.gen_range(0..key_count);

    (0..key_count)
        .map(|lane| (lane + offset) % key_count)
        .collect()
}
```

### S-RANDOM (Super Random)

Each note is randomly assigned to a lane, with some restrictions.

```rust
pub struct SRandomizer {
    rng: StdRng,
    key_count: usize,
    last_lane: Option<usize>,
}

impl SRandomizer {
    pub fn new(key_count: usize, seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            key_count,
            last_lane: None,
        }
    }

    /// Get next lane for a note at given time
    pub fn next_lane(&mut self, occupied_lanes: &[usize]) -> usize {
        let available: Vec<usize> = (0..self.key_count)
            .filter(|lane| !occupied_lanes.contains(lane))
            .collect();

        if available.is_empty() {
            // All lanes occupied at this time - use any lane
            self.rng.gen_range(0..self.key_count)
        } else {
            available[self.rng.gen_range(0..available.len())]
        }
    }
}
```

### H-RANDOM (Hyper Random)

S-RANDOM with additional rules to prevent excessive same-lane repetition.

```rust
pub struct HRandomizer {
    rng: StdRng,
    key_count: usize,
    lane_history: Vec<usize>,  // Recent lanes used
    max_consecutive: usize,     // Max notes in same lane consecutively
}

impl HRandomizer {
    pub fn new(key_count: usize, seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            key_count,
            lane_history: Vec::new(),
            max_consecutive: 2,  // Typical limit
        }
    }

    pub fn next_lane(&mut self, occupied_lanes: &[usize]) -> usize {
        let mut available: Vec<usize> = (0..self.key_count)
            .filter(|lane| !occupied_lanes.contains(lane))
            .collect();

        // Remove lanes that have been used too consecutively
        if self.lane_history.len() >= self.max_consecutive {
            let recent = &self.lane_history[self.lane_history.len() - self.max_consecutive..];
            if recent.iter().all(|&l| l == recent[0]) {
                available.retain(|&lane| lane != recent[0]);
            }
        }

        let lane = if available.is_empty() {
            self.rng.gen_range(0..self.key_count)
        } else {
            available[self.rng.gen_range(0..available.len())]
        };

        self.lane_history.push(lane);
        if self.lane_history.len() > self.max_consecutive * 2 {
            self.lane_history.remove(0);
        }

        lane
    }
}
```

## Implementation

### Data Structures

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RandomOption {
    Off,
    Mirror,
    Random,
    RRandom,
    SRandom,
    HRandom,
    Spiral,
    AllScratch,
    Converge,
    Cross,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RandomUnit {
    None,   // No randomization
    Lane,   // Shuffle lanes at song start
    Note,   // Per-note randomization
    Player, // Per-player (for DP)
}

impl RandomOption {
    pub fn unit(&self) -> RandomUnit {
        match self {
            Self::Off => RandomUnit::None,
            Self::Mirror | Self::Random | Self::RRandom => RandomUnit::Lane,
            Self::SRandom | Self::HRandom | Self::Spiral => RandomUnit::Note,
            _ => RandomUnit::Lane,
        }
    }

    pub fn affects_scratch(&self) -> bool {
        matches!(self, Self::AllScratch)
    }
}
```

### Pattern Modifier

```rust
pub struct PatternModifier {
    option: RandomOption,
    key_count: usize,
    lane_map: Vec<usize>,       // For lane-based options
    note_randomizer: Option<Box<dyn NoteRandomizer>>,
}

impl PatternModifier {
    pub fn new(option: RandomOption, key_count: usize, seed: u64) -> Self {
        let lane_map = match option {
            RandomOption::Off => (0..key_count).collect(),
            RandomOption::Mirror => Self::create_mirror_map(key_count),
            RandomOption::Random => Self::create_random_map(key_count, seed),
            RandomOption::RRandom => Self::create_rotate_map(key_count, seed),
            _ => (0..key_count).collect(),
        };

        let note_randomizer: Option<Box<dyn NoteRandomizer>> = match option {
            RandomOption::SRandom => Some(Box::new(SRandomizer::new(key_count, seed))),
            RandomOption::HRandom => Some(Box::new(HRandomizer::new(key_count, seed))),
            _ => None,
        };

        Self {
            option,
            key_count,
            lane_map,
            note_randomizer,
        }
    }

    pub fn transform_lane(&mut self, original_lane: usize, occupied: &[usize]) -> usize {
        match &mut self.note_randomizer {
            Some(randomizer) => randomizer.next_lane(occupied),
            None => self.lane_map[original_lane],
        }
    }

    fn create_mirror_map(key_count: usize) -> Vec<usize> {
        (0..key_count).rev().collect()
    }

    fn create_random_map(key_count: usize, seed: u64) -> Vec<usize> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut lanes: Vec<usize> = (0..key_count).collect();
        for i in (1..lanes.len()).rev() {
            let j = rng.gen_range(0..=i);
            lanes.swap(i, j);
        }
        lanes
    }

    fn create_rotate_map(key_count: usize, seed: u64) -> Vec<usize> {
        let mut rng = StdRng::seed_from_u64(seed);
        let offset = rng.gen_range(0..key_count);
        (0..key_count).map(|l| (l + offset) % key_count).collect()
    }
}

trait NoteRandomizer {
    fn next_lane(&mut self, occupied: &[usize]) -> usize;
}
```

### Applying to Chart

```rust
pub fn apply_random_to_chart(
    chart: &mut Chart,
    option: RandomOption,
    seed: u64,
) {
    let key_count = 7; // For 7-key mode
    let mut modifier = PatternModifier::new(option, key_count, seed);

    // Group notes by time to track occupied lanes
    let mut notes_by_time: HashMap<i64, Vec<&mut Note>> = HashMap::new();
    for note in &mut chart.notes {
        notes_by_time.entry(note.time_ms).or_default().push(note);
    }

    // Process each time slice
    for (_, notes) in notes_by_time.iter_mut() {
        let mut occupied = Vec::new();

        for note in notes.iter_mut() {
            if !note.is_scratch() {
                let new_lane = modifier.transform_lane(note.lane, &occupied);
                note.lane = new_lane;
                occupied.push(new_lane);
            }
        }
    }
}
```

## Seed Management

### Reproducibility

For replays and IR, the random seed must be stored:

```rust
pub struct PlayOption {
    pub random: RandomOption,
    pub random_seed: u64,
    // For DP
    pub random_2p: RandomOption,
    pub random_seed_2p: u64,
}

impl PlayOption {
    pub fn generate_seed() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
```

### Display Format

When showing results:
```
Option: RANDOM (seed: 12345678)
```

## Score Considerations

- RANDOM/MIRROR do not affect score registration
- S-RANDOM/H-RANDOM typically disables IR ranking in some systems
- beatoraja allows all options for IR with option recorded

## Reference Links

- [beatoraja Random.java](https://github.com/exch-bms2/beatoraja/blob/master/src/bms/player/beatoraja/pattern/Random.java)
- [beatoraja Randomizer.java](https://github.com/exch-bms2/beatoraja/blob/master/src/bms/player/beatoraja/pattern/Randomizer.java)
- [Random Options Guide](https://iidx.org/compendium/random)
