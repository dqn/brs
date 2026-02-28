# lr2oraja Rust Porting

beatoraja fork (Java 313 files / 72k+ lines) → Rust. 26 crates, 158k lines. Source: `./lr2oraja-java`.

## Rules

- Workflow: `Read Java → Write Rust → Test → Next`. Copy Java verbatim, refactor ONLY after ALL tests pass.
- Translate one method → test immediately — green before moving on.
- Golden Master: export Java values as JSON, compare with Rust. Tolerance: ±2μs.
- Preserve ALL branch/loop/fallthrough structure. Copy constants/magic numbers AS-IS.
- Explicit type conversions — every implicit Java cast → explicit Rust cast.
- After completing a phase/task, update TODO.md and AGENTS.md.
- Worktree isolation: **always merge worktree branches before sending shutdown requests**.
- Deferred items: always tag with `→ **Phase XX**`. At phase completion, audit all deferred items.

## Type Mapping

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

## Tech Stack

| Java | Rust |
|------|------|
| LibGDX / PortAudio | wgpu / Kira 0.12 |
| LuaJ / SQLite (JDBC) | mlua / rusqlite |
| JavaFX / ImGui | egui (egui-wgpu 0.31) |
| Discord (JNA) / OBS (WebSocket) | discord-rich-presence / tokio-tungstenite |

## Structure

```
lr2oraja-java/       # Java source (read-only)
lr2oraja-rust/       # Cargo workspace
  crates/
    bms-model        # BMS/BME/BML parser + model
    bms-table        # Difficulty table parser
    beatoraja-types  # Shared types (circular dep breaker)
    beatoraja-pattern    # Note pattern (JavaRandom LCG)
    beatoraja-audio      # Audio (Kira 0.12)
    beatoraja-input      # Keyboard/controller input
    beatoraja-controller # gilrs controller manager
    beatoraja-render     # Rendering (wgpu)
    beatoraja-skin       # Skin loading/layout
    beatoraja-song       # Song DB (rusqlite)
    beatoraja-core       # State machine, main loop
    beatoraja-play       # Play state (gameplay)
    beatoraja-select     # Song select state
    beatoraja-decide     # Song decide state
    beatoraja-result     # Result state
    beatoraja-modmenu    # Mod menu state
    beatoraja-ir         # Internet ranking
    beatoraja-external   # Twitter, clipboard
    beatoraja-obs        # OBS WebSocket
    beatoraja-stream     # Streaming
    beatoraja-launcher   # Launcher UI (egui)
    beatoraja-system     # Platform utilities
    beatoraja-bin        # Entry point
    discord-rpc          # Discord Rich Presence
    md-processor         # Markdown processing
    ast-compare          # Test: AST Java↔Rust comparison
  golden-master/   # Golden Master test infra
  test-bms/        # Test BMS files
```

## Key Invariants

- Timing: i64 microseconds. JavaRandom LCG in `beatoraja-pattern` (**never** `StdRng`/`rand`). LR2 MT19937. LR2 judge: pure integer arithmetic. LongNote: index-based.

## Testing

- **Test runner:** `just test` (excludes slow render snapshot tests) or `just test-all` (full).
- **Golden Master:** `just golden-master-gen`. Fixtures: `filename.ext.json`.
- **TDD:** Red-Green-Refactor. **ast-compare:** `just ast-map` / `just ast-compare` / `just ast-constants` / `just ast-full`.

## Status

**2940 tests.** Phases 1–56 complete. Zero clippy warnings.
**Migration audit**: 97.90% method resolution (4,189/4,279). 90 genuinely missing. 0 constant mismatches. 0 Rust-side regressions.
**Phase 54 finding**: ast-compare "missing" 257 methods → 88% false positives (architectural redesign).
**Phase 55**: 28 genuine gaps audited → 15 already implemented (false positives), 7 newly implemented, 6 blocked by circular deps.
**Phase 56**: Method-level ignore added to ast-compare. 170 false positives registered (136 patterns). Accurate gap count: 90 methods.

### Resolved (Phase 45–53)

All 7 critical gaps, the StdRng regression, and BytePCM regressions resolved:
- PlayerResource.loadBMSModel() — BMS files load (Phase 46a)
- MainState.load_skin() — screens render (Phase 47c)
- PlayerResource.SongData unified — get_songdata() returns real data (Phase 46b)
- read_chart/read_course — select→play works (Phase 48c)
- CourseResult MainState — course results functional with IR (Phase 50a/b)
- FloatPropertyFactory — delegates to MainState (Phase 47a)
- SkinTextFont.draw_with_offset() — TrueType text renders (Phase 51d)
- RandomizerBase — JavaRandom LCG restored (Phase 45a)
- ScoreData serde — Java JSON field names compatible (Phase 45b)
- BytePCM float→byte — `as i32 as i8` matches Java truncation (Phase 54b)
- ast-compare ignore list — bmson/osu POJOs added (Phase 54a)

## Remaining Stubs (~2,872 lines across 10 stubs.rs)

| Crate | stubs.rs | Status |
|-------|:--------:|--------|
| beatoraja-launcher | 527 | Skin header wired, async DB wired |
| beatoraja-result | 510 | CourseResult functional, IR thread wired |
| beatoraja-external | 500 | Permanent (`bail!()`, Twitter API deprecated) + screen_type wired |
| beatoraja-skin | 495 | Timer/Float/Boolean delegates wired, Lua 20 functions done |
| beatoraja-select | 278 | Bar Clone resolved, 7 get_children() done, read_chart done |
| beatoraja-modmenu | 205 | SkinWidget stubs remain |
| beatoraja-decide | 154 | load_skin wired, AudioProcessor stubs remain |
| beatoraja-input | 114 | MouseScratchInput position hardcoded |
| beatoraja-types | 88 | 7 resolved re-exports, 1 partial (BarSorter) |
| beatoraja-core | 1 | exit/save_config wired, loadBMSModel wired |

### Remaining Regressions (0)

BytePCM float saturation and negative overflow resolved in Phase 54b.
Fix: `(f * 127.0) as i32 as i8` matches Java's `(byte)(int)(f * 127)` truncation semantics.

### Genuine Gaps (Phase 56 audit: 90 remaining)

**Phase 56**: ast-compare method-level ignore added. `.ast-compare-method-ignore` in workspace root.
170 methods ignored as false positives (arch redesign, platform, already-impl, thread inner classes).

**Remaining 90 by domain:**
- SkinConfiguration (13): skin selection/switching UI — launcher integration
- KeyConfiguration (13): keyboard/controller/midi key assignment management
- LR2 Skin Loaders (9): loadSkin methods + CSV helpers
- Randomizer/Pattern (7): Randomizer base + LaneShuffleModifier
- AbstractAudioDriver (5): non-abstract methods (getKeySound, getSound, getSampleRate, etc.)
- PlayerResource (4): getBGAManager, reloadBMSFile, setTableinfo, getAnalysisTask
- bms-model (4): BMSONDecoder.getTimeLine, ChartDecoder.printLog, Section.getTimeLine, DataProcessor.process
- beatoraja-play (4): JudgeManager (2), JudgeWindowRule, SkinLane.init
- beatoraja-song (3): SongData.getTimelines, SongDatabaseAccessor.updateSongDatas, SongInformation.parseInt36
- beatoraja-audio (5): BMSLoudnessAnalyzer (2), BMSRenderer, GdxAudioDeviceDriver, PCM
- beatoraja-core misc (13): Config.validatePath, MainController.create, TimerManager, etc.
- Other (10): IR, select, obs, input, stream

**Blocked by architecture (non-blocking):**
- MainState defaults (4): loadSkin, getOffsetValue, getImage, getSound — trait override points
- MainController.updateTable — needs TableBar from beatoraja-select (circular dep)
- MainController IRSendStatus.send — needs IRConnection from beatoraja-ir (circular dep)

## Lessons Learned

- **Encoding:** `encoding_rs::SHIFT_JIS` for MS932. **Serde:** `BPM`→`Bpm`, `URL`→`Url`, `#[serde(alias)]`.
- **Borrow checker:** `&mut` conflicts → scoped block. Self-ref → `Option::take()`. Parent ref → callback trait.
- **Stubs:** `stubs.rs` per crate → replace via `pub use`. Always `cargo check` after removal.
- **Circular deps:** `beatoraja-types` for shared types. Core cannot import: song, skin, play, select, result, ir, modmenu.
- **Lua→JSON coercion:** 3-layer: numbers→strings, float→int truncation, empty `{}`→remove.
- **Bar Clone:** `Box<dyn Trait>` blocks Clone → use `Arc<dyn Trait>` for shared trait objects.
- **Property delegate pattern:** `integer_value(id)` / `float_value(id)` / `boolean_value(id)` on MainState — skin property factories delegate via ID lookup.
- **Dead crate removal:** beatoraja-common (785 lines, 0 callers) removed in Phase 53d. Always audit before removing: check Cargo.toml deps, re-exports, test imports.
- **ast-compare false positives:** ~88% of "missing" methods are architectural redesigns (inner class→closure, abstract→enum dispatch, getter→pub field). Always verify Java↔Rust manually before implementing.
- **ast-compare method-level ignore:** `.ast-compare-method-ignore` supports `ClassName.methodName` (exact) and `ClassName.*` (wildcard). Run `just ast-map` to use. 136 patterns → 170 methods ignored.
- **Java float→int→byte truncation:** Use `as i32 as i8` in Rust (via i32 to get truncation). Direct `as i8` saturates since Rust 1.45.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
