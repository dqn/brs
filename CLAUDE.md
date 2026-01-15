# BMS Player Project

Rust で実装する BMS (Be-Music Source) リズムゲームプレイヤー。

## Tech Stack

- bms-rs: BMS パーシング
- kira: オーディオ (クロックベースタイミング)
- macroquad: 2D グラフィックス
- anyhow: エラーハンドリング

## Project Structure

```
src/
├── main.rs           # エントリーポイント
├── bms/              # BMS データ処理
│   ├── loader.rs     # ファイル読み込み
│   ├── chart.rs      # Chart 構造体
│   └── timing.rs     # タイミング計算
├── audio/            # オーディオシステム
│   ├── manager.rs    # Kira ラッパー
│   ├── keysound.rs   # キー音管理
│   └── scheduler.rs  # スケジューリング
├── game/             # ゲームプレイ
│   ├── state.rs      # ゲーム状態
│   ├── judge.rs      # 判定システム
│   ├── input.rs      # 入力処理
│   └── score.rs      # スコア計算
├── render/           # グラフィックス
│   ├── highway.rs    # ノーツレーン
│   ├── notes.rs      # ノーツ描画
│   └── ui.rs         # HUD
├── scene/            # シーン管理
│   ├── select.rs     # 選曲画面
│   ├── play.rs       # プレイ画面
│   └── result.rs     # リザルト画面
└── config/           # 設定
    ├── settings.rs   # ユーザー設定
    └── keybinds.rs   # キーバインド
```

## Task List

### Phase 1: Foundation
- [ ] プロジェクトセットアップ (Cargo.toml)
- [ ] bms-rs で BMS ファイル読み込み
- [ ] 内部 Chart 構造体への変換
- [ ] タイミング計算実装
- [ ] 静的ノーツ表示

### Phase 2: Audio System
- [ ] Kira キー音読み込み
- [ ] オーディオスケジューラー
- [ ] BGM 再生
- [ ] STOP 対応

### Phase 3: Core Gameplay
- [ ] 入力システム
- [ ] ハイウェイスクロール
- [ ] ノーツ判定
- [ ] スコア・コンボ

### Phase 4: Song Selection
- [ ] フォルダスキャン
- [ ] 選曲 UI

### Phase 5: Polish
- [ ] リザルト画面
- [ ] ロングノート
- [ ] エフェクト
- [ ] 設定画面

### Future
- [ ] Mirror/Random
- [ ] BGA
- [ ] PMS 対応

## Documentation

- `docs/bms-specification.md` - BMS フォーマット仕様
- `docs/architecture.md` - アーキテクチャ
- `docs/technical-challenges.md` - 技術課題

## Key Implementation Notes

### Audio-Visual Sync
Kira のクロックを時間の唯一のソースとして使用。ビジュアルはオーディオクロックから導出。

### Timing Calculation
BPM 変更・拍子変更・STOP を正確に処理するため fraction で分数計算。

### Judgment Windows (beatoraja-style)
- PGREAT: ±20ms
- GREAT: ±60ms
- GOOD: ±150ms
- BAD: ±280ms

### Key Layout (7-key)
```
S 1 2 3 4 5 6 7
```
Channel mapping: 16, 11, 12, 13, 14, 15, 18, 19
