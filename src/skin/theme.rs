//! Runtime skin theme configuration
//!
//! Contains color and style settings extracted from skin definitions.

use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};

use crate::bms::PlayMode;

/// Color representation for serialization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SkinColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32,
}

fn default_alpha() -> f32 {
    1.0
}

impl From<SkinColor> for Color {
    fn from(c: SkinColor) -> Self {
        Color::new(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for SkinColor {
    fn from(c: Color) -> Self {
        SkinColor {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

/// Lane color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneColors {
    pub scratch: SkinColor,
    pub white_key: SkinColor,
    pub black_key: SkinColor,
    pub background: SkinColor,
    pub border: SkinColor,
}

impl Default for LaneColors {
    fn default() -> Self {
        Self {
            scratch: SkinColor {
                r: 1.0,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
            white_key: SkinColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            black_key: SkinColor {
                r: 0.3,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            background: SkinColor {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            border: SkinColor {
                r: 0.3,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
        }
    }
}

/// PMS (9-key) specific lane colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmsLaneColors {
    pub colors: [SkinColor; 9],
    pub background: SkinColor,
    pub border: SkinColor,
}

impl Default for PmsLaneColors {
    fn default() -> Self {
        Self {
            colors: [
                SkinColor {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }, // Key1 - White
                SkinColor {
                    r: 1.0,
                    g: 0.9,
                    b: 0.2,
                    a: 1.0,
                }, // Key2 - Yellow
                SkinColor {
                    r: 0.2,
                    g: 0.9,
                    b: 0.3,
                    a: 1.0,
                }, // Key3 - Green
                SkinColor {
                    r: 0.3,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                }, // Key4 - Blue
                SkinColor {
                    r: 1.0,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                }, // Key5 - Red (center)
                SkinColor {
                    r: 0.3,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                }, // Key6 - Blue
                SkinColor {
                    r: 0.2,
                    g: 0.9,
                    b: 0.3,
                    a: 1.0,
                }, // Key7 - Green
                SkinColor {
                    r: 1.0,
                    g: 0.9,
                    b: 0.2,
                    a: 1.0,
                }, // Key8 - Yellow
                SkinColor {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }, // Key9 - White
            ],
            background: SkinColor {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            border: SkinColor {
                r: 0.3,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
        }
    }
}

/// Note color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteColors {
    /// Long note body color
    pub long_note: SkinColor,
    /// Long note head/tail color
    pub long_note_edge: SkinColor,
    /// Invisible note color (for practice mode)
    pub invisible: SkinColor,
    /// Landmine note color
    pub landmine: SkinColor,
}

impl Default for NoteColors {
    fn default() -> Self {
        Self {
            long_note: SkinColor {
                r: 0.0,
                g: 0.8,
                b: 0.4,
                a: 0.7,
            },
            long_note_edge: SkinColor {
                r: 0.0,
                g: 1.0,
                b: 0.5,
                a: 1.0,
            },
            invisible: SkinColor {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 0.5,
            },
            landmine: SkinColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 0.8,
            },
        }
    }
}

/// Judge line configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeLineConfig {
    pub color: SkinColor,
    pub thickness: f32,
}

impl Default for JudgeLineConfig {
    fn default() -> Self {
        Self {
            color: SkinColor {
                r: 1.0,
                g: 0.8,
                b: 0.0,
                a: 1.0,
            },
            thickness: 3.0,
        }
    }
}

/// Lane cover colors (SUDDEN+, HIDDEN+, LIFT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneCoverColors {
    /// SUDDEN+/HIDDEN+ cover color (semi-transparent)
    pub cover: SkinColor,
    /// LIFT cover color (opaque)
    #[serde(default = "default_lift_cover")]
    pub lift_cover: SkinColor,
    pub text: SkinColor,
}

fn default_lift_cover() -> SkinColor {
    SkinColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }
}

impl Default for LaneCoverColors {
    fn default() -> Self {
        Self {
            cover: SkinColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.9,
            },
            lift_cover: default_lift_cover(),
            text: SkinColor {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
        }
    }
}

/// Complete skin theme containing all visual settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkinTheme {
    /// BMS 7-key lane colors
    #[serde(default)]
    pub bms_lane_colors: LaneColors,
    /// PMS 9-key lane colors
    #[serde(default)]
    pub pms_lane_colors: PmsLaneColors,
    /// Note colors
    #[serde(default)]
    pub note_colors: NoteColors,
    /// Judge line settings
    #[serde(default)]
    pub judge_line: JudgeLineConfig,
    /// Lane cover colors
    #[serde(default)]
    pub lane_cover: LaneCoverColors,
}

impl SkinTheme {
    /// Get background color for a play mode
    pub fn background_color(&self, mode: PlayMode) -> Color {
        match mode {
            PlayMode::Pms9Key => self.pms_lane_colors.background.into(),
            _ => self.bms_lane_colors.background.into(),
        }
    }

    /// Get border color for a play mode
    pub fn border_color(&self, mode: PlayMode) -> Color {
        match mode {
            PlayMode::Pms9Key => self.pms_lane_colors.border.into(),
            _ => self.bms_lane_colors.border.into(),
        }
    }

    /// Get lane color for BMS/DP modes
    /// lane_type: 0 = scratch, 1 = white key, 2 = black key
    pub fn bms_lane_color(&self, lane_type: u8) -> Color {
        match lane_type {
            0 => self.bms_lane_colors.scratch.into(),
            1 => self.bms_lane_colors.white_key.into(),
            _ => self.bms_lane_colors.black_key.into(),
        }
    }

    /// Get lane color for PMS 9-key mode
    pub fn pms_lane_color(&self, lane: usize) -> Color {
        if lane < 9 {
            self.pms_lane_colors.colors[lane].into()
        } else {
            Color::new(0.5, 0.5, 0.5, 1.0)
        }
    }

    /// Get long note body color
    pub fn long_note_color(&self) -> Color {
        self.note_colors.long_note.into()
    }

    /// Get long note edge color
    pub fn long_note_edge_color(&self) -> Color {
        self.note_colors.long_note_edge.into()
    }

    /// Get invisible note color
    pub fn invisible_note_color(&self) -> Color {
        self.note_colors.invisible.into()
    }

    /// Get landmine note color
    pub fn landmine_note_color(&self) -> Color {
        self.note_colors.landmine.into()
    }

    /// Get judge line color
    pub fn judge_line_color(&self) -> Color {
        self.judge_line.color.into()
    }

    /// Get judge line thickness
    pub fn judge_line_thickness(&self) -> f32 {
        self.judge_line.thickness
    }

    /// Get lane cover color
    pub fn lane_cover_color(&self) -> Color {
        self.lane_cover.cover.into()
    }

    /// Get LIFT cover color (opaque)
    pub fn lift_cover_color(&self) -> Color {
        self.lane_cover.lift_cover.into()
    }

    /// Get lane cover text color
    pub fn lane_cover_text_color(&self) -> Color {
        self.lane_cover.text.into()
    }
}

/// Judgment effect colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeColors {
    pub pgreat: SkinColor,
    pub great: SkinColor,
    pub good: SkinColor,
    pub bad: SkinColor,
    pub poor: SkinColor,
}

impl Default for JudgeColors {
    fn default() -> Self {
        Self {
            pgreat: SkinColor {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            great: SkinColor {
                r: 1.0,
                g: 0.8,
                b: 0.0,
                a: 1.0,
            },
            good: SkinColor {
                r: 0.0,
                g: 1.0,
                b: 0.5,
                a: 1.0,
            },
            bad: SkinColor {
                r: 0.5,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            poor: SkinColor {
                r: 1.0,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
        }
    }
}

/// Combo milestone colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboColors {
    /// Default combo color (< 100)
    pub default: SkinColor,
    /// Combo >= 100
    pub milestone_100: SkinColor,
    /// Combo >= 500
    pub milestone_500: SkinColor,
    /// Combo >= 1000
    pub milestone_1000: SkinColor,
}

impl Default for ComboColors {
    fn default() -> Self {
        Self {
            default: SkinColor {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            milestone_100: SkinColor {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            milestone_500: SkinColor {
                r: 1.0,
                g: 0.5,
                b: 0.0,
                a: 1.0,
            },
            milestone_1000: SkinColor {
                r: 1.0,
                g: 0.8,
                b: 0.0,
                a: 1.0,
            },
        }
    }
}

/// Key beam configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBeamConfig {
    /// Maximum alpha at the bottom (near judge line)
    pub max_alpha: f32,
    /// Minimum alpha at the top
    pub min_alpha: f32,
    /// Height ratio of the beam relative to judge line position
    pub height_ratio: f32,
    /// Enable/disable key beams
    pub enabled: bool,
}

impl Default for KeyBeamConfig {
    fn default() -> Self {
        Self {
            max_alpha: 0.6,
            min_alpha: 0.1,
            height_ratio: 0.5,
            enabled: true,
        }
    }
}

/// Effect configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectConfig {
    /// Judgment display colors
    pub judge_colors: JudgeColors,
    /// Combo milestone colors
    pub combo_colors: ComboColors,
    /// Judge effect display duration (seconds)
    pub judge_duration: f32,
    /// Combo effect display duration (seconds)
    pub combo_duration: f32,
    /// Lane flash duration (seconds)
    pub lane_flash_duration: f32,
    /// Lane flash max alpha
    pub lane_flash_alpha: f32,
    /// Judge text font size
    pub judge_font_size: f32,
    /// Combo text font size
    pub combo_font_size: f32,
    /// Key beam configuration
    #[serde(default)]
    pub key_beam: KeyBeamConfig,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            judge_colors: JudgeColors::default(),
            combo_colors: ComboColors::default(),
            judge_duration: 0.3,
            combo_duration: 0.15,
            lane_flash_duration: 0.1,
            lane_flash_alpha: 0.5,
            judge_font_size: 40.0,
            combo_font_size: 36.0,
            key_beam: KeyBeamConfig::default(),
        }
    }
}

impl EffectConfig {
    /// Get color for judge result
    pub fn judge_color(&self, result: crate::game::JudgeResult) -> Color {
        use crate::game::JudgeResult;
        match result {
            JudgeResult::PGreat => self.judge_colors.pgreat.into(),
            JudgeResult::Great => self.judge_colors.great.into(),
            JudgeResult::Good => self.judge_colors.good.into(),
            JudgeResult::Bad => self.judge_colors.bad.into(),
            JudgeResult::Poor => self.judge_colors.poor.into(),
        }
    }

    /// Get color for combo value
    pub fn combo_color(&self, combo: u32) -> Color {
        if combo >= 1000 {
            self.combo_colors.milestone_1000.into()
        } else if combo >= 500 {
            self.combo_colors.milestone_500.into()
        } else if combo >= 100 {
            self.combo_colors.milestone_100.into()
        } else {
            self.combo_colors.default.into()
        }
    }
}
