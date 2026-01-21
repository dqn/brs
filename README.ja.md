# brs

[English](./README.md)

Rust で実装された BMS (Be-Music Source) リズムゲームプレイヤー。

## 特徴

- **LR2/beatoraja 互換の判定**: LR2 と beatoraja の判定システムを切り替え可能
- **複数のゲージシステム**: ASSIST EASY、EASY、NORMAL、HARD、EX-HARD、HAZARD
- **Gauge Auto Shift (GAS)**: 全ゲージタイプを並列追跡し、最高クリアを自動達成
- **ロングノート対応**: LN、CN、HCN モードを完全サポート
- **動画 BGA 再生**: ffmpeg を使用し MPG、AVI、MP4、WebM などに対応
- **IIDX 専コン対応**: アナログ軸入力に対応した IIDX アーケードスタイルコントローラーのネイティブサポート
- **ビジュアルオプション**: SUDDEN+、HIDDEN+、LIFT、フローティング HI-SPEED
- **レーンオプション**: MIRROR、RANDOM、R-RANDOM
- **FAST/SLOW 表示**: ミリ秒精度のタイミングフィードバック

## インストール

### ビルド済みバイナリのダウンロード

[GitHub Releases](https://github.com/dqn/brs/releases) から対応プラットフォーム用の最新リリースをダウンロードしてください。

### ffmpeg の要件

brs は動画 BGA 再生に ffmpeg を必要とします。実行前にインストールしてください。

**macOS:**
```bash
brew install ffmpeg
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

**Windows:**

[gyan.dev](https://www.gyan.dev/ffmpeg/builds/) からプリビルトバイナリをダウンロードし、PATH に追加してください。

## ソースからビルド

### 必要条件

- Rust 1.85 以降
- ffmpeg 開発ライブラリ（上記参照）

### ビルド

```bash
cargo build --release
```

バイナリは `target/release/brs` に生成されます。

## 使い方

BMS フォルダのパスを指定して brs を実行します：

```bash
./brs /path/to/bms/folder
```

### 操作方法

7 キーモードのデフォルトキーボードレイアウト：

| キー | レーン |
|------|--------|
| 左 Shift | スクラッチ |
| Z | 1 |
| S | 2 |
| X | 3 |
| D | 4 |
| C | 5 |
| F | 6 |
| V | 7 |

### 設定

選曲画面から設定画面にアクセスし、以下を設定できます：

- キーバインド
- HI-SPEED と Green Number
- SUDDEN+/HIDDEN+/LIFT
- ゲージタイプ
- 判定システム（beatoraja/LR2）
- FAST/SLOW 表示オプション

## 対応フォーマット

### 譜面
- .bms、.bme、.bml

### 音声
- WAV、OGG、MP3、FLAC

### 動画
- MPG、MPEG、AVI、WMV、MP4、WebM、M4V（ffmpeg 必須）

## 機能比較

| 機能 | LR2 | beatoraja | brs |
|------|-----|-----------|-----|
| プラットフォーム | Windows | クロスプラットフォーム | クロスプラットフォーム |
| 判定システム | LR2 | beatoraja | 両方（切替可能） |
| Gauge Auto Shift | - | あり | あり |
| EX-HARD ゲージ | - | あり | あり |
| 動画 BGA | 部分的 | あり | あり |
| IIDX 専コン | あり | あり | あり |

詳細な比較は [docs/brs-comparison.md](./docs/brs-comparison.md) を参照してください。

## ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。詳細は [LICENSE](./LICENSE) ファイルを参照してください。

### サードパーティライセンス

このプロジェクトは以下のフォントを使用しています：
- **Noto Sans JP** - SIL Open Font License 1.1 の下でライセンスされています。[assets/fonts/OFL.txt](./assets/fonts/OFL.txt) を参照してください。
