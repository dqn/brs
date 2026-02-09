// JSON skin deserialization structures.
//
// Matches Java's JsonSkin.java data classes exactly.
// All field names preserve the original JSON format for compatibility.
//
// Property fields (timer, value, draw, act, event) use PropertyRef which
// can deserialize as either a numeric ID or a Lua script string.
// ID-like fields (id, src, font) use FlexId which accepts string or integer.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ---------------------------------------------------------------------------
// FlexId — a string or integer identifier
// ---------------------------------------------------------------------------

/// A value that can appear as either a string or integer in JSON.
/// Always stored and compared as a String.
///
/// Used for id/src/font reference fields that Java's libgdx Json library
/// auto-converts to String.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FlexId(pub String);

impl FlexId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FlexId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<i32> for FlexId {
    fn from(n: i32) -> Self {
        Self(n.to_string())
    }
}

impl Serialize for FlexId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for FlexId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FlexIdVisitor;
        impl<'de> Visitor<'de> for FlexIdVisitor {
            type Value = FlexId;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or integer")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(FlexId(v.to_string()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(FlexId(v))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(FlexId(v.to_string()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(FlexId(v.to_string()))
            }
        }
        deserializer.deserialize_any(FlexIdVisitor)
    }
}

// ---------------------------------------------------------------------------
// PropertyRef — property ID or Lua script reference
// ---------------------------------------------------------------------------

/// A property reference that deserializes as either a numeric ID or a
/// Lua script string.
///
/// In beatoraja JSON skins, property fields can be:
/// - A number: resolved via PropertyFactory to a built-in property
/// - A string: passed to SkinLuaAccessor for script evaluation
#[derive(Debug, Clone)]
pub enum PropertyRef {
    /// Numeric property ID.
    Id(i32),
    /// Lua script name/expression.
    Script(String),
}

impl Default for PropertyRef {
    fn default() -> Self {
        Self::Id(0)
    }
}

impl PropertyRef {
    /// Returns the numeric ID if this is an Id variant.
    pub fn as_id(&self) -> Option<i32> {
        match self {
            Self::Id(id) => Some(*id),
            Self::Script(_) => None,
        }
    }
}

impl Serialize for PropertyRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Id(id) => serializer.serialize_i32(*id),
            Self::Script(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for PropertyRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PropertyRefVisitor;
        impl<'de> Visitor<'de> for PropertyRefVisitor {
            type Value = PropertyRef;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an integer or string")
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(PropertyRef::Id(v as i32))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(PropertyRef::Id(v as i32))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(PropertyRef::Script(v.to_string()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(PropertyRef::Script(v))
            }
        }
        deserializer.deserialize_any(PropertyRefVisitor)
    }
}

// ---------------------------------------------------------------------------
// Root Skin structure
// ---------------------------------------------------------------------------

/// The root JSON skin structure.
///
/// Matches Java's `JsonSkin.Skin` class exactly.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonSkinData {
    /// Skin type ID (-1 = unset).
    #[serde(rename = "type")]
    pub skin_type: i32,
    pub name: Option<String>,
    pub author: Option<String>,
    /// Source resolution width.
    pub w: i32,
    /// Source resolution height.
    pub h: i32,
    /// Fade-out duration (ms).
    pub fadeout: i32,
    /// Input delay (ms).
    pub input: i32,
    /// Scene duration (ms).
    pub scene: i32,
    /// Close timing (ms).
    pub close: i32,
    /// Load-end timing (ms).
    pub loadend: i32,
    /// Play start timing (ms).
    pub playstart: i32,
    /// Judge timer mode (default 1).
    pub judgetimer: i32,
    /// Finish margin (ms).
    pub finishmargin: i32,

    // Customization
    pub category: Vec<JsonCategory>,
    pub property: Vec<JsonProperty>,
    pub filepath: Vec<JsonFilepath>,
    pub offset: Vec<JsonOffset>,

    // Assets
    pub source: Vec<JsonSource>,
    pub font: Vec<JsonFont>,

    // Objects
    pub image: Vec<JsonImage>,
    pub imageset: Vec<JsonImageSet>,
    pub value: Vec<JsonValue>,
    pub floatvalue: Vec<JsonFloatValue>,
    pub text: Vec<JsonText>,
    pub slider: Vec<JsonSlider>,
    pub graph: Vec<JsonGraph>,
    pub gaugegraph: Vec<JsonGaugeGraph>,
    pub judgegraph: Vec<JsonJudgeGraph>,
    pub bpmgraph: Vec<JsonBpmGraph>,
    pub hiterrorvisualizer: Vec<JsonHitErrorVisualizer>,
    pub timingvisualizer: Vec<JsonTimingVisualizer>,
    pub timingdistributiongraph: Vec<JsonTimingDistributionGraph>,

    // Skin-type specific
    pub note: Option<JsonNoteSet>,
    pub gauge: Option<JsonGauge>,
    #[serde(rename = "hiddenCover")]
    pub hidden_cover: Vec<JsonHiddenCover>,
    #[serde(rename = "liftCover")]
    pub lift_cover: Vec<JsonLiftCover>,
    pub bga: Option<JsonBga>,
    pub judge: Vec<JsonJudge>,
    pub songlist: Option<JsonSongList>,
    pub pmchara: Vec<JsonPmChara>,
    #[serde(rename = "skinSelect")]
    pub skin_select: Option<JsonSkinConfigProperty>,
    #[serde(rename = "customEvents")]
    pub custom_events: Vec<JsonCustomEvent>,
    #[serde(rename = "customTimers")]
    pub custom_timers: Vec<JsonCustomTimer>,

    // Rendering
    pub destination: Vec<JsonDestination>,
}

impl Default for JsonSkinData {
    fn default() -> Self {
        Self {
            skin_type: -1,
            name: None,
            author: None,
            w: 1280,
            h: 720,
            fadeout: 0,
            input: 0,
            scene: 0,
            close: 0,
            loadend: 0,
            playstart: 0,
            judgetimer: 1,
            finishmargin: 0,
            category: Vec::new(),
            property: Vec::new(),
            filepath: Vec::new(),
            offset: Vec::new(),
            source: Vec::new(),
            font: Vec::new(),
            image: Vec::new(),
            imageset: Vec::new(),
            value: Vec::new(),
            floatvalue: Vec::new(),
            text: Vec::new(),
            slider: Vec::new(),
            graph: Vec::new(),
            gaugegraph: Vec::new(),
            judgegraph: Vec::new(),
            bpmgraph: Vec::new(),
            hiterrorvisualizer: Vec::new(),
            timingvisualizer: Vec::new(),
            timingdistributiongraph: Vec::new(),
            note: None,
            gauge: None,
            hidden_cover: Vec::new(),
            lift_cover: Vec::new(),
            bga: None,
            judge: Vec::new(),
            songlist: None,
            pmchara: Vec::new(),
            skin_select: None,
            custom_events: Vec::new(),
            custom_timers: Vec::new(),
            destination: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Customization types
// ---------------------------------------------------------------------------

/// Custom option property (user-selectable).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonProperty {
    pub category: Option<String>,
    pub name: Option<String>,
    pub item: Vec<JsonPropertyItem>,
    pub def: Option<String>,
}

/// A single item within a custom property option.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonPropertyItem {
    pub name: Option<String>,
    pub op: i32,
}

/// Custom file path.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonFilepath {
    pub category: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub def: Option<String>,
}

/// Custom offset definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonOffset {
    pub category: Option<String>,
    pub name: Option<String>,
    pub id: i32,
    pub x: bool,
    pub y: bool,
    pub w: bool,
    pub h: bool,
    pub r: bool,
    pub a: bool,
}

/// Category grouping for custom items.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonCategory {
    pub name: Option<String>,
    pub item: Vec<String>,
}

// ---------------------------------------------------------------------------
// Asset types
// ---------------------------------------------------------------------------

/// Image/movie source file reference.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonSource {
    pub id: FlexId,
    pub path: Option<String>,
}

/// Font definition (TrueType or bitmap).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonFont {
    pub id: FlexId,
    pub path: Option<String>,
    /// 0 = TrueType, 1 = bitmap (.fnt).
    #[serde(rename = "type")]
    pub font_type: i32,
}

// ---------------------------------------------------------------------------
// Image objects
// ---------------------------------------------------------------------------

/// Single image definition with optional animation grid.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonImage {
    pub id: FlexId,
    /// Source ID reference.
    pub src: FlexId,
    /// Crop region.
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// Grid divisions.
    pub divx: i32,
    pub divy: i32,
    /// Animation timer.
    pub timer: Option<PropertyRef>,
    /// Animation cycle (ms).
    pub cycle: i32,
    /// Number of animation sets (len > 1 splits grid into len groups).
    pub len: i32,
    /// Reference property ID for multi-set selection.
    #[serde(rename = "ref")]
    pub ref_id: i32,
    /// Click action event.
    pub act: Option<PropertyRef>,
    /// Click type: 0=plus, 1=minus, 2=LR split, 3=UD split.
    pub click: i32,
}

impl Default for JsonImage {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            len: 0,
            ref_id: 0,
            act: None,
            click: 0,
        }
    }
}

/// Image set: switches between images based on an integer property.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonImageSet {
    pub id: FlexId,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<PropertyRef>,
    pub images: Vec<FlexId>,
    pub act: Option<PropertyRef>,
    pub click: i32,
}

// ---------------------------------------------------------------------------
// Number display
// ---------------------------------------------------------------------------

/// Integer number display (digits from image grid).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonValue {
    pub id: FlexId,
    pub src: FlexId,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub divx: i32,
    pub divy: i32,
    pub timer: Option<PropertyRef>,
    pub cycle: i32,
    pub align: i32,
    pub digit: i32,
    pub padding: i32,
    pub zeropadding: i32,
    pub space: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<PropertyRef>,
    pub offset: Option<Vec<JsonValueOffset>>,
}

impl Default for JsonValue {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            align: 0,
            digit: 0,
            padding: 0,
            zeropadding: 0,
            space: 0,
            ref_id: 0,
            value: None,
            offset: None,
        }
    }
}

/// Per-digit offset for Value display.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonValueOffset {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Float number display.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonFloatValue {
    pub id: FlexId,
    pub src: FlexId,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub divx: i32,
    pub divy: i32,
    pub timer: Option<PropertyRef>,
    pub cycle: i32,
    pub align: i32,
    /// Fractional digit count.
    pub fketa: i32,
    /// Integer digit count.
    pub iketa: i32,
    /// Scale factor.
    #[serde(default = "default_gain")]
    pub gain: f32,
    /// Show +/- sign.
    #[serde(rename = "isSignvisible")]
    pub is_sign_visible: bool,
    pub padding: i32,
    pub zeropadding: i32,
    pub space: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<PropertyRef>,
    pub offset: Option<Vec<JsonValueOffset>>,
}

fn default_gain() -> f32 {
    1.0
}

impl Default for JsonFloatValue {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            align: 0,
            fketa: 0,
            iketa: 0,
            gain: 1.0,
            is_sign_visible: false,
            padding: 0,
            zeropadding: 0,
            space: 0,
            ref_id: 0,
            value: None,
            offset: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// Text display object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonText {
    pub id: FlexId,
    /// Font ID reference.
    pub font: FlexId,
    pub size: i32,
    pub align: i32,
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub value: Option<PropertyRef>,
    #[serde(rename = "constantText")]
    pub constant_text: Option<String>,
    pub wrapping: bool,
    /// Overflow mode: 0=overflow, 1=shrink, 2=truncate.
    pub overflow: i32,
    #[serde(rename = "outlineColor")]
    pub outline_color: String,
    #[serde(rename = "outlineWidth")]
    pub outline_width: f32,
    #[serde(rename = "shadowColor")]
    pub shadow_color: String,
    #[serde(rename = "shadowOffsetX")]
    pub shadow_offset_x: f32,
    #[serde(rename = "shadowOffsetY")]
    pub shadow_offset_y: f32,
    #[serde(rename = "shadowSmoothness")]
    pub shadow_smoothness: f32,
}

impl Default for JsonText {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            font: FlexId::default(),
            size: 0,
            align: 0,
            ref_id: 0,
            value: None,
            constant_text: None,
            wrapping: false,
            overflow: 0,
            outline_color: "ffffff00".to_string(),
            outline_width: 0.0,
            shadow_color: "ffffff00".to_string(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_smoothness: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Slider
// ---------------------------------------------------------------------------

/// Interactive slider control.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonSlider {
    pub id: FlexId,
    pub src: FlexId,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub divx: i32,
    pub divy: i32,
    pub timer: Option<PropertyRef>,
    pub cycle: i32,
    /// Direction: 0=up, 1=right, 2=down, 3=left.
    pub angle: i32,
    pub range: i32,
    #[serde(rename = "type")]
    pub slider_type: i32,
    pub changeable: bool,
    pub value: Option<PropertyRef>,
    pub event: Option<PropertyRef>,
    #[serde(rename = "isRefNum")]
    pub is_ref_num: bool,
    pub min: i32,
    pub max: i32,
}

impl Default for JsonSlider {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            angle: 0,
            range: 0,
            slider_type: 0,
            changeable: true,
            value: None,
            event: None,
            is_ref_num: false,
            min: 0,
            max: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// Value graph or distribution graph.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonGraph {
    pub id: FlexId,
    pub src: FlexId,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub divx: i32,
    pub divy: i32,
    pub timer: Option<PropertyRef>,
    pub cycle: i32,
    /// Direction: 0=right (for distribution), 1=right (value graph).
    pub angle: i32,
    #[serde(rename = "type")]
    pub graph_type: i32,
    pub value: Option<PropertyRef>,
    #[serde(rename = "isRefNum")]
    pub is_ref_num: bool,
    pub min: i32,
    pub max: i32,
}

impl Default for JsonGraph {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            angle: 1,
            graph_type: 0,
            value: None,
            is_ref_num: false,
            min: 0,
            max: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Gauge graph
// ---------------------------------------------------------------------------

/// Gauge graph with colored sections.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonGaugeGraph {
    pub id: FlexId,
    /// 24 color strings: 6 gauge types × 4 states.
    pub color: Option<Vec<String>>,
    #[serde(rename = "assistClearBGColor")]
    pub assist_clear_bg_color: String,
    #[serde(rename = "assistAndEasyFailBGColor")]
    pub assist_and_easy_fail_bg_color: String,
    #[serde(rename = "grooveFailBGColor")]
    pub groove_fail_bg_color: String,
    #[serde(rename = "grooveClearAndHardBGColor")]
    pub groove_clear_and_hard_bg_color: String,
    #[serde(rename = "exHardBGColor")]
    pub ex_hard_bg_color: String,
    #[serde(rename = "hazardBGColor")]
    pub hazard_bg_color: String,
    #[serde(rename = "assistClearLineColor")]
    pub assist_clear_line_color: String,
    #[serde(rename = "assistAndEasyFailLineColor")]
    pub assist_and_easy_fail_line_color: String,
    #[serde(rename = "grooveFailLineColor")]
    pub groove_fail_line_color: String,
    #[serde(rename = "grooveClearAndHardLineColor")]
    pub groove_clear_and_hard_line_color: String,
    #[serde(rename = "exHardLineColor")]
    pub ex_hard_line_color: String,
    #[serde(rename = "hazardLineColor")]
    pub hazard_line_color: String,
    #[serde(rename = "borderlineColor")]
    pub borderline_color: String,
    #[serde(rename = "borderColor")]
    pub border_color: String,
}

impl Default for JsonGaugeGraph {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            color: None,
            assist_clear_bg_color: "440044".to_string(),
            assist_and_easy_fail_bg_color: "004444".to_string(),
            groove_fail_bg_color: "004400".to_string(),
            groove_clear_and_hard_bg_color: "440000".to_string(),
            ex_hard_bg_color: "444400".to_string(),
            hazard_bg_color: "444444".to_string(),
            assist_clear_line_color: "ff00ff".to_string(),
            assist_and_easy_fail_line_color: "00ffff".to_string(),
            groove_fail_line_color: "00ff00".to_string(),
            groove_clear_and_hard_line_color: "ff0000".to_string(),
            ex_hard_line_color: "ffff00".to_string(),
            hazard_line_color: "cccccc".to_string(),
            borderline_color: "ff0000".to_string(),
            border_color: "440000".to_string(),
        }
    }
}

/// Judge distribution graph.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonJudgeGraph {
    pub id: FlexId,
    #[serde(rename = "type")]
    pub graph_type: i32,
    #[serde(rename = "backTexOff")]
    pub back_tex_off: i32,
    pub delay: i32,
    #[serde(rename = "orderReverse")]
    pub order_reverse: i32,
    #[serde(rename = "noGap")]
    pub no_gap: i32,
    #[serde(rename = "noGapX")]
    pub no_gap_x: i32,
}

impl Default for JsonJudgeGraph {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            graph_type: 0,
            back_tex_off: 0,
            delay: 500,
            order_reverse: 0,
            no_gap: 0,
            no_gap_x: 0,
        }
    }
}

/// BPM timeline graph.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonBpmGraph {
    pub id: FlexId,
    pub delay: i32,
    #[serde(rename = "lineWidth")]
    pub line_width: i32,
    #[serde(rename = "mainBPMColor")]
    pub main_bpm_color: String,
    #[serde(rename = "minBPMColor")]
    pub min_bpm_color: String,
    #[serde(rename = "maxBPMColor")]
    pub max_bpm_color: String,
    #[serde(rename = "otherBPMColor")]
    pub other_bpm_color: String,
    #[serde(rename = "stopLineColor")]
    pub stop_line_color: String,
    #[serde(rename = "transitionLineColor")]
    pub transition_line_color: String,
}

impl Default for JsonBpmGraph {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            delay: 0,
            line_width: 2,
            main_bpm_color: "00ff00".to_string(),
            min_bpm_color: "0000ff".to_string(),
            max_bpm_color: "ff0000".to_string(),
            other_bpm_color: "ffff00".to_string(),
            stop_line_color: "ff00ff".to_string(),
            transition_line_color: "7f7f7f".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Visualizers
// ---------------------------------------------------------------------------

/// Hit error visualizer.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonHitErrorVisualizer {
    pub id: FlexId,
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

impl Default for JsonHitErrorVisualizer {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
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

/// Timing visualizer.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonTimingVisualizer {
    pub id: FlexId,
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

impl Default for JsonTimingVisualizer {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
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

/// Timing distribution graph.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonTimingDistributionGraph {
    pub id: FlexId,
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

impl Default for JsonTimingDistributionGraph {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
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

// ---------------------------------------------------------------------------
// Play-specific types
// ---------------------------------------------------------------------------

/// Note set definitions (lane graphics for play skins).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonNoteSet {
    pub id: FlexId,
    pub note: Vec<FlexId>,
    pub lnstart: Vec<FlexId>,
    pub lnend: Vec<FlexId>,
    pub lnbody: Vec<FlexId>,
    #[serde(rename = "lnbodyActive")]
    pub lnbody_active: Vec<FlexId>,
    pub lnactive: Vec<FlexId>,
    pub hcnstart: Vec<FlexId>,
    pub hcnend: Vec<FlexId>,
    pub hcnbody: Vec<FlexId>,
    pub hcnactive: Vec<FlexId>,
    #[serde(rename = "hcnbodyActive")]
    pub hcnbody_active: Vec<FlexId>,
    pub hcndamage: Vec<FlexId>,
    #[serde(rename = "hcnbodyMiss")]
    pub hcnbody_miss: Vec<FlexId>,
    pub hcnreactive: Vec<FlexId>,
    #[serde(rename = "hcnbodyReactive")]
    pub hcnbody_reactive: Vec<FlexId>,
    pub mine: Vec<FlexId>,
    pub hidden: Vec<FlexId>,
    pub processed: Vec<FlexId>,
    pub dst: Vec<JsonAnimation>,
    pub dst2: Option<i32>,
    pub expansionrate: Vec<i32>,
    pub size: Vec<f32>,
    pub group: Vec<JsonDestination>,
    pub bpm: Vec<JsonDestination>,
    pub stop: Vec<JsonDestination>,
    pub time: Vec<JsonDestination>,
}

/// Gauge display configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonGauge {
    pub id: FlexId,
    pub nodes: Vec<FlexId>,
    pub parts: i32,
    #[serde(rename = "type")]
    pub gauge_type: i32,
    pub range: i32,
    pub cycle: i32,
    pub starttime: i32,
    pub endtime: i32,
}

impl Default for JsonGauge {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            nodes: Vec::new(),
            parts: 50,
            gauge_type: 0,
            range: 3,
            cycle: 33,
            starttime: 0,
            endtime: 500,
        }
    }
}

/// Hidden cover (darkness effect).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonHiddenCover {
    pub id: FlexId,
    pub src: FlexId,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub divx: i32,
    pub divy: i32,
    pub timer: Option<PropertyRef>,
    pub cycle: i32,
    #[serde(rename = "disapearLine")]
    pub disapear_line: i32,
    #[serde(rename = "isDisapearLineLinkLift")]
    pub is_disapear_line_link_lift: bool,
}

impl Default for JsonHiddenCover {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            disapear_line: -1,
            is_disapear_line_link_lift: true,
        }
    }
}

/// Lift cover effect.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonLiftCover {
    pub id: FlexId,
    pub src: FlexId,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub divx: i32,
    pub divy: i32,
    pub timer: Option<PropertyRef>,
    pub cycle: i32,
    #[serde(rename = "disapearLine")]
    pub disapear_line: i32,
    #[serde(rename = "isDisapearLineLinkLift")]
    pub is_disapear_line_link_lift: bool,
}

impl Default for JsonLiftCover {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            divx: 1,
            divy: 1,
            timer: None,
            cycle: 0,
            disapear_line: -1,
            is_disapear_line_link_lift: false,
        }
    }
}

/// BGA display.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonBga {
    pub id: FlexId,
}

/// Judge display (result screen).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonJudge {
    pub id: FlexId,
    pub index: i32,
    pub images: Vec<JsonDestination>,
    pub numbers: Vec<JsonDestination>,
    pub shift: bool,
}

/// Song list (music select screen).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonSongList {
    pub id: FlexId,
    pub center: i32,
    pub clickable: Vec<i32>,
    pub listoff: Vec<JsonDestination>,
    pub liston: Vec<JsonDestination>,
    pub text: Vec<JsonDestination>,
    pub level: Vec<JsonDestination>,
    pub lamp: Vec<JsonDestination>,
    pub playerlamp: Vec<JsonDestination>,
    pub rivallamp: Vec<JsonDestination>,
    pub trophy: Vec<JsonDestination>,
    pub label: Vec<JsonDestination>,
    pub graph: Option<JsonDestination>,
}

// ---------------------------------------------------------------------------
// Character / Skin config
// ---------------------------------------------------------------------------

/// POMYU character definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonPmChara {
    pub id: FlexId,
    pub src: FlexId,
    pub color: i32,
    #[serde(rename = "type")]
    pub chara_type: i32,
    pub side: i32,
}

impl Default for JsonPmChara {
    fn default() -> Self {
        Self {
            id: FlexId::default(),
            src: FlexId::default(),
            color: 1,
            chara_type: i32::MIN,
            side: 1,
        }
    }
}

/// Skin configuration (skin select) properties.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonSkinConfigProperty {
    #[serde(rename = "customBMS")]
    pub custom_bms: Option<Vec<String>>,
    #[serde(rename = "defaultCategory")]
    pub default_category: i32,
    #[serde(rename = "customPropertyCount")]
    pub custom_property_count: i32,
    #[serde(rename = "customOffsetStyle")]
    pub custom_offset_style: i32,
}

/// Custom event definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonCustomEvent {
    pub id: i32,
    pub action: Option<PropertyRef>,
    pub condition: Option<PropertyRef>,
    #[serde(rename = "minInterval")]
    pub min_interval: i32,
}

/// Custom timer definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonCustomTimer {
    pub id: i32,
    pub timer: Option<PropertyRef>,
}

// ---------------------------------------------------------------------------
// Destination and animation
// ---------------------------------------------------------------------------

/// Rendering destination for a skin object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonDestination {
    pub id: FlexId,
    pub blend: i32,
    pub filter: i32,
    pub timer: Option<PropertyRef>,
    #[serde(rename = "loop")]
    pub loop_time: i32,
    pub center: i32,
    pub offset: i32,
    pub offsets: Vec<i32>,
    #[serde(default = "default_stretch")]
    pub stretch: i32,
    pub op: Vec<i32>,
    pub draw: Option<PropertyRef>,
    pub dst: Vec<JsonAnimation>,
    #[serde(rename = "mouseRect")]
    pub mouse_rect: Option<JsonRect>,
}

fn default_stretch() -> i32 {
    -1
}

/// A single animation keyframe.
///
/// Fields default to `i32::MIN` (sentinel for "inherit from previous frame").
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonAnimation {
    pub time: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub acc: i32,
    pub a: i32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub angle: i32,
}

impl Default for JsonAnimation {
    fn default() -> Self {
        Self {
            time: i32::MIN,
            x: i32::MIN,
            y: i32::MIN,
            w: i32::MIN,
            h: i32::MIN,
            acc: i32::MIN,
            a: i32::MIN,
            r: i32::MIN,
            g: i32::MIN,
            b: i32::MIN,
            angle: i32::MIN,
        }
    }
}

/// Rectangle for mouse click regions.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct JsonRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_id_from_string() {
        let json = r#""hello""#;
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id.as_str(), "hello");
    }

    #[test]
    fn test_flex_id_from_int() {
        let json = "42";
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id.as_str(), "42");
    }

    #[test]
    fn test_flex_id_from_negative_int() {
        let json = "-100";
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id.as_str(), "-100");
    }

    #[test]
    fn test_property_ref_from_int() {
        let json = "42";
        let p: PropertyRef = serde_json::from_str(json).unwrap();
        assert_eq!(p.as_id(), Some(42));
    }

    #[test]
    fn test_property_ref_from_string() {
        let json = r#""my_timer_script""#;
        let p: PropertyRef = serde_json::from_str(json).unwrap();
        assert!(p.as_id().is_none());
        if let PropertyRef::Script(s) = &p {
            assert_eq!(s, "my_timer_script");
        } else {
            panic!("Expected Script variant");
        }
    }

    #[test]
    fn test_animation_defaults() {
        let json = r#"{"x": 100, "y": 200}"#;
        let anim: JsonAnimation = serde_json::from_str(json).unwrap();
        assert_eq!(anim.x, 100);
        assert_eq!(anim.y, 200);
        assert_eq!(anim.time, i32::MIN);
        assert_eq!(anim.w, i32::MIN);
        assert_eq!(anim.a, i32::MIN);
    }

    #[test]
    fn test_destination_defaults() {
        let json = r#"{"id": "test", "dst": [{"x": 0, "y": 0, "w": 100, "h": 50}]}"#;
        let dst: JsonDestination = serde_json::from_str(json).unwrap();
        assert_eq!(dst.id.as_str(), "test");
        assert_eq!(dst.stretch, -1);
        assert_eq!(dst.blend, 0);
        assert_eq!(dst.dst.len(), 1);
        assert_eq!(dst.dst[0].x, 0);
        assert_eq!(dst.dst[0].w, 100);
    }

    #[test]
    fn test_parse_minimal_skin() {
        let json = r#"{
            "type": 6,
            "name": "test skin",
            "w": 1280,
            "h": 720,
            "fadeout": 500,
            "scene": 3000,
            "source": [{"id": 0, "path": "system.png"}],
            "image": [{"id": "blank", "src": 0, "x": 0, "y": 0, "w": 8, "h": 8}],
            "destination": [
                {"id": -100, "dst": [{"x": 0, "y": 0, "w": 1280, "h": 720}]}
            ]
        }"#;
        let skin: JsonSkinData = serde_json::from_str(json).unwrap();
        assert_eq!(skin.skin_type, 6);
        assert_eq!(skin.name.as_deref(), Some("test skin"));
        assert_eq!(skin.w, 1280);
        assert_eq!(skin.h, 720);
        assert_eq!(skin.fadeout, 500);
        assert_eq!(skin.scene, 3000);
        assert_eq!(skin.source.len(), 1);
        assert_eq!(skin.source[0].id.as_str(), "0");
        assert_eq!(skin.source[0].path.as_deref(), Some("system.png"));
        assert_eq!(skin.image.len(), 1);
        assert_eq!(skin.image[0].id.as_str(), "blank");
        assert_eq!(skin.image[0].src.as_str(), "0");
        assert_eq!(skin.destination.len(), 1);
        assert_eq!(skin.destination[0].id.as_str(), "-100");
    }

    #[test]
    fn test_parse_with_text() {
        let json = r#"{
            "text": [
                {"id": "title", "font": 0, "size": 30, "ref": 12}
            ]
        }"#;
        let skin: JsonSkinData = serde_json::from_str(json).unwrap();
        assert_eq!(skin.text.len(), 1);
        assert_eq!(skin.text[0].font.as_str(), "0");
        assert_eq!(skin.text[0].size, 30);
        assert_eq!(skin.text[0].ref_id, 12);
    }

    #[test]
    fn test_parse_with_timer_property() {
        let json = r#"{
            "destination": [
                {"id": "img", "timer": 2, "dst": [{"x": 0, "y": 0}]},
                {"id": "img2", "timer": "my_timer", "dst": [{"x": 0, "y": 0}]}
            ]
        }"#;
        let skin: JsonSkinData = serde_json::from_str(json).unwrap();
        assert_eq!(skin.destination[0].timer.as_ref().unwrap().as_id(), Some(2));
        assert!(
            skin.destination[1]
                .timer
                .as_ref()
                .unwrap()
                .as_id()
                .is_none()
        );
    }

    #[test]
    fn test_parse_image_with_grid() {
        let json = r#"{"id": "nums", "src": "0", "x": 0, "y": 0, "w": 240, "h": 24, "divx": 10, "divy": 1}"#;
        let img: JsonImage = serde_json::from_str(json).unwrap();
        assert_eq!(img.divx, 10);
        assert_eq!(img.divy, 1);
        assert_eq!(img.w, 240);
    }

    #[test]
    fn test_parse_gauge_defaults() {
        let json = r#"{"id": "gauge", "nodes": ["g1", "g2", "g3", "g4"]}"#;
        let gauge: JsonGauge = serde_json::from_str(json).unwrap();
        assert_eq!(gauge.parts, 50);
        assert_eq!(gauge.range, 3);
        assert_eq!(gauge.cycle, 33);
        assert_eq!(gauge.endtime, 500);
        assert_eq!(gauge.nodes.len(), 4);
    }

    #[test]
    fn test_parse_custom_events() {
        let json = r#"{
            "customEvents": [
                {"id": 1000, "action": 100, "condition": -50, "minInterval": 200}
            ],
            "customTimers": [
                {"id": 10000, "timer": 41}
            ]
        }"#;
        let skin: JsonSkinData = serde_json::from_str(json).unwrap();
        assert_eq!(skin.custom_events.len(), 1);
        assert_eq!(skin.custom_events[0].id, 1000);
        assert_eq!(skin.custom_events[0].min_interval, 200);
        assert_eq!(skin.custom_timers.len(), 1);
        assert_eq!(skin.custom_timers[0].id, 10000);
    }

    #[test]
    fn test_parse_default_skin_data() {
        let skin = JsonSkinData::default();
        assert_eq!(skin.skin_type, -1);
        assert_eq!(skin.w, 1280);
        assert_eq!(skin.h, 720);
        assert_eq!(skin.judgetimer, 1);
        assert!(skin.source.is_empty());
        assert!(skin.destination.is_empty());
    }

    #[test]
    fn test_parse_hit_error_visualizer() {
        let json = r#"{"id": "hev", "width": 401, "judgeWidthMillis": 200}"#;
        let hev: JsonHitErrorVisualizer = serde_json::from_str(json).unwrap();
        assert_eq!(hev.width, 401);
        assert_eq!(hev.judge_width_millis, 200);
        assert_eq!(hev.line_width, 1); // default
        assert_eq!(hev.ema_mode, 1); // default
    }

    #[test]
    fn test_parse_note_set() {
        let json = r#"{
            "id": "notes",
            "note": ["n1", "n2", "n3"],
            "lnstart": ["ls1"],
            "mine": ["m1"],
            "dst": [{"time": 0, "x": 0, "y": 0, "w": 100, "h": 10}]
        }"#;
        let ns: JsonNoteSet = serde_json::from_str(json).unwrap();
        assert_eq!(ns.note.len(), 3);
        assert_eq!(ns.lnstart.len(), 1);
        assert_eq!(ns.mine.len(), 1);
        assert_eq!(ns.dst.len(), 1);
    }
}
