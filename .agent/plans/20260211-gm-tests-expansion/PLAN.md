# GM Tests 拡充計画

Code Name: `gm-tests-expansion`
Created: 2026-02-11
Status: In Progress

## 概要

`lr2oraja-rust/golden-master` の比較テストを、現行の回帰検知中心から「未実装フェーズを先回りできる検証基盤」へ拡張する。
重点は以下の 3 点。

- 比較対象の網羅不足（未比較フィールド・fixture 件数不足）の解消
- RenderSnapshot の strict parity 化（`#[ignore]` 依存の段階的撤廃）
- Phase 16-20 相当機能で再利用できる Java exporter + Rust comparator の雛形整備

## 背景

現状調査より、以下を確認した。

- `cargo test -p golden-master -- --nocapture` は全通過（RenderSnapshot 7件・SSIM 7件は意図的 ignore）
- `compare_render_snapshot` の regression guard は通過し、既知 diff budget は `ecfn_decide=28`、その他 6 ケースは `command_count` 差分 1 件
- Java 側 `GoldenMasterExporter` は `timelines` / `timeline_count` を出力しているが、Rust 側 `compare_model` はこの情報を比較していない
- `GoldenMasterExporter` の fixture 出力名は拡張子を落としており、同名ベース（例: `9key_pms.bms` / `9key_pms.pms`）で衝突余地がある
- `random_seeds.json` による deterministic 制御は parser/autoplay/database で既に実装済み

## 目標

- Parser / Autoplay / Database の fixture 行列を拡充し、仕様分岐の盲点を減らす
- RenderSnapshot の 7 ignore テストを段階的に unignore して strict parity へ近づける
- これから実装する Phase 16-20 機能のために、GM exporter / comparator を先に設計して実装速度を上げる
- すべての拡張を deterministic かつローカル再現可能なコマンドで運用する

## 全体方針

- 比較ルールと fixture 仕様を SSOT 化し、Rust/Java 双方の重複定義を避ける
- いきなり strict parity にせず、`regression guard -> strict` の二段階を採用する
- 追加する fixture は「仕様差分が起きる入力」を優先し、件数だけを増やさない
- `just` コマンドに生成・比較の入口を集約し、手順の属人化を防ぐ

## Phase List

| Phase | File | 要約 | Status |
|-------|------|------|--------|
| 1 | [phase-1-baseline-and-harness.md](./phase-1-baseline-and-harness.md) | ベースライン可視化とハーネス健全化（命名衝突・未比較項目の解消） | Completed |
| 2 | [phase-2-fixture-matrix-expansion.md](./phase-2-fixture-matrix-expansion.md) | Parser/Autoplay/Database/Judge 系 fixture 行列の拡張 | Completed |
| 3 | [phase-3-render-snapshot-parity.md](./phase-3-render-snapshot-parity.md) | RenderSnapshot parity 改善と ignore 撤廃 | In Progress |
| 4 | [phase-4-new-module-exporters.md](./phase-4-new-module-exporters.md) | Phase 16-20 向け新規 GM exporter/comparator 追加 | Not Started |

## 最新進捗（2026-02-12）

- Java screenshot/render-snapshot exporter に出力解像度反映を追加し、`ecfn_decide` の geometry 差分を解消（28 diff -> 4 diff）。
- `compare_render_snapshot` に `command_count` 差分の型別内訳（`type_delta` / `visible_type_delta`）を追加。
- regression guard の `known_diff_budget` を更新（`ecfn_decide=5`）し、`cargo test -p golden-master --test compare_render_snapshot -- --nocapture` を通過確認。
- RenderSnapshot capture に option 条件と special ID の整合処理を追加し、Java との object 列挙差分の切り分け精度を改善。
- Rust `render_snapshot` に Java `SkinText.prepare()` 相当の可視判定（空文字 hidden）を導入し、`tablefull(1003)` の GM mock 既定値 `"null"` を反映。
- `ecfn_decide` が 0 diff となり、`known_diff_budget` を `0` に更新。`render_snapshot_ecfn_decide` の `#[ignore]` を解除。
- `render_snapshot` の option 条件評価を skin type aware に拡張し、Java `Skin.prepare()` の static prune に近い判定を導入。
- `command_count` の絶対差を大幅に縮小（`ecfn_result_*: 208->109 vs java 111`、`ecfn_play7_*: 207->167 vs java 166`、`ecfn_select: 282->281 vs java 280`）。
- Rust `json_loader` に未実装だった JSON skin object builder（`songlist` / `note` / `judge` / `gauge` / `bga` / `hidden/lift` / `gaugegraph` / `judgegraph` / `float`）を追加し、`image` より前に解決するよう列挙順を調整。
- `render_snapshot` の object type 名を Java exporter のクラス名へ整合（`SkinBar` / `SkinNote` / `SkinJudge` / `SkinNoteDistributionGraph` / `SkinGaugeGraphObject`）。
- 上記反映後の `--ignored` 実測を更新:
  - `ecfn_result_*`: `java 111 / rust 110`（`type_delta: Image:-1`）
  - `ecfn_play7_*`: `java 166 / rust 172`（`type_delta: Image:+14, Text:-8`）
  - `ecfn_select`: `java 280 / rust 282`（`type_delta: Image:+2`）
- `render_snapshot_parity_regression_guard`（非 ignored）の通過は維持。
- `json_ecfn_select_snapshot` 回帰を解消（`STRING_SEARCHWORD` の除外を `json_loader` から `render_snapshot` 側へ移動し、Skin snapshot 比較と RenderSnapshot 比較を分離）。
- `cargo test -p golden-master -- --nocapture` と `cargo test -p bms-skin -- --nocapture` の全通過を再確認。
- `compare_render_snapshot` に `sequence_delta`（LCS ベース）を追加し、`command_count` 不一致時の `java_only/rust_only` コマンド位置を先頭 5 件ずつ出力するように改善。

## 未対応ステップ（2026-02-12 追記）

1. `ecfn_result_*` の `Image:-1` 差分の確定
   - 現状: `idx=30..129` 区間で Java 側にのみ hidden `Image` が 1 command 残る。
   - 補足: 試験的に `idx=30` を強制 include すると `command_count` は一致するが、visibility/geometry/detail 差分が 19 件露出したため未採用。
   - 次アクション: `obj.validate()` と `draw` 条件の適用順を Java 実装に合わせて再現し、隠れ command の生存条件を特定する。

2. `ecfn_play7_*` の `Image:+14` / `Text:-8` の解消
   - 現状: command 数は `java 166 / rust 172` のまま。
   - 次アクション: `type_delta` と `visible_type_delta` を object index 単位で突合し、`draw` 条件と source 解決の差を切り分ける。

3. `ecfn_select` の `Image:+2` 差分の解消
   - 現状: `java 280 / rust 282`。
   - 次アクション: select 固有 object（Graph/Bar 周辺と panel 系）の prune 条件を Java exporter と 1:1 で照合する。

4. `sequence_delta` で観測された先頭近傍の列挙ズレの確定
   - 現状:
     - `ecfn_play7_*` で `java_only pos=10..14`（hidden Number 連続）と `rust_only pos=11..14`（visible Image 連続）が発生
     - `ecfn_select` で `pos=1` の Image visible 判定が反転
   - 影響: `command_count` だけでなく object 列挙順・可視判定タイミングがずれており、strict parity で大量差分が再露出するリスクがある
   - 次アクション: `render_snapshots_debug/*__java.json` / `*__rust.json` の該当 `pos` を起点に、`draw` 条件評価順と source 選択条件を object 単位で突合する

5. `ecfn_result_*` の Number 可視偏り（`visible_type_delta: Number:+8`）の解消
   - 現状: `command_count` は `-1` だが、同時に Number の visible 偏りが残る
   - 影響: 隠れ Image 1件差の解消後に Number の visibility/detail 差分が前面化する可能性が高い
   - 次アクション: result ケースで Number の参照 ID ごとに Java `prepare()` の hidden 条件を抽出し、Rust capture 側の可視判定に必要最小限で反映する

## 次ステップ（直近）

1. Java `Skin.prepare()` と Rust capture 前処理の差を詰める  
`draw=function(...)` と Java `obj.validate()` 相当の除外条件を突合し、`command_count` 差分の主因を縮小する。
2. `screenshot_states` と Java mock state の既定値を一致させる  
可視数の乖離（`visible_type_delta` の `Image/Number` 偏在）を潰すため、timer/integer/float/boolean の既定値をケース別に同期する。
3. ケース別 `type_delta` をゼロへ寄せる  
`result: Image:-1`、`play: Image:+14, Text:-8`、`select: Image:+2` を object id 単位で解消する。
4. 差分 0 ケースから `#[ignore]` を段階解除する  
ケース単位で `known_diff_budget` を下げ、`ignored` から通常実行へ移行する（`ecfn_decide` は解除済み）。

## 保留事項

1. `ecfn_select` / `ecfn_play7_*` / `ecfn_result_*` の `command_count` 差分（6ケース）は未解消
   - 状態: `ecfn_decide` は strict 化済みだが、残り 6 ケースは `#[ignore]` 継続
   - 影響: RenderSnapshot 全ケース strict parity には未到達
   - 次アクション: `type_delta` 上位（`Image` / `Text`）から、Java exporter と Rust capture の列挙基準・前処理を順に一致させる

2. 残 `type_delta` のうち object type 未整合が存在
   - 状態:
     - `ecfn_result_*`: `Image:-1`
     - `ecfn_play7_*`: `Image:+14`, `Text:-8`
     - `ecfn_select`: `Image:+2`
   - 影響: object 列挙順/前処理の不一致が残り、strict 化の阻害要因となる
   - 次アクション: object type ごとに Java exporter 側の出力対象と Rust `capture_render_snapshot` 側の対象を 1:1 で突合する（builder 未実装起因は解消済み）

3. `option_conditions` の static 判定は暫定ルール（skin type aware ヒューリスティック）
   - 状態: Java `BooleanPropertyFactory` 全量移植ではなく、RenderSnapshot で効く static 条件のみを優先実装
   - 影響: 条件 ID の網羅漏れがあると、ケース追加時に `command_count` が再増加するリスクがある
   - 次アクション: `screenshot_states` で使う Boolean ID を固定リスト化し、Java 側 `isStatic` 分類との対応表を docs 化して回帰テスト化する

4. Java `draw/validate` 相当の前処理差
   - 状態: Rust 側は `command_count` 比較まで寄せているが、Java `Skin.prepare()` の `draw=function(...)` / `validate()` 由来の除外を十分再現できていない
   - 影響: `visible_type_delta` の `Image/Number` 偏在が残り、`command_count` ゼロ化後に geometry/detail の大量差分化リスクがある
   - 次アクション: `draw` 条件の静的評価可能分を capture 前に適用し、Java と同じ除外タイミングへ合わせる

## Research Summary

- 実行コマンド:
  - `cargo test -p golden-master -- --nocapture`
  - `cargo test -p golden-master --test compare_render_snapshot render_snapshot_parity_regression_guard -- --nocapture`
  - `cargo test -p golden-master --test compare_render_snapshot -- --nocapture --ignored`
- 差分状況:
  - `ecfn_decide`: 可視性・座標・解像度差を含む 28 diff
  - `ecfn_select` / `ecfn_play7_*` / `ecfn_result_*`: `command_count` の 1 diff
- 主要観測ファイル:
  - `CLAUDE.md`
  - `AGENTS.local.md`
  - `lr2oraja-rust/golden-master/tests/compare_render_snapshot.rs`
  - `lr2oraja-rust/golden-master/src/lib.rs`
  - `lr2oraja-java/golden-master/src/bms/model/golden/GoldenMasterExporter.java`
  - `lr2oraja-rust/justfile`

## リスクと緩和策

| リスク | 影響 | 緩和策 |
|------|--------|------------|
| fixture 追加で差分が大量発生し、原因切り分けが困難 | High | Phase 1 で diff 分類出力を先に整備し、比較失敗時の情報密度を上げる |
| Java exporter 変更で既存 fixture を壊す | Medium | 生成コマンドを分割維持し、変更対象 fixture だけ更新できるようにする |
| RenderSnapshot strict 化を急ぎすぎて開発速度を落とす | Medium | ケース単位で unignore し、budget 減少をマイルストーン化する |
| 仕様差（Java/Rust 実装差）が設計上正当な可能性 | Medium | 「一致必須」「許容差」「比較除外」を明文化して comparator に反映する |

## 並列実行性

- 並列可: Phase 2 と Phase 3（テスト対象領域が独立）
- 部分並列可: Phase 4 は exporter 設計を先に進め、Rust 実装完了待ちで comparator を接続
- 直列推奨: Phase 1 は全フェーズの前提なので最優先

## 完了判定

- [ ] Phase 1-4 のチェックリストが完了している
- [ ] `cargo test -p golden-master -- --nocapture` が安定通過
- [ ] RenderSnapshot の known diff budget が段階的に縮小され、最終的に strict parity に移行可能
- [ ] 新規 module（Phase 16-20 相当）で exporter/comparator の再利用パターンが成立している
