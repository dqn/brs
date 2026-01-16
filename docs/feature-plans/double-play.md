# Double Play (DP) Mode

ダブルプレイモードの実装。14 レーン（P1: Scratch+7keys, P2: 7keys+Scratch）を 1 人でプレイ。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 低 |
| 難易度 | 高 |
| 推定工数 | 4-5日 |
| 依存関係 | PMS 対応（可変レーン数の基盤として有用） |

## Background

DP (Double Play) は IIDX の上級プレイモード。1 人で P1 側と P2 側の両方を同時にプレイする。

### Lane Layout

```
P1 Side              P2 Side
S 1 2 3 4 5 6 7 | 8 9 10 11 12 13 14 S
```

- P1: Scratch + Key1-7 (lanes 0-7)
- P2: Key8-14 + Scratch (lanes 8-15)

### BMS Channel Mapping for DP

| チャンネル | P1 | P2 |
|-----------|----|----|
| Scratch | 16 | 26 |
| Key 1/8 | 11 | 21 |
| Key 2/9 | 12 | 22 |
| Key 3/10 | 13 | 23 |
| Key 4/11 | 14 | 24 |
| Key 5/12 | 15 | 25 |
| Key 6/13 | 18 | 28 |
| Key 7/14 | 19 | 29 |

## Dependencies

- PMS 対応（可変レーン数の基盤）- オプショナル

## Files to Modify

| ファイル | 変更内容 |
|----------|----------|
| `src/bms/chart.rs` | `NoteChannel` に P2 側を追加、`PlayMode::Dp14Key` |
| `src/bms/loader.rs` | P2 チャンネル (21-29) のパース |
| `src/render/config.rs` | 16 レーン対応 |
| `src/render/highway.rs` | デュアルハイウェイ描画 |
| `src/game/input.rs` | 16 キー入力対応 |
| `src/game/game_state.rs` | 16 レーン状態管理 |
| `src/config/settings.rs` | DP キーバインド追加 |

## Implementation Phases

### Phase 1: Extended NoteChannel

```rust
// src/bms/chart.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteChannel {
    // P1 Side (existing, renamed for clarity)
    Scratch1,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7,

    // P2 Side (new)
    Key8, Key9, Key10, Key11, Key12, Key13, Key14,
    Scratch2,
}

impl NoteChannel {
    /// Get lane index for the given play mode
    pub fn lane_index_for_mode(&self, mode: PlayMode) -> usize {
        match mode {
            PlayMode::Dp14Key => match self {
                // P1 side: 0-7
                Self::Scratch1 => 0,
                Self::Key1 => 1,
                Self::Key2 => 2,
                Self::Key3 => 3,
                Self::Key4 => 4,
                Self::Key5 => 5,
                Self::Key6 => 6,
                Self::Key7 => 7,
                // P2 side: 8-15
                Self::Key8 => 8,
                Self::Key9 => 9,
                Self::Key10 => 10,
                Self::Key11 => 11,
                Self::Key12 => 12,
                Self::Key13 => 13,
                Self::Key14 => 14,
                Self::Scratch2 => 15,
            },
            PlayMode::Bms7Key => self.lane_index(), // Existing SP logic
            PlayMode::Pms9Key => self.lane_index_pms(),
        }
    }

    /// Check if this channel is P2 side
    pub fn is_p2(&self) -> bool {
        matches!(
            self,
            Self::Key8 | Self::Key9 | Self::Key10 | Self::Key11 |
            Self::Key12 | Self::Key13 | Self::Key14 | Self::Scratch2
        )
    }

    /// Check if this is a scratch channel
    pub fn is_scratch(&self) -> bool {
        matches!(self, Self::Scratch1 | Self::Scratch2)
    }

    /// Convert from BMS channel number for DP
    pub fn from_bms_channel_dp(channel: u32) -> Option<Self> {
        match channel {
            // P1 visible notes
            16 => Some(Self::Scratch1),
            11 => Some(Self::Key1),
            12 => Some(Self::Key2),
            13 => Some(Self::Key3),
            14 => Some(Self::Key4),
            15 => Some(Self::Key5),
            18 => Some(Self::Key6),
            19 => Some(Self::Key7),
            // P2 visible notes
            26 => Some(Self::Scratch2),
            21 => Some(Self::Key8),
            22 => Some(Self::Key9),
            23 => Some(Self::Key10),
            24 => Some(Self::Key11),
            25 => Some(Self::Key12),
            28 => Some(Self::Key13),
            29 => Some(Self::Key14),
            // P1 invisible notes
            36 => Some(Self::Scratch1),
            31 => Some(Self::Key1),
            // ... (for autoplay/invisible notes)
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    #[default]
    Bms7Key,    // SP: 7 keys + scratch
    Pms9Key,    // PMS: 9 keys
    Dp14Key,    // DP: 14 keys + 2 scratches
}

impl PlayMode {
    pub fn lane_count(&self) -> usize {
        match self {
            Self::Bms7Key => 8,
            Self::Pms9Key => 9,
            Self::Dp14Key => 16,
        }
    }
}
```

### Phase 2: Loader Extension for DP

```rust
// src/bms/loader.rs

fn channel_id_to_note_channel(
    channel_id: &NoteChannelId,
    play_mode: PlayMode,
) -> Option<NoteChannel> {
    let mapping = channel_id.try_into_map::<KeyLayoutBeat>()?;
    let key = mapping.key();
    let player = mapping.player();

    match play_mode {
        PlayMode::Dp14Key => {
            match (player, key) {
                (Player::P1, Key::Scratch(_)) => Some(NoteChannel::Scratch1),
                (Player::P1, Key::Key(1)) => Some(NoteChannel::Key1),
                (Player::P1, Key::Key(2)) => Some(NoteChannel::Key2),
                (Player::P1, Key::Key(3)) => Some(NoteChannel::Key3),
                (Player::P1, Key::Key(4)) => Some(NoteChannel::Key4),
                (Player::P1, Key::Key(5)) => Some(NoteChannel::Key5),
                (Player::P1, Key::Key(6)) => Some(NoteChannel::Key6),
                (Player::P1, Key::Key(7)) => Some(NoteChannel::Key7),
                (Player::P2, Key::Scratch(_)) => Some(NoteChannel::Scratch2),
                (Player::P2, Key::Key(1)) => Some(NoteChannel::Key8),
                (Player::P2, Key::Key(2)) => Some(NoteChannel::Key9),
                (Player::P2, Key::Key(3)) => Some(NoteChannel::Key10),
                (Player::P2, Key::Key(4)) => Some(NoteChannel::Key11),
                (Player::P2, Key::Key(5)) => Some(NoteChannel::Key12),
                (Player::P2, Key::Key(6)) => Some(NoteChannel::Key13),
                (Player::P2, Key::Key(7)) => Some(NoteChannel::Key14),
                _ => None,
            }
        }
        PlayMode::Bms7Key => {
            // Existing SP logic (ignore P2 channels)
            match (player, key) {
                (Player::P1, Key::Scratch(_)) => Some(NoteChannel::Scratch1),
                (Player::P1, Key::Key(n)) => NoteChannel::from_key_lane(n as usize),
                _ => None,
            }
        }
        PlayMode::Pms9Key => {
            // PMS logic
            // ...
        }
    }
}

/// Detect if chart is DP based on P2 channel presence
fn detect_play_mode(bms: &Bms) -> PlayMode {
    let has_p2_notes = bms.notes().any(|note| {
        let channel = note.channel_id().raw();
        // P2 visible channels: 21-29
        (21..=29).contains(&channel) || channel == 26
    });

    if has_p2_notes {
        PlayMode::Dp14Key
    } else {
        PlayMode::Bms7Key
    }
}
```

### Phase 3: DP Highway Rendering

```rust
// src/render/highway.rs

impl Highway {
    pub fn draw_dp(
        &self,
        chart: &Chart,
        play_state: &GamePlayState,
        current_time_ms: f64,
        scroll_speed: f32,
    ) {
        let lane_count = 16;
        let p1_lanes = 8;
        let center_gap = 20.0; // Gap between P1 and P2

        let total_width = self.config.lane_width * lane_count as f32 + center_gap;
        let highway_x = (screen_width() - total_width) / 2.0;

        // Draw P1 side (lanes 0-7)
        for i in 0..p1_lanes {
            let x = highway_x + i as f32 * self.config.lane_width;
            self.draw_lane(i, x);
        }

        // Draw center divider
        let center_x = highway_x + p1_lanes as f32 * self.config.lane_width;
        draw_rectangle(
            center_x,
            0.0,
            center_gap,
            screen_height(),
            Color::new(0.1, 0.1, 0.1, 1.0),
        );

        // Draw P2 side (lanes 8-15)
        for i in p1_lanes..lane_count {
            let x = highway_x + i as f32 * self.config.lane_width + center_gap;
            self.draw_lane(i, x);
        }

        // Draw notes
        self.draw_notes_dp(chart, play_state, highway_x, center_gap, current_time_ms, scroll_speed);

        // Draw judge lines for both sides
        self.draw_judge_line(highway_x, p1_lanes as f32 * self.config.lane_width);
        self.draw_judge_line(
            highway_x + p1_lanes as f32 * self.config.lane_width + center_gap,
            p1_lanes as f32 * self.config.lane_width,
        );
    }

    fn lane_color_dp(&self, lane_index: usize) -> Color {
        // P1/P2 symmetric coloring
        let normalized = if lane_index < 8 {
            lane_index
        } else {
            15 - lane_index // Mirror P2 to match P1
        };

        match normalized {
            0 => Color::new(1.0, 0.3, 0.3, 1.0),  // Scratch (red)
            1 | 3 | 5 | 7 => WHITE,                // White keys
            2 | 4 | 6 => Color::new(0.3, 0.5, 1.0, 1.0), // Blue keys
            _ => WHITE,
        }
    }
}
```

### Phase 4: DP Input Bindings

```rust
// src/config/settings.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingsDp {
    // P1 Side
    pub p1_scratch: String,
    pub p1_key1: String,
    pub p1_key2: String,
    pub p1_key3: String,
    pub p1_key4: String,
    pub p1_key5: String,
    pub p1_key6: String,
    pub p1_key7: String,
    // P2 Side
    pub p2_key1: String,
    pub p2_key2: String,
    pub p2_key3: String,
    pub p2_key4: String,
    pub p2_key5: String,
    pub p2_key6: String,
    pub p2_key7: String,
    pub p2_scratch: String,
}

impl Default for KeyBindingsDp {
    fn default() -> Self {
        Self {
            // P1: Left hand (same as SP)
            p1_scratch: "LeftShift".to_string(),
            p1_key1: "Z".to_string(),
            p1_key2: "S".to_string(),
            p1_key3: "X".to_string(),
            p1_key4: "D".to_string(),
            p1_key5: "C".to_string(),
            p1_key6: "F".to_string(),
            p1_key7: "V".to_string(),
            // P2: Right hand
            p2_key1: "M".to_string(),
            p2_key2: "K".to_string(),
            p2_key3: "Comma".to_string(),
            p2_key4: "L".to_string(),
            p2_key5: "Period".to_string(),
            p2_key6: "Semicolon".to_string(),
            p2_key7: "Slash".to_string(),
            p2_scratch: "RightShift".to_string(),
        }
    }
}

impl KeyBindingsDp {
    pub fn to_keycodes(&self) -> [KeyCode; 16] {
        [
            // P1
            string_to_keycode(&self.p1_scratch).unwrap_or(KeyCode::LeftShift),
            string_to_keycode(&self.p1_key1).unwrap_or(KeyCode::Z),
            string_to_keycode(&self.p1_key2).unwrap_or(KeyCode::S),
            string_to_keycode(&self.p1_key3).unwrap_or(KeyCode::X),
            string_to_keycode(&self.p1_key4).unwrap_or(KeyCode::D),
            string_to_keycode(&self.p1_key5).unwrap_or(KeyCode::C),
            string_to_keycode(&self.p1_key6).unwrap_or(KeyCode::F),
            string_to_keycode(&self.p1_key7).unwrap_or(KeyCode::V),
            // P2
            string_to_keycode(&self.p2_key1).unwrap_or(KeyCode::M),
            string_to_keycode(&self.p2_key2).unwrap_or(KeyCode::K),
            string_to_keycode(&self.p2_key3).unwrap_or(KeyCode::Comma),
            string_to_keycode(&self.p2_key4).unwrap_or(KeyCode::L),
            string_to_keycode(&self.p2_key5).unwrap_or(KeyCode::Period),
            string_to_keycode(&self.p2_key6).unwrap_or(KeyCode::Semicolon),
            string_to_keycode(&self.p2_key7).unwrap_or(KeyCode::Slash),
            string_to_keycode(&self.p2_scratch).unwrap_or(KeyCode::RightShift),
        ]
    }
}
```

### Phase 5: Game State Extension

```rust
// src/game/game_state.rs

// Change LANE_COUNT to dynamic
const LANE_COUNT_SP: usize = 8;
const LANE_COUNT_DP: usize = 16;

impl GameState {
    pub fn new_with_mode(mode: PlayMode) -> Self {
        let lane_count = mode.lane_count();
        Self {
            // ... initialize with dynamic lane count
            lane_index: vec![Vec::new(); lane_count].try_into().ok(),
            active_long_notes: vec![None; lane_count].try_into().ok(),
            hcn_damage_timers: vec![0.0; lane_count].try_into().ok(),
            // ...
        }
    }
}
```

### Phase 6: DP Random Options

```rust
// src/game/options.rs

/// Apply mirror for DP (mirrors each side independently)
pub fn apply_mirror_dp(chart: &mut Chart) {
    for note in &mut chart.notes {
        let new_channel = match note.channel {
            // P1 mirror
            NoteChannel::Scratch1 => NoteChannel::Scratch1,
            NoteChannel::Key1 => NoteChannel::Key7,
            NoteChannel::Key2 => NoteChannel::Key6,
            NoteChannel::Key3 => NoteChannel::Key5,
            NoteChannel::Key4 => NoteChannel::Key4,
            NoteChannel::Key5 => NoteChannel::Key3,
            NoteChannel::Key6 => NoteChannel::Key2,
            NoteChannel::Key7 => NoteChannel::Key1,
            // P2 mirror
            NoteChannel::Scratch2 => NoteChannel::Scratch2,
            NoteChannel::Key8 => NoteChannel::Key14,
            NoteChannel::Key9 => NoteChannel::Key13,
            NoteChannel::Key10 => NoteChannel::Key12,
            NoteChannel::Key11 => NoteChannel::Key11,
            NoteChannel::Key12 => NoteChannel::Key10,
            NoteChannel::Key13 => NoteChannel::Key9,
            NoteChannel::Key14 => NoteChannel::Key8,
        };
        note.channel = new_channel;
    }
}

/// Apply FLIP (swap P1 and P2 sides)
pub fn apply_flip(chart: &mut Chart) {
    for note in &mut chart.notes {
        let new_channel = match note.channel {
            // P1 -> P2
            NoteChannel::Scratch1 => NoteChannel::Scratch2,
            NoteChannel::Key1 => NoteChannel::Key8,
            NoteChannel::Key2 => NoteChannel::Key9,
            NoteChannel::Key3 => NoteChannel::Key10,
            NoteChannel::Key4 => NoteChannel::Key11,
            NoteChannel::Key5 => NoteChannel::Key12,
            NoteChannel::Key6 => NoteChannel::Key13,
            NoteChannel::Key7 => NoteChannel::Key14,
            // P2 -> P1
            NoteChannel::Scratch2 => NoteChannel::Scratch1,
            NoteChannel::Key8 => NoteChannel::Key1,
            NoteChannel::Key9 => NoteChannel::Key2,
            NoteChannel::Key10 => NoteChannel::Key3,
            NoteChannel::Key11 => NoteChannel::Key4,
            NoteChannel::Key12 => NoteChannel::Key5,
            NoteChannel::Key13 => NoteChannel::Key6,
            NoteChannel::Key14 => NoteChannel::Key7,
        };
        note.channel = new_channel;
    }
}
```

## Verification

1. DP 専用 BMS ファイル（P2 チャンネル使用）をロード
2. 16 レーンが正しく表示されることを確認
3. P1/P2 両側の入力が正しく動作することを確認
4. MIRROR/FLIP オプションが正しく動作することを確認
5. スコア・ゲージ計算が DP でも正しく動作することを確認

## Notes

- DP 譜面は SP 譜面より物量が多いため、パフォーマンス確認が重要
- P1/P2 で異なるランダムオプション（RANDOM/MIRROR 組み合わせ）は将来課題
- コントローラー対応は専コン 2 台接続を想定
- AUTO SCRATCH は P1/P2 両方に適用
