# BMS プレイヤー機能比較表

LR2、beatoraja、brs の 3 プレイヤーを比較。

## 概要

| 項目 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| 開発者 | Lavalse 他 | exch | - |
| 言語 | C++ | Java | Rust |
| プラットフォーム | Windows | クロスプラットフォーム | クロスプラットフォーム |
| ライセンス | プロプライエタリ | オープンソース | MIT |
| 最終更新 | 2010年2月 | 継続開発中 | 継続開発中 |
| 解像度 | SD/HD/FHD | 設定可能 | 設定可能 |

## 判定タイミング

### EASY 判定（ミリ秒）

| 判定 | LR2 | beatoraja (7keys) | brs (beatoraja) | brs (LR2) |
|------|-----|-------------------|-------------------|--------------|
| PGREAT | ±21ms | ±20ms | ±20ms | ±21ms |
| GREAT | ±60ms | ±60ms | ±60ms | ±60ms |
| GOOD | ±120ms | ±150ms | ±150ms | ±120ms |
| BAD | ±200ms | +220ms/-280ms | +220ms/-280ms | ±200ms |

brs は `JudgeSystemType` で beatoraja/LR2 の判定を切り替え可能。非対称 BAD 判定にも対応。

### 判定難易度による倍率

beatoraja/brs は EASY 判定に対して倍率を適用:

| 難易度 | 倍率 |
|--------|------|
| VERY EASY | 1.25x |
| EASY | 1.00x |
| NORMAL | 0.75x |
| HARD | 0.50x |
| VERY HARD | 0.25x |

### 空POOR（カラプア）

| 動作 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| 発生方向 | ノート手前のみ | ノート前後両方 | 切替可能（beatoraja/LR2） |
| 1ノートあたりの回数 | 複数回 | 1回（9keys） | beatoraja 仕様 |
| コンボ維持 | 維持 | 途切れる（9keys） | beatoraja 仕様 |

## ゲージシステム

### ゲージ種類

| ゲージ | LR2 | beatoraja | brs | 説明 |
|--------|-----|-----------|--------|------|
| ASSIST EASY | - | ○ | ○ | 60%クリア、ダメージ軽減 |
| EASY | ○ | ○ | ○ | LR2: 60%クリア、回復+20% / beatoraja: 80%クリア |
| NORMAL | ○ | ○ | ○ | 80%クリア |
| HARD | ○ | ○ | ○ | 生存型、100%開始 |
| EX-HARD | - | ○ | ○ | HARD + POOR/BAD ダメージ2倍 |
| HAZARD | ○ | ○ | ○ | BAD/POOR で即失敗 |

### NORMAL ゲージ回復/ダメージ

TOTAL 値に基づく: `a = TOTAL / ノート数`

| 判定 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| PGREAT | +a | +a | +a（LR2 モード時 TOTAL スケーリング適用） |
| GREAT | +a | +a | +a |
| GOOD | +0.6a | +0.5a | +0.5a |
| BAD | -4.0 | -2.0 | -3.0 |
| POOR | -6.0 | -6.0 | -6.0 |
| 空POOR | -2.0 | -6.0 | -2.0 |

### HARD ゲージダメージ軽減

| 動作 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| 軽減開始 | 32% | 50% | beatoraja: 50% / LR2: 32% |
| 軽減率 | 固定50%軽減 | 段階的（HP%に比例） | 切替可能 |
| 失敗閾値 | 2% | 0% | 0%（beatoraja 仕様） |

### 低 TOTAL ペナルティ

| プレイヤー | 動作 |
|------------|------|
| LR2 | 低 TOTAL + 低ノート数で HARD ダメージ増加 |
| beatoraja | TOTAL < 250 で HARD/EX-HARD の回復量低下 |
| brs | LR2 モード時: ノート数ベースのダメージ倍率（1000未満で増加） |

### Gauge Auto Shift (GAS)

| プレイヤー | 対応 | 説明 |
|------------|------|------|
| LR2 | - | - |
| beatoraja | ○ | 全ゲージを並列追跡、最高クリアを自動達成 |
| brs | ○ | 全ゲージを並列追跡、失敗時に次ゲージへ自動シフト |

## ロングノート

### モード比較

| モード | 終端判定 | ノート数 | リリース判定 |
|--------|----------|----------|--------------|
| LN | なし | 1 | GOOD 幅内なら開始と同じ判定 |
| CN | あり | 2 | タイミング判定あり |
| HCN | あり | 2 | 離している間ダメージ継続 |

### 早離し時の判定

| プレイヤー | LN モード | CN/HCN モード |
|------------|-----------|---------------|
| LR2 | 早BAD | 限定的サポート |
| beatoraja | 早POOR | 早POOR |
| brs | 早POOR | 早POOR |

### CN リリース判定幅（brs）

| 判定 | 幅 |
|------|-----|
| PGREAT | ±120ms |
| GREAT | ±160ms |
| GOOD | ±200ms |
| BAD | ±280ms |

## 対応フォーマット

### 譜面フォーマット

| 形式 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| .bms | ○ | ○ | ○ |
| .bme | ○ | ○ | ○ |
| .bml | ○ | ○ | ○ |
| .pms | ○ | ○ | - |
| .bmson | - | ○ | - |

### 音声フォーマット

| 形式 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| WAV | ○ | ○ (8/16/24/32bit) | ○ |
| OGG | ○ | ○ | ○ |
| FLAC | 要パッチ | ○ | ○ |
| MP3 | - | ○ | ○ |

### 動画フォーマット

| 形式 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| MPG/MPEG | ○ | ○ | ○ |
| AVI | ○ | ○ | ○ |
| WMV | △ | ○ | ○ |
| MP4 | - | ○ | ○ |
| WebM | - | ○ | ○ |
| M4V | - | ○ | ○ |

brs は ffmpeg-next を使用し、ffmpeg 対応フォーマットをサポート。

## プレイオプション

### レーンオプション

| オプション | LR2 | beatoraja | brs | 説明 |
|------------|-----|-----------|--------|------|
| MIRROR | ○ | ○ | ○ | レーンを左右反転 |
| RANDOM | ○ | ○ | ○ | レーン位置をランダム化 |
| S-RANDOM | ○ | ○ | ○ | ノートごとにランダム化 |
| R-RANDOM | ○ | ○ | ○ | 回転式ランダム |
| H-RANDOM | - | ○ | ○ | 高密度制限ランダム |

### ビジュアルオプション

| オプション | LR2 | beatoraja | brs | 説明 |
|------------|-----|-----------|--------|------|
| SUDDEN+ | ○ | ○ | ○ (0-900) | レーン上部をカバー |
| HIDDEN+ | ○ | ○ | ○ (0-500) | 判定線下をカバー |
| LIFT | ○ | ○ | ○ (0-500) | 判定線位置を上昇 |
| HI-SPEED | ○ | ○ | ○ | ノートスクロール速度 |
| フローティングHI-SPEED | ○ | ○ | ○ | BPM に応じた自動調整 |

### Green Number

| プレイヤー | 単位 | 計算 |
|------------|------|------|
| LR2 | フレーム×10 | Green 300 = 30フレーム = 500ms (60fps) |
| beatoraja | ミリ秒 | 直接表示 |
| brs | ミリ秒 | 直接表示 |

### アシストオプション

| オプション | LR2 | beatoraja | brs | 説明 |
|------------|-----|-----------|--------|------|
| AUTO SCRATCH | ○ | ○ | - | スクラッチ自動プレイ |
| LEGACY NOTE | - | ○ | ○ | LN を通常ノートに変換 |
| EXPAND JUDGE | - | ○ | ○ | 判定幅拡大 |
| BATTLE | ○ | ○ | ○ | 1P/2P 反転 |

## FAST/SLOW 表示

| 機能 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| FAST/SLOW 表示 | ○（要HDパッチ） | ○ | ○ |
| ミリ秒表示 | - | ○ | ○ |
| 統計表示 | △ | 詳細グラフ | カウント表示 |
| EXACT 閾値 | - | - | ±1ms |

## インターネットランキング (IR)

| 機能 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| 公式IR | LR2IR（単一） | 複数サーバー | - |
| コミュニティ規模 | 最大 | 拡大中（分散） | - |
| スコア送信 | 自動 | 設定可能 | - |
| 段位認定 | ○ | ○ | - |

## スキンシステム

| 機能 | LR2 | beatoraja | brs |
|------|-----|-----------|--------|
| カスタムスキン | ○ | ○ | - |
| LR2 スキン互換 | - | ○（部分的） | - |
| 人気スキン | 多数 | LITONE, ModernChic | - |

## 既知の問題
### LR2

- 2010年以降更新なし
- メモリリーク
- うるう年バグ（2/29に全曲NEW扱い）
- 高リフレッシュレートモニター非対応（フルスクリーン時）
- FLAC/MP4 は限定的サポート

### beatoraja

- リソース消費量が多い（Java ベース）
- IR コミュニティの分散
- 一部 LR2 スキン非互換

### brs

- PMS 未対応
- BMSON 未対応
- IR 未実装
- スコア保存未完成

## 派生版・フォーク

### LR2 派生版

| 派生版 | 説明 |
|--------|------|
| LR2HD | 1280x720 解像度対応 |
| LR2FHD | 1920x1080 解像度対応 |
| FLAC パッチ | FLAC 音声再生対応 |

### beatoraja フォーク

| フォーク | 説明 |
|----------|------|
| LR2oraja | LR2 判定・ゲージ仕様を使用 |
| LR2oraja ~Endless Dream~ | LR2oraja + レート変更、ランダム練習 |
| IIDXoraja | IIDX ライク仕様 |

## brs 固有機能

| 機能 | 説明 |
|------|------|
| 判定システム切替 | JudgeSystemType で beatoraja/LR2 を選択可能 |
| ゲージシステム切替 | GaugeSystem で beatoraja/LR2 を選択可能 |
| GAS | Gauge Auto Shift - 全ゲージを並列追跡 |
| IIDX 専コン対応 | gilrs による軸入力対応 |

## 参考リンク

- [IIDX LR2 beatoraja differences](https://iidx.org/misc/iidx_lr2_beatoraja_diff)
- [beatoraja/LR2 仕様比較まとめ](https://ralba-gear.hateblo.jp/entry/2023/11/06/141139)
- [BMS Community Resources](https://bms-community.github.io/resources/)
- [beatoraja GitHub](https://github.com/exch-bms2/beatoraja)
- [beatoraja English Guide](https://wcko87.github.io/beatoraja-english-guide/)
- [Lunatic Rave 2 Guidance](https://bmsoffighters.net/lr2/)
