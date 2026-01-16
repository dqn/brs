# PMS Support (9-Key)

PMS (Pop'n Music Script) フォーマットのサポート。9 キーレイアウト（スクラッチなし）。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 中 |
| 難易度 | 中 |
| 推定工数 | 2-3日 |
| 依存関係 | なし |

## Background

PMS は Pop'n Music 風の 9 キーリズムゲーム用フォーマット。BMS と同じテキストベースだが、チャンネルマッピングとキーレイアウトが異なる。

### Key Differences from BMS

| 項目 | BMS (7-key) | PMS (9-key) |
|------|-------------|-------------|
| レーン数 | 8 (Scratch + 7 keys) | 9 (keys only) |
| スクラッチ | あり | なし |
| チャンネル | 16, 11-15, 18-19 | 11-19 |

## Dependencies

- なし

## Files to Modify

| ファイル | 変更内容 |
|----------|----------|
| `src/bms/chart.rs` | `NoteChannel` に Key8, Key9 追加、`PlayMode` enum 追加 |
| `src/bms/loader.rs` | PMS チャンネルマッピング追加 |
| `src/render/config.rs` | `LANE_COUNT` を動的化または PMS 用定数追加 |
| `src/render/highway.rs` | 9 レーン描画対応 |
| `src/game/input.rs` | 9 キー入力対応 |
| `src/config/settings.rs` | `KeyBindings9Key` 追加 |
| `src/scene/song_select.rs` | `.pms` 拡張子対応 |

## Implementation Phases

### Phase 1: Data Model Extension

```rust
// src/bms/chart.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    #[default]
    Bms7Key,  // 7 keys + scratch
    Pms9Key,  // 9 keys (no scratch)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteChannel {
    Scratch,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7,
    Key8, Key9,  // PMS additions
}

impl NoteChannel {
    pub fn lane_index_for_mode(&self, mode: PlayMode) -> usize {
        match mode {
            PlayMode::Pms9Key => match self {
                Self::Key1 => 0,
                Self::Key2 => 1,
                Self::Key3 => 2,
                Self::Key4 => 3,
                Self::Key5 => 4,
                Self::Key6 => 5,
                Self::Key7 => 6,
                Self::Key8 => 7,
                Self::Key9 => 8,
                Self::Scratch => 0, // Fallback (should not occur)
            },
            PlayMode::Bms7Key => self.lane_index(), // Existing logic
        }
    }

    pub fn from_pms_channel(channel: u32) -> Option<Self> {
        match channel {
            11 => Some(Self::Key1),
            12 => Some(Self::Key2),
            13 => Some(Self::Key3),
            14 => Some(Self::Key4),
            15 => Some(Self::Key5),
            22 => Some(Self::Key6),  // PMS uses 22 for key 6
            23 => Some(Self::Key7),  // PMS uses 23 for key 7
            24 => Some(Self::Key8),  // PMS uses 24 for key 8
            25 => Some(Self::Key9),  // PMS uses 25 for key 9
            _ => None,
        }
    }
}

// Add to Metadata
pub struct Metadata {
    // ... existing fields
    pub play_mode: PlayMode,
}
```

### Phase 2: Loader Extension

```rust
// src/bms/loader.rs

impl BmsLoader {
    pub fn load_full<P: AsRef<Path>>(path: P) -> Result<BmsLoadResult> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        let is_pms = ext.as_deref() == Some("pms");

        // ... existing parsing

        let mut chart = convert_to_chart(&bms);
        if is_pms {
            chart.metadata.play_mode = PlayMode::Pms9Key;
        }

        // ...
    }
}

fn channel_id_to_note_channel(
    channel_id: &NoteChannelId,
    play_mode: PlayMode,
) -> Option<NoteChannel> {
    match play_mode {
        PlayMode::Pms9Key => {
            // PMS channel mapping
            // ...
        }
        PlayMode::Bms7Key => {
            // Existing BMS mapping
            // ...
        }
    }
}
```

### Phase 3: Dynamic Lane Count

```rust
// src/render/config.rs

pub const LANE_COUNT_BMS: usize = 8;
pub const LANE_COUNT_PMS: usize = 9;

pub fn lane_count(mode: PlayMode) -> usize {
    match mode {
        PlayMode::Bms7Key => LANE_COUNT_BMS,
        PlayMode::Pms9Key => LANE_COUNT_PMS,
    }
}
```

### Phase 4: Highway Rendering

```rust
// src/render/highway.rs

impl Highway {
    pub fn draw_with_state(
        &self,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        scroll_speed: f32,
    ) {
        let lane_count = lane_count(chart.metadata.play_mode);
        let total_width = self.config.lane_width * lane_count as f32;
        let highway_x = (screen_width() - total_width) / 2.0;

        // PMS uses rainbow colors
        let lane_colors = if chart.metadata.play_mode == PlayMode::Pms9Key {
            PMS_LANE_COLORS
        } else {
            BMS_LANE_COLORS
        };

        // Draw lanes
        for i in 0..lane_count {
            // ...
        }
    }
}

const PMS_LANE_COLORS: [Color; 9] = [
    WHITE,   // Key1
    YELLOW,  // Key2
    GREEN,   // Key3
    BLUE,    // Key4
    RED,     // Key5 (center)
    BLUE,    // Key6
    GREEN,   // Key7
    YELLOW,  // Key8
    WHITE,   // Key9
];
```

### Phase 5: Input Extension

```rust
// src/config/settings.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings9Key {
    pub key1: String,
    pub key2: String,
    pub key3: String,
    pub key4: String,
    pub key5: String,
    pub key6: String,
    pub key7: String,
    pub key8: String,
    pub key9: String,
}

impl Default for KeyBindings9Key {
    fn default() -> Self {
        Self {
            key1: "A".to_string(),
            key2: "S".to_string(),
            key3: "D".to_string(),
            key4: "F".to_string(),
            key5: "Space".to_string(),
            key6: "J".to_string(),
            key7: "K".to_string(),
            key8: "L".to_string(),
            key9: "Semicolon".to_string(),
        }
    }
}
```

### Phase 6: Song Select Extension

```rust
// src/scene/song_select.rs

// Add .pms to extension check
if ext_lower == "bms" || ext_lower == "bme" || ext_lower == "bml"
   || ext_lower == "pms" {
    // ...
}
```

## Verification

1. PMS サンプルファイルをダウンロード（BMS Archive 等から）
2. 9 レーンが正しく表示されることを確認
3. チャンネルマッピングが正しいことを確認（11-15, 22-25 → Key1-Key9）
4. 9 キー入力が正しく動作することを確認
5. スコア保存が PMS でも動作することを確認

## Notes

- PMS のチャンネルマッピングは BMS と微妙に異なる（Key6-9 が 22-25）
- 一部の PMS は BMS 互換チャンネル（18-19 を Key6-7 として使用）を使う場合もある
- PMS の難易度表記は BMS と異なることがある（1-50 等）
