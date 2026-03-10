use super::deserializers::deserialize_optional_string_from_int;
use super::visual_objects::default_one;
use serde::{Deserialize, Serialize};

/// Corresponds to JsonSkin.Slider
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Slider {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub src: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default = "default_one")]
    pub divx: i32,
    #[serde(default = "default_one")]
    pub divy: i32,
    pub timer: Option<i32>,
    pub cycle: i32,
    pub angle: i32,
    pub range: i32,
    #[serde(rename = "type")]
    pub slider_type: i32,
    #[serde(default = "default_true")]
    pub changeable: bool,
    pub value: Option<i32>,
    pub event: Option<i32>,
    #[serde(rename = "isRefNum")]
    pub is_ref_num: bool,
    pub min: i32,
    pub max: i32,
}

fn default_true() -> bool {
    true
}

/// Corresponds to JsonSkin.Graph
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Graph {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub src: Option<String>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default = "default_one")]
    pub divx: i32,
    #[serde(default = "default_one")]
    pub divy: i32,
    pub timer: Option<i32>,
    pub cycle: i32,
    #[serde(default = "default_one")]
    pub angle: i32,
    #[serde(rename = "type")]
    pub graph_type: i32,
    pub value: Option<i32>,
    #[serde(rename = "isRefNum")]
    pub is_ref_num: bool,
    pub min: i32,
    pub max: i32,
}

/// Corresponds to JsonSkin.GaugeGraph
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GaugeGraph {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub color: Option<Vec<String>>,
    #[serde(rename = "assistClearBGColor", default = "default_assist_clear_bg")]
    pub assist_clear_bg_color: String,
    #[serde(
        rename = "assistAndEasyFailBGColor",
        default = "default_assist_easy_fail_bg"
    )]
    pub assist_and_easy_fail_bg_color: String,
    #[serde(rename = "grooveFailBGColor", default = "default_groove_fail_bg")]
    pub groove_fail_bg_color: String,
    #[serde(
        rename = "grooveClearAndHardBGColor",
        default = "default_groove_clear_hard_bg"
    )]
    pub groove_clear_and_hard_bg_color: String,
    #[serde(rename = "exHardBGColor", default = "default_exhard_bg")]
    pub ex_hard_bg_color: String,
    #[serde(rename = "hazardBGColor", default = "default_hazard_bg")]
    pub hazard_bg_color: String,
    #[serde(rename = "assistClearLineColor", default = "default_assist_clear_line")]
    pub assist_clear_line_color: String,
    #[serde(
        rename = "assistAndEasyFailLineColor",
        default = "default_assist_easy_fail_line"
    )]
    pub assist_and_easy_fail_line_color: String,
    #[serde(rename = "grooveFailLineColor", default = "default_groove_fail_line")]
    pub groove_fail_line_color: String,
    #[serde(
        rename = "grooveClearAndHardLineColor",
        default = "default_groove_clear_hard_line"
    )]
    pub groove_clear_and_hard_line_color: String,
    #[serde(rename = "exHardLineColor", default = "default_exhard_line")]
    pub ex_hard_line_color: String,
    #[serde(rename = "hazardLineColor", default = "default_hazard_line")]
    pub hazard_line_color: String,
    #[serde(rename = "borderlineColor", default = "default_borderline")]
    pub borderline_color: String,
    #[serde(rename = "borderColor", default = "default_border")]
    pub border_color: String,
}

fn default_assist_clear_bg() -> String {
    "440044".to_string()
}
fn default_assist_easy_fail_bg() -> String {
    "004444".to_string()
}
fn default_groove_fail_bg() -> String {
    "004400".to_string()
}
fn default_groove_clear_hard_bg() -> String {
    "440000".to_string()
}
fn default_exhard_bg() -> String {
    "444400".to_string()
}
fn default_hazard_bg() -> String {
    "444444".to_string()
}
fn default_assist_clear_line() -> String {
    "ff00ff".to_string()
}
fn default_assist_easy_fail_line() -> String {
    "00ffff".to_string()
}
fn default_groove_fail_line() -> String {
    "00ff00".to_string()
}
fn default_groove_clear_hard_line() -> String {
    "ff0000".to_string()
}
fn default_exhard_line() -> String {
    "ffff00".to_string()
}
fn default_hazard_line() -> String {
    "cccccc".to_string()
}
fn default_borderline() -> String {
    "ff0000".to_string()
}
fn default_border() -> String {
    "440000".to_string()
}

/// Corresponds to JsonSkin.JudgeGraph
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct JudgeGraph {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub graph_type: i32,
    #[serde(rename = "backTexOff")]
    pub back_tex_off: i32,
    #[serde(default = "default_500")]
    pub delay: i32,
    #[serde(rename = "orderReverse")]
    pub order_reverse: i32,
    #[serde(rename = "noGap")]
    pub no_gap: i32,
    #[serde(rename = "noGapX")]
    pub no_gap_x: i32,
}

pub(super) fn default_500() -> i32 {
    500
}

/// Corresponds to JsonSkin.BPMGraph
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BPMGraph {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub delay: i32,
    #[serde(rename = "lineWidth", default = "default_two")]
    pub line_width: i32,
    #[serde(rename = "mainBPMColor", default = "default_main_bpm_color")]
    pub main_bpm_color: String,
    #[serde(rename = "minBPMColor", default = "default_min_bpm_color")]
    pub min_bpm_color: String,
    #[serde(rename = "maxBPMColor", default = "default_max_bpm_color")]
    pub max_bpm_color: String,
    #[serde(rename = "otherBPMColor", default = "default_other_bpm_color")]
    pub other_bpm_color: String,
    #[serde(rename = "stopLineColor", default = "default_stop_line_color")]
    pub stop_line_color: String,
    #[serde(
        rename = "transitionLineColor",
        default = "default_transition_line_color"
    )]
    pub transition_line_color: String,
}

fn default_two() -> i32 {
    2
}
fn default_main_bpm_color() -> String {
    "00ff00".to_string()
}
fn default_min_bpm_color() -> String {
    "0000ff".to_string()
}
fn default_max_bpm_color() -> String {
    "ff0000".to_string()
}
fn default_other_bpm_color() -> String {
    "ffff00".to_string()
}
fn default_stop_line_color() -> String {
    "ff00ff".to_string()
}
fn default_transition_line_color() -> String {
    "7f7f7f".to_string()
}

/// Corresponds to JsonSkin.HitErrorVisualizer
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HitErrorVisualizer {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub width: i32,
    #[serde(rename = "judgeWidthMillis")]
    pub judge_width_millis: i32,
    #[serde(rename = "lineWidth")]
    pub line_width: i32,
    #[serde(rename = "colorMode")]
    pub color_mode: i32,
    #[serde(rename = "hiterrorMode")]
    pub hiterror_mode: i32,
    #[serde(rename = "emaMode")]
    pub ema_mode: i32,
    #[serde(rename = "lineColor")]
    pub line_color: String,
    #[serde(rename = "centerColor")]
    pub center_color: String,
    #[serde(rename = "PGColor")]
    pub pg_color: String,
    #[serde(rename = "GRColor")]
    pub gr_color: String,
    #[serde(rename = "GDColor")]
    pub gd_color: String,
    #[serde(rename = "BDColor")]
    pub bd_color: String,
    #[serde(rename = "PRColor")]
    pub pr_color: String,
    #[serde(rename = "emaColor")]
    pub ema_color: String,
    pub alpha: f32,
    #[serde(rename = "windowLength")]
    pub window_length: i32,
    pub transparent: i32,
    #[serde(rename = "drawDecay")]
    pub draw_decay: i32,
}

impl Default for HitErrorVisualizer {
    fn default() -> Self {
        Self {
            id: None,
            width: 301,
            judge_width_millis: 150,
            line_width: 1,
            color_mode: 1,
            hiterror_mode: 1,
            ema_mode: 1,
            line_color: "99CCFF80".to_string(),
            center_color: "FFFFFFFF".to_string(),
            pg_color: "99CCFF80".to_string(),
            gr_color: "F2CB3080".to_string(),
            gd_color: "14CC8f80".to_string(),
            bd_color: "FF1AB380".to_string(),
            pr_color: "CC292980".to_string(),
            ema_color: "FF0000FF".to_string(),
            alpha: 0.1,
            window_length: 30,
            transparent: 0,
            draw_decay: 1,
        }
    }
}

/// Corresponds to JsonSkin.TimingVisualizer
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TimingVisualizer {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub width: i32,
    #[serde(rename = "judgeWidthMillis")]
    pub judge_width_millis: i32,
    #[serde(rename = "lineWidth")]
    pub line_width: i32,
    #[serde(rename = "lineColor")]
    pub line_color: String,
    #[serde(rename = "centerColor")]
    pub center_color: String,
    #[serde(rename = "PGColor")]
    pub pg_color: String,
    #[serde(rename = "GRColor")]
    pub gr_color: String,
    #[serde(rename = "GDColor")]
    pub gd_color: String,
    #[serde(rename = "BDColor")]
    pub bd_color: String,
    #[serde(rename = "PRColor")]
    pub pr_color: String,
    pub transparent: i32,
    #[serde(rename = "drawDecay")]
    pub draw_decay: i32,
}

impl Default for TimingVisualizer {
    fn default() -> Self {
        Self {
            id: None,
            width: 301,
            judge_width_millis: 150,
            line_width: 1,
            line_color: "00FF00FF".to_string(),
            center_color: "FFFFFFFF".to_string(),
            pg_color: "000088FF".to_string(),
            gr_color: "008800FF".to_string(),
            gd_color: "888800FF".to_string(),
            bd_color: "880000FF".to_string(),
            pr_color: "000000FF".to_string(),
            transparent: 0,
            draw_decay: 1,
        }
    }
}

/// Corresponds to JsonSkin.TimingDistributionGraph
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TimingDistributionGraph {
    #[serde(deserialize_with = "deserialize_optional_string_from_int", default)]
    pub id: Option<String>,
    pub width: i32,
    #[serde(rename = "lineWidth")]
    pub line_width: i32,
    #[serde(rename = "graphColor")]
    pub graph_color: String,
    #[serde(rename = "averageColor")]
    pub average_color: String,
    #[serde(rename = "devColor")]
    pub dev_color: String,
    #[serde(rename = "PGColor")]
    pub pg_color: String,
    #[serde(rename = "GRColor")]
    pub gr_color: String,
    #[serde(rename = "GDColor")]
    pub gd_color: String,
    #[serde(rename = "BDColor")]
    pub bd_color: String,
    #[serde(rename = "PRColor")]
    pub pr_color: String,
    #[serde(rename = "drawAverage")]
    pub draw_average: i32,
    #[serde(rename = "drawDev")]
    pub draw_dev: i32,
}

impl Default for TimingDistributionGraph {
    fn default() -> Self {
        Self {
            id: None,
            width: 301,
            line_width: 1,
            graph_color: "00FF00FF".to_string(),
            average_color: "FFFFFFFF".to_string(),
            dev_color: "FFFFFFFF".to_string(),
            pg_color: "000088FF".to_string(),
            gr_color: "008800FF".to_string(),
            gd_color: "888800FF".to_string(),
            bd_color: "880000FF".to_string(),
            pr_color: "000000FF".to_string(),
            draw_average: 1,
            draw_dev: 1,
        }
    }
}
