# Porting TODO — Remaining Work

All phases (1–20) complete. 1274 tests pass. See AGENTS.md for full status.

## Completed Phases (recent)

### Phase 19: SkinData→Skin Loading Pipeline (complete)

+1,469 lines across 6 files, +20 tests. All sub-phases done:

- [x] **19a:** JsonSkinObjectLoader base — complete conversion methods for all JsonSkin types (Image, ImageSet, Text, Value, Slider, Graph, GaugeGraph, JudgeGraph, BpmGraph, NoteSet, SongList, PMchara, HiddenCover, LiftCover, BGA, Judge). 820+ lines added to json_skin_object_loader.rs
- [x] **19b:** Screen-specific loaders — PlaySkinObjectLoader (note field, gauge, judge, lane cover, BGA), SelectSkinObjectLoader (bar list rendering). Decide/Result/Course/KeyConfig/SkinConfig remain minimal (delegate to base, matching Java)
- [x] **19c:** LuaSkinLoader — `load_header()` and `load_skin()` implemented via mlua. `from_lua_value()` recursive converter: LuaTable → JsonSkin data tree. 280 lines
- [x] **19d:** SkinLoader entry points — `load()` routes to JSONSkinLoader or LuaSkinLoader based on file extension. `load_skin()` wired to screen-specific object loader creation. JSONSkinLoader `load_skin()` fully connected

### Phase 20: IRConnection Integration (complete)

+263 lines across 6 files + 2 new files, +13 tests:

- [x] `IRSendStatus` — full `send()` implementation: calls `connection.send_play_data()`, checks response, updates `is_sent`/`retry`. `send_course()` for course results. 250 lines
- [x] `IRInitializer` — `initialize_ir()` method: iterates player IR configs, creates connections via `IRConnectionManager`, calls login, returns `Vec<IRStatus>`. 107 lines
- [x] `IRResend` — `IRResendLoop` with exponential backoff (`4^retry * 1000ms`), periodic retry via `tokio::time::interval`, configurable max retries. 232 lines
- [x] `IRStatus` — updated with `connection: Arc<dyn IRConnection>`, `config`, `player` fields
- [x] IR stub comments updated to "real implementations (Phase 20)" in beatoraja-result/stubs.rs

## Blocked Tasks

### Phase 16b: Golden Master Test Activation (partially complete)

- [ ] Add missing fixtures for modules not yet covered (modmenu, select bar, stream) — deferred until Rust-side APIs are implemented
- [ ] Reactivate `compare_render_snapshot.rs` — blocked: rendering pipeline not yet connected to wgpu. SkinLoader now functional but SkinObject→GPU rendering gap remains

### Phase 18e: Stub replacement (remaining items blocked)

- [ ] Replace `MainState` stubs with real trait impls — blocked: requires per-screen concrete types (PlayState, SelectState, etc.)
- [ ] Remove all `stubs.rs` files — blocked: depends on above + rendering/database implementations
- [ ] beatoraja-external LibGDX stubs (Pixmap/GdxGraphics/BufferUtils/PixmapIO) — blocked on wgpu rendering pipeline

### Phase 18f: Integration verification (remaining items blocked)

- [ ] Activate `compare_render_snapshot.rs` — partially unblocked: skin loading pipeline done, but SkinObject→GPU rendering not connected
- [ ] E2E gameplay flow test: select → decide → play → result screen transitions — blocked: requires per-screen MainState impls
- [ ] Final verification: all tests pass, zero clippy warnings, clean `cargo fmt` — blocked: final gate

### Known Issues (open)

- [ ] SkinObject→GPU rendering gap: SkinLoader produces Skin with SkinObjects, but no wgpu draw calls yet
- [ ] Remaining stubs: ~2,540 lines across 16 stubs.rs files — blocked by rendering, database, per-screen implementations
- [ ] MainController has ~20 stub methods (state transitions, state management, database access) — blocked on Phase 21

## Next Phases (planned)

### Phase 21: Per-Screen MainState Implementations

Unblocks: Phase 18e MainState stub removal, E2E screen transitions, MainController state management stubs

- [ ] `PlayState` (BMSPlayer) — implements `MainState` for gameplay screen (~1,219 lines Java). render() game loop, judge processing, BGA, gauge
- [ ] `SelectState` (MusicSelector) — implements `MainState` for song select screen (~742 lines Java). Bar rendering, preview music, song DB
- [ ] `DecideState` (MusicDecide) — implements `MainState` for decide screen (71 lines Java). Already partially translated (114 lines Rust)
- [ ] `ResultState` (MusicResult) — implements `MainState` for result screen (~498 lines Java). Score display, IR upload, replay save. Already partially translated (569 lines Rust)
- [ ] `KeyConfigState` / `SkinConfigState` — implements `MainState` for config screens
- [ ] Wire screen transitions: select → decide → play → result via MainController.change_state()
- [ ] Connect MainController lifecycle: getCurrentState, getStateType, initializeStates

### Phase 22: Rendering Pipeline (SkinObject→GPU)

Unblocks: Phase 16b render snapshot tests, Phase 18f E2E tests, visual output

- [ ] Wire SkinObject draw calls to wgpu render pass
- [ ] Implement SkinText/SkinNumber/SkinImage GPU rendering
- [ ] Connect SkinObjectRenderer dispatch
- [ ] Frame timing and animation system

### Phase 23: Database Integration

Unblocks: SongDatabaseAccessor stubs, PlayDataAccessor stubs

- [ ] Wire rusqlite SongDatabaseAccessor with real schema
- [ ] Implement PlayDataAccessor for score persistence
- [ ] Connect to MusicSelector song list loading
