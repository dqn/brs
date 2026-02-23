# Porting TODO — Remaining Work

Phases 1–33 complete. **1816 tests, 22 ignored (9 explicit #[ignore] + 13 fixture-absent runtime skips).** 27 crates, 125k lines. See AGENTS.md.

**残 Phase 34–43 計画 (Java↔Rust 徹底比較に基づく)**

## Phase 26: スキンパイプライン完成 → 22 ignored テスト解除

Resolves: `beatoraja-skin/stubs.rs` (287 lines), `beatoraja-launcher/stubs.rs` (partial)

- [x] **26a:** `PixmapResourcePool` (wgpu テクスチャ ロード/キャッシュ/解放) + `SkinSourceImage`
- [x] **26b:** `SkinLoader.load_skin()` (SkinData→Skin 変換 + テクスチャバインド)
- [x] **26c:** Lua→JSON 型変換 + 13 テスト `#[ignore]` 解除 (残り 9 ignored: main_state API / SkinNote/SkinBar)
- [x] **26d:** バナー/ステージファイル画像 + `ReplayData::exists()` (依存: 26a)

## Phase 27: 楽曲 DB 拡張 + 検索

- [x] **27a:** `rayon::par_iter()` による BMS 並列走査
- [x] **27b:** SQLite FTS5 全文検索 (`get_song_datas_by_text()`)
- [x] **27c:** `SongInformationAccessor` trait + SQLite CRUD

## Phase 28: プラットフォーム固有 + 入力

Resolves: `beatoraja-input/stubs.rs` (44 lines)

- [x] **28a:** gilrs コントローラ + hotplug
- [x] **28b:** KeyCommand (F キー, Alt+Enter, ESC)
- [x] **28c:** Windows named pipe (`#[cfg(windows)]`)
- [x] **28d:** winit モニター列挙
- [x] **28e:** Discord Rich Presence (discord-rpc crate + DiscordListener 完全実装済み、MainController 接続配線 → **Phase 29a**)

## Phase 29: リファクタリング + スタブ解消

Resolves: rendering stubs (result/decide/select/modmenu ~972 lines), `beatoraja-types/stubs.rs` (549 lines), `beatoraja-external/stubs.rs` (partial)

- [x] **29a-1:** MainStateListener trait 統合 + Discord/OBS 接続配線 (StateAccessAdapter パターン)
- **29a-2:** rendering stubs 削減 (result/decide/select/modmenu ~972 lines) — → **Phase 33**
- **29a-3:** Property traits 統合 (beatoraja-skin vs beatoraja-external) — → **Phase 33**
- [x] **29b:** PlayerResource trait 分析完了 — 32メソッド中31が使用中、最小化不要
- [x] **29c:** dhat ヒーププロファイリング (`--features dhat-heap` で有効化、`dhat-heap.json` 出力)
- [x] **29d:** 入力ポーリング分析完了 — 同期で十分、スキップ

## Phase 30: 非レンダリングスタブ整理

レンダリングパイプライン非依存のスタブを独自ファイルへ移動・実装。

- [x] **30a:** `beatoraja-types` enum 移動 — `JudgeAlgorithm`, `BMSPlayerRule`, `BarSorter` + modifier enums を stubs.rs から専用ファイルへ
- [x] **30b:** `beatoraja-types` DTO 実装 — `KeyInputLog`, `PatternModifyLog` を stubs.rs から移動
- [x] **30c:** `beatoraja-input` `SkinWidgetManager::get_focus()` 実装 — skin_widget_focus を beatoraja-types に抽出
- [x] **30d:** `beatoraja-select` `DownloadTask` 系型 — stubs 削除、md-processor の実装を re-export

## Phase 31: Lua main_state API 拡張 → 5 ignored テスト解除

`compare_render_snapshot.rs` で `#[ignore]` されている 5 テストのブロッカー解消。

- [x] **31a:** `main_state.number(key)` Lua API — DelegateIntegerProperty でスキンLuaから数値参照
- [x] **31b:** `main_state.text(key)` Lua API — DelegateStringProperty でスキンLuaからテキスト参照

Resolves: `render_snapshot_ecfn_result_clear`, `_result_fail`, `_play14_active`, `_course_result`, `timeline_result_has_stable_visible_set`

## Phase 32: SkinNote/SkinBar/SkinJudge 型実装 → 4 ignored テスト解除

`compare_render_snapshot.rs` の残り 4 テストのブロッカー解消。

- [x] **32a:** `SkinNote` — SkinObject::Note variant + skin_data_converter 変換
- [x] **32b:** `SkinBar` — SkinObject::Bar variant + skin_data_converter 変換
- [x] **32c:** `SkinJudge` — SkinObject::Judge variant + skin_data_converter 変換

Resolves: `rust_only_snapshot_ecfn_play7_mid_song`, `_select_with_song`, `skin_state_objects_play_has_note_judge`, `skin_state_objects_select_has_bar`

## Phase 33: フルレンダリングパイプライン完成

Phase 29a-2/29a-3 の blocking 解消。レンダリングスタブの分析・置換。

- [x] **33a:** `SkinText::draw()`, `SkinNumber::draw()`, `SkinImage::draw()` — 検証完了。全て beatoraja-skin に完全実装済み (Phase 26/32 で完了済み)
- [x] **33b:** `SkinObjectRenderer::draw()` — 検証完了。beatoraja-skin/skin_object.rs に完全実装済み。select/modmenu のスタブは API 不一致のため直接置換不可 (異なるフィールド構造・mutability)
- [x] **33c:** `BooleanPropertyFactory` — `StubBooleanProperty` を `DelegateBooleanProperty` に置換。`MainState::boolean_value(id)` trait メソッド追加 (Integer/String と同パターン)
- [x] **33d:** stubs 整理 — result: `SkinObjectData` スタブ削除→ beatoraja-skin re-export。modmenu: 未使用 import (`Skin`, `MusicSelector`) 削除。select/modmenu/decide: API 不一致スタブは保持 (将来の SkinBar/SkinWidget リライト時に解消)

**残留スタブ分析:**
- `beatoraja-result/stubs.rs` (385 lines): MainController/PlayerResource ライフサイクルスタブ、IR re-export — 必須。SkinObjectData は削除済み
- `beatoraja-select/stubs.rs` (278 lines): SkinText/SkinNumber/SkinImage/SkinObject/SkinObjectRenderer — API 不一致 (stub は `&self`/simple fields、real は `&mut self`/SkinObjectData)。SkinBar リライト時に解消
- `beatoraja-modmenu/stubs.rs` (203 lines): Skin/SkinObject/SkinObjectDestination — SkinWidgetManager が Clone+Debug derive に依存。リライト時に解消
- `beatoraja-decide/stubs.rs` (108 lines): MainControllerRef/SkinStub — ライフサイクルスタブ、必須

---

## Phase 34: BMSPlayer 初期化ロジック移植

最大のギャップ。Java版 BMSPlayer コンストラクタの ~390行の初期化処理を移植。ゲームプレイの実動作に必須。

- [ ] **34a:** PatternModifier 生成・適用 — PlayerConfig から Random/Mirror/Scatter 等のオプション読み取り → PatternModifier チェーン構築 → BMSModel に適用
- [ ] **34b:** 乱数シード管理 — リプレイ再現用シード保存/復元、JavaRandom LCG シード初期化パス
- [ ] **34c:** リプレイデータ復元 — ReplayData からパターン/ゲージ/設定を復元、リプレイモード時のオプション上書き
- [ ] **34d:** アシストレベル計算 — BPMガイド、カスタムジャッジ、定速等のアシストフラグ判定
- [ ] **34e:** DP→SP オプション変換 — ダブルプレイ時の 2P 側オプション処理
- [ ] **34f:** 周波数トレーナー統合 — FreqTrainer との速度変更連携

## Phase 35: スコアデータ統計完成

BMSPlayer.createScoreData() の統計計算部分。リザルト画面/DB保存に影響。

- [ ] **35a:** タイミング分散計算 — avgduration, average, stddev (ジャッジタイミングの統計)
- [ ] **35b:** デバイス種別トラッキング — キーボード/コントローラ/MIDI の入力デバイス判定・記録
- [ ] **35c:** スキル値計算 — スコアに基づくスキルレーティング算出

## Phase 36: 入力/リプレイ統合

KeyInputProcessor の完全実装。リプレイ再生・オートプレイに必須。

- [ ] **36a:** `startJudge()` 完全実装 — KeyInputLog[] リプレイキーログからの入力再生
- [ ] **36b:** オートプレイ押下シミュレーション — auto_presstime 配列によるノート自動押下スケジューリング
- [ ] **36c:** クイックリトライ検出 — START+SELECT 同時押しによる即リトライ
- [ ] **36d:** JudgeManager.init() 完全実装 — カスタムジャッジレート、コース制約 (NO_GREAT/NO_GOOD)、LN タイプ設定

## Phase 37: イベントディスパッチ実装

beatoraja-skin EventFactory の StubEvent を実際のイベントロジックに置換。

- [ ] **37a:** MusicSelector 操作イベント — ソート変更、ゲージ変更、オプション切替、リプレイ操作
- [ ] **37b:** 設定変更イベント — プレイコンフィグ変更 (ガイドSE, BGA, レーンカバー等)
- [ ] **37c:** IR 操作イベント — IR 接続/ランキング取得/スコア送信のトリガー
- [ ] **37d:** その他イベント — スクリーンショット、Twitter (スキップ)、キーコンフィグリセット等

## Phase 38: オーディオプロセッサ統合

BMSPlayer/MainController とオーディオシステムの接続。

- [ ] **38a:** グローバルピッチ制御 — `AudioProcessor.setGlobalPitch()` を MainController 経由で BMSPlayer に接続
- [ ] **38b:** ガイド SE 設定 — Config.isGuideSE() に基づくガイドサウンドパスの解決・再生
- [ ] **38c:** ラウドネス分析統合 — BMSLoudnessAnalyzer の結果をレンダリング時のボリューム正規化に適用
- [ ] **38d:** 状態遷移 BGM — Select/Decide/Result 間の BGM 再生・フェード制御

## Phase 39: BGA 動画処理

beatoraja-play/bga の動画プロセッサ実装。

- [ ] **39a:** FFmpegProcessor 実装 — 外部 FFmpeg プロセスによる動画デコード (フレーム抽出 → テクスチャ)
- [ ] **39b:** MovieProcessor/GdxVideoProcessor — 動画再生パイプライン統合
- [ ] **39c:** BGA 表示統合 — BMSPlayer の BGA レイヤーとスキンレンダリングの接続

## Phase 40: SkinWidget リライト + レンダリングスタブ解消

API 不整合スタブ (~481行) の解消。select/modmenu のレンダリングパイプライン完成。

- [ ] **40a:** SkinWidget API 設計 — `&self` + simple fields → `&mut self` + SkinObjectData の borrow 問題を解決するアーキテクチャ設計
- [ ] **40b:** beatoraja-select レンダリングスタブ置換 (278行) — SkinText/SkinNumber/SkinImage/SkinObjectRenderer を実 API に接続
- [ ] **40c:** beatoraja-modmenu レンダリングスタブ置換 (203行) — Skin/SkinObject/SkinObjectDestination + MusicSelector 結合
- [ ] **40d:** ImGuiRenderer egui 統合 — modmenu の egui レンダリングパイプライン接続

## Phase 41: ライフサイクルスタブ統合

クロスクレート API 境界のライフサイクルスタブ (~939行) を実オブジェクトに置換。

- [ ] **41a:** PlayerResource クロスクレート統合 — result/decide/external の PlayerResource スタブを trait ベースの実オブジェクトに置換
- [ ] **41b:** MainController クロスクレート統合 — result/decide の MainControllerRef を実 MainController 参照に置換
- [ ] **41c:** AudioProcessor 統合 — result/decide の AudioProcessorStub を実オーディオドライバに接続

## Phase 42: Launcher egui 完全移行

JavaFX 設定 UI の egui 完全移行。設定ビューの動的動作実装。

- [ ] **42a:** 設定ビュー initialize/update/commit — PlayConfigurationView 等 14 ビューの初期化・更新・保存ロジック
- [ ] **42b:** エディタビュー — CourseEditorView, FolderEditorView, TableEditorView の実動作
- [ ] **42c:** DisplayMode/MonitorInfo 統合 — winit からのモニター情報を Launcher UI に反映

## Phase 43: BMSPlayer.create() + Skin ロード統合

BMSPlayer のスキンロード/初期化完成。

- [ ] **43a:** `BMSPlayer.create()` 完成 — loadSkin(), ガイドSEパス解決, 入力プロセッサモード設定
- [ ] **43b:** プラクティスモード統合 — PracticeConfiguration のプロパティを BMSModel に適用 (周波数/LNモード/乱数シード/時間範囲)

---

## 軽微な未移植項目

| 項目 | 影響 | 備考 |
|------|------|------|
| `BMSModel.compareTo()` | 低 | 必要時に Ord 実装可 |
| `BMSModel.getEventLane()` / `getLanes()` | 低 | オンデマンド生成 (呼び出し側で対応可) |
| `BMSModelUtils.getAverageNotesPerTime()` | 低 | 使用頻度低 |
| OBS reconnect lifecycle | 低 | server_uri/password の inner 保持が必要 |

## Permanent Stubs

- **Twitter4j** (`beatoraja-external`): ~446 lines, `bail!()` — API 廃止済みのため意図的に未実装
- **ShortDirectPCM** (`beatoraja-audio`): Java 固有の DirectBuffer — Rust では不要
