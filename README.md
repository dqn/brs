# brs - BMS Player in Rust

A rhythm game player for BMS (Be-Music Source) files, written in Rust.

## Features

- 7-key + scratch gameplay
- Multiple gauge types (Normal, Hard, EX-Hard, etc.)
- Pattern modifiers (Mirror, Random, S-Random)
- SUDDEN+/HIDDEN+/LIFT lane cover
- Replay recording and playback
- Score database with persistent storage
- Lua skin support (ECFN compatible)
- Gamepad support via gilrs
- Low-latency audio via kira

## Requirements

- Rust 1.85 or later
- BMS files to play

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| Left Shift | Scratch |
| Z, S, X, D, C, F, V | Keys 1-7 |
| Up/Down | Adjust hi-speed |
| Enter | Confirm selection |
| Escape | Back / Return to select |

## Configuration

Key bindings can be configured in `keyconfig.json`.

## Development

### Running tests

```bash
cargo test
```

### Running benchmarks

```bash
cargo bench
```

### Building with profiling

```bash
cargo build --features profiling
```

## Architecture

```
src/
├── app/          # Application core
├── audio/        # Audio system (kira)
├── config/       # Configuration management
├── database/     # Score and song database (rusqlite)
├── input/        # Input handling (gilrs + keyboard)
├── model/        # BMS data model
├── pattern/      # Pattern modifiers
├── render/       # Rendering (macroquad)
├── replay/       # Replay system
├── skin/         # Skin system (mlua)
├── state/        # Game states
└── util/         # Utilities (logging, profiler)
```

## License

MIT
