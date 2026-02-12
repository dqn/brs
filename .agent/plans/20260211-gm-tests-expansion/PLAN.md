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

## 次ステップ（直近）

1. `command_count` 差分の発生源を object 種別ごとに縮小する  
`ecfn_select` / `ecfn_play7_*` / `ecfn_result_*` について、`type_delta` を指標に `SkinBGA` / `Gauge` / `Number` / `Text` / `Image` の列挙基準を順次一致させる。
2. 差分 0 ケースから `#[ignore]` を段階解除する  
ケース単位で `known_diff_budget` を下げ、`ignored` から通常実行へ移行する（`ecfn_decide` は解除済み）。
3. strict parity へ向けた残課題を固定化する  
解消不能な仕様差がある場合は comparator 側の明示除外ルールとして文書化し、diff budget を暫定値として管理する。

## 保留事項

1. `ecfn_select` / `ecfn_play7_*` / `ecfn_result_*` の `command_count` 差分（6ケース）は未解消
   - 状態: `ecfn_decide` は strict 化済みだが、残り 6 ケースは `#[ignore]` 継続
   - 影響: RenderSnapshot 全ケース strict parity には未到達
   - 次アクション: `type_delta` 上位（`Image` / `Number` / `Gauge` / `SkinBGA` / `SkinNote`）から、Java exporter と Rust capture の列挙基準を順に一致させる

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
