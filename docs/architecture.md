# BMS Player Architecture

This document describes the high-level architecture of the BMS player.

## Tech Stack

| Component | Library | Reason |
|-----------|---------|--------|
| BMS Parsing | bms-rs | Mature Rust parser, handles all formats |
| Audio | kira | Clock-based scheduling, low latency |
| Graphics | macroquad | Simple 2D, cross-platform |
| Error Handling | anyhow | Ergonomic error propagation |

## Module Structure

```
src/
├── main.rs              # Entry point, game state machine
├── lib.rs               # Re-exports for testing
│
├── bms/                 # BMS data handling
│   ├── loader.rs        # Load and validate BMS files
│   ├── chart.rs         # Processed chart representation
│   └── timing.rs        # Timing calculations
│
├── audio/               # Audio system
│   ├── manager.rs       # Kira AudioManager wrapper
│   ├── keysound.rs      # Keysound loading and playback
│   └── scheduler.rs     # Note-to-audio timing sync
│
├── game/                # Core gameplay
│   ├── state.rs         # Play state, current position
│   ├── judge.rs         # Judgment logic
│   ├── input.rs         # Input mapping
│   └── score.rs         # Score/combo calculation
│
├── render/              # Graphics
│   ├── highway.rs       # Note highway rendering
│   ├── notes.rs         # Note sprite rendering
│   ├── ui.rs            # HUD, combo display
│   └── effects.rs       # Judgment effects
│
├── scene/               # Game scenes
│   ├── select.rs        # Song selection
│   ├── play.rs          # Gameplay scene
│   └── result.rs        # Result screen
│
└── config/              # Configuration
    ├── settings.rs      # User settings
    └── keybinds.rs      # Key configuration
```

## Data Flow

```
┌─────────────┐    parse    ┌─────────────┐    convert   ┌──────────┐
│  .bms file  │────────────▶│   bms-rs    │─────────────▶│  Chart   │
└─────────────┘             └─────────────┘              └──────────┘
                                                              │
                                                              ▼
┌─────────────┐    load     ┌─────────────┐         ┌──────────────┐
│   .wav/.ogg │────────────▶│    Kira     │◀────────│  Scheduler   │
│   keysounds │             │  AudioMgr   │         └──────────────┘
└─────────────┘             └─────────────┘
                                   ▲
                                   │
                            ┌──────┴──────┐
                            │  Game Loop  │
                            │  Input →    │
                            │  Judge →    │
                            │  Score →    │
                            │  Render     │
                            └─────────────┘
```

## Core Data Structures

### Chart

```rust
pub struct Chart {
    pub metadata: Metadata,
    pub timing_data: TimingData,
    pub notes: Vec<Note>,
    pub bgm_events: Vec<BgmEvent>,
}

pub struct Note {
    pub time: Fraction,        // Precise measure position
    pub time_ms: f64,          // Pre-calculated milliseconds
    pub channel: NoteChannel,
    pub keysound_id: ObjId,
    pub note_type: NoteType,
}
```

### Timing

```rust
pub struct TimingData {
    pub initial_bpm: f64,
    pub bpm_changes: Vec<BpmChange>,
    pub stops: Vec<StopEvent>,
    pub measure_lengths: Vec<MeasureLength>,
}
```

### Judgment

```rust
pub struct JudgeConfig {
    pub pgreat_window: f64,  // ±ms
    pub great_window: f64,
    pub good_window: f64,
    pub bad_window: f64,
}

pub enum JudgeResult {
    PGreat,
    Great,
    Good,
    Bad,
    Poor,
}
```

## Key Design Decisions

### Audio-Visual Synchronization

Use Kira's clock as the single source of truth:

```rust
pub struct GameClock {
    kira_clock: kira::clock::ClockHandle,
}

impl GameClock {
    pub fn current_time_ms(&self) -> f64 {
        // Derive visual position from audio clock
    }
}
```

### Timing Precision

Use fraction arithmetic for measure-based calculations:

```rust
use fraction::Fraction;

pub fn calculate_time_ms(
    measure_position: Fraction,
    timing_data: &TimingData,
) -> f64 {
    // Process BPM changes with exact fractions
    // Convert to milliseconds only at final step
}
```

### Input Latency Compensation

Timestamp input events and process retroactively:

```rust
pub struct InputEvent {
    key: KeyCode,
    pressed: bool,
    timestamp: f64,  // System clock time
}
```

### Keysound Loading

Progressive loading with priority queue:

1. Load first 30 seconds of keysounds synchronously
2. Background load remainder
3. Queue by first occurrence time

## Scene State Machine

```
         ┌─────────┐
         │  Title  │
         └────┬────┘
              │
         ┌────▼────┐
    ┌───▶│ Select  │◀───┐
    │    └────┬────┘    │
    │         │         │
    │    ┌────▼────┐    │
    │    │  Play   │    │
    │    └────┬────┘    │
    │         │         │
    │    ┌────▼────┐    │
    └────│ Result  │────┘
         └─────────┘
```

## References

- [kira Documentation](https://docs.rs/kira)
- [macroquad Documentation](https://docs.rs/macroquad)
- [bms-rs Documentation](https://docs.rs/bms-rs)
