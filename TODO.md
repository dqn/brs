# Porting TODO — Remaining Work

All phases (1–18g) complete. 1241 tests pass. See AGENTS.md for full status.

## Blocked Tasks

### Phase 16b: Golden Master Test Activation (partially complete)

- [ ] Add missing fixtures for modules not yet covered (modmenu, select bar, stream) — deferred until Rust-side APIs are implemented
- [ ] Reactivate `compare_render_snapshot.rs` — blocked: old crate names, SkinData→Skin pipeline gap, Lua loader stubbed. Requires full API rewrite + loading pipeline

### Phase 18e: Stub replacement (remaining items blocked)

- [ ] Replace `MainState` stubs with real trait impls — blocked: requires per-screen concrete types (PlayState, SelectState, etc.)
- [ ] Remove all `stubs.rs` files — blocked: depends on above + rendering/IR/database implementations
- [ ] beatoraja-external LibGDX stubs (Pixmap/GdxGraphics/BufferUtils/PixmapIO) — blocked on wgpu rendering pipeline

### Phase 18f: Integration verification (remaining items blocked)

- [ ] Activate `compare_render_snapshot.rs` — blocked: SkinData→Skin pipeline, Lua loader
- [ ] E2E gameplay flow test: select → decide → play → result screen transitions — blocked: requires all stubs removed
- [ ] Final verification: all tests pass, zero clippy warnings, clean `cargo fmt` — blocked: final gate

### Known Issues (open)

- [ ] JSONSkinLoader returns `SkinData` (intermediate), not `Skin` — full loading pipeline not connected
- [ ] LuaSkinLoader completely stubbed — `load_header()` and `load_skin()` return None
- [ ] All remaining stubs (16 files, ~2,440 lines) exhaustively audited (4 rounds) — blocked by rendering, IR network, database, per-screen implementations

## Next Phases (planned)

### Phase 19: SkinData→Skin Loading Pipeline (~2,600 lines)

Unblocks: Phase 16b render snapshot tests, Phase 18f E2E tests, rendering stub removal

#### 19a: SkinData→Skin Conversion Function
- [ ] Implement `SkinData::to_skin()` / `Skin::from_skin_data()` — converts intermediate SkinData into renderable Skin
- [ ] Wire destination/source/timer/op data from SkinObjectData → SkinObject draw parameters

#### 19b: Screen-Specific JsonSkinObjectLoaders (7 screens)
- [ ] PlaySkinObjectLoader — note field, gauge, judge, lane cover, BGA
- [ ] SelectSkinObjectLoader — bar list, table/difficulty/lamp display
- [ ] DecideSkinObjectLoader — preview, metadata display
- [ ] ResultSkinObjectLoader — score graph, ranking, clear lamp
- [ ] CourseSkinObjectLoader — course result display
- [ ] KeyConfigSkinObjectLoader — key binding display
- [ ] SkinConfigSkinObjectLoader — skin option display
- Each loader: Java `loadXXX()` → Rust `load_xxx()`, returns typed SkinObject variants

#### 19c: LuaSkinLoader Implementation
- [ ] Implement `load_header()` — parse Lua table → SkinHeaderData
- [ ] Implement `load_skin()` — parse Lua table → SkinData via `from_lua_value()`
- [ ] `from_lua_value()` recursive converter: LuaTable → SkinObjectData tree

#### 19d: SkinLoader Entry Points
- [ ] Wire `SkinLoader::load()` to call JSONSkinLoader or LuaSkinLoader based on file extension
- [ ] Connect SkinData→Skin conversion at load site
- [ ] Reactivate `compare_render_snapshot.rs` test

### Phase 20: IRConnection Integration (~300 lines)

Unblocks: beatoraja-result MainController IR methods, IRSendStatusMain::send()

- [ ] Implement `LR2IRConnectionImpl` — concrete `IRConnection` trait impl using `LR2IRClient`
- [ ] Implement `IRResponseImpl` — wrap HTTP response into `IRResponse` trait
- [ ] Wire `IRConnectionManager::register()` call in MainController initialization
- [ ] Implement `IRSendStatusMain::send()` — actual IR score submission
- [ ] Remove MainController IR-related stubs in beatoraja-result (6 methods)

### Phase 21: Per-Screen MainState Implementations

Unblocks: Phase 18e MainState stub removal, E2E screen transitions

- [ ] `PlayState` — implements `MainState` for gameplay screen
- [ ] `SelectState` — implements `MainState` for song select screen
- [ ] `DecideState` — implements `MainState` for decide screen
- [ ] `ResultState` — implements `MainState` for result screen
- [ ] `KeyConfigState` / `SkinConfigState` — implements `MainState` for config screens
- [ ] Wire screen transitions: select → decide → play → result
