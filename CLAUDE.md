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
```

Each state has:
- `update()` - Process frame logic
- `draw()` - Render to screen
- `take_transition()` - Handle state changes

### Directory Structure

Feature-based organization (NOT layer-based):

```
src/
├── audio/      # Kira audio engine (keysounds, BGM)
├── config/     # Application configuration
├── database/   # SQLite persistence (song.db, score.db)
├── input/      # Keyboard + gamepad input
├── model/      # BMS data structures
├── pattern/    # Pattern modifiers (Mirror, Random)
├── render/     # Note/lane rendering
├── replay/     # Replay recording/playback
├── skin/       # Lua-based skin system
├── state/      # Game state machines
│   ├── select/
│   ├── decide/
│   ├── play/
│   ├── result/
│   ├── course/
│   └── config/
└── util/       # Logging, profiling, errors
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

### Timing Precision

- Microsecond precision for input timestamps
- Millisecond precision for note timing
- Critical for audio/input synchronization

## Testing

### Integration Tests

```bash
cargo test
```

Tests cover:
- Gauge behavior (all 6 types)
- Score tracking
- Pattern modifiers

### Benchmarks

```bash
cargo bench
```

Criterion-based benchmarks for:
- Gauge update performance
- Judge rank logic

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

- `song.db` - Scanned BMS library metadata
- `score.db` - Player scores and clear types

### Replay Format

Binary format with:
- Input logs (microsecond timestamps)
- Metadata (hi-speed, gauge_type, recorded_at)
- flate2 compression
