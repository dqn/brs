# lr2oraja Rust Porting — Mechanical Line-by-Line Translation

lr2oraja (beatoraja fork, Java 313 files / 72k+ lines) → Rust. All features incl. peripherals in scope.
**FRESH START. All previous Rust code discarded.**

## Rules

- **NEVER** explore/investigate/plan. Workflow: `Read Java → Write Rust → Test → Next`.
- **NEVER** read previous implementation/plans/old commits. ONLY source: `./lr2oraja-java`.
- **ZERO improvements** — copy Java verbatim. Refactor ONLY after ALL tests pass.
- **Translate one method → test immediately** — green before moving on.
- **Golden Master** — export Java values as JSON, compare with Rust. Tolerance: ±2μs.
- **Preserve ALL branch/loop/fallthrough structure.** Copy constants/magic numbers AS-IS.
- **Explicit type conversions** — every implicit Java cast → explicit Rust cast.

## Type Mapping

| Java | Rust |
|------|------|
| `null` check / `try-catch` | `Option<T>` / `Result<T>` + `anyhow` |
| `ArrayList<T>` / `HashMap<K,V>` | `Vec<T>` / `HashMap<K,V>` |
| `TreeMap<K,V>` / `TreeMap<Double,V>` | `BTreeMap<K,V>` / `BTreeMap<u64,V>` via `to_bits()` |
| `TreeMap.lowerEntry(y)` | `BTreeMap::range(..y).next_back()` |
| `synchronized` / `static` field | `Mutex`/`RwLock` / `lazy_static!`/`OnceLock` |
| Abstract class + `instanceof` | Enum + shared `Data` struct + `match` |
| Interface with lambdas | Enum + `modify()`, or `Box<dyn Trait>` |
| Abstract class + factory | Trait + `Box<dyn Trait>` factory |
| POJO config | `pub` fields + `#[derive(Serialize, Deserialize)]` + `#[serde(default)]` |
| `@JsonIgnoreProperties` / LibGDX Json | `serde_json::from_str` + `#[serde(default)]` |
| `PreparedStatement` + `ResultSet` | rusqlite `prepare` + `query_map` + `params![]` |
| `ByteBuffer.slice()` | `Arc<Vec<T>>` + offset/length |
| `TextureRegion[]` (nullable) | `Vec<Option<TextureRegion>>` |
| `java_websocket.WebSocketClient` | tokio + `futures_util::SplitSink/SplitStream` |
| JavaFX views / `TableView<T>` | Plain structs; `Vec<T>` + `Vec<usize>` (selected) |

## Tech Stack

| Java | Rust |
|------|------|
| LibGDX (LWJGL3) / PortAudio | Bevy / Kira |
| LuaJ / SQLite (JDBC) | mlua / rusqlite |
| JavaFX / ImGui | egui |
| JNA IPC (Discord) / WebSocket (OBS) | discord-rich-presence / tokio-tungstenite |
| long (μs) | i64 (μs) |

## Directory & Structure

```
brs/
  lr2oraja-java/    # Java source (read-only except debug output)
  lr2oraja-rust/    # Cargo workspace
    crates/         # Rust crates
    golden-master/  # Test infra (Java exporter + fixtures)
    test-bms/       # Test BMS files
```

## Key Invariants

- Timing: i64 microseconds, no floating-point drift.
- `java.util.Random(seed)` LCG: multiplier=`0x5DEECE66D`, addend=`0xB`, mask=`(1L<<48)-1`. Reproduce exactly.
- LR2 Mersenne Twister: custom MT19937, LR2-specific seeding, `u32` wrapping arithmetic.
- LR2 judge scaling: pure integer arithmetic. LongNote refs: index-based.

## Testing

- **Golden Master:** Java state → JSON → Rust comparison. Java BMSDecoder hardcodes MS932. `#RANDOM` deterministic via `random_seeds.json`. Regenerate: `just golden-master-gen`. Fixtures: `filename.ext.json`.
- **TDD:** Red-Green-Refactor for every method. Java mods allowed for debug/JSON export.

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

**Circular dep stubs (cannot replace):** SongData/SkinType/GrooveGauge in core; TextureRegion/Texture in play.
**Structural mismatches:** SongDatabaseAccessor/IRConnection (struct vs trait); BMSPlayerInputProcessor (i32 vs usize).
**Lifecycle stubs:** MainController, PlayerResource, MainState in all downstream crates.
**External `todo!()`:** PortAudio, LibGDX, ebur128, 7z, MIDI, FLAC/MP3, BGA video, ImGui→egui, Twitter4j, AWT clipboard, LR2 score import, Windows named pipe.

## Lessons Learned

### Encoding & Platform
- **MS932:** `encoding_rs::SHIFT_JIS.decode(raw_bytes)`. **LR2IR:** Shift_JIS HTTP via `encoding_rs`, XML via `quick-xml`.
- **Platform:** `#[cfg(unix)]`/`#[cfg(windows)]` for Discord IPC, named pipes.

### Borrow Checker
- Parent `this` ref → callback trait (`&mut dyn Trait`). Constructor with sibling → pass primitives.
- LongNote pairing → section-based tracking with index lookups.

### Parallel Agents
- Independent crates → parallel agents. Create workspace `Cargo.toml` + all crate scaffolding BEFORE launching.
- Verify `git status` after — files can be missed. Group by dependency level.

### Stub Management
- Forward stubs in `stubs.rs` per crate. Replace via `pub use real_crate::module::Type;`.
- Add Java-style getters to real types rather than modifying callers.
- Remaining: rendering types, lifecycle types, structural mismatches only.

### Circular Dependencies
- Core cannot import: song, skin, play, select, result, ir, modmenu.
- Solution: `beatoraja-types` crate; core re-exports via `pub use`. BMKeys moved with PlayModeConfig.

### API Incompatibility (Stub → Real)

| Mismatch | Fix |
|---|---|
| `String` vs `Option<String>` | `.unwrap_or_default()` |
| `i32` vs `Mode` | Update callers or adapter methods |
| Struct vs Trait | Keep stub |
| Struct vs Enum | Update to enum method calls |
| `set_field(v)` → pub field | Direct assignment |

### Phase-Specific
- **P1:** CommandWord enum → match dispatch. **P2:** switch fallthrough → explicit next-branch call; 16 classes → single `mod.rs`.
- **P3:** MS-ADPCM: `&[u8]` → `Vec<i16>`, static coefficients.
- **P8:** OBS auth: SHA-256 + base64. IRResponse: generic `IRResponse<T>`. IRConnectionManager: `OnceLock` registry. FontAwesome: ~1016 `pub const`. Ghost RLE: 40+ char mappings verbatim.
- **P9:** SkinHeader + items need `#[derive(Clone)]`. **P10:** Custom CRC32 poly `0xEDB88320`, appends `\\\0`. RobustFile: double-write + `sync_all()`.
- **P12:** winit: `create→resumed`, `render→RedrawRequested`, `resize→Resized`, `pause→suspended`, `dispose→CloseRequested`, `ControlFlow::Poll`. CLI: `clap::Parser`; `--replay N`. Deferred: egui launcher, fullscreen (GLFW).
