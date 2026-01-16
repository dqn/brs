# FLAC Support

FLAC オーディオフォーマットのサポート追加。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 低 |
| 難易度 | 低 |
| 推定工数 | 0.5日 |
| 依存関係 | なし |

## Background

現在のオーディオシステムは Kira 0.10 を使用しており、WAV, MP3, OGG をサポートしている。FLAC は可逆圧縮フォーマットで、一部の BMS 作者が高音質キー音として使用している。

## Dependencies

- なし（Kira の feature flag 追加のみ）

## Files to Modify

| ファイル | 変更内容 |
|----------|----------|
| `Cargo.toml` | Kira の features に `flac` を追加 |

## Implementation

### Single Change

```toml
# Cargo.toml
[dependencies]
kira = { version = "0.10", features = ["cpal", "mp3", "ogg", "wav", "flac"] }
```

`AudioManager::load_sound()` は `StaticSoundData::from_file()` を使用しており、ファイル形式は自動検出される。コード変更は不要。

## Verification

1. WAV キー音を FLAC に変換
   ```bash
   ffmpeg -i sound.wav -c:a flac sound.flac
   ```

2. BMS ファイルの `#WAV` 定義を FLAC ファイルに変更

3. ゲームを起動し、FLAC キー音が正しく再生されることを確認

4. レイテンシに問題がないか確認（FLAC はデコード負荷が若干高い）

## Notes

- Kira の FLAC サポートは `symphonia` crate を使用
- FLAC はロスレス圧縮のため、ファイルサイズは WAV より小さいがデコード負荷あり
- 一部の古い BMS では FLAC 未対応の場合があるため、WAV へのフォールバックは不要
