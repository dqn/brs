# brs — beatoraja Rust Port

Mechanical line-by-line translation of [lr2oraja](https://github.com/exch-bms2/beatoraja) (Java) to Rust.

## Project Structure

```
brs/
  lr2oraja-java/           # Java source (reference implementation, read-only)
  lr2oraja-rust/           # Rust port (Cargo workspace)
    crates/
      bms-model/           # BMS/BMSON/osu! format parser
      bms-table/           # LR2 course table parser
      beatoraja-common/    # Exceptions, file utilities, generic pair
      discord-rpc/         # Discord Rich Presence IPC client
      beatoraja-input/     # Keyboard/controller/MIDI input processing
      beatoraja-audio/     # Audio playback, PCM processing, WAV decoding
      md-processor/        # Music download and processing
      beatoraja-core/      # Config, central state, data models, DB accessors
    golden-master/         # Test infrastructure
    test-bms/              # Test BMS files
```

## Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `bms-model` | BMS, BMSON, osu! format parser and decoder | Phase 1-2 complete |
| `bms-table` | LR2 course table parser | Phase 1 complete |
| `beatoraja-common` | Exceptions, RobustFile, generic Pair utility | Phase 3 complete |
| `discord-rpc` | Discord Rich Presence IPC client (Unix/Windows) | Phase 3 complete |
| `beatoraja-input` | Keyboard, controller, MIDI, mouse scratch input | Phase 3 complete |
| `beatoraja-audio` | Audio driver, PCM formats, WAV/ADPCM decoding | Phase 3 complete |
| `md-processor` | HTTP/IPFS music download and processing | Phase 3 complete |
| `beatoraja-core` | Config, central state, data models, DB accessors | Phase 4 complete |

## Implementation Progress

- **Phase 1** (Core Foundation): `bms.model` (15 modules), `bms.table` (11 modules)
- **Phase 2** (Format Variants): `bms.model.bmson` (BMSONDecoder + 16 model types), `bms.model.osu` (OSUDecoder + 9 model types)
- **Phase 3** (Low-level Subsystems): `beatoraja-common` (3 modules), `discord-rpc` (4 modules), `beatoraja-input` (9 modules), `beatoraja-audio` (13 modules), `md-processor` (10 modules)
- **Phase 4** (Configuration & Central State): `beatoraja-core` (47 modules — config, data models, DB accessors, core types)

See [TODO.md](TODO.md) for the full porting roadmap.

## Building

```sh
cd lr2oraja-rust
cargo check
cargo test
```

## Tech Stack

| Area | Java (original) | Rust (port) |
|------|-----------------|-------------|
| Graphics | LibGDX (LWJGL3) | Bevy |
| Audio | PortAudio / GDX | Kira |
| Skin (Lua) | LuaJ | mlua |
| Database | SQLite (JDBC) | rusqlite |
| GUI | JavaFX / ImGui | egui |
| Discord RPC | JNA IPC | discord-rich-presence |
| OBS | WebSocket | tokio-tungstenite |
