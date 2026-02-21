# Porting TODO — Mechanical Line-by-Line Translation

Dependency graph order. Each module is ported only after its dependencies are complete.

## Phase 1: Core Foundation (~11,178 lines)

Zero internal dependencies. Port these first.

- [x] `bms.model` (19 files, 8,070 lines) — BMS format parser
- [x] `bms.table` (10 files, 3,108 lines) — LR2 course table parser

## Phase 2: Format Variants (~1,802 lines)

Depends on: bms.model

- [x] `bms.model.bmson` (16 files, 526 lines) — BMSON parser
- [x] `bms.model.osu` (9 files, 1,276 lines) — osu! format converter

## Phase 3: Low-level Subsystems (~12,050 lines)

Isolated subsystems with minimal internal deps.

- [x] `beatoraja.exceptions` (1 file, 7 lines) — Exception definitions
- [x] `beatoraja.system` (1 file, 139 lines) — File utilities
- [x] `tool.util` (1 file, 121 lines) — Generic utilities
- [x] `beatoraja.controller` (3 files, 762 lines) — Gamepad management
- [x] `beatoraja.external.DiscordRPC` (4 files, 634 lines) — Discord RPC
- [x] `beatoraja.input` (8 files, 4,188 lines) — Keyboard/MIDI input
- [x] `beatoraja.audio` (14 files, 7,086 lines) — Audio playback (Kira)
- [x] `tool.mdprocessor` (9 files, 2,264 lines) — Download/song processing

## Phase 4: Configuration & Central State (~26,290 lines)

Hub module — most others depend on this.

- [x] `beatoraja.config` (4 files, 2,582 lines) — Config definitions
- [x] `beatoraja` root (44 files, 23,708 lines) — Central state/data classes

## Phase 5: Pattern & Gameplay (~17,692 lines)

Core gameplay logic. Depends on: config, model, input, audio.

- [x] `beatoraja.pattern` (14 files, 4,108 lines) — Lane/note shuffle
- [x] `beatoraja.play` (23 files, 13,584 lines) — Judge, gauge, game loop
  - [x] `beatoraja.play.bga` (5 files, 1,802 lines) — BGA playback

## Phase 6: Skin System (~15,594 lines)

Multi-format skin rendering. Depends on: config, model, play.

- [x] `beatoraja.skin` base (34 files, 15,594 lines) — Skin rendering engine
  - [x] `beatoraja.skin.json` (11 files, 5,456 lines) — JSON skin loader
  - [x] `beatoraja.skin.lr2` (10 files, 6,482 lines) — LR2 skin loader
  - [x] `beatoraja.skin.lua` (5 files, 2,480 lines) — Lua skin loader
  - [x] `beatoraja.skin.property` (13 files, 8,216 lines) — Property binding

## Phase 7: Screen Implementations (~11,900 lines)

UI screens. Depends on: skin, config, play.

- [x] `beatoraja.select` (13 files, 8,386 lines) — Song select screen
  - [x] `beatoraja.select.bar` (17 files, 3,514 lines) — Bar rendering
- [x] `beatoraja.result` (7 files, 3,122 lines) — Result screen
- [x] `beatoraja.decide` (2 files, 172 lines) — Decide screen

## Phase 8: Advanced Features (~15,946 lines)

Optional/peripheral features.

- [x] `beatoraja.ir` (14 files, 3,572 lines) — Internet ranking
- [x] `beatoraja.external` (7 files, 2,076 lines) — OBS, webhooks
- [x] `beatoraja.obs` (2 files, 1,502 lines) — OBS WebSocket
- [x] `beatoraja.modmenu` (15 files, 8,468 lines) — In-game mod menu
- [x] `beatoraja.stream` (3 files, 402 lines) — Stream commands

## Phase 9: Launcher (~9,210 lines)

Standalone GUI. Can be deferred.

- [x] `beatoraja.launcher` (21 files, 9,210 lines) — Settings GUI (egui)

## Phase 10: Remaining Modules (~2,726 lines)

Untranslated Java files not covered by Phase 1–9.

- [x] `beatoraja.song` (8 files, 2,206 lines) — Song data model & DB accessor
- [x] `beatoraja.controller` (3 files, 381 lines) — Lwjgl3 gamepad (LibGDX-dependent)
- [x] `beatoraja.system` (1 file, 139 lines) — RobustFile I/O utility

## Phase 11: Integration & Wiring

Replace stubs with real cross-crate imports. No new translation — just connecting existing code.

- [x] Replace `SongData` stubs with `beatoraja-song` import (ir, skin, play, result)
- [x] Replace `MainController` internal stubs in `beatoraja-core` with real import
- [x] Replace `TextureRegion`/`Texture`/`Color`/`Pixmap`/`Rectangle` stubs in `beatoraja-select`/`beatoraja-result` with `beatoraja-skin` import
- [x] Replace `SkinProperty` constants in `beatoraja-external` with `beatoraja-skin` import
- [x] Replace `MessageRenderer` stub in `beatoraja-stream` with `beatoraja-core` import
- [x] Replace `SoundType`/`bms_model::Mode` stubs in `beatoraja-select` with real imports
- [x] Remove 11 unused stubs from `beatoraja-core`, 13 from `beatoraja-play`
- [x] Add 30+ getter methods to `SongData` and `ScoreData` for stub API compatibility
- [x] Add `beatoraja-song` dependency to 7 downstream crates
- [x] Resolve circular dependency issues (documented: core↔song, core↔skin, core↔play, play↔skin, input↔core, audio↔core)
- [x] Verify: all 66 tests pass, zero clippy warnings, clean `cargo fmt`

## Phase 12: Binary Entry Point

Create an executable binary target.

- [ ] Add `[[bin]]` target to workspace (e.g. `beatoraja-bin` crate)
- [ ] Implement `main()` wiring `MainLoader` → `BeatorajaGame` → game loop
- [ ] Replace LibGDX `Lwjgl3Application` with Bevy app runner or equivalent

## Phase 13: External Library Integration

Replace `todo!()` stubs with real library calls (~377 `todo!()` total).

- [ ] LibGDX rendering → Bevy (TextureRegion, SpriteBatch, ShaderProgram, etc.)
- [ ] JavaFX UI → egui (launcher views, ~40 `todo!()`)
- [ ] LuaJ → mlua (Lua skin loader, ~40 `todo!()`)
- [ ] PortAudio → cpal/Kira (audio driver, ~20 `todo!()`)
- [ ] FFmpeg → ffmpeg-next (BGA video processing)
- [ ] javax.sound.midi → midir (MIDI device enumeration)
- [ ] 7z extraction → sevenz-rust

---

## Testing Checkpoints

| After Phase | What you can test |
|-------------|-------------------|
| 1 | BMS parsing independently (Golden Master) |
| 2 | All format variants (BMS, BMSON, osu!) |
| 3 | Input/audio subsystems |
| 5 | Full gameplay logic with judge calculations |
| 6 | Skin rendering with actual skins |
| 7 | Full game flow (select → play → result) |
| 9 | Launcher settings GUI |
| 10 | Song database operations |
| 11 | Cross-crate compilation without stubs |
| 12 | Application launches (blank window) |
| 13 | Full game playable |
