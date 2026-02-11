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
  - [x] judge / gauge モジュールの export 対応 (JudgeManagerExporter.java — 26 テストケース)
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
  - [x] GM テスト: 全5モード × 全ランク × 4ノートタイプ × 3 judgeWindowRate (480ケース通過)
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
  - [x] GM テスト: 全ゲージタイプの初期値・ボーダー・死亡閾値・増減値 (225ケース通過)
- [x] **2-4. GrooveGauge (ゲージ更新)**
  - [x] `update()` — 判定 → 増減 → guts 軽減 → clamp → 即死判定
  - [x] `value > 0` のときのみ更新 (復帰不可)
  - [x] guts テーブルによるダメージ軽減
  - [x] GM テスト: 判定シーケンス → 各ステップのゲージ値比較 (80ケース通過)
- [x] **2-5. JudgeManager (判定処理)** ※ Phase 11C で実装
  - [x] 通常ノート: 判定窓内で JudgeAlgorithm による最適選択
  - [x] LN: 押下開始 + releasemargin 後の離し判定
  - [x] CN: 離し → releasemargin 以内の再押下で復帰
  - [x] HCN: 200000μs 毎ゲージ増減 + 離し再押下復帰
  - [x] BSS: 反転入力で終端判定
  - [x] MultiBadCollector: BAD 窓内の複数ノート同時 BAD
  - [x] GM テスト: 入力シーケンス (key, time_us, pressed) + BMS → ScoreData 比較 (26 ケース × 6 グループ通過)
- [x] **2-6. ScoreData / ClearType**
  - [x] スコアデータ (epg/lpg, egr/lgr 等の早遅分離)
  - [x] クリアタイプ判定

### Phase 3: Pattern Shuffle (`bms-pattern`)

参照: `LaneShuffleModifier.java`, `NoteShuffleModifier.java`

- [x] **3-1. java.util.Random LCG 再現**
  - [x] `seed = (seed * 0x5DEECE66D + 0xB) & ((1 << 48) - 1)`
  - [x] `next_int(bound)` の実装
  - [x] Java との出力列一致テスト
- [x] **3-2. LaneShuffleModifier (7種)**
  - [x] Mirror
  - [x] Rotate (n)
  - [x] Random
  - [x] Cross
  - [x] Flip
  - [x] Battle
  - [x] PlayableRandom (9! 全探索 + murioshiChords テーブル)
- [x] **3-3. NoteShuffleModifier**
  - [x] タイムライン単位の置換 (SRandom, Spiral, AllScr, NoMurioshi, Converge — 5種)
  - [x] TimelineView によるグループ化 + LN 追跡 (ln_active HashMap)
  - [x] TimeBasedState (閾値ベース縦連打回避)
  - [x] BUTTON_COMBINATION_TABLE (PMS 無理押し防止 10パターン)
  - [x] enum dispatch (RandomizerImpl) による trait object 回避
- [x] **3-4. LongNote pair 整合性維持**
- [x] **3-5. GM テスト**: 固定 seed × 各パターン → 出力配列比較 (lane shuffle 102 + playable random 8 + battle 6 = 116 ケース通過)

### Phase 4: Replay / Ghost (`bms-replay`)

参照: `ReplayData.java`, `LR2GhostData.java`, `KeyInputLog.java`

- [x] **4-1. ReplayData** — serialize / deserialize
  - [x] 全フィールド (player, sha256, mode, keylog, keyinput, gauge, pattern, lane_shuffle_pattern, rand, date, randomoption/seed, doubleoption, config)
  - [x] `shrink()`: keylog → 9byte records (1byte keycode + 8byte i64 LE) → GZIP → Base64 URL-safe
  - [x] `validate()`: Base64 decode → GZIP decompress → 9byte parse → keylog 復元
  - [x] `.brd` ファイル I/O (`read_brd` / `write_brd`: GZIP JSON)
- [x] **4-2. LR2GhostData** — ゴーストデータ
  - [x] `GhostJudgment` enum, `LR2RandomOption` enum
  - [x] `parse(csv)`: CSV パース → options/seed/ghost デコード
  - [x] `decode_play_ghost(data)`: 文字置換テーブル (Java 順序厳守) → RLE デコード → judgment 配列
  - [x] レーン shuffle: LR2Random(seed) + Fisher-Yates → decimal encoded lane order
- [x] **4-3. KeyInputLog** — 入力ログ
  - [x] struct (presstime, keycode, pressed, time) + serde
  - [x] `get_time()` (presstime 優先、legacy time*1000 fallback), `validate()`
- [x] **4-4. LR2Random** — MT19937 variant
  - [x] 非標準 seeding: `69069 * seed + 1` (i32 wrapping)
  - [x] `generate_mt()`: twist + tempering
  - [x] `next_int(max)`: `(rand_mt() as u64 * max as u64) >> 32`
- [x] **4-5. GM テスト**: エンコード/デコード ラウンドトリップ + Java 出力比較
  - [x] LR2Random: 9 seeds × raw (700値) + nextInt (7 bounds × 20値) = 7560 assertions
  - [x] Ghost decode: 16 cases (置換テーブル + RLE)
  - [x] Keylog round-trip: 2 cases (basic + all keycodes 0..25)
  - [x] Lane order: 8 seeds

### Phase 5: Database (`bms-database`)

参照: `SQLiteSongDatabaseAccessor.java`, `ScoreDatabaseAccessor.java`, `SongData.java`, `FolderData.java`

- [x] **5-1. SongDatabase** — 楽曲 DB スキーマ + CRUD (rusqlite)
  - [x] `SongData` struct (29 DB カラム + feature/content flags + `from_model()`)
  - [x] `FolderData` struct (10 DB カラム)
  - [x] `schema.rs` — `ColumnDef`/`TableDef`, `ensure_table()` (CREATE TABLE + ALTER TABLE ADD COLUMN)
  - [x] `SongDatabase` — `open()`, `get_song_datas(key, value)`, `get_song_datas_by_hashes()`, `get_song_datas_by_text()`, `set_song_datas()`, `get_folder_datas()`, `set_folder_datas()`
  - [x] SQL injection prevention via key whitelist validation
- [x] **5-2. ScoreDatabase** — スコア DB スキーマ + CRUD
  - [x] `PlayerData` struct (17 フィールド, all i64)
  - [x] `PlayerInformation` struct (id, name, rank)
  - [x] `ScoreDatabase` — `open()`, `get_score_data()`, `get_score_datas()` (1000件チャンクローディング), `set_score_data()`, `delete_score_data()`, `get_player_datas()`, `set_player_data()`, `get_information()`, `set_information()`
  - [x] DB `combo` カラム ↔ ScoreData `maxcombo` フィールドの名前マッピング
  - [x] `avgjudge` DEFAULT 2147483647 (Java Integer.MAX_VALUE)
- [x] **5-3. ScoreDataLogDatabase** — スコアログ
  - [x] `ScoreDataLogDatabase` — `open()`, `set_score_data_log()` (INSERT OR REPLACE)
- [x] **5-4. PlayMode.mode_id()** — bms-model に mode_id()/from_mode_id() 追加
- [x] **5-5. ユニットテスト** — 15 テスト通過 (schema 3 + song_database 5 + score_database 4 + score_log_database 1 + 他)
- [x] **5-6. GM テスト**: SongData::from_model() の Java 出力比較 (18 BMS/bmson × 22 フィールド = 396+ assertions 通過)

### Phase 6: Config (`bms-config`)

参照: `Config.java`, `PlayerConfig.java`, `PlayConfig.java`, `PlayModeConfig.java`, `SkinConfig.java`, `AudioConfig.java`, `IRConfig.java`

- [x] **6-1. Config structs** — `#[serde(rename_all = "camelCase")]`
- [x] **6-2. PlayerConfig / PlayConfig / PlayModeConfig**
- [x] **6-3. SkinConfig / AudioConfig / IRConfig**
- [x] **6-4. GM テスト**: JSON 読み書き互換テスト (6 テスト通過: system/player deserialize + validate + round-trip)

### Phase 7: Input (`bms-input`)

参照: `BMSPlayerInputProcessor.java`, `KeyBoardInputProcesseor.java`, `BMControllerInputProcessor.java`, `MidiInputProcessor.java`

- [x] **7-1. Core types** — device.rs (DeviceType, InputEvent), controller_keys.rs (bm_keys), control_keys.rs (ControlKeys 31種), key_command.rs (KeyCommand 15種)
- [x] **7-2. KeyStateManager** — key_state.rs (256 key states, timestamps, analog tracking, 14 tests)
- [x] **7-3. AnalogScratch** — analog_scratch.rs (V1/V2 algorithms, compute_analog_diff, 18 tests)
- [x] **7-4. キーボード入力** — keyboard.rs (KeyboardBackend trait, VirtualKeyboardBackend, debounce, 14 tests)
- [x] **7-5. マウススクラッチ** — mouse_scratch.rs (MouseBackend trait, V1/V2 algorithms, MouseToAnalog, 16 tests)
- [x] **7-6. ゲームパッド入力** — controller.rs (poll_with generic, JKOC hack, analog scratch, 11 tests)
- [x] **7-7. MIDI 入力** — midi.rs (midir + mpsc, NOTE_ON/OFF, PITCH_BEND 14-bit, CC, 17 tests)
- [x] **7-8. InputProcessor + Autoplay** — input_processor.rs (key state + key logger orchestration, 14 tests), autoplay.rs (create_autoplay_log with BSS, 8 tests)
- [x] **7-9. Quality checks** — 121 tests passed, clippy clean, fmt applied

### Phase 8: Audio (`bms-audio`)

参照: `KeySoundProcessor.java`, `AbstractAudioDriver.java`, `BMSRenderer.java`, `BMSLoudnessAnalyzer.java`, PCM 系, `MSADPCMDecoder.java`, `FlacProcessor.java`

- [x] **8-1. PCM デコーダー** (WAV, OGG, FLAC, MP3, MS-ADPCM)
  - [x] `pcm.rs` — 統一 f32 PCM struct (change_sample_rate, change_channels, change_frequency, slice, strip_trailing_silence)
  - [x] `msadpcm.rs` — MS-ADPCM デコーダー (Java MSADPCMDecoder.java 忠実移植)
  - [x] `wav.rs` — WAV パーサー (PCM 8/16/24/32bit, MS-ADPCM, IEEE float, MP3-in-WAV)
  - [x] `ogg.rs` — OGG Vorbis デコーダー (lewton)
  - [x] `flac.rs` — FLAC デコーダー (claxon, 8/16/24/32bit)
  - [x] `mp3.rs` — MP3 デコーダー (minimp3)
  - [x] `decode.rs` — 統合ローダー (拡張子判定 + .wav→.flac→.ogg→.mp3 fallback)
- [x] **8-2. キー音プロセッサー**
  - [x] `key_sound.rs` — KeySoundProcessor (BGM 自動再生, タイムラインポインタ方式)
- [x] **8-3. オーディオドライバー**
  - [x] `driver.rs` — AudioDriver trait + OfflineAudioDriver (wav_map, slice_map, set_model)
  - [x] channel_id = wav_id * 256 + pitch + 128 (Java 互換)
  - [x] BgNote struct + bg_notes を bms-model に追加 (ch 0x01 パース + bmson BGM)
  - [x] Note に micro_starttime, micro_duration 追加 (bmson 音切り対応)
- [x] **8-4. ラウドネス解析**
  - [x] `loudness.rs` — LoudnessAnalyzer (ebur128 LUFS + SHA256 キャッシュ)
- [x] **8-5. BGM レンダラー**
  - [x] `renderer.rs` — BmsRenderer (オフライン f32 バッファミックス, -6dB headroom)
- [x] **8-6. テスト** — 52 テスト通過 (PCM 12 + MS-ADPCM 8 + WAV 6 + FLAC 3 + decode 4 + driver 3 + key_sound 3 + renderer 5 + loudness 5 + その他 3), clippy clean, fmt applied

### Phase 9: Skin System (`bms-skin`) — LARGEST

参照: 52 Java ファイル (`JSONSkinLoader`, `LR2SkinCSVLoader`, `LuaSkinLoader`, SkinObject 各種, `SkinProperty`, `IntegerPropertyFactory`, `BooleanPropertyFactory`)

- [x] **9-1. 基盤型定義**
  - [x] `property_id.rs` — 全プロパティ ID 定数 (Timer/Integer/Boolean/Float/String/Event newtypes + 2400+ 定数)
  - [x] `property_mapper.rs` — SkinPropertyMapper (player/key → timer/value ID 計算)
  - [x] `skin_object.rs` — SkinObjectBase (Destination, Rect, Color, SkinOffset, アニメーション補間, 描画条件)
  - [x] `stretch_type.rs` — StretchType enum
  - [x] `image_handle.rs` — ImageHandle, ImageRegion, ImageLoader trait, StubImageLoader
- [x] **9-2. SkinObject 具象型**
  - [x] Image, Number, Text, Slider, Graph, Gauge, Judge
  - [x] BpmGraph, HitErrorVisualizer, NoteDistributionGraph, TimingDistributionGraph, TimingVisualizer
  - [x] SkinObjectType enum dispatch (11 variants)
- [x] **9-3. SkinSource 階層**
  - [x] `skin_source.rs` — SkinImageSource (Reference/Frames), MovieSource
- [x] **9-4. SkinHeader**
  - [x] `skin_header.rs` — SkinHeader, CustomOption, CustomFile, CustomOffset, SkinFormat
- [x] **9-5. Skin コンテナ**
  - [x] `skin.rs` — Skin struct (全 SkinObject 保持, resolution scaling)
  - [x] `custom_event.rs` — CustomEventDef, CustomTimerDef
- [x] **9-6. JSON スキンローダー**
  - [x] `json_skin.rs` — JsonSkinData デシリアライズ構造体 (serde)
  - [x] `json_loader.rs` — JSON → Skin 変換 (SRC/DST, option/offset 解決, property mapping)
- [x] **9-7. LR2 CSV スキンローダー**
  - [x] `lr2_csv_loader.rs` — CSV パース + コマンドディスパッチ (SRC_IMAGE/NUMBER/TEXT/SLIDER/BARGRAPH/BUTTON, DST, #IF/#ELSE)
  - [x] `lr2_header_loader.rs` — ヘッダー解析 (INFORMATION, CUSTOMOPTION, CUSTOMFILE, CUSTOMOFFSET, RESOLUTION)
  - [x] `decode_ms932()` — MS932/Shift_JIS デコーディング (encoding_rs)
- [x] **9-8. Lua スキンローダー**
  - [x] `lua_loader.rs` — Lua 実行 → JSON 変換 → json_loader 委譲 (mlua)
  - [x] Lua 環境セットアップ (package.path, skin_config, skin_property)
  - [x] Lua table → serde_json::Value 再帰変換 (array/object 自動判定)
- [x] **9-9. テスト** — 193 テスト通過, clippy clean, fmt applied
  - [x] GM テスト: 各ローダーでスキン読込 → SkinSnapshot 構造比較 (JSON 4 + Lua 4 + LR2 CSV 3 = 11 テスト通過, 3 ignored)

### Phase 10: Rendering (`bms-render`)

参照: `SpriteBatchHelper.java`, `ShaderManager.java`, `MessageRenderer.java`

- [x] **10-1. Bevy セットアップ** (window, camera, render pipeline)
  - [x] `plugin.rs` — BmsRenderPlugin (Camera2d, skin_render_system)
  - [x] `coord.rs` — 座標変換 (skin 左上原点 → Bevy 中央原点, SkinRect/ScreenSize/RotationParams 構造体)
  - [x] 10 テスト通過
- [x] **10-2. スプライトバッチ** (Bevy Sprite)
  - [x] `texture_map.rs` — ImageHandle → Handle\<Image\> + dimensions マッピング (6 テスト)
  - [x] `image_loader_bevy.rs` — BevyImageLoader (ImageLoader trait 実装, image crate → Bevy Image 変換)
  - [x] `skin_renderer.rs` — SkinRenderState (Resource), setup_skin(), skin_render_system() (10 テスト)
  - [x] `state_provider.rs` — SkinStateProvider trait + StaticStateProvider (15 テスト)
  - [x] `draw/` モジュール — 7 サブモジュール (image, number, slider, graph, text, gauge, visualizer)
    - [x] image: SkinImage 描画 (6 テスト)
    - [x] number: 桁分解 + alignment/padding (10 テスト)
    - [x] slider: 4 方向オフセット (4 テスト)
    - [x] graph: Right/Up クリップ (4 テスト)
    - [x] text: alignment 計算 (3 テスト)
    - [x] gauge: ノード描画 + red zone threshold (5 テスト)
    - [x] visualizer: スタブ (Phase 11 依存)
- [x] **10-3. ブレンドモード** (Alpha, Additive, Invert)
  - [x] `blend.rs` — BlendState ヘルパー (alpha/additive/invert, 5 テスト)
- [x] **10-4. フォントレンダリング** (TTF + ビットマップ)
  - [x] `FontType` enum + BMFont .fnt パーサー (`bmfont.rs`)
  - [x] JSON ローダーフォント解決 (`json_loader.rs`)
  - [x] `FontMap` リソース (`font_map.rs`)
  - [x] BMFont テキストレイアウト (`draw/bmfont_text.rs`)
  - [x] TTF + BMFont テキストエンティティ生成・描画 (`skin_renderer.rs`)
  - [x] distance_field WGSL シェーダ + Material2d 定義
  - [x] BMFont 子グリフスプライト管理 (`CachedBmFontText` キャッシュ + `despawn_descendants`)
  - [x] TTF / BMFont シャドウ描画 (RGB/2 パターン, `TtfShadowMarker` エンティティ)
  - [x] Distance field レンダラー統合 (`Mesh2d` + `MeshMaterial2d<DistanceFieldMaterial>`)
  - [x] `Material2dPlugin<DistanceFieldMaterial>` 登録 + `embedded_asset!` シェーダ埋め込み
  - [x] DF ユニフォーム計算ヘルパー (`compute_outline_distance`, `compute_shadow_offset`, `compute_shadow_smoothing`)
- [x] **10-5. テスト**: Screenshot + SSIM 比較 (image-compare crate)
  - [x] Bevy headless rendering ハーネス (WinitPlugin 無効化, RenderTarget::Image オフスクリーン)
  - [x] TestSkinBuilder (プログラム的テストスキン構築, 外部ファイル不要)
  - [x] SSIM 比較ユーティリティ (UPDATE_SCREENSHOTS=1 fixture 生成/更新, diff 画像保存)
  - [x] 10 テストケース通過 (blank, single_image, image_alpha, z_order, animation_midpoint, draw_condition_false, timer_inactive, four_corners, slider, graph)
  - [x] harness=false カスタムテストランナー (--ignored フラグで GPU テスト実行)
  - [x] justfile ターゲット (screenshot-test, screenshot-update)
- [x] **10-6. テスト** — 108 テスト通過, clippy clean, fmt applied

### Phase 11: State Machine (`brs`)

参照: `MainController.java`, `MainState.java`, `TimerManager.java`, 各 State クラス

- [x] **11-1. MainState enum** (Select, Decide, Play, Result, CourseResult, KeyConfig, SkinConfig)
  - [x] `AppStateType` enum + `StateRegistry` ステートマシン (5 テスト)
  - [x] `GameStateHandler` trait (create/prepare/render/input/shutdown/dispose)
  - [x] `StateContext` — タイマー・リソース・設定・遷移要求のコンテキスト
- [x] **11-2. TimerManager**
  - [x] Java TimerManager.java 忠実移植 (μs 精度, TIMER_OFF = i64::MIN)
  - [x] reset/update/now_time/is_timer_on/set_timer_on/switch_timer/frozen (12 テスト)
- [x] **11-3. MusicSelect state** (スタブ: BMS ロード済みなら即 Decide 遷移)
- [x] **11-4. MusicDecide state**
  - [x] タイマーベース scene/fadeout/input フロー (8 テスト)
  - [x] confirm/cancel 入力ハンドリング
- [x] **11-5. Play state** (スタブ: 即 Result 遷移)
- [x] **11-6. Result state** (スタブ: 即 MusicSelect 遷移)
- [x] **11-7. CourseResult state** (スタブ: 即 MusicSelect 遷移)
- [x] **11-8. KeyConfiguration / SkinConfiguration states** (スタブ: 即 MusicSelect 遷移)
- [x] **11-A. PlayerResource** — ステート間共有データコンテナ
- [x] **11-B. SharedGameState + GameStateProvider** — SkinStateProvider 実装 (5 テスト)
  - [x] `sync_timer_state()` — TimerManager → SharedGameState 同期
- [x] **11-C. Bevy App 統合** — CLI `--bms`, システムチェーン, リソースラッパー
- [x] **11-D. JudgeManager** — 判定処理エンジン (Java JudgeManager.java 1,063行の忠実移植)
  - [x] LaneState: per-lane ステートマシン (cursor, processing, passing, release tracking)
  - [x] MultiBadCollector: PMS 用複数ノート同時 BAD 判定
  - [x] 5-phase update: pass → HCN gauge → key input → release margin → miss
  - [x] JudgeConfig / JudgeEvent: 初期化パラメータ / 出力イベント型
  - [x] 通常ノート / LN / CN / HCN / BSS / Mine / Autoplay 全対応
  - [x] 31 テスト通過 (171 テスト全通過), clippy clean, fmt applied
  - [x] now_judge / now_combo: レーン→プレイヤーインデックス変換 + スキン表示用更新
  - [x] LN release margin: worst-of-three の結果を ln_end_judge に正しく保存
  - [x] BSS `sckey` 追跡: LaneProperty による物理キー→レーン変換 + sckey 追跡 (6箇所), per-physical-key API 移行完了
- [x] **11-AB テスト** — 30 テスト通過, clippy clean, fmt applied
- [x] **11-9. E2E テスト**: パース → Judge → Gauge → ScoreData 一気通貫
  - [x] 5グループ 20テスト通過 (Autoplay 8 + Manual 5 + Gauge 4 + LN 2 + Cross-mode 1)
  - [x] LN pair_index 対応: build_judge_notes() で start/end 分割、autoplay/miss 全 LN タイプ対応

### Phase 12: Internet Ranking (`bms-ir`)

参照: `IRConnection.java`, `LR2IRConnection.java`, `IRConnectionManager.java`, `IRScoreData.java`

- [x] **12-1. IR データ型定義**
  - [x] `response.rs` — IRResponse\<T\> (success/failure コンストラクタ)
  - [x] `score_data.rs` — IRScoreData + ScoreData 相互変換 (Java passnotes バグ忠実再現)
  - [x] `chart_data.rs` — IRChartData + SongData → 変換 (feature flags → boolean)
  - [x] `player_data.rs` — IRPlayerData (id, name, rank)
  - [x] `account.rs` — IRAccount (id, password, name)
  - [x] `course_data.rs` — CourseDataConstraint enum (14 variants) + IRCourseData + IRTrophyData
  - [x] `table_data.rs` — IRTableData + IRTableFolder
  - [x] `leaderboard.rs` — LeaderboardEntry + IRType enum
- [x] **12-2. LR2IR 互換接続**
  - [x] `connection.rs` — IRConnection async trait (11 メソッド, デフォルト実装)
  - [x] `lr2ir.rs` — LR2IRConnection (POST getrankingxml.cgi, GET getghost.cgi, Shift_JIS デコード, XML パース, LR2 clear 変換, キャッシュ)
  - [x] `connection_manager.rs` — IRConnectionManager + IRConnectionType enum dispatch (async trait dyn 互換)
- [x] **12-3. ランキングデータ / キャッシュ**
  - [x] `ranking_data.rs` — RankingData (updateScore: exscore 降順ソート, 同率順位, irrank/localrank/prevrank, lamps 集計)
  - [x] `ranking_cache.rs` — RankingDataCache (4 スロット LN mode 分離, course hash SHA-256)
- [x] **12-4. テスト** — 55 テスト通過, clippy clean, fmt applied
  - [x] データ型テスト: serde ラウンドトリップ, ScoreData ↔ IRScoreData 変換, exscore 計算, CourseDataConstraint 変換
  - [x] XML デシリアライズテスト: 空/単一/複数スコア, LR2 clear 変換 (7 ケース)
  - [x] RankingData テスト: update_score の順位計算, 同率順位, irrank/localrank, lamps 集計, prevrank (10 ケース)
  - [x] RankingDataCache テスト: put/get (LN mode 分離), course hash 計算, cache miss (7 ケース)

### Phase 13: Peripherals

- [x] **13-1. Discord RPC** (`bms-external`)
  - [x] `discord/ipc.rs` — IpcConnection trait + Unix/Windows 実装 (UnixStream / named pipe)
  - [x] `discord/rich_presence.rs` — RichPresenceClient (ハンドシェイク, SET_ACTIVITY, uuid nonce)
  - [x] `discord/listener.rs` — DiscordListener (AppStateType → Rich Presence 更新)
- [x] **13-2. OBS 連携** (`bms-external`) — tokio-tungstenite
  - [x] `obs/protocol.rs` — OBS WS v5 メッセージ型 (Hello/Identify/Request/Event) + auth 計算
  - [x] `obs/client.rs` — ObsWsClient (mpsc コマンドチャンネル + バックグラウンド WebSocket task, 自動再接続)
  - [x] `obs/listener.rs` — ObsListener (Config obs_scenes/obs_actions マッピング)
- [x] **13-3. ストリーミング** (`bms-stream`)
  - [x] `command.rs` — StreamCommand trait + StreamRequestCommand (!!req パース, bounded VecDeque)
  - [x] `controller.rs` — StreamController (Windows named pipe + non-Windows stub, shutdown via watch)
- [x] **13-4. 楽曲ダウンローダー** (`bms-download`)
  - [x] `task.rs` — DownloadTask + DownloadTaskStatus enum (6 states, auto time_finished)
  - [x] `source/konmai.rs` — KonmaiDownloadSource (API クエリ → JSON → song_url)
  - [x] `source/wriggle.rs` — WriggleDownloadSource (URL パターン置換)
  - [x] `processor.rs` — HttpDownloadProcessor (tokio::Semaphore 同時5DL, streaming progress)
  - [x] `extract.rs` — tar.gz 展開 (flate2 + tar)
- [x] **13-5. 設定 GUI** (`bms-launcher`) — スケルトン
  - [x] `view.rs` — LauncherView trait (name/has_changes/apply)
  - [x] `lib.rs` — LauncherApp struct (Config 保持)
- [x] **13-6. ModMenu** — スケルトン
  - [x] `mod_menu.rs` — ModMenuPlugin struct (enabled toggle)
- [x] **13-7. その他** (BMSSearch, Screenshot, Webhook, ScoreImporter)
  - [x] `screenshot/mod.rs` — ScreenshotExporter trait + clear_type_name/rank_name ヘルパー
  - [x] `screenshot/file_exporter.rs` — FileExporter (PNG 保存, YYYYMMDD_HHmmss 命名)
  - [x] `webhook/mod.rs` — WebhookHandler (multipart/form-data POST)
  - [x] `webhook/payload.rs` — Discord Embed ペイロード構築 (色付き clear type)
  - [x] `bms_search.rs` — BmsSearchAccessor (api.bmssearch.net/v1 クエリ)
  - [x] `score_importer.rs` — ScoreDataImporter (LR2 SQLite → bms-database, clear 値マッピング)
- [x] **13-8. テスト・品質** — 1,114 テスト通過 (全 workspace), clippy clean, fmt applied
  - [x] bms-external: 45 テスト, bms-stream: 18 テスト, bms-download: 36 テスト, bms-launcher: 2 テスト

### Phase 14: Integration & GUI Testing

- [x] **14-1. Screenshot Testing パイプライン**
  - [x] Java reference screenshots 生成 (`just golden-master-screenshot-gen`)
  - [x] Rust test screenshots 生成 (`just screenshot-update`)
  - [x] SSIM 比較 (`just golden-master-screenshot-compare`)
    - [x] `ScreenshotTestCase` struct でテスト別閾値設定可能
    - [x] **SSIM 閾値方針**: Java-Rust 間は 0.85 (LibGDX/Bevy のレンダリングエンジン差異のため 0.95 は非現実的), Rust 内部回帰テストは 0.99
    - [x] 7 テストケース (ecfn_select, decide, play7_active/fullcombo/danger, result_clear/fail)
- [x] **14-2. E2E テスト**
  - [x] ReplayData E2E: Java `ReplayE2EExporter` → Rust `compare_replay_e2e.rs` (12 テストケース通過)
  - [x] 網羅 E2E テスト: `exhaustive_e2e.rs` — 4 モード × 6 ゲージ × 3 入力 = 72 テスト通過
  - [x] E2E ヘルパー抽出: `e2e_helpers.rs` (既存 `e2e_judge.rs` 20 テストもリファクタリング済み)
  - [x] PMS テストファイル: `test-bms/9key_pms.pms` 作成 (PopN9K モード検出用)
- [x] **14-3. パフォーマンステスト**
  - [x] criterion ベンチマーク: bms-model (parse + build_judge_notes), bms-rule (JudgeManager + Gauge), bms-pattern (lane/note shuffle + PlayableRandom)
  - [x] `just bench` ターゲット
