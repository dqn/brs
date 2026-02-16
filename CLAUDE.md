# lr2oraja Rust Porting Project (Bevy/Kira)

## Overview

lr2oraja (beatoraja fork, Java 313 files / 72,000+ lines) を Rust へ完全移植するプロジェクト。
周辺機能 (Launcher, ModMenu, OBS, Discord RPC, Downloader) を含む全機能が対象。
**このドキュメントは常に最新に保ち続けること。**

## Directory Structure

```
brs/
  lr2oraja-java/           # Java source (reference implementation)
  lr2oraja-rust/           # Rust port (Cargo workspace)
    crates/
      bms-model/           # BMS parser (self-made)
      bms-rule/            # Judge, gauge, score
      bms-pattern/         # Lane/note shuffle
      bms-replay/          # Replay, ghost data
      bms-database/        # Song/score DB (rusqlite)
      bms-config/          # Config (serde)
      bms-input/           # Keyboard, gamepad, MIDI
      bms-audio/           # Audio (kira)
      bms-skin/            # Skin system (mlua)
      bms-render/          # Rendering (bevy)
      bms-ir/              # Internet ranking (reqwest)
      bms-external/        # Discord RPC, OBS, webhook
      bms-launcher/        # Settings GUI (egui)
      bms-stream/          # Streaming integration
      bms-download/        # Song downloader
      brs/                 # Main binary
    golden-master/         # Test infrastructure
    test-bms/              # Test BMS files
  .claude/plans/           # Detailed porting plans and knowledge docs
```

## Development Guidelines

- **Strict Accuracy:** Judge calculations, BMS parsing, and timing management must be bit-identical to Java.
- **Autonomous Porting:** Claude analyzes `./lr2oraja-java` code and ports module by module to `./lr2oraja-rust`, starting from core modules with fewest dependencies.
- **Deferred Task Tracking:** 作業完了時に未完了のタスクがある場合、実装順序を考慮して `Deferred / Stub Items` セクションに追記すること。次回着手時に漏れなく把握できるようにする。

## Testing Rules

- **Golden Master Testing:** Export Java internal state as JSON, compare against Rust output.
- **TDD:** Red-Green-Refactor for every module.
- **GUI Screenshot Testing:** Capture screenshots from both Java and Rust, compare with SSIM.
- **Test BMS Files:** Claude creates minimal BMS files for each feature.
- **Java Modifications Allowed:** Adding debug output / export methods to Java code is permitted for verification.

### Golden Master Testing Lessons

Lessons learned from Phase 0-3 implementation. Refer to these when implementing future GM tests.

- **Watch for Java-Rust semantic differences:** The same field name may have different semantics (e.g., `wav_id` — Java uses wavlist array index 0-based with -2 for undefined, Rust uses raw base36 value). Verify that compared fields share the same semantics; skip or add conversion logic if they differ.
- **Use ±2μs tolerance for timing comparisons:** BPM → μs conversion produces floating-point rounding differences. ±1μs causes false negatives.
- **Java BMSDecoder hardcodes MS932:** UTF-8 BMS metadata and hashes are garbled on the Java side. Keep UTF-8 tests as `#[ignore]` until Java-side encoding detection is added.
- **`#RANDOM` is deterministic via `random_seeds.json`:** Java exporter reads per-file selectedRandom arrays from `test-bms/random_seeds.json`, and Rust tests must use matching `decode_with_randoms(...)` inputs.
- **Avoid JavaFX dependencies:** `core:compileJava` fails due to JavaFX. Keep the GM exporter in the separate `golden-master` Gradle subproject depending only on jbms-parser + Jackson. Apply the same pattern when adding exports for new modules.
- **Regenerate fixtures with `just golden-master-gen`:** Always regenerate after modifying the Java exporter to keep Rust tests in sync.
- **Parser fixture names must keep source extensions:** Use `filename.ext.json` (e.g. `9key_pms.bms.json`) to avoid collisions across `.bms/.pms` variants sharing the same stem.
- **RenderSnapshot parity triage should use category summaries:** Prefer `command_count / visibility / geometry / detail` counts to quickly detect whether regressions are structural or field-level.
- **Lua functions in skin DST fields must be preserved through JSON serialization:** `lua_value_to_json()` converts Lua functions to a `"__lua_function__"` sentinel string so that `PropertyRef::Script` preserves the "draw field is present" semantics. Without this, Lua `draw = function()` becomes null, causing `op` to be incorrectly used as `option_conditions` (Java ignores `op` when `dst.draw` is non-null).

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

## Key Principles

- All timing uses integer microseconds (i64) to avoid floating-point drift
- LongNote references use index-based approach (no circular references)
- `java.util.Random(seed)` LCG must be reproduced exactly for pattern shuffle
- LR2 judge scaling (`lr2JudgeScaling`) uses pure integer arithmetic

## Plans

- Detailed porting plan: `.claude/plans/iridescent-tumbling-swan.md`
- Critical algorithms: `.claude/plans/critical-algorithms.md`
- Java module analysis: `.claude/plans/java-module-analysis.md`
- Java vs Rust 機能差分分析: `.claude/plans/cryptic-frolicking-bengio.md`

## Implementation Status

Phase 0-23 全完了（16 crate, ~61,000行）。全 RenderSnapshot GM テストが strict parity 達成済み。

### Deferred / Stub Items

- **Launcher GUI** — Java 対応パネル全完了 (12パネル: audio, discord, input, ir, music_select, obs, play_option, resource, skin, stream, table_editor, video)。**Rust 独自拡張候補 (低優先):** スキンプレビュー (Java でも未実装)、ゲージ可視化、タイマー表示、イベントトレース、プロファイラ、スキンパネル強化 (カスタムオプション/ファイル編集)
- **Window 管理** — 完了 — 起動時モニター選択 + F6 フルスクリーントグル (モニター保持) + ModMenu Window Settings パネル (解像度/表示モード/VSync ランタイム変更) を実装済み。**未実装 (低優先):** ランチャーでのモニター自動列挙 (現在フリーテキスト入力)
- **IR プラグインシステム** — Java は `IRConnectionManager` でカスタム IR を動的ロードするが、Rust は LR2IR のみ静的実装
- **ライバルスコア表示 UI** — データ構造は存在するが MusicSelect 画面での表示統合が不明確
- **スクリーンショット Twitter 投稿** — ファイルエクスポートのみ。Java の `ScreenShotTwitterExporter` 相当なし
- **スキンロードエラー時のフォールバック UI** — スキン読み込み失敗時の代替表示なし
- **オーディオ障害リカバリ** — ゲームプレイ中の Kira オーディオ障害に対するフォールバック処理なし
- **ホットリロード (スキン/コンフィグ)** — `SkinManager` に `request_load()` は存在するが実際のリロードは未配線
- **Stream Controller (Windows Named Pipes)** — Unix ソケットは完了。Windows の Named Pipe (`\\.\pipe\beatoraja`) は未検証
- **選曲画面バータイプ** — Java 17種に対し Rust 5種 (Song, Folder, Course, TableRoot, HashFolder)。未実装: ContextMenuBar, RandomCourseBar, SameFolderBar, LeaderBoardBar, SearchWordBar, ExecutableBar, ContainerBar 等
- **選曲ソート** — Java 8種+LASTUPDATE に対し Rust 4種 (Default, Title, Artist, Level)。未実装: BPM, LENGTH, CLEAR, SCORE, MISSCOUNT
- **MusicSelectCommand** — Java 13種のコマンド (ハッシュコピー、IPFS/HTTP ダウンロード、コンテキストメニュー、同フォルダ表示等) が未移植
- **アーカイブ展開 (zip/lzh)** — `bms-download/extract.rs` は tar.gz のみ。BMS 主要配布形式の .zip と LR2 時代の .lzh が未対応
- **CLI 引数** — Java の `-a` (autoplay), `-p` (practice), `-r` (replay) 引数が未移植。Rust は `--bms` のみ
- **RhythmTimerProcessor** — PMS リズムノート拡大用タイマー (小節線/4分音符タイミング) 未実装
- **GhostBattlePlay** — ゴーストバトル用パターン共有 (Random + lane sequence の static 管理) 未実装
- **GithubVersionChecker** — バージョンチェック/自動更新なし
- **Config 細部** — songPreview (OFF/LOOP/SINGLE), skipDecideScreen, frameskip, analogScroll, cacheSkinImage, scrollduration, maxSearchBarCount, setClipboardScreenshot, updatesong 等が未移植
- **スクリーンショット SSIM テスト** — テストコード追加済みだが `skin_render_system` の Bevy ECS クエリ競合 (`Query` の `Transform`/`Visibility` が `MultiEntityMarker` 有無で衝突) により全テスト実行不可。`Without<T>` または `ParamSet` での修正が必要
- **result2.luaskin** — Lua スキンのデシリアライズエラーにより RenderSnapshot テストから除外中。JSON シリアライズパス要調査
- **新規スキン Java fixture** — play14, play7wide, course_result の Java RenderSnapshot fixture 未生成。`RenderSnapshotExporter` に skinTypeId=2 (PLAY_14KEYS) 等の MockState 対応追加 + `justfile` の `golden-master-render-snapshot-gen` 拡張が必要
