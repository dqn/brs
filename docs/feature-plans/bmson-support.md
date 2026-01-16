# BMSON Support

BMSON (BMS Object Notation) フォーマットのサポート。JSON ベースの現代的な BMS フォーマット。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 中 |
| 難易度 | 高 |
| 推定工数 | 4-5日 |
| 依存関係 | なし |

## Background

BMSON は BMS の後継フォーマットとして設計された JSON ベースのフォーマット。beatoraja で標準サポートされている。テキストパースの曖昧さを排除し、拡張性を持たせた設計。

### Key Differences from BMS

| 項目 | BMS | BMSON |
|------|-----|-------|
| フォーマット | テキスト | JSON |
| タイミング | 小節ベース (Fraction) | Tick ベース (240/beat) |
| BPM 変更 | チャンネル埋め込み | 明示的イベント配列 |
| 音声ファイル | #WAV 定義 | sound_channels 配列 |
| メタデータ | #TITLE 等 | info オブジェクト |

## Dependencies

- `serde_json` (既存)

## Files to Modify/Create

| ファイル | 変更内容 |
|----------|----------|
| `src/bms/bmson.rs` (新規) | BMSON パーサー |
| `src/bms/loader.rs` | フォーマット検出と分岐 |
| `src/bms/mod.rs` | bmson モジュール追加 |
| `src/scene/song_select.rs` | `.bmson` 拡張子対応 |

## BMSON Format Structure

```json
{
  "version": "1.0.0",
  "info": {
    "title": "Song Title",
    "subtitle": "",
    "artist": "Artist",
    "subartists": ["Arranger"],
    "genre": "Genre",
    "mode_hint": "beat-7k",
    "chart_name": "ANOTHER",
    "level": 12,
    "init_bpm": 150.0,
    "total": 300,
    "judge_rank": 100
  },
  "lines": [
    { "y": 0 },
    { "y": 960 }
  ],
  "bpm_events": [
    { "y": 3840, "bpm": 180.0 }
  ],
  "stop_events": [
    { "y": 7680, "duration": 48 }
  ],
  "sound_channels": [
    {
      "name": "bgm.ogg",
      "notes": [
        { "x": 0, "y": 0, "l": 0, "c": true }
      ]
    }
  ],
  "bga": {
    "bga_header": [
      { "id": 0, "name": "bga.mpg" }
    ],
    "bga_events": [
      { "y": 0, "id": 0 }
    ],
    "layer_events": [],
    "poor_events": []
  }
}
```

## Implementation Phases

### Phase 1: BMSON Data Structures

```rust
// src/bms/bmson.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Bmson {
    pub version: String,
    pub info: BmsonInfo,
    #[serde(default)]
    pub lines: Vec<BarLine>,
    #[serde(default)]
    pub bpm_events: Vec<BpmEvent>,
    #[serde(default)]
    pub stop_events: Vec<StopEvent>,
    pub sound_channels: Vec<SoundChannel>,
    #[serde(default)]
    pub bga: Option<BgaData>,
}

#[derive(Debug, Deserialize)]
pub struct BmsonInfo {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub subartists: Vec<String>,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub mode_hint: String,
    #[serde(default)]
    pub chart_name: String,
    #[serde(default)]
    pub level: u32,
    pub init_bpm: f64,
    #[serde(default)]
    pub total: Option<f64>,
    #[serde(default)]
    pub judge_rank: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BarLine {
    pub y: i64,
}

#[derive(Debug, Deserialize)]
pub struct BpmEvent {
    pub y: i64,
    pub bpm: f64,
}

#[derive(Debug, Deserialize)]
pub struct StopEvent {
    pub y: i64,
    pub duration: i64,
}

#[derive(Debug, Deserialize)]
pub struct SoundChannel {
    pub name: String,
    pub notes: Vec<BmsonNote>,
}

#[derive(Debug, Deserialize)]
pub struct BmsonNote {
    /// Lane: 0=BGM, 1-7=keys, 8=scratch (for beat-7k)
    pub x: i32,
    /// Position in ticks (resolution: 240 per beat)
    pub y: i64,
    /// Long note length in ticks (0 for normal notes)
    #[serde(default)]
    pub l: i64,
    /// Continue flag (for sliced sounds)
    #[serde(default)]
    pub c: bool,
}

#[derive(Debug, Deserialize)]
pub struct BgaData {
    #[serde(default)]
    pub bga_header: Vec<BgaHeader>,
    #[serde(default)]
    pub bga_events: Vec<BgaEvent>,
    #[serde(default)]
    pub layer_events: Vec<BgaEvent>,
    #[serde(default)]
    pub poor_events: Vec<BgaEvent>,
}

#[derive(Debug, Deserialize)]
pub struct BgaHeader {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BgaEvent {
    pub y: i64,
    pub id: u32,
}
```

### Phase 2: BMSON to Chart Conversion

```rust
// src/bms/bmson.rs

const TICKS_PER_BEAT: f64 = 240.0;

impl Bmson {
    pub fn to_chart(&self) -> Result<Chart> {
        let mut notes = Vec::new();
        let mut bgm_events = Vec::new();
        let mut keysound_id = 0u32;
        let mut wav_files = HashMap::new();

        // Build timing data from BPM events
        let timing_data = self.build_timing_data();

        for channel in &self.sound_channels {
            // Register sound file
            wav_files.insert(keysound_id, channel.name.clone());

            for bmson_note in &channel.notes {
                let time_ms = self.ticks_to_ms(bmson_note.y, &timing_data);

                if bmson_note.x == 0 {
                    // BGM
                    bgm_events.push(BgmEvent {
                        time_ms,
                        keysound_id,
                    });
                } else {
                    // Note
                    let note_channel = Self::x_to_channel(bmson_note.x)?;
                    let note_type = if bmson_note.l > 0 {
                        NoteType::LongStart
                    } else {
                        NoteType::Normal
                    };

                    let (measure, position) = self.ticks_to_measure_position(bmson_note.y);

                    notes.push(Note {
                        measure,
                        position,
                        time_ms,
                        channel: note_channel,
                        keysound_id,
                        note_type,
                        long_end_time_ms: if bmson_note.l > 0 {
                            Some(self.ticks_to_ms(bmson_note.y + bmson_note.l, &timing_data))
                        } else {
                            None
                        },
                    });

                    // Add LongEnd for long notes
                    if bmson_note.l > 0 {
                        let end_time_ms = self.ticks_to_ms(
                            bmson_note.y + bmson_note.l,
                            &timing_data
                        );
                        let (end_measure, end_position) = self.ticks_to_measure_position(
                            bmson_note.y + bmson_note.l
                        );

                        notes.push(Note {
                            measure: end_measure,
                            position: end_position,
                            time_ms: end_time_ms,
                            channel: note_channel,
                            keysound_id,
                            note_type: NoteType::LongEnd,
                            long_end_time_ms: None,
                        });
                    }
                }
            }

            keysound_id += 1;
        }

        // Sort notes by time
        notes.sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap());

        Ok(Chart {
            metadata: Metadata {
                title: self.info.title.clone(),
                artist: self.info.artist.clone(),
                bpm: self.info.init_bpm,
                total: self.info.total.unwrap_or(300.0),
                rank: self.info.judge_rank.unwrap_or(100),
                ln_type: LnType::Cn, // BMSON defaults to CN
                play_mode: PlayMode::Bms7Key,
            },
            timing_data,
            notes,
            bgm_events,
            bga_events: self.convert_bga_events()?,
        })
    }

    fn ticks_to_ms(&self, ticks: i64, timing: &TimingData) -> f64 {
        // Calculate time considering BPM changes
        let mut current_time = 0.0;
        let mut current_tick = 0i64;
        let mut current_bpm = self.info.init_bpm;

        let ms_per_tick = |bpm: f64| 60000.0 / bpm / TICKS_PER_BEAT;

        for bpm_event in &timing.bpm_changes {
            let event_tick = (bpm_event.measure as f64 * 4.0 * TICKS_PER_BEAT) as i64;

            if event_tick >= ticks {
                break;
            }

            current_time += (event_tick - current_tick) as f64 * ms_per_tick(current_bpm);
            current_tick = event_tick;
            current_bpm = bpm_event.bpm;
        }

        current_time + (ticks - current_tick) as f64 * ms_per_tick(current_bpm)
    }

    fn x_to_channel(x: i32) -> Result<NoteChannel> {
        match x {
            1 => Ok(NoteChannel::Key1),
            2 => Ok(NoteChannel::Key2),
            3 => Ok(NoteChannel::Key3),
            4 => Ok(NoteChannel::Key4),
            5 => Ok(NoteChannel::Key5),
            6 => Ok(NoteChannel::Key6),
            7 => Ok(NoteChannel::Key7),
            8 => Ok(NoteChannel::Scratch),
            _ => Err(anyhow!("Invalid BMSON lane: {}", x)),
        }
    }
}
```

### Phase 3: Loader Integration

```rust
// src/bms/loader.rs

impl BmsLoader {
    pub fn load_full<P: AsRef<Path>>(path: P) -> Result<BmsLoadResult> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        match ext.as_deref() {
            Some("bmson") => Self::load_bmson(path),
            _ => Self::load_bms(path),
        }
    }

    fn load_bmson(path: &Path) -> Result<BmsLoadResult> {
        let content = std::fs::read_to_string(path)?;
        let bmson: Bmson = serde_json::from_str(&content)?;

        let chart = bmson.to_chart()?;
        let wav_files = bmson.collect_wav_files();
        let bmp_files = bmson.collect_bmp_files();

        Ok(BmsLoadResult {
            chart,
            wav_files,
            bmp_files,
        })
    }
}
```

### Phase 4: Song Select Extension

```rust
// src/scene/song_select.rs

if ext_lower == "bms" || ext_lower == "bme" || ext_lower == "bml"
   || ext_lower == "pms" || ext_lower == "bmson" {
    // ...
}
```

## Verification

1. BMSON テストファイルをダウンロード（beatoraja コミュニティから）
2. JSON パースが正しく動作することを確認
3. タイミング計算が beatoraja と一致することを確認（±1ms 程度の誤差は許容）
4. BPM 変更イベントが正しく処理されることを確認
5. ロングノート変換が正しいことを確認
6. 音声同期を確認

## Notes

- BMSON の tick 解像度は 240/beat が標準
- `c` (continue) フラグはスライス音声用だが、初期実装では無視可能
- mode_hint で 7k/9k/14k 等を判別可能
- BMSON 1.0 と 0.21 で互換性の違いがある場合がある
