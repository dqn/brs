# Custom Skin System

カスタムスキンシステムの実装。UI 要素（レーン、ノーツ、ゲージ等）のカスタマイズ機能。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 低 |
| 難易度 | 非常に高 |
| 推定工数 | 7-10日 |
| 依存関係 | なし |

## Background

カスタムスキンは beatoraja や LR2 で広く使われている機能。プレイヤーが自分好みの見た目にカスタマイズできる。

### Skinnable Elements

- 背景画像
- レーン（位置、幅、色、画像）
- ノーツ（形状、色、画像）
- ジャッジライン
- ゲージ（位置、形状、色）
- コンボ表示
- 判定表示（PGREAT, GREAT, etc.）
- FAST/SLOW 表示
- BGA 表示領域

## Dependencies

- なし（core gameplay が安定してから実装推奨）

## Files to Modify/Create

| ファイル | 変更内容 |
|----------|----------|
| `src/skin/mod.rs` (新規) | Skin モジュールルート |
| `src/skin/definition.rs` (新規) | スキン定義構造体 |
| `src/skin/loader.rs` (新規) | スキンファイル読み込み |
| `src/skin/renderer.rs` (新規) | スキンベース描画 |
| `src/skin/elements.rs` (新規) | UI 要素定義 |
| `src/render/highway.rs` | スキン対応リファクタ |
| `src/render/effects.rs` | スキン対応リファクタ |
| `src/config/settings.rs` | スキン設定追加 |

## Implementation Phases

### Phase 1: Skin Definition Structure

```rust
// src/skin/definition.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SkinDefinition {
    /// Skin metadata
    pub info: SkinInfo,
    /// Target resolution
    pub resolution: Resolution,
    /// UI elements
    pub elements: SkinElements,
}

#[derive(Debug, Deserialize)]
pub struct SkinInfo {
    pub name: String,
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct SkinElements {
    /// Background
    pub background: BackgroundElement,
    /// Play area (highway)
    pub play_area: PlayAreaElement,
    /// Lane definitions
    pub lanes: Vec<LaneElement>,
    /// Note styles
    pub notes: NoteElements,
    /// Judge line
    pub judge_line: JudgeLineElement,
    /// Gauge display
    pub gauge: GaugeElement,
    /// Combo display
    pub combo: TextElement,
    /// Judgment display
    pub judgment: JudgmentElements,
    /// BGA area
    pub bga: BgaElement,
    /// FAST/SLOW display
    pub timing: TimingElement,
}
```

### Phase 2: Element Definitions

```rust
// src/skin/elements.rs

#[derive(Debug, Deserialize)]
pub struct BackgroundElement {
    pub image: Option<String>,
    pub color: Option<[f32; 4]>,
}

#[derive(Debug, Deserialize)]
pub struct PlayAreaElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize)]
pub struct LaneElement {
    pub x: f32,
    pub width: f32,
    pub background_color: Option<[f32; 4]>,
    pub background_image: Option<String>,
    pub flash_color: Option<[f32; 4]>,
}

#[derive(Debug, Deserialize)]
pub struct NoteElements {
    pub scratch: NoteStyle,
    pub white_key: NoteStyle,
    pub black_key: NoteStyle,
    pub long_bar: LongNoteStyle,
}

#[derive(Debug, Deserialize)]
pub struct NoteStyle {
    pub width: f32,
    pub height: f32,
    pub color: Option<[f32; 4]>,
    pub image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LongNoteStyle {
    pub start: NoteStyle,
    pub end: NoteStyle,
    pub bar_color: Option<[f32; 4]>,
    pub bar_image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JudgeLineElement {
    pub y: f32,
    pub thickness: f32,
    pub color: [f32; 4],
    pub image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GaugeElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub direction: GaugeDirection,
    pub background_color: [f32; 4],
    pub fill_colors: GaugeFillColors,
}

#[derive(Debug, Deserialize)]
pub enum GaugeDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Deserialize)]
pub struct GaugeFillColors {
    pub groove_low: [f32; 4],
    pub groove_high: [f32; 4],
    pub survival_normal: [f32; 4],
    pub survival_danger: [f32; 4],
}

#[derive(Debug, Deserialize)]
pub struct TextElement {
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub color: [f32; 4],
    pub alignment: TextAlignment,
}

#[derive(Debug, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Deserialize)]
pub struct JudgmentElements {
    pub x: f32,
    pub y: f32,
    pub pgreat: JudgmentStyle,
    pub great: JudgmentStyle,
    pub good: JudgmentStyle,
    pub bad: JudgmentStyle,
    pub poor: JudgmentStyle,
}

#[derive(Debug, Deserialize)]
pub struct JudgmentStyle {
    pub image: Option<String>,
    pub color: Option<[f32; 4]>,
    pub text: Option<String>,
    pub font_size: f32,
}

#[derive(Debug, Deserialize)]
pub struct BgaElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize)]
pub struct TimingElement {
    pub x: f32,
    pub y: f32,
    pub fast_color: [f32; 4],
    pub slow_color: [f32; 4],
    pub font_size: f32,
}
```

### Phase 3: Skin JSON Format

```json
{
  "info": {
    "name": "Default Skin",
    "author": "brs",
    "version": "1.0.0",
    "description": "Default skin for brs"
  },
  "resolution": {
    "width": 1920,
    "height": 1080
  },
  "elements": {
    "background": {
      "color": [0.05, 0.05, 0.1, 1.0]
    },
    "play_area": {
      "x": 560,
      "y": 0,
      "width": 400,
      "height": 1080
    },
    "lanes": [
      { "x": 0, "width": 60, "background_color": [0.3, 0.1, 0.1, 1.0] },
      { "x": 60, "width": 45, "background_color": [0.15, 0.15, 0.15, 1.0] },
      { "x": 105, "width": 45, "background_color": [0.1, 0.1, 0.2, 1.0] },
      { "x": 150, "width": 45, "background_color": [0.15, 0.15, 0.15, 1.0] },
      { "x": 195, "width": 45, "background_color": [0.1, 0.1, 0.2, 1.0] },
      { "x": 240, "width": 45, "background_color": [0.15, 0.15, 0.15, 1.0] },
      { "x": 285, "width": 45, "background_color": [0.1, 0.1, 0.2, 1.0] },
      { "x": 330, "width": 45, "background_color": [0.15, 0.15, 0.15, 1.0] }
    ],
    "notes": {
      "scratch": {
        "width": 56,
        "height": 12,
        "color": [1.0, 0.3, 0.3, 1.0]
      },
      "white_key": {
        "width": 41,
        "height": 10,
        "color": [1.0, 1.0, 1.0, 1.0]
      },
      "black_key": {
        "width": 41,
        "height": 10,
        "color": [0.3, 0.5, 1.0, 1.0]
      },
      "long_bar": {
        "start": { "width": 41, "height": 10, "color": [1.0, 1.0, 0.0, 1.0] },
        "end": { "width": 41, "height": 10, "color": [1.0, 1.0, 0.0, 1.0] },
        "bar_color": [1.0, 1.0, 0.0, 0.5]
      }
    },
    "judge_line": {
      "y": 900,
      "thickness": 3.0,
      "color": [1.0, 0.8, 0.0, 1.0]
    },
    "gauge": {
      "x": 1000,
      "y": 50,
      "width": 300,
      "height": 20,
      "direction": "Horizontal",
      "background_color": [0.2, 0.2, 0.2, 1.0],
      "fill_colors": {
        "groove_low": [0.2, 0.6, 1.0, 1.0],
        "groove_high": [0.0, 1.0, 0.5, 1.0],
        "survival_normal": [1.0, 0.2, 0.2, 1.0],
        "survival_danger": [1.0, 0.5, 0.0, 1.0]
      }
    },
    "combo": {
      "x": 760,
      "y": 500,
      "font_size": 48.0,
      "color": [1.0, 1.0, 1.0, 1.0],
      "alignment": "Center"
    },
    "judgment": {
      "x": 760,
      "y": 450,
      "pgreat": { "text": "PGREAT", "color": [0.0, 1.0, 1.0, 1.0], "font_size": 32.0 },
      "great": { "text": "GREAT", "color": [1.0, 1.0, 0.0, 1.0], "font_size": 32.0 },
      "good": { "text": "GOOD", "color": [0.0, 1.0, 0.0, 1.0], "font_size": 32.0 },
      "bad": { "text": "BAD", "color": [0.5, 0.0, 1.0, 1.0], "font_size": 32.0 },
      "poor": { "text": "POOR", "color": [1.0, 0.0, 0.0, 1.0], "font_size": 32.0 }
    },
    "bga": {
      "x": 50,
      "y": 100,
      "width": 400,
      "height": 300
    },
    "timing": {
      "x": 760,
      "y": 520,
      "fast_color": [0.0, 0.8, 1.0, 1.0],
      "slow_color": [1.0, 0.5, 0.0, 1.0],
      "font_size": 24.0
    }
  }
}
```

### Phase 4: Skin Loader

```rust
// src/skin/loader.rs

use std::collections::HashMap;
use macroquad::prelude::*;

pub struct LoadedSkin {
    pub definition: SkinDefinition,
    pub textures: HashMap<String, Texture2D>,
}

impl LoadedSkin {
    pub async fn load<P: AsRef<Path>>(skin_dir: P) -> Result<Self> {
        let skin_dir = skin_dir.as_ref();
        let json_path = skin_dir.join("skin.json");

        let json_content = std::fs::read_to_string(&json_path)?;
        let definition: SkinDefinition = serde_json::from_str(&json_content)?;

        let mut textures = HashMap::new();

        // Collect all image paths from definition
        let image_paths = definition.collect_image_paths();

        for image_path in image_paths {
            let full_path = skin_dir.join(&image_path);
            if full_path.exists() {
                let texture = load_texture(full_path.to_str().unwrap()).await?;
                textures.insert(image_path, texture);
            }
        }

        Ok(Self {
            definition,
            textures,
        })
    }

    pub fn get_texture(&self, path: &str) -> Option<&Texture2D> {
        self.textures.get(path)
    }
}
```

### Phase 5: Skin Renderer

```rust
// src/skin/renderer.rs

pub struct SkinRenderer {
    skin: LoadedSkin,
    scale_x: f32,
    scale_y: f32,
}

impl SkinRenderer {
    pub fn new(skin: LoadedSkin) -> Self {
        let scale_x = screen_width() / skin.definition.resolution.width as f32;
        let scale_y = screen_height() / skin.definition.resolution.height as f32;

        Self { skin, scale_x, scale_y }
    }

    fn scale_x(&self, x: f32) -> f32 {
        x * self.scale_x
    }

    fn scale_y(&self, y: f32) -> f32 {
        y * self.scale_y
    }

    pub fn draw_background(&self) {
        let bg = &self.skin.definition.elements.background;

        if let Some(ref image_path) = bg.image {
            if let Some(texture) = self.skin.get_texture(image_path) {
                draw_texture_ex(
                    texture,
                    0.0, 0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(screen_width(), screen_height())),
                        ..Default::default()
                    },
                );
            }
        } else if let Some(color) = bg.color {
            clear_background(Color::new(color[0], color[1], color[2], color[3]));
        }
    }

    pub fn draw_lane(&self, lane_index: usize) {
        let lane = &self.skin.definition.elements.lanes[lane_index];
        let play_area = &self.skin.definition.elements.play_area;

        let x = self.scale_x(play_area.x + lane.x);
        let y = self.scale_y(play_area.y);
        let width = self.scale_x(lane.width);
        let height = self.scale_y(play_area.height);

        if let Some(ref image_path) = lane.background_image {
            if let Some(texture) = self.skin.get_texture(image_path) {
                draw_texture_ex(texture, x, y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    ..Default::default()
                });
            }
        } else if let Some(color) = lane.background_color {
            draw_rectangle(x, y, width, height,
                Color::new(color[0], color[1], color[2], color[3]));
        }
    }

    pub fn draw_note(&self, lane_index: usize, y: f32, note_type: NoteType) {
        let style = self.get_note_style(lane_index, note_type);
        let lane = &self.skin.definition.elements.lanes[lane_index];
        let play_area = &self.skin.definition.elements.play_area;

        let x = self.scale_x(play_area.x + lane.x + (lane.width - style.width) / 2.0);
        let scaled_y = self.scale_y(y);
        let width = self.scale_x(style.width);
        let height = self.scale_y(style.height);

        if let Some(ref image_path) = style.image {
            if let Some(texture) = self.skin.get_texture(image_path) {
                draw_texture_ex(texture, x, scaled_y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    ..Default::default()
                });
            }
        } else if let Some(color) = style.color {
            draw_rectangle(x, scaled_y, width, height,
                Color::new(color[0], color[1], color[2], color[3]));
        }
    }

    fn get_note_style(&self, lane_index: usize, note_type: NoteType) -> &NoteStyle {
        let notes = &self.skin.definition.elements.notes;

        if lane_index == 0 {
            &notes.scratch
        } else if lane_index % 2 == 1 {
            &notes.white_key
        } else {
            &notes.black_key
        }
    }

    // ... more drawing methods for gauge, combo, judgment, etc.
}
```

## Skin Directory Structure

```
skins/
├── default/
│   ├── skin.json
│   ├── background.png
│   ├── notes/
│   │   ├── scratch.png
│   │   ├── white.png
│   │   └── black.png
│   └── judge/
│       ├── pgreat.png
│       ├── great.png
│       └── ...
└── custom_skin/
    └── ...
```

## Verification

1. デフォルトスキンを JSON で定義
2. スキンなしと同等の表示になることを確認
3. カスタムスキンを作成してテスト
4. 異なる解像度でのスケーリングを確認
5. パフォーマンス影響を計測

## Notes

- 初期実装では画像よりも色定義を優先
- 解像度スケーリングはアスペクト比を維持
- テクスチャキャッシュでメモリ効率化
- LR2/beatoraja スキンとの互換性は将来課題
