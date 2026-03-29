# rubato

beatoraja fork (Java 313 files / 72k+ lines) → Rust. 15 crates, 167k lines.

## Status

**5757 tests.** All 62 phases complete. Zero clippy warnings. Zero regressions.
4,279 Java methods resolved (4,049 direct + 230 architectural redesigns). 0 functional gaps.

**Permanent stubs (intentional):**
- Twitter4j (`rubato-external`): ~446 lines, `bail!()` -- API deprecated
- ShortDirectPCM (`rubato-audio`): Java-specific DirectBuffer -- unnecessary in Rust
- JavaFX find_parent_by_class_simple_name (`rubato-launcher`): No egui equivalent
- randomtrainer.dat (`rubato-modmenu`): Binary resource from Java, uses empty HashMap fallback

## Architecture

### Crate Structure

```
rubato/              # Cargo workspace (15 crates) at repo root
  crates/
    bms-model        # BMS/BME/BML parser + model
    bms-table        # Difficulty table parser
    rubato-types     # Shared types (circular dep breaker)
    rubato-audio     # Audio (Kira 0.12)
    rubato-input     # Keyboard/controller input (+ controller)
    rubato-render    # Rendering (wgpu)
    rubato-skin      # Skin loading/layout
    rubato-song      # Song DB (rusqlite, + md-processor)
    rubato-core      # State machine, main loop (+ pattern)
    rubato-play      # Play state (gameplay)
    rubato-state     # Select/Decide/Result/Modmenu/Stream states
    rubato-ir        # Internet ranking
    rubato-external  # Twitter, clipboard, Discord RPC, OBS WebSocket
    rubato-launcher  # Launcher UI (egui)
    rubato-bin       # Entry point
  golden-master/   # Golden Master test infra
  test-bms/        # Test BMS files
  .claude/tmp/beatoraja/  # Original Java source code (beatoraja fork)
```

### Tech Stack

| Java | Rust |
|------|------|
| LibGDX / PortAudio | wgpu / Kira 0.12 |
| LuaJ / SQLite (JDBC) | mlua / rusqlite |
| JavaFX / ImGui | egui (egui-wgpu 0.31) |
| Discord (JNA) / OBS (WebSocket) | discord-rich-presence / tokio-tungstenite |

### Type Mapping

| Java | Rust |
|------|------|
| `null` / `try-catch` | `Option<T>` / `Result<T>` + `anyhow` |
| `ArrayList` / `HashMap` / `TreeMap` | `Vec` / `HashMap` / `BTreeMap` (`TreeMap<Double>` → `BTreeMap<u64>` via `to_bits()`) |
| `synchronized` / `static` | `Mutex`/`RwLock` / `OnceLock` |
| Abstract class + `instanceof` | Enum + `Data` struct + `match` |
| Interface / Abstract factory | `Box<dyn Trait>` / Enum + `modify()` |
| POJO config | `pub` fields + `#[derive(Serialize, Deserialize)]` + `#[serde(default)]` |
| `PreparedStatement` + `ResultSet` | rusqlite `prepare` + `query_map` + `params![]` |
| `ByteBuffer.slice()` | `Arc<Vec<T>>` + offset/length |
| JavaFX `TableView<T>` | `Vec<T>` + `Vec<usize>` (selected) |

### Key Invariants

- Timing: i64 microseconds. JavaRandom LCG in `rubato-core::pattern` (**never** `StdRng`/`rand`). LR2 MT19937. LR2 judge: pure integer arithmetic. LongNote: index-based.
- Circular deps: `rubato-types` for shared types. Core cannot import: song, skin, play, select, result, ir, modmenu.

## Testing

- **Test runner:** `just test` (excludes slow render snapshot tests and `rubato-bin` which requires ffmpeg system library) or `just test-all` (full, requires ffmpeg).
- **Golden Master:** `just golden-master-test`. Fixtures: `golden-master/fixtures/*.json` (pre-generated). Tolerance: +-2us.
- **TDD:** Red-Green-Refactor.

## Development Rules

### General

- Preserve ALL branch/loop/fallthrough structure. Copy constants/magic numbers AS-IS.
- Explicit type conversions -- every implicit Java cast → explicit Rust cast.
- When choosing between similar-sounding APIs (e.g., `has_long_note()` vs `has_undefined_long_note()`, `maxbpm` vs `mainbpm`), trace back to the Java source to verify which semantic is needed.
- NEVER use blocking I/O (`rx.recv()`, synchronous HTTP) on the main/render thread. Use background threads + `try_recv()` or non-blocking poll.
- After broad renames, file splits, or structural refactors, run runtime smoke on every affected entrypoint immediately. Build/lint green is insufficient.

### Java→Rust Gotchas

- **Panic surface:** Java implicit safety (null returns, silent overflow, bounds defaults) becomes panics in Rust. Audit every: array/slice index, division, `as` cast (especially to unsigned), `.unwrap()`, and enum match.
- **float→int→byte truncation:** Use `as i32 as i8` (via i32 for truncation). Direct `as i8` saturates since Rust 1.45.
- **Explicit Drop required:** Every ported background thread, network connection, audio handle, and GPU resource needs `Drop` impl or `dispose()`. Add cleanup paths immediately.
- **Borrow checker patterns:** `&mut` conflicts → scoped block. Self-ref → `Option::take()`. Parent ref → callback trait. `Box<dyn Trait>` blocks Clone → use `Arc<dyn Trait>`.
- **UTF-8 byte processing:** Never use `byte as char` (`u8 as char` expands to Latin-1 code point, corrupts Japanese text). Use `Vec<u8>` output buffers with `String::from_utf8()`.

### Encoding & Serialization

- **SHIFT_JIS everywhere:** All file readers for Japanese formats (.chp, .lr2skin CSV, .bms) must use `encoding_rs::SHIFT_JIS`, not UTF-8. Use `std::fs::read()` + `SHIFT_JIS.decode()`.
- **CRC32 path encoding:** CRC32 over file paths must encode as Shift_JIS bytes, not UTF-8, or Japanese paths produce different hashes.
- **Serde alias vs rename:** `#[serde(alias)]` is deserialization-only. For bidirectional Java parity, use `#[serde(rename)]`. Convention: `BPM`→`Bpm`, `URL`→`Url`.
- **Lua→JSON coercion:** 3-layer: numbers→strings, float→int truncation, empty `{}`→remove.
- **String byte slicing safety:** Never slice at byte positions from external data without `is_char_boundary()` / `floor_char_boundary()` / `strip_prefix()`.

### State Lifecycle & Wiring

- **Wiring-first debugging:** For black screens, no-op inputs, broken transitions, or silent state desync, investigate wiring and lifecycle boundaries before rewriting business logic. Checklist: input, timer, audio, `skin.prepare()`, skin property delegation, state transitions, controller/resource sync, interactive mouse context.
- **Global shared state capture-at-construction:** Globals must be initialized BEFORE subsystems that capture them are constructed. Late initialization creates a silent, undetectable split (writer and reader hold different instances). Document initialization order and add startup assertions.
- **State create() must init all subsystems:** Every subsystem referenced in render/input paths (gauge, replay, BGA, callbacks, timers, skin) must be initialized in `create()`. Omitting gauge init causes silent failures: 0% rendering, skipped stage-failed, empty gauge log.
- **Handoff struct completeness:** When passing data between states via handoff structs (`ScoreHandoff`, `StateCreateEffects`), verify every field is populated at the source and consumed at the destination. Empty/default fields are bugs.
- **Controller wiring across crates:** States that cannot own `&mut MainController` use a queued `MainControllerAccess` proxy + MainController-side drain step. `Arc<Mutex<State>>` wrappers must sync `MainStateData` bidirectionally.
- **Config propagation via command queue:** Menu config changes must push back to MainController via `MainControllerCommandQueue` variants (e.g., `UpdateSkinConfig`, `UpdateSkinHistory`). Writing only to the local `PlayerConfig` clone loses changes between sessions.
- **state_factory take/put-back panic safety:** `MainController.state_factory` is `Option::take()`-ed before `create_state()`. If the create call panics, the factory is lost (None). Wrap in `catch_unwind` and restore before resuming.
- **Play config remap must clear live key state:** `set_play_config()` must clear live key/button/MIDI state before installing new mappings, or play starts with stuck beams and false autoplay.
- **Ranking cache sharing:** IR ranking cache handles must stay shared across MainController, proxies, and result wrappers. Fresh per-wrapper caches break select→play/result reuse.
- **Practice mode reload must refresh derived fields:** When `receive_reloaded_model()` replaces the BMS model, also update `song_data`, `song_metadata`, and `lnmode_override`.
- **Dispose ordering:** BGA decoders before skin. Skin disposal releases textures that BGA background threads may still be reading.
- **Frozen harness timer sync:** `MainController.timer` and active state's `main_state_data.timer` are separate clocks. Test harnesses must advance both in lockstep.

### Skin & Rendering

- **Audit the full chain in one pass:** asset load → font/texture resolve → coordinate scaling → positioning → draw call. Single-layer fixes cause cascading commit chains.
- **Path resolution:** Skin/config asset paths resolve against both configured skin root AND ancestor directories of CWD. Font paths must be resolved relative to the current skin file. Absolute paths must skip directory prefix joining (`Path::join()` on an absolute component silently discards the prefix).
- **Lua/JSON source resolution:** Preserve filemap substitutions and wildcard expansion through runtime texture resolution. Do not reject image objects with `*` in source path early; resolve filemap first.
- **Bitmap font parity:** Scale both destination rectangle AND text `size` by the destination-width ratio. Glyph `yoffset` uses BMFont top-origin without reapplying font base offset.
- **Select bar subobject scaling:** `SelectBarData` children bypass `Skin::set_destination()`. Apply `src`→`dstr` scaling manually.
- **Skin loader safe division:** LR2 CSV src.width/src.height can be zero. All `dst / src` divisions must use `safe_div_f32()`.
- **Skin property ref parity:** LR2 numeric refs and image-index refs reuse the same ID for different meanings. Keep `integer_value(id)` and `image_index_value(id)` separate.
- **LR2 option handoff:** Copy `#SETOPTION` / custom option selections from loader into `Skin` before `prepare()`.
- **Property delegate pattern:** `integer_value(id)` / `float_value(id)` / `boolean_value(id)` on MainState; skin property factories delegate via ID lookup.
- **Pixmap-backed textures must re-upload every frame:** `ensure_uploaded()` must always re-upload `__pixmap_*` textures.
- **Lua draw sentinel:** `Some(-1)` sentinel for "has Lua expression" must not reach `boolean_property(id)` dispatch. Guard with explicit sentinel checks.
- **ImageSet entries must go through image resolution.** Missing resolution causes blank skin elements.
- **Render-capture testing:** Runtime glyph quads use generated keys (`__pixmap_*`), not source `.fnt` paths. Filter on actual emitted identifiers + target widget bounds.
- **Render context adapter delegation:** Java IS-A→composition ports must explicitly delegate every `SkinRenderContext` method. Missing delegations silently return trait defaults (None, false, 0, empty Vec). When adding or modifying the `SkinRenderContext` trait, enumerate ALL callers and verify each adapter delegates.

### Skin Event & Mouse Handling

- **Skin event wiring:** `SkinDrawable` mouse/custom-event paths need a state-aware `SkinRenderContext`, not a timer-only adapter. Interactive states build their own context for mouse/render passes.
- **Mouse bridge parity:** States with clickable skins must override `handle_skin_mouse_pressed()` / `handle_skin_mouse_dragged()`, take ownership of `skin` + `TimerManager`, and build state-aware context.
- **Lua skin load-time context:** Lua skins read `main_state.*` during `load_header()` / `load()`, not only render. Pass a state-aware adapter into the loader.
- **Mouse context cannot dispatch custom events:** Custom events (1000-1999) from `DelegateEvent` clickevents are silently dropped during mouse handling due to skin `take()` borrow pattern. Fixing requires either queued event dispatch or restructuring the borrow pattern.
- **Active skin verification:** Green tests on default JSON skin do not verify the user's configured Lua/bitmap-font skin. Check `config_player.json` and active profile first.

### Input & Gameplay

- **Play input handoff:** Explicitly copy START/SELECT/key/control/scroll/device state from `BMSPlayerInputProcessor`, and write back consumed flags after processing.
- **Analog input handoff:** Snapshot `is_analog`, `get_analog_diff()`, `get_time_since_last_analog_reset()` during `sync_input_from()`, flush with `reset_analog_input()` in `sync_input_back_to()`.
- **Key beam release parity:** Only the pressed branch is gated by `isJudgeStarted` / autoplay. Release must always flip KEYON → KEYOFF when the timer is on.
- **Judge timer parity:** Queue and apply judge/combo timer side effects (`46/47/247`, `446/447/448`, bomb timers) on the main thread when a judgment lands.
- **Play note render timing:** `LaneRenderer::draw_lane()` expects timer start timestamps, not elapsed durations. Pass `timer(TIMER_PLAY)` and let the renderer subtract from `now_time()`.
- **Lane renderer coordinate parity:** SpriteBatch uses Y-up projection. Keep Java's `hu`/`hl`/upward break condition/positive LN span semantics. Do not rewrite as Y-down.
- **Selected play-config lookup:** Resolve play config from the selected bar's actual mode, not only the current selector mode.
- **Target list fallback:** When `TargetProperty`'s global list is empty, fall back to `player_config.targetlist`.
- **egui event consumption gates game input:** When egui reports `wants_pointer_input` or `wants_keyboard_input`, do not forward those events to game state. CursorMoved events must also propagate into drag/move callbacks via `SharedKeyState.mouse_dragged`.
- **Rotated sprite V-flip:** `draw_region_rotated()` must apply Y-up V flip (matching `push_quad()`).
- **FFmpeg frame pixel format:** FFmpegProcessor outputs RGBA directly. The `fs_ffmpeg` shader must NOT apply R/B swizzle.
- **Course-mode-only data flows escape single-song testing.** Course gauge constraints, previous-stage restoration, aggregate score checks, and gauge history are all gated by `is_course_mode()`. Always test course-specific paths explicitly.

### Audio Pipeline

- **Periodic audio handle cleanup:** Sweep finished audio handles (`cleanup_stopped_handles`) to prevent unbounded accumulation.
- **WAV format tag distinction:** 32-bit WAV: check format_tag 1 (PCM integer) vs 3 (IEEE float). Misinterpreting PCM as float produces garbage audio.
- **RIFF word-aligned padding:** Odd-length WAV chunks have a pad byte after the data. `seek_to_chunk` must account for this.
- **Loader thread: drop handle, don't join.** Background loader threads using `rayon::par_iter` can take a long time. Drop the handle to detach instead.
- **Audio side effects:** States needing the real `AudioDriver` after render/shutdown should flush through `MainState::sync_audio()` from `MainController`, not through queued commands.

### Testing Practices

- **GUI subprocess smoke tests:** Use explicit timeouts, not `Command::output()` alone. Healthy GUI event loops run indefinitely, turning smoke tests into hangs.
