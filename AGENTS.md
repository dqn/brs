# lr2oraja Rust Porting Project — Mechanical Line-by-Line Translation

## Overview

lr2oraja (beatoraja fork, Java 313 files / 72,000+ lines) to Rust.
All features including peripherals (Launcher, ModMenu, OBS, Discord RPC, Downloader) are in scope.

**CRITICAL: This is a FRESH START. All previous Rust code has been discarded.**

## Absolute Rules

### No Investigation, No Planning — Just Translate

- **NEVER** explore, investigate, or analyze before translating. **NEVER** enter plan mode.
- Workflow: `Read Java file → Write Rust file → Test → Next file`.

### Prohibition on Past History

- **NEVER** read previous implementation, plans, notes, or old commits.
- The ONLY source of truth is `./lr2oraja-java` Java source code.

### Mechanical Line-by-Line Translation

| Java | Rust |
|------|------|
| `if (a != null && a.x > 0)` | `if let Some(a) = &a { if a.x > 0 { ... } }` |
| `for (int i=0; i<n; i++)` | `for i in 0..n { ... }` |
| `switch-case` with fallthrough | Replicate exact control flow |
| `int→long` / `float→double` | Explicit `as i64` / `as f64` |
| `ArrayList<T>` / `HashMap<K,V>` | `Vec<T>` / `HashMap<K,V>` |
| `null` / `try-catch` | `Option<T>` / `Result<T>` + `anyhow` |
| `synchronized` / `static` field | `Mutex`/`RwLock` / `lazy_static!`/`OnceLock` |

### Six Principles

1. **ZERO improvements** — Copy Java verbatim. Refactor ONLY after ALL tests pass.
2. **Translate one method → test immediately** — Green before moving on.
3. **Golden Master** — Export Java intermediate values as JSON, compare with Rust output.
4. **Preserve ALL branch/loop structure** — Including fallthrough. NEVER change control flow.
5. **Copy constants/magic numbers AS-IS** — No renaming.
6. **Explicit type conversions** — Every implicit Java cast → explicit Rust cast.

## Directory Structure

```
brs/
  lr2oraja-java/           # Java source (read-only except debug output)
  lr2oraja-rust/           # Rust port (Cargo workspace)
    crates/                # Rust crates
    golden-master/         # Test infrastructure (Java exporter + fixtures)
    test-bms/              # Test BMS files
```

## Tech Stack

| Java | Rust |
|------|------|
| LibGDX (LWJGL3) | Bevy |
| PortAudio / GDX | Kira |
| LuaJ | mlua |
| SQLite (JDBC) | rusqlite |
| long (μs) | i64 (μs) |
| JavaFX / ImGui | egui |
| JNA IPC (Discord) | discord-rich-presence |
| WebSocket (OBS) | tokio-tungstenite |

## Key Invariants

- Timing: integer microseconds (i64), no floating-point drift.
- `java.util.Random(seed)` LCG must be reproduced EXACTLY for pattern shuffle.
- LR2 judge scaling: pure integer arithmetic.
- LongNote references: index-based (no circular references).

## Testing Rules

- **Golden Master:** Export Java state as JSON, compare against Rust. Tolerance: ±2μs for timing.
- **TDD:** Red-Green-Refactor for every method.
- **Java modifications allowed** for debug output / JSON export.
- **GM Lessons:** Java BMSDecoder hardcodes MS932 (UTF-8 metadata garbled); `#RANDOM` deterministic via `random_seeds.json`; keep GM exporter in separate Gradle subproject; regenerate with `just golden-master-gen`; fixture names: `filename.ext.json`.

## Implementation Status

| Phase | Crates | Modules |
|-------|--------|---------|
| 1 | `bms-model`, `bms-table` | 26 |
| 2 | `bmson`, `osu` | 27 |
| 3 | `beatoraja-common`, `discord-rpc`, `beatoraja-input`, `beatoraja-audio`, `md-processor` | 39 |
| 4 | `beatoraja-core` | 47 |
| 5 | `beatoraja-pattern`, `beatoraja-play` | 42 |
| 6 | `beatoraja-skin` | 50+ |
| 7 | `beatoraja-select`, `beatoraja-result`, `beatoraja-decide` | 39 |
| 8 | `beatoraja-ir`, `beatoraja-external`, `beatoraja-obs`, `beatoraja-modmenu`, `beatoraja-stream` | 41 |
| 9 | `beatoraja-launcher` | 21 |
| 10 | `beatoraja-song`, `beatoraja-controller`, `beatoraja-system` | 12 |
| 11 | Integration & wiring (stub replacement across 12 crates) | — |
| 12 | `beatoraja-bin` (CLI + winit event loop) | — |
| 14 | `beatoraja-types` (15 modules, circular dep resolution) | 15 |

## Deferred / Stub Items

### Remaining circular dependency stubs (cannot be replaced)
- `beatoraja-core`: SongData, SkinType, GrooveGauge (song/skin/play depend on core)
- `beatoraja-play`: TextureRegion/Texture (skin depends on play)

### Remaining structural mismatches
- SongDatabaseAccessor, IRConnection: struct in stubs vs trait in real impl
- BMSPlayerInputProcessor: parameter types differ (i32 vs usize)

### Complex lifecycle stubs (need full runtime)
- MainController, PlayerResource, MainState in all downstream crates

### External library stubs (`todo!()`)
- PortAudio, LibGDX, ebur128, 7z, javax.sound.midi, MIDI enumeration
- FLAC/MP3 decoding, BGA video (FFmpeg/Gdx), ImGui→egui rendering
- Twitter4j, AWT clipboard, LR2 SQLite score import, Windows named pipe

## Translation Lessons Learned

> Living document. Update after every phase.

### Java→Rust Type Mapping Patterns

| Java Pattern | Rust Solution |
|---|---|
| Abstract class + `instanceof` | Enum with shared `Data` struct + `match` |
| Interface with lambdas | Enum + `modify()` method, or `Box<dyn Trait>` |
| Abstract class + factory | Trait + `Box<dyn Trait>` factory return |
| POJO config (private fields + getters) | `pub` fields + `#[derive(Serialize, Deserialize)]` + `#[serde(default)]` |
| `TreeMap<K,V>` | `BTreeMap<K,V>` (preserves key order) |
| `TreeMap<Double,V>` | `BTreeMap<u64,V>` via `f64::to_bits()` |
| `TreeMap.lowerEntry(y)` | `BTreeMap::range(..y).next_back()` |
| `TreeMap.subMap(y1,false,y2,true)` | `BTreeMap::range((Excluded(y1), Included(y2)))` |
| `@JsonIgnoreProperties(ignoreUnknown=true)` | `#[serde(default)]` + `#[derive(Deserialize)]` |
| LibGDX `Json` + `setIgnoreUnknownFields` | `serde_json::from_str` + `#[serde(default)]` |
| `PreparedStatement` + `ResultSet` | rusqlite `prepare` + `query_map`/`query_row` + `params![]` |
| `ByteBuffer.slice()` | `Arc<Vec<T>>` + offset/length fields |
| `TextureRegion[]` (nullable) | `Vec<Option<TextureRegion>>` |
| `java_websocket.WebSocketClient` | tokio + `futures_util::SplitSink/SplitStream` |
| JavaFX views (`@FXML`) | Plain structs with `pub` fields; rendering deferred |
| JavaFX `TableView<T>` | `Vec<T>` + `Vec<usize>` (selected indices) |

### Encoding & Platform

- **MS932:** `encoding_rs::SHIFT_JIS.decode(raw_bytes)` for Java's hardcoded MS932.
- **Platform detection:** `#[cfg(unix)]`/`#[cfg(windows)]` for Discord RPC IPC, Windows named pipes.
- **LR2IR:** Shift_JIS HTTP responses via `encoding_rs`, XML via `quick-xml` with serde.

### Borrow Checker Patterns

- **Parent `this` reference:** Use callback trait (`&mut dyn Trait`) instead of permanent reference.
- **Constructor with sibling:** Pass extracted primitives instead of `Option<&Section>`.
- **LongNote pairing:** Section-based tracking with index lookups instead of direct object references.

### Random Number Generation

- **Java `Random(seed)` LCG:** multiplier=`0x5DEECE66D`, addend=`0xB`, mask=`(1L<<48)-1`. Implement manually if exact reproduction needed.
- **LR2 Mersenne Twister:** Custom MT19937 with LR2-specific seeding. Use `u32` with wrapping arithmetic.

### Parallel Agent Strategy

- Independent crates → parallel agents writing to separate directories.
- **Pre-requisites:** Create workspace `Cargo.toml`, all crate `Cargo.toml`, stub `lib.rs` for all members BEFORE launching agents.
- **Pitfall:** Verify `git status` after agents complete — files (especially `Cargo.toml`) can be missed.
- **Grouping:** By dependency level (foundational → data models → DB accessors → core types).
- **Phase agent counts:** Phase 4: 4 agents, Phase 5: 2 crates, Phase 6: 5 agents (73 files), Phase 8: 5 agents (41 files), Phase 9: 4 agents, Phase 10: 3 agents.

### Stub Management

- **Forward stubs:** Create `stubs.rs` in each crate for types from later phases. Replace when translated.
- **Hub crate stubs (Phase 4):** Comprehensive stubs for Phase 5+ types; methods use `todo!("Phase N")`.
- **Rendering stubs:** LibGDX types → `#[derive(Clone, Default, Debug, PartialEq)]` stubs; deferred to graphics integration.
- **Stub replacement via `pub use`:** Replace struct definition with `pub use real_crate::module::Type;` — all existing imports continue working.
- **Getter compatibility:** Add Java-style getters to real types rather than modifying all callers.
- **Remaining stubs after replacement:** Only rendering types, lifecycle types (MainController), and structural mismatches.

### Circular Dependency Resolution

- **Core cannot import from:** song, skin, play, select, result, ir, modmenu (all depend ON core).
- **Solution (Phase 14):** Extract shared types to `beatoraja-types` crate; core re-exports via `pub use`.
- **BMKeys moved with PlayModeConfig** to avoid types→input→types cycle.
- **Associated constants:** Bridge module-level constants as `impl Type { pub const X: T = module::X; }`.

### API Incompatibility Patterns (Stub → Real)

| Mismatch | Resolution |
|---|---|
| Field type (`String` vs `Option<String>`) | Add `.unwrap_or_default()` at call sites |
| Method signature (`i32` vs `Mode`) | Update callers or add adapter methods |
| Struct vs Trait | Keep stub (structural refactoring needed) |
| Struct vs Enum (`Resolution`) | Update callers to enum method calls |
| `set_field(v)` → pub field | Direct assignment `obj.field = v` |

### Phase-Specific Notes

- **switch-case fallthrough (Phase 2, Osu.java):** Explicitly call next branch handler at end of current branch.
- **CommandWord enum (Phase 1):** Java enum with `BiFunction` → match-based dispatch.
- **Submodule consolidation (Phase 2):** 16 small Java classes → single `mod.rs`.
- **MS-ADPCM (Phase 3):** Stateless function `&[u8]` → `Vec<i16>`, static coefficient tables.
- **OBS auth (Phase 8):** SHA-256 challenge-response: `sha2::Sha256` + `base64::engine::general_purpose::STANDARD`.
- **IRResponse (Phase 8):** Generic struct `IRResponse<T>` with `Option<T>`, not a trait.
- **IRConnectionManager (Phase 8):** Manual `OnceLock` registry instead of Java reflection.
- **FontAwesomeIcons (Phase 8):** ~1016 icon codepoints → `pub const` strings.
- **Ghost data RLE (Phase 8):** 40+ char substitution mappings, copy verbatim.
- **Custom CRC32 (Phase 10):** Polynomial `0xEDB88320`, appends `\\\0` before hashing. LR2-specific.
- **RobustFile (Phase 10):** Double-write (backup→temp→atomic rename), `File::sync_all()` for fsync.
- **SkinHeader Clone (Phase 9):** Added `#[derive(Clone)]` to SkinHeader + custom item types.
- **winit event loop (Phase 12):** `create→resumed`, `render→RedrawRequested`, `resize→Resized`, `pause→suspended`, `dispose→CloseRequested`. Use `ControlFlow::Poll`.
- **CLI args (Phase 12):** `clap::Parser` derive; `--replay N` replaces `-r1`..`-r4`.
- **Deferred entry points (Phase 12):** Launcher UI (egui) and fullscreen mode (GLFW monitor APIs).
