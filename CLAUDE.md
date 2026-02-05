# brs - BMS Player in Rust

## Project Overview

A BMS (Be-Music Source) player written in Rust. Rhythm game player supporting 7-key + scratch layout.

## Development Commands

```bash
# Build
cargo build --release

# Run
cargo run --release

# Test
cargo test

# Benchmark
cargo bench

# Build with profiling
cargo build --features profiling
```

## Architecture

### State Machine

The application uses an enum-based state machine:

```
Select → Decide → Play → Result
   ↑_______________________↓
   (Config accessible from Select)
```

Each state has:
- `update()` - Process frame logic
- `draw()` - Render to screen
- `take_transition()` - Handle state changes

### Module Structure

```
src/
├── app/        # Application controller (MainController)
├── audio/      # Kira audio engine
│   ├── audio_driver.rs      # Main audio management (dual tracks)
│   ├── keysound_processor.rs # BGM event playback
│   ├── sound_pool.rs        # Audio sample caching
│   └── preview_player.rs    # Song preview for selection
├── config/     # Application configuration (JSON + serde)
├── database/   # SQLite persistence
│   ├── connection.rs   # Database wrapper
│   ├── song_db.rs      # Song metadata (SHA256-indexed)
│   ├── score_db.rs     # Player scores
│   └── scanner.rs      # BMS folder scanning
├── input/      # Keyboard + gamepad input
│   ├── input_manager.rs  # Microsecond-precision timestamps
│   ├── key_config.rs     # JSON key binding
│   └── hotkey.rs         # In-game hotkeys
├── model/      # BMS data structures
│   ├── bms_model.rs   # Core BMSModel struct
│   ├── note.rs        # Note and Lane types
│   ├── timing.rs      # TimingEngine for sync
│   └── lane.rs        # LaneLayout and LaneConfig
├── pattern/    # Pattern modifiers (Mirror, Random, S-Random)
├── render/     # macroquad-based rendering
├── replay/     # Replay recording/playback (flate2 compression)
├── skin/       # Lua-based skin system
│   ├── lua/           # MLua 5.4 integration
│   ├── object/        # SkinObject types
│   └── font/          # Bitmap font support (FNT)
├── state/      # Game state machines
│   ├── select/   # Song selection with bar manager
│   ├── decide/   # Loading with resource preparation
│   ├── play/     # Main gameplay
│   │   ├── play_state.rs   # Core gameplay logic
│   │   ├── groove_gauge.rs # 8 gauge types
│   │   ├── judge_manager.rs # Judgement logic
│   │   ├── autoplay.rs     # Automatic play mode
│   │   └── score.rs        # Score tracking
│   ├── result/   # Results screen
│   ├── course/   # Dan/Course mode
│   └── config/   # Configuration screens
└── util/       # Logging, profiling, errors
    ├── error.rs    # UserError (bilingual)
    ├── logging.rs  # Tracing-based logging
    └── profiler.rs # Feature-gated profiling
```

## Key Types

### Lane System (`src/model/note.rs`)

- 16 total lanes: `Scratch + Key1-7` (1P) + `Scratch2 + Key8-14` (2P)
- Helper methods: `Lane::all_7k()`, `Lane::all_14k()`
- Note types: `Normal`, `LongStart`, `LongEnd`, `Invisible`, `Mine`

### Judge System (`src/state/play/judge_manager.rs`)

```rust
pub enum JudgeRank {
    PerfectGreat,  // index 0
    Great,         // index 1
    Good,          // index 2
    Bad,           // index 3
    Poor,          // index 4
    Miss,          // index 5
}
```

Judge windows (base): PG=20ms, GR=50ms, GD=100ms, BD=150ms, PR=200ms

### Gauge System (`src/state/play/groove_gauge.rs`)

8 gauge types with different behaviors:
- **Normal gauges** (start at 20%, border at 60-80%): AssistEasy, LightAssistEasy, Easy, Normal
- **Survival gauges** (start at 100%, border at 0%): Hard, ExHard, Hazard, Class

Guts system: Hard/Class gauges have reduced damage below thresholds (10-50%).

### Clear Types (`src/database/models.rs`)

11-level enum (beatoraja compatible), ordered for comparison:
```rust
NoPlay < Failed < AssistEasy < ... < ExHard < FullCombo < Perfect < Max
```

### Play Modes

```rust
pub enum Mode {
    Beat5K = 5, Beat7K = 7, Beat10K = 10, Beat14K = 14,
    PopN5K = 25, PopN9K = 29,
}
```

## Coding Conventions

### Error Handling

- Use `anyhow::Result<T>` for all fallible operations
- Convert Option to Result with `.ok_or_else(|| anyhow!("message"))?`
- User-facing errors use `UserError` with bilingual (EN/JP) messages

### State Transitions

- States own their resources (databases, input managers)
- Use transition enums (e.g., `SelectTransition::Decide(Box<SongData>)`)
- Resources are transferred via Box to avoid copies

### Resource Transfer

Use `std::mem::replace()` for owned types during state transitions:
```rust
pub fn take_transition(&mut self) -> SelectTransition {
    std::mem::replace(&mut self.transition, SelectTransition::None)
}
```

### Configuration Pattern

- JSON files with `serde` + `#[serde(default)]` for backward compatibility
- Graceful fallback to defaults if file missing
- Files: `config.json`, `hotkey.json`, `favorites.json`

### Timing Precision

- Microsecond precision for input timestamps (replay recording)
- Millisecond precision for note timing (gameplay)
- Critical for audio/input synchronization

### Derive Patterns

- `#[repr(i32)]` on enums for database/beatoraja compatibility
- `#[default]` variant marker with `derive(Default)`
- Value types: `Debug, Clone, Copy, PartialEq, Eq`

## Logging & Profiling

- **Logging**: `tracing` crate with daily log rotation to `logs/`
- **Profiling**: Feature-gated via `#[cfg(feature = "profiling")]`

```bash
# Run with profiling
cargo build --features profiling
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| macroquad 0.4 | Graphics/window/input |
| kira 0.10 | Low-latency audio |
| bms-rs 0.10 | BMS file parsing |
| mlua 0.10 | Lua 5.4 scripting (vendored) |
| gilrs 0.11 | Gamepad support |
| rusqlite 0.32 | SQLite (bundled) |
| anyhow | Error handling |
| serde + serde_json | Configuration |
| flate2 | Replay compression |
| tracing | Logging |

## Testing

### Unit Tests

Co-located in `#[cfg(test)]` modules within source files.

```bash
cargo test
```

Tests cover:
- Gauge behavior (all 8 types)
- Score tracking
- Pattern modifiers
- Clear type ordering

### Benchmarks

Criterion-based benchmarks for hot paths:

```bash
cargo bench
```

## Important Notes

### Bilingual Support

Error messages include both English and Japanese:
```rust
UserError::new("File not found", "ファイルが見つかりません")
```

### Skin System

- Lua 5.4 scripting via mlua
- ECFN format compatibility
- Objects: ImageObject, NumberObject, TextObject
- FNT bitmap fonts supported

### Databases

- `song.db` - Scanned BMS library metadata (SHA256 + MD5 keys)
- `score.db` - Player scores and clear types

### Replay Format

Binary format with:
- Input logs (microsecond timestamps)
- Metadata (hi-speed, gauge_type, recorded_at)
- flate2 compression

## Visual Development Workflow

Claude Code can autonomously capture and compare screenshots for UI development.

### Screenshot Capture

```bash
# Capture select screen
cargo run --release --bin brs -- --screenshot select

# Capture play screen (requires --bms)
cargo run --release --bin brs -- --screenshot play --bms /path/to/song.bms

# Capture result screen (uses mock data)
cargo run --release --bin brs -- --screenshot result

# Custom output directory
cargo run --release --bin brs -- --screenshot select --screenshot-output /path/to/output
```

Default output: `.agent/screenshots/current/<state>.png`

### Screenshot Comparison

```bash
# Compare reference with current
cargo run --release --bin screenshot_diff -- \
  '.agent/screenshots/select.png' \
  '.agent/screenshots/current/select.png'
```

Output format:
```
Screenshot Comparison Report
============================
Reference: .agent/screenshots/select.png
Current:   .agent/screenshots/current/select.png

Size: MATCH (1920x1080)

Region Analysis (4x4 grid):
  [0,0]: 2.3% diff (top-left)
  [1,1]: 45.2% diff - SIGNIFICANT (middle-center)
  ...

Overall Similarity: 78.4%
Status: SIGNIFICANT - Major differences detected

Significant Differences:
  - middle-center (45.2% diff): pixels (480, 270) to (960, 540)
```

### Claude Code Autonomous Workflow

1. Capture screenshot: `cargo run --release --bin brs -- --screenshot select`
2. Compare with reference: `cargo run --release --bin screenshot_diff -- '<ref>' '<current>'`
3. Read the diff report to identify changed regions
4. Modify rendering code in the relevant files
5. Repeat until similarity reaches acceptable level

### Screen-Specific Files

| Screen | Primary Files |
|--------|---------------|
| Select | `src/state/select/select_state.rs` |
| Play | `src/state/play/play_state.rs`, `src/render/` |
| Result | `src/state/result/result_state.rs` |

### CI Usage

For headless environments (CI):
```bash
xvfb-run cargo run --release --bin brs -- --screenshot select
```

### Directory Structure

```
.agent/screenshots/
├── play.png          # Reference screenshots
├── select.png
├── result.png
└── current/          # Captured screenshots
    └── select.png
```
