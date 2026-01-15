# BMS Player Project

Rust で実装する BMS (Be-Music Source) リズムゲームプレイヤー。

## Tech Stack

- bms-rs: BMS パーシング
- kira: オーディオ (クロックベースタイミング)
- macroquad: 2D グラフィックス
- gilrs: ゲームパッド入力 (IIDX 専コン対応)
- ffmpeg-next: 動画 BGA デコード
- image: 静止画読み込み
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
│   └── scheduler.rs  # スケジューリング
├── game/             # ゲームプレイ
│   ├── game_state.rs # ゲーム状態
│   ├── judge.rs      # 判定システム
│   ├── gauge.rs      # ゲージシステム
│   ├── input.rs      # 入力処理
│   ├── gamepad.rs    # ゲームパッド入力
│   ├── options.rs    # RANDOM/MIRROR オプション
│   ├── score.rs      # スコア計算
│   └── result.rs     # リザルト処理
├── render/           # グラフィックス
│   ├── highway.rs    # ノーツレーン
│   ├── effects.rs    # 判定エフェクト
│   ├── lane_cover.rs # SUDDEN+/HIDDEN/LIFT
│   ├── bga.rs        # BGA 管理
│   └── video.rs      # 動画デコード
├── scene/            # シーン管理
│   ├── song_select.rs # 選曲画面
│   ├── gameplay.rs    # プレイ画面
│   ├── result.rs      # リザルト画面
│   └── settings.rs    # 設定画面
├── database/         # データ永続化
│   ├── repository.rs # リポジトリ
│   └── score.rs      # スコアデータ
└── config/           # 設定
    └── settings.rs   # ユーザー設定
```

## Task List

### Phase 1: Foundation ✅
- [x] プロジェクトセットアップ (Cargo.toml)
- [x] bms-rs で BMS ファイル読み込み
- [x] 内部 Chart 構造体への変換
- [x] タイミング計算実装
- [x] 静的ノーツ表示

### Phase 2: Audio System ✅
- [x] Kira キー音読み込み
- [x] オーディオスケジューラー
- [x] BGM 再生
- [x] STOP 対応

### Phase 3: Core Gameplay ✅
- [x] 入力システム
- [x] ハイウェイスクロール
- [x] ノーツ判定
- [x] スコア・コンボ
- [x] ロングノート判定 (LN/CN/HCN)

### Phase 4: Song Selection ✅
- [x] フォルダスキャン
- [x] 選曲 UI

### Phase 5: Polish ✅
- [x] リザルト画面
- [x] 設定画面
- [x] キーバインド設定
- [x] コントローラー対応 (IIDX 専コン軸入力対応)
- [x] BGA (静止画 + 動画)
- [x] ロングノート描画
- [x] エフェクト

### Phase 6: Essential Features ✅
- [x] ゲージシステム (EASY/NORMAL/HARD/EX-HARD)
- [x] Gauge Auto Shift (GAS)
- [x] LR2/beatoraja 判定・ゲージ切替
- [x] MIRROR/RANDOM オプション
- [x] スコア・クリアランプ保存
- [x] SUDDEN+/HIDDEN/LIFT
- [x] FAST/SLOW 表示 (ミリ秒表示対応)
- [x] Green Number 表示

### Future

#### フォーマット対応
- [ ] PMS 対応
- [ ] BMSON 対応
- [ ] FLAC 対応

#### レーンオプション
- [x] S-RANDOM
- [x] H-RANDOM

#### アシストオプション
- [ ] AUTO SCRATCH
- [ ] LEGACY NOTE (LN → 通常ノート変換)
- [ ] EXPAND JUDGE (判定幅拡大)
- [ ] BATTLE (1P/2P 反転)

#### インターネットランキング
- [ ] IR スコア送信
- [ ] 段位認定

#### その他
- [ ] カスタムスキン
- [ ] ダブルプレイ (DP)

## Documentation

- `docs/bms-specification.md` - BMS フォーマット仕様
- `docs/bms-player-comparison.md` - BMS プレイヤー機能比較表（LR2/beatoraja/bms-rs）

## Design Principles

### LR2/beatoraja Compatibility
ゲーム体験に関わる部分は他の BMS プレイヤーと同じ挙動を再現する。

- **判定タイミング**: LR2 / beatoraja を選択可能
- **ゲージシステム**: LR2 / beatoraja を選択可能
- **Gauge Auto Shift (GAS)**: 全ゲージを並列計算し、最高クリアを自動達成

### Key Differences: LR2 vs beatoraja

| 項目 | LR2 | beatoraja |
|------|-----|-----------|
| PGREAT | ±21ms | ±20ms |
| 空POOR | 手前のみ | 前後両方 |
| ダメージ軽減 | 32%以下で半減 | 50%から徐々に軽減 |
| LN早離し | BAD | POOR |

## Key Implementation Notes

### Audio-Visual Sync
Kira のクロックを時間の唯一のソースとして使用。ビジュアルはオーディオクロックから導出。

### Timing Calculation
BPM 変更・拍子変更・STOP を正確に処理するため fraction で分数計算。

### Judgment Windows

| Judge | beatoraja (EASY) | LR2 (EASY) |
|-------|------------------|------------|
| PGREAT | ±20ms | ±21ms |
| GREAT | ±60ms | ±60ms |
| GOOD | ±150ms | ±120ms |
| BAD | +220ms/-280ms | ±200ms |

beatoraja の BAD 判定は非対称（早押し +220ms、遅押し -280ms）。

### Key Layout (7-key)
```
S 1 2 3 4 5 6 7
```
Channel mapping: 16, 11, 12, 13, 14, 15, 18, 19

### BGA Video Support
動画 BGA には ffmpeg が必要:
- **macOS**: `brew install ffmpeg`
- **Linux**: `apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev`
- **Windows**: vcpkg または事前ビルド済みバイナリ

対応フォーマット: MPG, MPEG, AVI, WMV, MP4, WebM, M4V
