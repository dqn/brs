# lr2oraja Rust Porting Project — Mechanical Line-by-Line Translation

## Overview

lr2oraja (beatoraja fork, Java 313 files / 72,000+ lines) to Rust.
All features including peripherals (Launcher, ModMenu, OBS, Discord RPC, Downloader) are in scope.

**CRITICAL: This is a FRESH START. All previous Rust code has been discarded.**

## Absolute Rules

### No Investigation, No Planning — Just Translate

- **NEVER** spend time exploring, investigating, or analyzing Java code structure before translating.
- **NEVER** enter plan mode or create plans for translation work.
- **NEVER** use Explore agents or research agents to "understand" the codebase.
- **Just read each Java file and translate it to Rust immediately.** The translation is mechanical — no design decisions needed.
- The workflow is: `Read Java file → Write Rust file → Test → Next file`.

### Prohibition on Past History

- **NEVER** read, reference, or consult any previous implementation, plans, or notes.
- **NEVER** use `git log`, `git show`, or browse old commits for implementation guidance.
- The ONLY source of truth is `./lr2oraja-java` Java source code.

### Mechanical Line-by-Line Translation

Every Java method/class MUST be translated mechanically to Rust. No shortcuts.

| Java | Rust |
|------|------|
| `if (a != null && a.x > 0)` | `if let Some(a) = &a { if a.x > 0 { ... } }` |
| `for (int i=0; i<n; i++)` | `for i in 0..n { ... }` |
| `switch-case` with fallthrough | Replicate exact control flow (no simplification) |
| `int → long` implicit cast | Explicit `as i64` cast |
| `float → double` implicit cast | Explicit `as f64` cast |
| `ArrayList<T>` | `Vec<T>` |
| `HashMap<K,V>` | `HashMap<K,V>` |
| `null` | `Option<T>` |
| `try-catch` | `Result<T>` / `anyhow` |
| `synchronized` | `Mutex` / `RwLock` |
| `static` field | `lazy_static!` / `OnceLock` |

### Six Principles

1. **ZERO improvements allowed** — If Java is verbose, copy it verbatim. Refactoring happens ONLY after ALL tests pass.
2. **Translate one method → test immediately** — Extract expected values from Java, write test, move to next only when green.
3. **Inject debug output into Java** — Export intermediate values as JSON. Compare line by line with Rust output (Golden Master).
4. **Preserve ALL branch/loop structure** — Including `switch-case` fallthrough. NEVER change control flow.
5. **Copy constants and magic numbers AS-IS** — Do NOT rename. Do NOT "improve" names. Make it work first.
6. **Write ALL type conversions explicitly** — Every Java implicit cast must become an explicit Rust cast.

## Workflow (Per Method)

```
1. Read the ENTIRE Java method (never skip lines)
2. Translate line-by-line to Rust (preserve structure exactly)
3. Add JSON debug output to Java side for intermediate values
4. Run both Java and Rust with same input
5. Compare outputs — fix until diff is ZERO
6. Move to next method
```

## Directory Structure

```
brs/
  lr2oraja-java/           # Java source (reference implementation, read-only except debug output)
  lr2oraja-rust/           # Rust port (Cargo workspace)
    crates/                # Rust crates (to be created incrementally)
    golden-master/         # Test infrastructure (Java exporter + fixtures)
    test-bms/              # Test BMS files
  .claude/                 # Claude workspace (plans only)
```

## Tech Stack

| Area        | Java            | Rust                  |
| ----------- | --------------- | --------------------- |
| Graphics    | LibGDX (LWJGL3) | Bevy                  |
| Audio       | PortAudio / GDX | Kira                  |
| Skin (Lua)  | LuaJ            | mlua                  |
| Database    | SQLite (JDBC)   | rusqlite              |
| Timing      | long (μs)       | i64 (μs)              |
| GUI         | JavaFX / ImGui  | egui                  |
| Discord RPC | JNA IPC         | discord-rich-presence |
| OBS         | WebSocket       | tokio-tungstenite     |

## Key Invariants

- All timing uses integer microseconds (i64) — no floating-point drift.
- `java.util.Random(seed)` LCG must be reproduced EXACTLY for pattern shuffle.
- LR2 judge scaling (`lr2JudgeScaling`) uses pure integer arithmetic.
- LongNote references use index-based approach (no circular references).

## Testing Rules

- **Golden Master Testing:** Export Java internal state as JSON, compare against Rust output.
- **TDD:** Red-Green-Refactor for every method.
- **Java Modifications Allowed:** Adding debug output / JSON export methods to Java code is permitted.
- **Test BMS Files:** Create minimal BMS files for each feature under test.
- **Tolerance:** Use ±2μs tolerance for timing comparisons (BPM → μs rounding).

### Golden Master Testing Lessons

- **Java BMSDecoder hardcodes MS932:** UTF-8 BMS metadata may be garbled on Java side.
- **`#RANDOM` is deterministic via `random_seeds.json`:** Use matching `decode_with_randoms(...)` inputs.
- **Avoid JavaFX dependencies:** Keep GM exporter in separate `golden-master` Gradle subproject.
- **Regenerate fixtures with `just golden-master-gen`:** Always regenerate after modifying Java exporter.
- **Parser fixture names keep source extensions:** Use `filename.ext.json` to avoid collisions.

## Implementation Status

- **Phase 1 complete:** `bms-model` (15 modules), `bms-table` (11 modules)
- **Phase 2 complete:** `bmson` (16 model types + BMSONDecoder), `osu` (9 model types + OSUDecoder)
- **Phase 3 complete:** `beatoraja-common` (3 modules), `discord-rpc` (4 modules), `beatoraja-input` (9 modules), `beatoraja-audio` (13 modules), `md-processor` (10 modules)
- **Phase 4 complete:** `beatoraja-core` (47 modules — config types, data models, DB accessors, core/resource types, config subpackage)

## Deferred / Stub Items

- Phase 5+ type dependencies (JudgeAlgorithm, BMSPlayerRule, SkinType, GrooveGauge, BarSorter, pattern modifiers, IRConnectionManager) are stubbed in `beatoraja-core/src/stubs.rs`
- Phase 4 type dependencies (Config, PlayModeConfig, etc.) are stubbed in each Phase 3 crate's `stubs.rs` (will be replaced with imports from `beatoraja-core`)
- PortAudio, LibGDX, ebur128, 7z extraction methods use `todo!()` pending external library integration
- javax.sound.midi equivalents stubbed (no direct Rust equivalent)
- MIDI device enumeration stubbed
- FLAC/MP3 decoding deferred to Phase 4+ library integration

## Translation Lessons Learned

> **This section is a living document.** Update it after every phase with new patterns, pitfalls, and decisions discovered during translation.

### Java Class Hierarchy → Rust Enum

Java abstract classes with `instanceof` checks translate best as Rust enums with a shared data struct:

```
Java:  abstract class Note { fields... }
       class NormalNote extends Note
       class LongNote extends Note { pair, end, type }
       class MineNote extends Note { damage }

Rust:  struct NoteData { /* shared fields */ }
       enum Note { Normal(NoteData), Long { data: NoteData, ... }, Mine { data: NoteData, ... } }
```

This preserves the `instanceof` pattern as `match` / `if let` and avoids trait-object complexity.

### f64 as BTreeMap Key

`f64` does not implement `Ord` in Rust. When Java uses `TreeMap<Double, V>`, convert the key to `u64` via `f64::to_bits()` for the `BTreeMap`, or use a newtype wrapper with manual `Ord` impl. Phase 1 used `to_bits()`.

### Borrow Checker vs. Java Constructor Patterns

Java constructors that take `this` and a sibling object (e.g., `Section(model, prev, ...)`) cause borrow issues when both `&self` and `&mut model` are needed. Solution: pass extracted primitive values (`prev_sectionnum`, `prev_rate`) instead of `Option<&Section>`.

### MS932 Encoding

Java's `BMSDecoder` hardcodes `"MS932"` (Shift_JIS superset). Use `encoding_rs::SHIFT_JIS` in Rust:

```rust
let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(raw_bytes);
```

### Parallel Agent Strategy

Independent crates (no cross-dependencies) should be translated by parallel agents writing to separate directories. This was done successfully with `bms-model` and `bms-table` in Phase 1.

**Pitfall:** Each agent must commit ALL files it creates, including `Cargo.toml`. Verify with `git status` after agents complete. In Phase 1, `bms-model/Cargo.toml` was missed and required a follow-up commit.

### Workspace Cargo.toml Must Exist Before Agents Start

Create the workspace `Cargo.toml` and all crate `Cargo.toml` files **before** launching translation agents. Agents need `cargo check` to work, which requires the workspace to be configured.

### Stub lib.rs for Sibling Crates

When translating crate A, the workspace may fail to compile if crate B (referenced as workspace member) has no `lib.rs`. Create a stub `lib.rs` (empty or with just module declarations) for all workspace members before starting translation.

### CommandWord Enum Translation

Java enums with `BiFunction` fields (like `CommandWord` in `BMSDecoder.java`) translate to a `match` on enum variants calling closures, or a function-dispatch table. Phase 1 used a match-based approach.

### Java TreeMap Iteration Order

`TreeMap` in Java iterates in key order. `BTreeMap` in Rust provides the same guarantee. Always use `BTreeMap` (not `HashMap`) when Java uses `TreeMap`, especially for section/timeline processing where order matters.

### BMSON JSON Model Types (Phase 2)

Java BMSON model classes with `@JsonIgnoreProperties(ignoreUnknown=true)` translate to Rust structs with `#[serde(default)]` and `#[derive(Deserialize)]`. Fields that can be `null` in JSON (checked with `!= null` in Java) should use `Option<T>`. Use `#[serde(default)]` on the struct to handle missing fields.

### Java switch-case Fallthrough (Phase 2)

The Java `Osu.java` parser has a `switch(section)` where the `"General"` case falls through to `"Editor"` (missing `break`). In Rust, `match` does not fall through. Replicate the fallthrough by explicitly calling the Editor parser at the end of the General branch.

### BTreeMap `lowerEntry` and `subMap` (Phase 2)

- Java `TreeMap.lowerEntry(y)` → Rust `BTreeMap::range(..y).next_back()`
- Java `TreeMap.subMap(y1, false, y2, true)` → Rust `BTreeMap::range((Excluded(y1), Included(y2)))`

### LongNote Pairing Without Direct References (Phase 2)

Java's BMSONDecoder uses direct object references for LN pairs (`ln.setPair(lnend)`). In Rust, use section-based tracking (`BmsonLnInfo { start_section, end_section, end_y }`) and timeline key lookups to modify pair notes. The `end_y` field allows locating the end note's timeline for wav/starttime/duration assignment.

### Submodule Organization (Phase 2)

When Java packages (like `bms.model.bmson` with 16 small classes) translate to Rust, consolidate all types into a single `mod.rs` file rather than one file per type. This reduces file count and simplifies imports.

### Platform-Specific Code with cfg (Phase 3)

Java platform detection (`System.getProperty("os.name")`) translates to `#[cfg(unix)]` / `#[cfg(windows)]` conditional compilation. Discord RPC's IPC uses Unix domain sockets on Linux/macOS and Named Pipes on Windows — keep both implementations with platform gates.

### Stub Modules for Phase 4 Dependencies (Phase 3)

Phase 3 crates reference Phase 4 types (Config, PlayModeConfig, Resolution, etc.) that don't exist yet. Create a `stubs.rs` module in each crate with minimal struct/trait definitions. These stubs will be replaced when Phase 4 is translated.

### Java Back-References → Callback Traits (Phase 3)

Java patterns where child objects hold `this` references to parent (e.g., `BMSPlayerInputProcessor` holding `BMSPlayer`) cause borrow conflicts in Rust. Solution: define a callback trait (e.g., `BMSPlayerInputDevice`) and pass `&mut dyn Trait` to methods instead of holding permanent references.

### PCM Arc Sharing for Slice (Phase 3)

Java's `ShortDirectPCM` uses `ByteBuffer.slice()` for zero-copy views. In Rust, use `Arc<Vec<T>>` with offset/length fields. The `slice()` method creates a new struct sharing the same `Arc` data with adjusted offset — avoids copying sample data.

### MS-ADPCM Decoder (Phase 3)

Java's MS-ADPCM decoder uses mutable coefficient arrays and predictor state. Translate as a stateless function taking `&[u8]` input and returning `Vec<i16>`. The adaptation table and coefficient sets are static constants.

### Java POJO Config Classes → Rust pub fields + serde (Phase 4)

Java config classes (Config, PlayerConfig, PlayModeConfig, AudioConfig, etc.) with private fields + getter/setter pairs translate best as Rust structs with `pub` fields and `#[derive(Clone, Debug, Default, Serialize, Deserialize)]`. Use `#[serde(default)]` on the struct level so missing JSON fields get default values. This avoids verbose getter/setter boilerplate while preserving JSON serialization compatibility.

### Java JDBC → rusqlite (Phase 4)

Java `PreparedStatement` + `ResultSet` patterns translate to rusqlite's `prepare` + `query_map`/`query_row`. Use `params![]` macro for bind parameters. The SQL strings can be copied nearly verbatim — rusqlite uses `?` placeholders just like JDBC. For database accessor inheritance (e.g., `ScoreDatabaseAccessor extends SQLiteDatabaseAccessor`), use composition (embed the base struct) instead of trait inheritance.

### Large Hub Crate with Phase 5+ Stubs (Phase 4)

Phase 4 is the "hub" module — most later phases depend on it, and it references many Phase 5+ types. Create a comprehensive `stubs.rs` with minimal implementations of external types (JudgeAlgorithm, BMSPlayerRule, SkinType, GrooveGauge, BarSorter, pattern modifiers, etc.). Methods that depend heavily on unimplemented subsystems should use `todo!("Phase N dependency")`.

### Parallel Agent Translation for Large Phases (Phase 4)

For phases with 40+ files, use a team of 4 agents translating in parallel. Group files by dependency:
- Group A: Foundational config types (no internal deps)
- Group B: Data models (depend on config types)
- Group C: Database accessors (depend on data models)
- Group D: Core/resource types (depend on everything)

Each agent writes to separate files, so no merge conflicts. Pre-create lib.rs with all module declarations and stubs.rs before launching agents.

### LibGDX JSON → serde_json (Phase 4)

Java's LibGDX `Json` class with `setIgnoreUnknownFields(true)` translates to `serde_json::from_str` with `#[serde(default)]` on structs. LibGDX's `Json.prettyPrint()` translates to `serde_json::to_string_pretty()`. The `setUsePrototypes(false)` flag (which disables skipping default values) has no direct equivalent — serde always serializes all fields.
