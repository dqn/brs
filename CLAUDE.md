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
- **`#RANDOM` is non-deterministic:** Java exporter has no fixed-seed support, so GM tests for BMS files containing `#RANDOM` require adding a seed argument to the Java exporter.
- **Avoid JavaFX dependencies:** `core:compileJava` fails due to JavaFX. Keep the GM exporter in the separate `golden-master` Gradle subproject depending only on jbms-parser + Jackson. Apply the same pattern when adding exports for new modules.
- **Regenerate fixtures with `just golden-master-gen`:** Always regenerate after modifying the Java exporter to keep Rust tests in sync.

## Tech Stack

| Area | Java | Rust |
|------|------|------|
| Graphics | LibGDX (LWJGL3) | Bevy |
| Audio | PortAudio / GDX | Kira |
| Skin (Lua) | LuaJ | mlua |
| Database | SQLite (JDBC) | rusqlite |
| Timing | long (μs) | i64 (μs) |
| GUI | JavaFX / ImGui | egui |
| Discord RPC | JNA IPC | discord-rich-presence |
| OBS | WebSocket | tokio-tungstenite |

## Key Principles

- All timing uses integer microseconds (i64) to avoid floating-point drift
- LongNote references use index-based approach (no circular references)
- `java.util.Random(seed)` LCG must be reproduced exactly for pattern shuffle
- LR2 judge scaling (`lr2JudgeScaling`) uses pure integer arithmetic

## Plans

- Detailed porting plan: `.claude/plans/iridescent-tumbling-swan.md`
- Critical algorithms: `.claude/plans/critical-algorithms.md`
- Java module analysis: `.claude/plans/java-module-analysis.md`

---

## Implementation Checklist

各フェーズを順番に実装するためのチェックリスト。完了したタスクには `[x]` をつけて進捗を管理する。

### Phase 0: Foundation

- [ ] **0-0. CLAUDE.md** — (完了済み)
- [x] **0-1. Rust workspace 初期化**
  - [x] `lr2oraja-rust/Cargo.toml` (workspace) 作成
  - [x] 全 crate の `Cargo.toml` + `src/lib.rs` スタブ作成 (bms-model, bms-rule, bms-pattern, bms-replay, bms-database, bms-config, bms-input, bms-audio, bms-skin, bms-render, bms-ir, bms-external, bms-launcher, bms-stream, bms-download, brs)
  - [x] `golden-master/` crate 作成
  - [x] `cargo check --workspace` 通過確認
- [x] **0-2. Java ビルド確認**
  - [x] `cd lr2oraja-java && git submodule update --init --recursive`
  - [x] `./gradlew golden-master:compileJava` 成功 (core:compileJava は JavaFX 依存欠落のため失敗; golden-master サブプロジェクトで回避)
- [x] **0-3. Golden Master テストインフラ**
  - [x] Java: `GoldenMasterExporter` クラス作成 (CLI で JSON 出力) — `lr2oraja-java/golden-master/`
  - [x] parse モジュールの export 対応 (全13テスト BMS の fixture JSON 生成済み)
  - [ ] judge / gauge モジュールの export 対応 (Phase 2 で追加)
  - [x] Rust: `golden-master/src/lib.rs` に比較ハーネス作成
  - [x] Rust: `golden-master/tests/compare_fixtures.rs` に各テスト BMS の比較テスト作成
  - [x] 4/11 テスト通過 (minimal_7k, 5key, 14key_dp, empty_measures), 7 テスト失敗 (bms-model パーサーバグ検出)
- [x] **0-4. テスト用 BMS ファイル作成**
  - [x] `test-bms/minimal_7k.bms`
  - [x] `test-bms/longnote_types.bms`
  - [x] `test-bms/bpm_change.bms`
  - [x] `test-bms/stop_sequence.bms`
  - [x] `test-bms/mine_notes.bms`
  - [x] `test-bms/empty_measures.bms`
  - [x] `test-bms/5key.bms`, `test-bms/9key_pms.bms`, `test-bms/14key_dp.bms`
  - [x] `test-bms/scratch_bss.bms`
  - [x] `test-bms/random_if.bms`
  - [x] `test-bms/encoding_sjis.bms`, `test-bms/encoding_utf8.bms`
- [x] **0-5. justfile 作成**

### Phase 1: BMS Data Model (`bms-model`)

参照: jbms-parser (`BMSModel`, `Note`, `TimeLine`, `Lane`, `Mode`, `BMSDecoder`)

- [x] **1-1. データ型定義**
  - [x] `PlayMode` enum (Beat5K, Beat7K, Beat10K, Beat14K, PopN9K, Keyboard24K)
  - [x] `NoteType` enum (Normal, LongNote, ChargeNote, HellChargeNote, Mine, Invisible)
  - [x] `Note` struct (lane, note_type, time_us, end_time_us, wav_id, damage)
  - [x] `BmsModel` struct (title, artist, mode, judge_rank, total, bpm, ln_mode, notes, etc.)
  - [x] `TimeLine`, `BpmChange`, `StopEvent` structs
- [x] **1-2. BMS パーサー**
  - [x] ヘッダコマンド (#PLAYER, #GENRE, #TITLE, #ARTIST, #BPM, #RANK, #TOTAL, #LNTYPE)
  - [x] WAV/BMP 定義 (#WAVxx, #BMPxx)
  - [x] チャンネルデータ (#xxxCH:data) パース
  - [x] BPM 変更チャンネル (03, 08)
  - [x] STOP チャンネル (09)
  - [x] LongNote (51-69 / LNTYPE 1,2)
  - [x] 地雷ノート (D1-D9)
  - [x] 不可視ノート (31-39)
  - [x] 小節長変更 (02)
  - [x] タイムライン構築 (BPM → μs 変換)
- [x] **1-3. エンコーディング対応**
  - [x] `encoding_rs` で Shift_JIS / EUC-JP / UTF-8 自動判定
- [x] **1-4. #RANDOM / #IF 対応**
- [x] **1-5. .bmson フォーマット対応**
  - [x] `bmson.rs` — JSON デシリアライズ構造体 (Bmson, BmsInfo, SoundChannel, SoundNote, MineChannel, BpmEvent, StopEvent, etc.)
  - [x] `bmson_decode.rs` — BmsonDecoder 実装 (Java BMSONDecoder.java 忠実再現)
  - [x] `PlayMode::from_mode_hint()` — mode_hint 文字列からモード判定
  - [x] テスト bmson ファイル 5 種作成 (minimal_7k, bpm_change, longnote, stop_sequence, mine_invisible)
  - [x] Java GoldenMasterExporter に .bmson 対応追加 (BMSONDecoder 使用)
  - [x] Golden Master テスト 5/5 通過 (wav_id 比較含む)
- [x] **1-6. ハッシュ計算** (MD5, SHA256)
- [x] **1-7. Golden Master テスト**
  - [x] Java で全テスト BMS/bmson をパース → fixture JSON 生成 (`just golden-master-gen`)
  - [x] Rust パース結果と fixture 比較 — 18/18 通過 (BMS 13 + bmson 5, ignore 0件)
    - [x] BPM 変更チャンネル (ch 03) の hex 変換修正 (base36→hex)
    - [x] STOP イベントの位置判定条件修正 (p <= pos → p < pos)
    - [x] ノート重複除去 (Normal > LN > Mine 優先度)
    - [x] Channel assignment table によるレーンマッピング修正
    - [x] PMS モード検出を .pms 拡張子ベースに修正
    - [x] Shift_JIS ハッシュ計算を raw bytes ベースに修正
    - [x] Java exporter に UTF-8 BOM 検出 + メタデータ上書きを追加 (encoding_utf8 テスト修正)
    - [x] Java exporter に random_seeds.json サポート追加 + Rust decode_with_randoms API (random_if テスト修正)
  - [x] LongNote pair 整合性テスト (7テスト: basic, sequential, multi_lane, unclosed, end_wav_id, charge_note, hell_charge_note)

### Phase 2: Rule Engine (`bms-rule`) — CRITICAL

参照: `JudgeProperty.java`, `JudgeAlgorithm.java`, `JudgeManager.java`, `GrooveGauge.java`, `GaugeProperty.java`
アルゴリズム詳細: `.claude/plans/critical-algorithms.md`

- [x] **2-1. JudgeProperty (判定窓)**
  - [x] `JudgeWindowRule` enum (Normal, Pms, Lr2)
  - [x] 5モードの判定窓ベーステーブル (FIVEKEYS, SEVENKEYS, PMS, KEYBOARD, LR2)
  - [x] NORMAL: judgerank [25,50,75,100,125] スケーリング
  - [x] PMS: judgerank [33,50,70,100,133] スケーリング (PG固定)
  - [x] LR2: `lr2_judge_scaling()` — LR2_SCALING テーブル + 2次元補間 (i64 整数演算のみ)
  - [x] `judge_window_rate` (fixjudge) 対応
  - [ ] GM テスト: 全5モード × 全ランク × 4ノートタイプ × 3 judgeWindowRate
- [x] **2-2. JudgeAlgorithm (判定アルゴリズム)**
  - [x] `Combo` — 最も近い1ノートを選択
  - [x] `Duration` — 判定窓内の最も古いノートを選択
  - [x] `Lowest` — 最もゆるい判定のノートを選択
  - [x] `Score` — スコア最大化ノートを選択
- [x] **2-3. GaugeProperty (ゲージ仕様)**
  - [x] 9種 × 5モード = 45 種のゲージ要素定義
  - [x] `GaugeModifier::Total` — `f * total / total_notes`
  - [x] `GaugeModifier::LimitIncrement` — 回復量制限
  - [x] `GaugeModifier::ModifyDamage` — TOTAL補正 × ノート数補正
  - [ ] GM テスト: 全ゲージタイプの初期値・ボーダー・死亡閾値・増減値
- [x] **2-4. GrooveGauge (ゲージ更新)**
  - [x] `update()` — 判定 → 増減 → guts 軽減 → clamp → 即死判定
  - [x] `value > 0` のときのみ更新 (復帰不可)
  - [x] guts テーブルによるダメージ軽減
  - [ ] GM テスト: 判定シーケンス → 各ステップのゲージ値比較
- [ ] **2-5. JudgeManager (判定処理)** ※ Phase 11 で実装 (AudioDriver, InputProcessor 等の依存が必要)
  - [ ] 通常ノート: 判定窓内で JudgeAlgorithm による最適選択
  - [ ] LN: 押下開始 + releasemargin 後の離し判定
  - [ ] CN: 離し → releasemargin 以内の再押下で復帰
  - [ ] HCN: 200000μs 毎ゲージ増減 + 離し再押下復帰
  - [ ] BSS: 反転入力で終端判定
  - [ ] MultiBadCollector: BAD 窓内の複数ノート同時 BAD
  - [ ] GM テスト: 入力シーケンス (key, time_us, pressed) + BMS → ScoreData 比較
- [x] **2-6. ScoreData / ClearType**
  - [x] スコアデータ (epg/lpg, egr/lgr 等の早遅分離)
  - [x] クリアタイプ判定

### Phase 3: Pattern Shuffle (`bms-pattern`)

参照: `LaneShuffleModifier.java`, `NoteShuffleModifier.java`

- [ ] **3-1. java.util.Random LCG 再現**
  - [ ] `seed = (seed * 0x5DEECE66D + 0xB) & ((1 << 48) - 1)`
  - [ ] `next_int(bound)` の実装
  - [ ] Java との出力列一致テスト
- [ ] **3-2. LaneShuffleModifier (7種)**
  - [ ] Mirror
  - [ ] Rotate (n)
  - [ ] Random
  - [ ] Cross
  - [ ] Flip
  - [ ] Battle
  - [ ] PlayableRandom (9! 全探索 + murioshiChords テーブル)
- [ ] **3-3. NoteShuffleModifier**
  - [ ] タイムライン単位の置換
- [ ] **3-4. LongNote pair 整合性維持**
- [ ] **3-5. GM テスト**: 固定 seed × 各パターン → 出力配列比較

### Phase 4: Replay / Ghost (`bms-replay`)

参照: `ReplayData.java`, `LR2GhostData.java`, `KeyInputLog.java`

- [ ] **4-1. ReplayData** — serialize / deserialize
- [ ] **4-2. LR2GhostData** — ゴーストデータ
- [ ] **4-3. KeyInputLog** — 入力ログ
- [ ] **4-4. GM テスト**: エンコード/デコード ラウンドトリップ + Java 出力比較

### Phase 5: Database (`bms-database`)

参照: `SQLiteSongDatabaseAccessor.java`, `ScoreDatabaseAccessor.java`, `SongData.java`, `FolderData.java`

- [ ] **5-1. SongDatabase** — 楽曲 DB スキーマ + CRUD (rusqlite)
- [ ] **5-2. ScoreDatabase** — スコア DB スキーマ + CRUD
- [ ] **5-3. ScoreDataLogDatabase** — スコアログ
- [ ] **5-4. GM テスト**: Java 生成 DB を Rust で読込 → 全レコード比較

### Phase 6: Config (`bms-config`)

参照: `Config.java`, `PlayerConfig.java`, `PlayConfig.java`, `PlayModeConfig.java`, `SkinConfig.java`, `AudioConfig.java`, `IRConfig.java`

- [ ] **6-1. Config structs** — `#[serde(rename_all = "camelCase")]`
- [ ] **6-2. PlayerConfig / PlayConfig / PlayModeConfig**
- [ ] **6-3. SkinConfig / AudioConfig / IRConfig**
- [ ] **6-4. GM テスト**: JSON 読み書き互換テスト

### Phase 7: Input (`bms-input`)

参照: `BMSPlayerInputProcessor.java`, `KeyBoardInputProcesseor.java`, `BMControllerInputProcessor.java`, `MidiInputProcessor.java`

- [ ] **7-1. キーボード入力** (winit)
- [ ] **7-2. ゲームパッド入力** (gilrs)
- [ ] **7-3. MIDI 入力** (midir)
- [ ] **7-4. マウススクラッチ**
- [ ] **7-5. KeyInputLog** serialize/deserialize
- [ ] **7-6. キーマッピング設定**

### Phase 8: Audio (`bms-audio`)

参照: `KeySoundProcessor.java`, `AbstractAudioDriver.java`, `BMSRenderer.java`, `BMSLoudnessAnalyzer.java`, PCM 系, `MSADPCMDecoder.java`, `FlacProcessor.java`

- [ ] **8-1. PCM デコーダー** (WAV, OGG, FLAC, MS-ADPCM)
- [ ] **8-2. キー音プロセッサー** (Kira)
- [ ] **8-3. オーディオドライバー**
- [ ] **8-4. ラウドネス解析**
- [ ] **8-5. BGM レンダラー**
- [ ] **8-6. テスト**: キー音発音タイミング + 音量計算

### Phase 9: Skin System (`bms-skin`) — LARGEST

参照: 52 Java ファイル (`JSONSkinLoader`, `LR2SkinCSVLoader`, `LuaSkinLoader`, SkinObject 各種, `SkinProperty`, `IntegerPropertyFactory`, `BooleanPropertyFactory`)

- [ ] **9-1. SkinObject 型定義**
  - [ ] Image, Number, Text, Slider, Graph, Gauge, Judge
  - [ ] BPMGraph, HitErrorVisualizer, NoteDistributionGraph, TimingDistributionGraph, TimingVisualizer
- [ ] **9-2. SkinProperty / SkinPropertyMapper**
  - [ ] IntegerProperty (150+ ID)
  - [ ] BooleanProperty
  - [ ] FloatProperty, StringProperty, TimerProperty
- [ ] **9-3. JSON スキンローダー**
- [ ] **9-4. LR2 CSV スキンローダー**
- [ ] **9-5. Lua スキンローダー** (mlua)
- [ ] **9-6. SkinHeader パーサー**
- [ ] **9-7. GM テスト**: 各ローダーでスキン読込 → SkinObject 構造 JSON 比較

### Phase 10: Rendering (`bms-render`)

参照: `SpriteBatchHelper.java`, `ShaderManager.java`, `MessageRenderer.java`

- [ ] **10-1. Bevy セットアップ** (window, camera, render pipeline)
- [ ] **10-2. スプライトバッチ** (Bevy Sprite)
- [ ] **10-3. ブレンドモード** (Alpha, Additive)
- [ ] **10-4. フォントレンダリング** (TTF + ビットマップ)
- [ ] **10-5. テスト**: Screenshot + SSIM 比較 (image-compare crate)

### Phase 11: State Machine (`brs`)

参照: `MainController.java`, `MainState.java`, `TimerManager.java`, 各 State クラス

- [ ] **11-1. MainState enum** (Select, Decide, Play, Result, CourseResult, KeyConfig, SkinConfig)
- [ ] **11-2. TimerManager**
- [ ] **11-3. MusicSelect state**
- [ ] **11-4. MusicDecide state**
- [ ] **11-5. Play state** (BMSPlayer)
- [ ] **11-6. Result state**
- [ ] **11-7. CourseResult state**
- [ ] **11-8. KeyConfiguration / SkinConfiguration states**
- [ ] **11-9. E2E テスト**: パース → Judge → Gauge → ScoreData 一気通貫

### Phase 12: Internet Ranking (`bms-ir`)

参照: `IRConnection.java`, `LR2IRConnection.java`, `IRConnectionManager.java`, `IRScoreData.java`

- [ ] **12-1. IR プロトコル** (reqwest)
- [ ] **12-2. LR2IR 互換接続**
- [ ] **12-3. ランキングデータ / キャッシュ**
- [ ] **12-4. テスト**: HTTP モック + serialize/deserialize

### Phase 13: Peripherals

- [ ] **13-1. Discord RPC** (`bms-external`) — discord-rich-presence
- [ ] **13-2. OBS 連携** (`bms-external`) — tokio-tungstenite
- [ ] **13-3. ストリーミング** (`bms-stream`)
- [ ] **13-4. 楽曲ダウンローダー** (`bms-download`)
- [ ] **13-5. 設定 GUI** (`bms-launcher`) — egui (18 JavaFX views → egui panels)
- [ ] **13-6. ModMenu** — egui (7 ImGui views → egui)
- [ ] **13-7. その他** (BMSSearch, Screenshot, Webhook, ScoreImporter)

### Phase 14: Integration & GUI Testing

- [ ] **14-1. Screenshot Testing パイプライン**
  - [ ] Java reference screenshots 生成
  - [ ] Rust test screenshots 生成
  - [ ] SSIM 比較 (> 0.95)
- [ ] **14-2. E2E テスト**
  - [ ] 同一 ReplayData を Java/Rust 再生 → ScoreData 比較
  - [ ] 全モード × 全ゲージタイプ × 全パターンの網羅テスト
- [ ] **14-3. パフォーマンステスト**
