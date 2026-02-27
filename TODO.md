# Porting TODO — Remaining Work

Phases 1–43 complete. **2346 tests, 0 ignored.** 27 crates, 127k lines. See AGENTS.md.

---

## Phase 40: SkinWidget リライト + レンダリングスタブ解消

API 不整合スタブ (~481行) の解消。select/modmenu のレンダリングパイプライン完成。

- [x] **40a:** SkinWidget API 設計 — `&self` + simple fields → `&mut self` + SkinObjectData の borrow 問題を解決するアーキテクチャ設計
- [x] **40b:** beatoraja-select レンダリングスタブ置換 (278行) — SkinText/SkinNumber/SkinImage/SkinObjectRenderer を実 API に接続
- [x] **40c:** beatoraja-modmenu レンダリングスタブ置換 (203行) — Skin/SkinObject/SkinObjectDestination + MusicSelector 結合
- [x] **40d:** ImGuiRenderer egui 統合 — modmenu の egui レンダリングパイプライン接続
- [x] **40e:** MovieProcessor 動画再生パイプライン統合 — GdxVideoProcessor のスキン統合

## Phase 41: ライフサイクルスタブ統合

クロスクレート API 境界のライフサイクルスタブを実オブジェクトに置換。

- [x] **41c:** AudioProcessor 統合 — result/decide の AudioProcessorStub を実オーディオドライバに接続
- [x] **41d:** デバイス種別トラッキング — create_score_data() で MainController.get_input_processor().get_device_type() を接続
- [x] **41e:** startJudge() 完全実装 — JudgeThread をスレッド化し KeyInputLog[] リプレイ入力再生を接続
- [x] **41f:** KeyInputProcessor.input() 実装 — auto_presstime + キービーム + スクラッチアニメーション
- [x] **41g:** ControlInputProcessor.input() 実装 — START+SELECT クイックリトライ + レーンカバー操作
- [x] **41h:** EventFactory 実イベント実装 — 108 StubEvent を MusicSelector/MainController 経由の実ロジックに置換
- [x] **41i:** オーディオプロセッサ統合 — グローバルピッチ、ガイドSE、ラウドネス、状態遷移BGM
- [x] **41j:** BGA 表示統合 — BMSPlayer の BGA レイヤーとスキンレンダリングの接続

## Phase 42: Launcher egui 完全移行

JavaFX 設定 UI の egui 完全移行。設定ビューの動的動作実装。

- [x] **42a:** 設定ビュー initialize/update/commit — PlayConfigurationView 等 14 ビューの初期化・更新・保存ロジック
- [x] **42b:** エディタビュー — CourseEditorView, FolderEditorView, TableEditorView の実動作
- [x] **42c:** DisplayMode/MonitorInfo 統合 — winit からのモニター情報を Launcher UI に反映

## Phase 43: BMSPlayer.create() + Skin ロード統合

BMSPlayer のスキンロード/初期化完成。

- [x] **43a:** `BMSPlayer.create()` 完成 — loadSkin(), ガイドSEパス解決, 入力プロセッサモード設定

---

## Phase 44: 統合テスト包括追加 ✅

`PlayerConfig::init()` 未呼び出しバグの教訓。wiring 統合テスト追加 + テスタビリティ改善。

### Phase 44a: テスタビリティ改善（コード変更） ✅

- [x] **44a-1:** `Config::read_from(dir)` / `Config::write_to(config, dir)` 追加
- [x] **44a-2:** `PlayerConfig::init()` の `create_dir` → `create_dir_all`
- [x] **44a-3:** `MainLoader::clear_illegal_songs()` / `clear_score_database_accessor()` を `pub fn` に
- [x] **44a-4:** `MainLoader::play()` → `anyhow::Result<MainController>`、`process::exit(1)` → `bail!()`
- [x] **44a-5:** `PlayConfigurationView::exit()` → `exit_requested` flag

### Phase 44b–d: 統合テスト (45 tests, 7 files) ✅

- [x] **44b-1:** `config_filesystem.rs` (8 tests)
- [x] **44b-2:** `player_config_lifecycle.rs` (10 tests, 1 ignored)
- [x] **44c-1:** `play_data_accessor_integration.rs` (6 tests)
- [x] **44c-2:** `main_loader_integration.rs` (8 tests)
- [x] **44d-1:** `song_db_init.rs` (5 tests)
- [x] **44d-2:** `launcher_wiring.rs` (5 tests)
- [x] **44d-3:** `cli_smoke.rs` (4 tests, 1 ignored)

---

## 軽微な未移植項目

| 項目 | 影響 | 備考 |
|------|------|------|
| `BMSModel.compareTo()` | 低 | 必要時に Ord 実装可。Java でも未使用 |
| `BMSModelUtils.getAverageNotesPerTime()` | 低 | Java でも未使用 (デッドコード) |
| OBS reconnect lifecycle | 低 | server_uri/password の inner 保持が必要 |
| Skill rating calculation | 低 | Java ソースに実装なし (移植元不在) |

## Permanent Stubs

- **Twitter4j** (`beatoraja-external`): ~446 lines, `bail!()` — API 廃止済みのため意図的に未実装
- **ShortDirectPCM** (`beatoraja-audio`): Java 固有の DirectBuffer — Rust では不要
