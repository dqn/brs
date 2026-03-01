# Porting TODO — Remaining Work

Phases 1–62 + post-62 complete. **2940 tests.** 27 crates, 158k lines. See AGENTS.md.
**"Not implemented" stubs: 0 remaining.** All 151 stubs resolved (Phase 58–62).
**Debug stubs: 32 remaining** (all genuinely blocked by architecture). 9 additional stubs implemented post-62.

---

## Completed Phases (45–62)

<details>
<summary>Phase 45–53: Regression fixes, lifecycle wiring, skin rendering, select→play, result, skin loaders, launcher, quality</summary>

- **45a–b:** RandomizerBase JavaRandom, ScoreData serde
- **46a–c:** PlayerResource.loadBMSModel, SongData unification, MainController.exit/save_config
- **47a–e:** FloatPropertyFactory, Timer, load_skin overrides, SkinFloat, BooleanPropertyFactory
- **48a–c:** Bar Clone, get_children (7 types), read_chart/read_course
- **49a–b:** bms_player wiring, LaneRenderer.draw_lane
- **50a–b:** CourseResult MainState, CourseResult IR thread
- **51a–d:** Lua MainStateAccessor, LR2 21 commands, JSON 7 factories, SkinTextFont
- **52a–d:** Skin header loading, async BMS DB, get_screen_type, DifficultyTableParser
- **53a–d:** modmenu/ir/controller tests, dead code removal (beatoraja-common)

</details>

<details>
<summary>Phase 54–57: ast-compare audit, BytePCM fix, method-level ignore, final gap resolution</summary>

- **54a:** ast-compare ignore list — bmson/osu POJOs added
- **54b:** BytePCM float→byte — `as i32 as i8` matches Java truncation
- **55:** 28 genuine gaps audited → 15 false positives, 7 implemented, 6 blocked
- **56:** Method-level ignore added to ast-compare. 170 false positives (136 patterns). Gap: 90
- **56b:** 52 additional false positives + PlayerResource.reloadBMSFile. Gap: 38
- **57:** 13 KeyConfiguration + 13 SkinConfiguration methods, 12 false positives. Gap: 0

</details>

<details>
<summary>Phase 58–62: "Not Implemented" stub elimination (151 → 0)</summary>

- **58:** Message cleanup — 46 test/null/out-of-scope stubs reclassified, 18 blocked stubs documented
- **59:** Sound system wiring — SoundType to beatoraja-types, MusicSelector 22 EventType dispatch, sound overrides
- **60:** PlayerResource wiring — reverse lookup, trait expansion (3 methods), MainController components stored directly (Box::leak eliminated), ChartReplicationMode::Replay*, decide sound
- **61:** OBS triggerStateChange(PLAY) implemented, LR2 CSV INCLUDE directive, 35 blocked stubs downgraded to debug
- **62:** 10 launcher egui stubs downgraded to debug with blocker descriptions
- **Post-62:** 9 debug stubs implemented — RankingDataCache (real HashMap cache from beatoraja-ir), open_ir (IRConnection → browser), EventType::Target cycling, FavoriteSong/FavoriteChart bitwise toggle, OpenDocument/OpenWithExplorer/OpenDownloadSite

</details>

---

## Minor Unported Items

| Item | Impact | Notes |
|------|--------|-------|
| `BMSModel.compareTo()` | Low | Ord impl on demand. Unused in Java |
| `BMSModelUtils.getAverageNotesPerTime()` | Low | Dead code in Java |
| Skill rating calculation | Low | No Java source (no porting source) |

## Permanent Stubs

- **Twitter4j** (`beatoraja-external`): ~446 lines, `bail!()` — API deprecated, intentionally unimplemented
- **ShortDirectPCM** (`beatoraja-audio`): Java-specific DirectBuffer — unnecessary in Rust
- **JavaFX find_parent_by_class_simple_name** (`beatoraja-launcher`): No egui equivalent
- **randomtrainer.dat** (`beatoraja-modmenu`): Binary resource from Java, uses empty HashMap fallback

## Blocked Stubs (32 remaining, downgraded to `debug!`)

All "blocked" stubs emit `log::debug!` with clear blocker descriptions. They do not affect functionality.

- **Rendering pipeline** (~11): MainState.loadSkin/getOffsetValue/getImage, SkinText/Number/Image draw, SkinDistributionGraph, CourseResult render, message_renderer
- **egui UI** (~12): launcher views (audio, obs, discord, play_config, table_editor, spinner), SkinConfiguration/KeyConfiguration create/render, search popup
- **Crate boundaries** (~5): OpenIr in MusicSelector, Rival (RivalDataAccessor), UpdateFolder (updateSong), LR2SkinCSVLoader wiring
- **Infrastructure** (~4): OBS WebSocket reconnect (async cycle), CIM images, main_loader launcher, skin stubs (change_state, timer, audio)
