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

## Remaining Implementation Checklist

Phase 0-14 完了済み（16 crate, 1,114 テスト, ~60,000行）。以下は未移植コードの実装チェックリスト。

### Phase 15: Pattern Modifiers 追加 (`bms-pattern`)

依存: Phase 3 完了。Java 7 ファイル / ~750行 → Rust 7 新規 + 2 修正

- [x] **15-1. AutoplayModifier** — 指定レーンを自動演奏に移動 (`autoplay_modifier.rs`)
- [x] **15-2. ExtraNoteModifier** — BG レーンからノーツ追加 (`extra_note_modifier.rs`)
- [x] **15-3. LongNoteModifier** — LN 除去/追加 (`longnote_modifier.rs`)
- [x] **15-4. MineNoteModifier** — 地雷ノート除去/追加 (`mine_note_modifier.rs`)
- [x] **15-5. ModeModifier** — プレイモード変換 7k→9k DP 等 (`mode_modifier.rs`)
- [x] **15-6. PracticeModifier** — 練習モード範囲外ノーツ移動 (`practice_modifier.rs`)
- [x] **15-7. ScrollSpeedModifier** — スクロール速度変更/固定 (`scroll_speed_modifier.rs`)
- [x] **15-8. PatternModifier trait 拡張** — `move_to_background()` ヘルパー, `rebuild_pair_indices()` 追加
- [x] **15-9. テスト** — 各 modifier のユニットテスト

### Phase 16: Song Analysis & Utility (`bms-database`)

依存: Phase 5 完了。Java 4 ファイル / ~700行 → Rust 4 新規 + 1 修正

- [x] **16-1. SongInformation** — 楽曲詳細情報 (密度, ピーク, 分布データ)
- [x] **16-2. SongInformationAccessor** — 楽曲情報 DB アクセス (rusqlite)
- [x] **16-3. SongUtils** — CRC32 パスハッシュ
- [x] **16-4. ScoreDataProperty** — スコアレート, ランク判定, ゴースト比較
- [x] **16-5. テスト** — GM テスト + ユニットテスト

### Phase 17: Course & Table データ (`bms-database`)

依存: Phase 5, 16 完了。Java 6 ファイル / ~740行 → Rust 5 新規 + 1 修正

- [x] **17-1. CourseData** — コースデータ構造 (constraints, trophies)
- [x] **17-2. CourseDataAccessor** — コースデータ JSON ファイル I/O
- [x] **17-3. RandomCourseData / RandomStageData** — ランダムコース選択
- [x] **17-4. TableData** — 難易度表データ (.bmt GZIP / .json)
- [x] **17-5. TableDataAccessor** — 難易度表ファイルアクセス + HTTP 取得
- [x] **17-6. テスト** — ラウンドトリップ + ファイル I/O テスト

### Phase 18: BGA システム (`bms-audio` + `bms-render`)

依存: Phase 8, 10, 11 完了。Java 5 ファイル / ~930行 → Rust 6 新規 + 3 修正

- [x] **18-1. BGA リソース管理基盤** — BGAProcessor, BGImageProcessor (画像キャッシュ)
- [x] **18-2. 動画デコーダー** — MovieProcessor trait 定義 + スタブ実装 (動画デコードは依存が重いため後回し)
- [x] **18-3. SkinBGA** — BGA スキンオブジェクト (bms-skin)
- [x] **18-4. BGA 描画統合** — prepare_bga(), draw_bga(), ミスレイヤー
- [ ] **18-5. PlayState 結線** — BGAProcessor 統合 (Phase 21 で部分統合済み、ランタイム結線は未完)
- [x] **18-6. テスト** — 画像キャッシュ, タイムライン走査, StretchType 計算

### Phase 19: State-Specific Skin Objects & Loaders (`bms-skin` + `bms-render`)

依存: Phase 9, 10, 18 完了。Java ~5,180行 → Rust 12 新規 + 6 修正

**19-A: Play スキンオブジェクト**
- [x] **19-A1. SkinNote** — ノート描画オブジェクト (テクスチャ配列, レーン位置)
- [x] **19-A2. SkinHidden / SkinLiftCover** — ヒドゥン・リフトカバー
- [x] **19-A3. SkinJudge** — 判定表示オブジェクト
- [x] **19-A4. PlaySkin** — Play 状態固有スキンコンテナ
- [x] **19-A5. PomyuCharaLoader** — ポミュキャラスキン (スタブ)
- [ ] **19-A6. JSON Play Loader 拡張** — note/bga/judge ハンドリング (struct 完成済み、ローダー組込み未実装)
- [x] **19-A7. LR2 Play Loader 拡張** — タイミングコマンド, Hidden/Lift/Judge フィールドパース実装済み (テクスチャ参照は Bevy 側で解決)

**19-B: Select スキンオブジェクト**
- [x] **19-B1. SkinBar** — 選曲バーオブジェクト
- [x] **19-B2. SkinDistributionGraph** — 分布グラフ
- [x] **19-B3. MusicSelectSkin** — Select 状態固有スキンコンテナ
- [ ] **19-B4. JSON Select Loader 拡張** — songlist ハンドリング (struct 完成済み、ローダー組込み未実装)
- [x] **19-B5. LR2 Select Loader 拡張** — collect_select_config() distribution_graph 抽出実装済み

**19-C: Result / Decide / CourseResult / Config スキン**
- [x] **19-C1. Result/Decide/CourseResult スキン** — 各状態スキンコンテナ
- [x] **19-C2. LR2 Result Loader** — SRC ハンドラーフィールドパース実装済み (gauge/note/bpm/timing chart)
- [x] **19-C3. LR2 CourseResult Loader** — Result Loader と共通ハンドラー、STARTINPUT rank_time 実装済み
- [x] **19-C4. KeyConfig / SkinConfig Loader** — Java 側空実装のため対応不要を確認済み

**19-D: 追加スキンオブジェクト**
- [x] **19-D1. SkinFloat** — 浮動小数点数表示オブジェクト

**19-E: テスト**
- [x] **19-E1.** SkinNote/SkinBar/SkinFloat ユニットテスト
- [ ] **19-E2.** JSON/LR2 スキン状態固有オブジェクト読み込み GM テスト (ローダー組込み後に実施)

### Phase 20: Lua Scripting Extensions & Remaining Features

依存: Phase 9, 16 完了。Java 4 ファイル / ~680行 → Rust 5 新規 + 3 修正

- [x] **20-1. Lua EventUtility** — Lua スクリプト用イベントヘルパー (mlua UserData)
- [x] **20-2. Lua TimerUtility** — Lua スクリプト用タイマーヘルパー (mlua UserData)
- [x] **20-3. MessageRenderer** — ゲーム内メッセージ表示
- [x] **20-4. RivalDataAccessor** — ライバルデータ DB アクセス
- [x] **20-5. テスト** — ユニットテスト

### Phase 21: State 完全実装 & 全機能統合

依存: Phase 15-20 全完了。Rust 8 修正 + 3 新規

- [x] **21-1. MusicSelectState 完全実装** — BarManager フル, コースバー統合 (SkinBar 描画・SongInformation 表示はローダー組込み後)
- [x] **21-2. ResultState 完全実装** — スコア保存, コーススコア蓄積, ステージ遷移 (IR 送信は bms-ir ランタイム接続が必要なため未実装)
- [x] **21-3. CourseResult 完全実装** — コース全曲スコア集計, トロフィー判定, DB 保存 (コース IR 送信は未実装)
- [x] **21-4. Course Mode 統合** — CourseData → Play → Result → 次曲ループ, PlayerResource コース管理
- [x] **21-5. Pattern Modifier 統合** — LN/Mine/Scroll modifier 適用フロー, assist レベル追跡
- [x] **21-6. E2E テスト** — コースフロー + modifier + Result/CourseResult ユニットテスト

### Reference Files

| Category | Java Reference | Rust Target |
|----------|---------------|-------------|
| Pattern Modifier | `lr2oraja-java/.../pattern/*.java` (14 files) | `bms-pattern/src/` |
| State-specific Skin | `lr2oraja-java/.../skin/json/Json*SkinObjectLoader.java` | `bms-skin/src/loader/json_loader.rs` |
| LR2 Skin Loader | `lr2oraja-java/.../skin/lr2/LR2*SkinLoader.java` | `bms-skin/src/loader/lr2_csv_loader.rs` |
| BGA | `lr2oraja-java/.../play/bga/*.java` | `bms-render/src/bga/` (new) |
| Course | `lr2oraja-java/.../CourseData.java` etc. | `bms-database/src/` |
| Song Analysis | `lr2oraja-java/.../song/SongInformation.java` | `bms-database/src/` |

### Dependency Graph

```
Phase 15 (Pattern Modifiers)  ←── Independent
Phase 16 (Song Analysis)      ←── Independent
Phase 17 (Course & Table)     ←── Phase 16
Phase 18 (BGA)                ←── Independent
Phase 19 (Skin Loaders)       ←── Phase 18
Phase 20 (Lua & Others)       ←── Phase 16
Phase 21 (Full Integration)   ←── Phase 15-20 all completed
```

Phase 15, 16, 18 are parallelizable.

### Verification per Phase

1. `cargo check --workspace`
2. `cargo test --workspace`
3. `cargo clippy --workspace --fix`
4. `cargo fmt`
5. Phase-specific GM tests
