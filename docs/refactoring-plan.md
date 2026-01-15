# BMS Player Refactoring Plan

## Summary

Phase 1 から順に実装する。低リスク・高価値の改善から着手。

## Phase 1: High Priority

### 1.1 Replace `unwrap()` with `total_cmp()`
- **Location**: `src/bms/loader.rs`
- **Issue**: `partial_cmp().unwrap()` は NaN で panic する
- **Fix**: `total_cmp()` を使用

### 1.2 Extract GameState to game module
- **Location**: `src/main.rs` → `src/game/game_state.rs`
- **Issue**: main.rs に GameState が埋め込まれていてテスト不可
- **Fix**: game モジュールに移動

### 1.3 Pre-index notes by lane
- **Location**: `src/bms/chart.rs`, `src/main.rs`
- **Issue**: 入力処理で毎フレーム全ノーツを走査 O(lanes * notes)
- **Fix**: レーン別インデックスを構築

## Phase 2: Medium Priority

### 2.1 Dependency injection for InputHandler
- テスト可能にするため trait ベースの抽象化

### 2.2 Consolidate note drawing logic
- `draw_notes_simple()` と `draw_notes_with_state()` の共通部分を抽出

### 2.3 Add BmsLoader integration tests
- テスト用 BMS ファイルを追加

## Phase 3: Low Priority

### 3.1 Extract rendering configuration
- ハードコードされた定数を設定可能に

### 3.2 Domain-specific error types
- `thiserror` を使った専用エラー型

### 3.3 Builder patterns
- JudgeConfig などにビルダーパターンを追加
