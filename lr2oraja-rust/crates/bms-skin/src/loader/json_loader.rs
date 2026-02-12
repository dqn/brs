// JSON skin loader.
//
// Loads beatoraja-format JSON skin files and converts them into the
// Skin data model.
//
// The loading pipeline:
// 1. Read JSON file (UTF-8, with Shift_JIS fallback)
// 2. Pre-process: resolve conditional branches and file includes
// 3. Deserialize into JsonSkinData
// 4. Convert to SkinHeader (for skin selection UI)
// 5. Convert to Skin (full skin with all objects)

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use bms_config::resolution::Resolution;
use bms_config::skin_type::SkinType;

use crate::custom_event::{CustomEventDef, CustomTimerDef};
use crate::image_handle::ImageHandle;
use crate::loader::json_skin::{FlexId, JsonAnimation, JsonDestination, JsonSkinData};
use crate::property_id::{
    BooleanId, EventId, OFFSET_ALL, OFFSET_JUDGE_1P, OFFSET_JUDGEDETAIL_1P, OFFSET_NOTES_1P,
    TimerId,
};
use crate::skin::Skin;
use crate::skin_header::{
    CustomCategory, CustomCategoryItem, CustomFile, CustomOffset, CustomOption, SkinFormat,
    SkinHeader,
};
use crate::skin_object::{Color, Destination, Rect, SkinObjectBase};
use crate::skin_object_type::SkinObjectType;
use crate::skin_text::{FontType, TextShadow};
use crate::skin_visualizer::parse_color;
use crate::stretch_type::StretchType;

/// All Resolution variants for dimension-based lookup.
const ALL_RESOLUTIONS: [Resolution; 15] = [
    Resolution::Sd,
    Resolution::Svga,
    Resolution::Xga,
    Resolution::Hd,
    Resolution::Quadvga,
    Resolution::Fwxga,
    Resolution::Sxgaplus,
    Resolution::Hdplus,
    Resolution::Uxga,
    Resolution::Wsxgaplus,
    Resolution::Fullhd,
    Resolution::Wuxga,
    Resolution::Qxga,
    Resolution::Wqhd,
    Resolution::Ultrahd,
];

// ---------------------------------------------------------------------------
// Conditional processing
// ---------------------------------------------------------------------------

/// Tests whether an option condition is satisfied.
///
/// Condition format (matching Java's JsonSkinSerializer.testOption):
/// - `901` → option 901 is enabled
/// - `-901` → option 901 is NOT enabled
/// - `[901, 911]` → 901 AND 911 enabled
/// - `[[901, 902], 911]` → (901 OR 902) AND 911
pub fn test_option(condition: &Value, enabled: &HashSet<i32>) -> bool {
    match condition {
        Value::Null => true,
        Value::Number(n) => {
            let op = n.as_i64().unwrap_or(0) as i32;
            test_option_number(op, enabled)
        }
        Value::Array(arr) => {
            for item in arr {
                let ok = match item {
                    Value::Number(n) => {
                        let op = n.as_i64().unwrap_or(0) as i32;
                        test_option_number(op, enabled)
                    }
                    Value::Array(sub) => {
                        // OR group: at least one must be enabled
                        sub.iter().any(|v| {
                            if let Value::Number(n) = v {
                                let op = n.as_i64().unwrap_or(0) as i32;
                                test_option_number(op, enabled)
                            } else {
                                false
                            }
                        })
                    }
                    _ => false,
                };
                if !ok {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn test_option_number(op: i32, enabled: &HashSet<i32>) -> bool {
    if op >= 0 {
        enabled.contains(&op)
    } else {
        !enabled.contains(&(-op))
    }
}

/// Pre-processes a JSON Value to resolve conditional branches.
///
/// For array elements with `{"if": condition, "value": obj}` or
/// `{"if": condition, "values": [objs]}`, evaluates the condition
/// and includes/excludes the items accordingly.
///
/// For objects with `{"include": "path"}`, loads the referenced file.
/// (File includes are NOT implemented in this phase — they return null.)
pub fn resolve_conditionals(value: Value, enabled: &HashSet<i32>) -> Value {
    match value {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Value::Object(ref obj) = item {
                    if obj.contains_key("if")
                        && (obj.contains_key("value") || obj.contains_key("values"))
                    {
                        // Conditional branch
                        let condition = obj.get("if").unwrap_or(&Value::Null);
                        if test_option(condition, enabled) {
                            if let Some(val) = obj.get("value") {
                                result.push(resolve_conditionals(val.clone(), enabled));
                            }
                            if let Some(Value::Array(vals)) = obj.get("values") {
                                for v in vals {
                                    result.push(resolve_conditionals(v.clone(), enabled));
                                }
                            }
                        }
                        continue;
                    }
                    if obj.contains_key("include") {
                        // File include — deferred to Phase 10 integration
                        continue;
                    }
                }
                result.push(resolve_conditionals(item, enabled));
            }
            Value::Array(result)
        }
        Value::Object(mut map) => {
            // Check if this object itself is a conditional branch
            if map.contains_key("if") && map.contains_key("value") {
                let condition = map.get("if").unwrap_or(&Value::Null);
                if test_option(condition, enabled)
                    && let Some(val) = map.remove("value")
                {
                    return resolve_conditionals(val, enabled);
                }
                return Value::Null;
            }
            // Recurse into object fields
            let resolved: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, resolve_conditionals(v, enabled)))
                .collect();
            Value::Object(resolved)
        }
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Header loading
// ---------------------------------------------------------------------------

/// Pre-processes lenient JSON (as used by beatoraja skins) into strict JSON.
///
/// Handles:
/// - Missing commas between objects/arrays: `}  {` → `}, {`
/// - Trailing commas before `}` or `]`: `, }` → `}`
pub fn preprocess_json(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = input.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        if escape_next {
            escape_next = false;
            result.push(c);
            continue;
        }
        if c == '\\' && in_string {
            escape_next = true;
            result.push(c);
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }
        if in_string {
            result.push(c);
            continue;
        }

        // Remove trailing commas: skip comma if next non-whitespace is } or ]
        if c == ',' {
            let next_nonws = chars[i + 1..].iter().find(|ch| !ch.is_ascii_whitespace());
            if matches!(next_nonws, Some('}') | Some(']')) {
                continue; // skip trailing comma
            }
        }

        // Insert missing commas: after } or ] if next non-whitespace is { or [ or " or digit/minus
        if c == '}' || c == ']' {
            result.push(c);
            let next_nonws = chars[i + 1..].iter().find(|ch| !ch.is_ascii_whitespace());
            if matches!(
                next_nonws,
                Some('{') | Some('[') | Some('"') | Some('0'..='9') | Some('-')
            ) {
                result.push(',');
            }
            continue;
        }

        result.push(c);
    }

    result
}

/// Loads only the skin header from a JSON skin file.
///
/// This is used for the skin selection UI — it reads metadata and
/// custom options without loading the full skin.
pub fn load_header(json_str: &str) -> Result<SkinHeader> {
    let preprocessed = preprocess_json(json_str);
    let data: JsonSkinData =
        serde_json::from_str(&preprocessed).context("Failed to parse JSON skin")?;
    build_header(&data, None)
}

/// Builds a SkinHeader from parsed JSON skin data.
pub fn build_header(data: &JsonSkinData, path: Option<&Path>) -> Result<SkinHeader> {
    if data.skin_type == -1 {
        anyhow::bail!("Skin type not specified (type = -1)");
    }

    let skin_type = SkinType::from_id(data.skin_type);

    // Build custom options
    let mut options = Vec::new();
    for prop in &data.property {
        let mut op_ids = Vec::new();
        let mut op_names = Vec::new();
        for item in &prop.item {
            op_ids.push(item.op);
            op_names.push(item.name.clone().unwrap_or_default());
        }
        let mut opt = CustomOption::new(prop.name.clone().unwrap_or_default(), op_ids, op_names);
        if let Some(def) = &prop.def {
            opt.default_label = Some(def.clone());
        }
        options.push(opt);
    }

    // Build custom files
    let parent_dir = path
        .and_then(|p| p.parent())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut files = Vec::new();
    for fp in &data.filepath {
        let full_path = if parent_dir.is_empty() {
            fp.path.clone().unwrap_or_default()
        } else {
            format!("{}/{}", parent_dir, fp.path.as_deref().unwrap_or(""))
        };
        files.push(CustomFile::new(
            fp.name.clone().unwrap_or_default(),
            full_path,
            fp.def.clone(),
        ));
    }

    // Build custom offsets
    let mut offsets = Vec::new();
    for off in &data.offset {
        offsets.push(CustomOffset::new(
            off.name.clone().unwrap_or_default(),
            off.id,
            off.x,
            off.y,
            off.w,
            off.h,
            off.r,
            off.a,
        ));
    }

    // Add standard play-mode offsets
    if is_play_type(skin_type) {
        offsets.push(CustomOffset::new(
            "All offset(%)".to_string(),
            OFFSET_ALL,
            true,
            true,
            true,
            true,
            false,
            false,
        ));
        offsets.push(CustomOffset::new(
            "Notes offset".to_string(),
            OFFSET_NOTES_1P,
            false,
            false,
            false,
            true,
            false,
            false,
        ));
        offsets.push(CustomOffset::new(
            "Judge offset".to_string(),
            OFFSET_JUDGE_1P,
            true,
            true,
            true,
            true,
            false,
            true,
        ));
        offsets.push(CustomOffset::new(
            "Judge Detail offset".to_string(),
            OFFSET_JUDGEDETAIL_1P,
            true,
            true,
            true,
            true,
            false,
            true,
        ));
    }

    // Build categories
    let mut categories = Vec::new();
    for cat in &data.category {
        let mut items = Vec::new();
        for cat_item_name in &cat.item {
            // Find matching option
            for (i, prop) in data.property.iter().enumerate() {
                if prop.category.as_deref() == Some(cat_item_name) {
                    items.push(CustomCategoryItem::Option(i));
                }
            }
            // Find matching file
            for (i, fp) in data.filepath.iter().enumerate() {
                if fp.category.as_deref() == Some(cat_item_name) {
                    items.push(CustomCategoryItem::File(i));
                }
            }
            // Find matching offset
            for (i, off) in data.offset.iter().enumerate() {
                if off.category.as_deref() == Some(cat_item_name) {
                    items.push(CustomCategoryItem::Offset(i));
                }
            }
        }
        categories.push(CustomCategory {
            name: cat.name.clone().unwrap_or_default(),
            items,
        });
    }

    // Detect source resolution from skin dimensions
    let source_resolution = ALL_RESOLUTIONS
        .iter()
        .find(|r| r.width() == data.w && r.height() == data.h)
        .copied();

    Ok(SkinHeader {
        format: SkinFormat::Beatoraja,
        path: path.map(|p| p.to_path_buf()),
        skin_type,
        name: data.name.clone().unwrap_or_default(),
        author: data.author.clone().unwrap_or_default(),
        options,
        files,
        offsets,
        categories,
        resolution: source_resolution.unwrap_or(Resolution::Hd),
        source_resolution,
        destination_resolution: None,
    })
}

/// Returns true if the skin type is a play screen type.
fn is_play_type(skin_type: Option<SkinType>) -> bool {
    matches!(
        skin_type,
        Some(
            SkinType::Play5Keys
                | SkinType::Play7Keys
                | SkinType::Play9Keys
                | SkinType::Play10Keys
                | SkinType::Play14Keys
                | SkinType::Play24Keys
                | SkinType::Play24KeysDouble
        )
    )
}

// ---------------------------------------------------------------------------
// Full skin loading
// ---------------------------------------------------------------------------

/// Loads a full Skin from a JSON skin string.
///
/// `enabled_options`: set of enabled option IDs (from user's skin config).
/// `dest_resolution`: the display resolution to scale to.
pub fn load_skin(
    json_str: &str,
    enabled_options: &HashSet<i32>,
    dest_resolution: Resolution,
    path: Option<&Path>,
) -> Result<Skin> {
    load_skin_with_images(
        json_str,
        enabled_options,
        dest_resolution,
        path,
        &HashMap::new(),
    )
}

/// Loads a full Skin from a JSON skin string with pre-loaded source images.
///
/// `source_images` maps source ID strings (from the `source` array) to
/// `ImageHandle` values. When an image/slider/graph references a source ID,
/// the corresponding handle is used to populate `SkinImage.sources`,
/// `SkinSlider.source_images`, or `SkinGraph.source_images`.
pub fn load_skin_with_images(
    json_str: &str,
    enabled_options: &HashSet<i32>,
    dest_resolution: Resolution,
    path: Option<&Path>,
    source_images: &HashMap<String, ImageHandle>,
) -> Result<Skin> {
    // Pre-process lenient JSON and resolve conditionals
    let preprocessed = preprocess_json(json_str);
    let raw: Value = serde_json::from_str(&preprocessed).context("Failed to parse JSON")?;
    let resolved = resolve_conditionals(raw, enabled_options);
    let data: JsonSkinData =
        serde_json::from_value(resolved).context("Failed to deserialize resolved JSON")?;

    // Build header
    let mut header = build_header(&data, path)?;
    header.destination_resolution = Some(dest_resolution);

    // Create skin
    let mut skin = Skin::new(header);
    skin.fadeout = data.fadeout;
    skin.input = data.input;
    if data.scene > 0 {
        skin.scene = data.scene;
    }

    // Build options map
    for opt in &data.property {
        for item in &opt.item {
            let is_enabled = enabled_options.contains(&item.op);
            skin.options.insert(item.op, if is_enabled { 1 } else { 0 });
        }
    }

    // Process destinations → create skin objects
    for dst in &data.destination {
        if let Some(obj) = build_skin_object(&data, dst, path, source_images) {
            skin.add(obj);
        }
    }

    // Custom events
    for evt in &data.custom_events {
        let condition = evt
            .condition
            .as_ref()
            .and_then(|p| p.as_id().map(BooleanId));
        if let Some(action_ref) = &evt.action
            && let Some(action_id) = action_ref.as_id()
        {
            skin.custom_events.push(CustomEventDef::new(
                EventId(action_id),
                condition,
                evt.min_interval,
            ));
        }
    }

    // Custom timers
    for timer in &data.custom_timers {
        let timer_func = timer.timer.as_ref().and_then(|p| p.as_id().map(TimerId));
        if let Some(func) = timer_func {
            skin.custom_timers
                .push(CustomTimerDef::active(TimerId(timer.id), func));
        } else {
            skin.custom_timers
                .push(CustomTimerDef::passive(TimerId(timer.id)));
        }
    }

    Ok(skin)
}

// ---------------------------------------------------------------------------
// Object building
// ---------------------------------------------------------------------------

/// Builds a SkinObjectType from a JSON destination and the skin data.
///
/// Matches destination IDs against image/value/text/slider/graph definitions.
/// Returns None if no matching object definition is found.
fn build_skin_object(
    data: &JsonSkinData,
    dst: &JsonDestination,
    skin_path: Option<&Path>,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<SkinObjectType> {
    let dst_id = &dst.id;

    // Check for negative numeric ID → reference image
    if let Ok(id) = dst_id.as_str().parse::<i32>()
        && id < 0
    {
        let mut img = crate::skin_image::SkinImage::from_reference(-id);
        apply_destination(&mut img.base, dst);
        return Some(img.into());
    }

    // Skin-type specific objects must be resolved before plain images.
    if let Some(obj) = try_build_song_list(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_note(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_judge(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_gauge(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_bga(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_hidden_cover(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_lift_cover(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_gauge_graph(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_judge_graph(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_float(data, dst, dst_id) {
        return Some(obj);
    }

    // Try matching against each object type
    if let Some(obj) = try_build_image(data, dst, dst_id, source_images) {
        return Some(obj);
    }
    if let Some(obj) = try_build_image_set(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_number(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_text(data, dst, dst_id, skin_path) {
        return Some(obj);
    }
    if let Some(obj) = try_build_slider(data, dst, dst_id, source_images) {
        return Some(obj);
    }
    if let Some(obj) = try_build_graph(data, dst, dst_id, source_images) {
        return Some(obj);
    }
    if let Some(obj) = try_build_bpm_graph(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_hit_error_visualizer(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_timing_visualizer(data, dst, dst_id) {
        return Some(obj);
    }
    if let Some(obj) = try_build_timing_distribution(data, dst, dst_id) {
        return Some(obj);
    }

    None
}

fn try_build_song_list(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let song_list = data.songlist.as_ref()?;
    if song_list.id != *dst_id {
        return None;
    }

    let mut bar = crate::skin_bar::SkinBar {
        position: song_list.center,
        ..Default::default()
    };
    apply_destination(&mut bar.base, dst);
    Some(bar.into())
}

fn try_build_note(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let note = data.note.as_ref()?;
    if note.id != *dst_id {
        return None;
    }

    let mut skin_note = crate::skin_note::SkinNote::default();
    apply_destination(&mut skin_note.base, dst);
    Some(skin_note.into())
}

fn try_build_judge(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let judge_def = data.judge.iter().find(|j| j.id == *dst_id)?;

    let mut judge = crate::skin_judge::SkinJudge {
        player: judge_def.index,
        shift: judge_def.shift,
        ..Default::default()
    };
    apply_destination(&mut judge.base, dst);
    Some(judge.into())
}

fn try_build_gauge(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let gauge = data.gauge.as_ref()?;
    if gauge.id != *dst_id {
        return None;
    }

    let mut skin_gauge = crate::skin_gauge::SkinGauge::new(gauge.parts);
    apply_destination(&mut skin_gauge.base, dst);
    Some(skin_gauge.into())
}

fn try_build_bga(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let bga = data.bga.as_ref()?;
    if bga.id != *dst_id {
        return None;
    }

    let mut skin_bga = crate::skin_bga::SkinBga::default();
    apply_destination(&mut skin_bga.base, dst);
    Some(skin_bga.into())
}

fn try_build_hidden_cover(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let hidden = data.hidden_cover.iter().find(|h| h.id == *dst_id)?;

    let mut skin_hidden = crate::skin_hidden::SkinHidden {
        disapear_line: hidden.disapear_line as f32,
        link_lift: hidden.is_disapear_line_link_lift,
        timer: hidden.timer.as_ref().and_then(|t| t.as_id()),
        cycle: hidden.cycle,
        ..Default::default()
    };
    apply_destination(&mut skin_hidden.base, dst);
    Some(skin_hidden.into())
}

fn try_build_lift_cover(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let lift = data.lift_cover.iter().find(|l| l.id == *dst_id)?;

    let mut skin_lift = crate::skin_hidden::SkinLiftCover {
        disapear_line: lift.disapear_line as f32,
        link_lift: lift.is_disapear_line_link_lift,
        timer: lift.timer.as_ref().and_then(|t| t.as_id()),
        cycle: lift.cycle,
        ..Default::default()
    };
    apply_destination(&mut skin_lift.base, dst);
    Some(skin_lift.into())
}

fn try_build_gauge_graph(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let _gauge_graph = data.gaugegraph.iter().find(|g| g.id == *dst_id)?;
    let mut graph = crate::skin_distribution_graph::SkinDistributionGraph::default();
    apply_destination(&mut graph.base, dst);
    Some(graph.into())
}

fn try_build_judge_graph(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let graph_def = data.judgegraph.iter().find(|g| g.id == *dst_id)?;
    let mut graph = crate::skin_visualizer::SkinNoteDistributionGraph::new(
        graph_def.graph_type,
        graph_def.delay,
    );
    graph.back_tex_off = graph_def.back_tex_off != 0;
    graph.order_reverse = graph_def.order_reverse != 0;
    graph.no_gap = graph_def.no_gap != 0;
    graph.no_gap_x = graph_def.no_gap_x != 0;
    apply_destination(&mut graph.base, dst);
    Some(graph.into())
}

fn try_build_float(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let float_def = data.floatvalue.iter().find(|f| f.id == *dst_id)?;
    let ref_id = if let Some(value) = &float_def.value {
        value.as_id().unwrap_or(float_def.ref_id)
    } else {
        float_def.ref_id
    };

    let mut float_obj = crate::skin_float::SkinFloat {
        ref_id: Some(crate::property_id::FloatId(ref_id)),
        iketa: float_def.iketa,
        fketa: float_def.fketa,
        sign_visible: float_def.is_sign_visible,
        gain: float_def.gain,
        zero_padding: float_def.zeropadding,
        align: float_def.align,
        ..Default::default()
    };
    apply_destination(&mut float_obj.base, dst);
    Some(float_obj.into())
}

fn try_build_image(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<SkinObjectType> {
    let img_def = data.image.iter().find(|i| i.id == *dst_id)?;

    let mut skin_img = crate::skin_image::SkinImage::default();
    apply_destination(&mut skin_img.base, dst);

    // Resolve source image handle
    if let Some(&handle) = source_images.get(img_def.src.as_str()) {
        let timer = img_def.timer.as_ref().and_then(|t| t.as_id());
        skin_img.sources = vec![crate::skin_image::SkinImageSource::Frames {
            images: vec![handle],
            timer,
            cycle: img_def.cycle,
        }];
    }

    // Record click event
    if let Some(act) = &img_def.act
        && let Some(id) = act.as_id()
    {
        skin_img.base.click_event = Some(EventId(id));
        skin_img.base.click_event_type = img_def.click;
    }

    Some(skin_img.into())
}

fn try_build_image_set(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let set_def = data.imageset.iter().find(|i| i.id == *dst_id)?;

    let mut skin_img = crate::skin_image::SkinImage::default();
    apply_destination(&mut skin_img.base, dst);

    // Set the ref selector
    let ref_id = if let Some(ref val) = set_def.value {
        val.as_id().unwrap_or(set_def.ref_id)
    } else {
        set_def.ref_id
    };
    if ref_id != 0 {
        skin_img.ref_id = Some(crate::property_id::IntegerId(ref_id));
    }

    if let Some(act) = &set_def.act
        && let Some(id) = act.as_id()
    {
        skin_img.base.click_event = Some(EventId(id));
        skin_img.base.click_event_type = set_def.click;
    }

    Some(skin_img.into())
}

fn try_build_number(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let val_def = data.value.iter().find(|v| v.id == *dst_id)?;

    let ref_id = if let Some(ref val) = val_def.value {
        val.as_id().unwrap_or(val_def.ref_id)
    } else {
        val_def.ref_id
    };

    let mut num = crate::skin_number::SkinNumber {
        base: SkinObjectBase::default(),
        ref_id: Some(crate::property_id::IntegerId(ref_id)),
        keta: val_def.digit,
        zero_padding: crate::skin_number::ZeroPadding::from_i32(val_def.zeropadding),
        align: crate::skin_number::NumberAlign::from_i32(val_def.align),
        space: val_def.space,
        ..Default::default()
    };
    apply_destination(&mut num.base, dst);

    // Record per-digit offsets
    if let Some(offsets) = &val_def.offset {
        num.digit_offsets = offsets
            .iter()
            .map(|o| crate::skin_object::SkinOffset {
                x: o.x as f32,
                y: o.y as f32,
                w: o.w as f32,
                h: o.h as f32,
                ..Default::default()
            })
            .collect();
    }

    Some(num.into())
}

fn try_build_text(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
    skin_path: Option<&Path>,
) -> Option<SkinObjectType> {
    let text_def = data.text.iter().find(|t| t.id == *dst_id)?;

    let ref_id = if let Some(ref val) = text_def.value {
        val.as_id().unwrap_or(text_def.ref_id)
    } else {
        text_def.ref_id
    };

    let outline_color = if text_def.outline_width > 0.0 {
        Some(parse_color(&text_def.outline_color))
    } else {
        None
    };

    let shadow = if text_def.shadow_offset_x != 0.0 || text_def.shadow_offset_y != 0.0 {
        Some(TextShadow {
            color: parse_color(&text_def.shadow_color),
            offset_x: text_def.shadow_offset_x,
            offset_y: text_def.shadow_offset_y,
            smoothness: text_def.shadow_smoothness,
        })
    } else {
        None
    };

    // Resolve font type from font ID
    let font_type = resolve_font_type(data, &text_def.font, skin_path);

    let mut text = crate::skin_text::SkinText {
        base: SkinObjectBase::default(),
        ref_id: Some(crate::property_id::StringId(ref_id)),
        constant_text: text_def.constant_text.clone(),
        font_size: text_def.size as f32,
        align: crate::skin_text::TextAlign::from_i32(text_def.align),
        wrapping: text_def.wrapping,
        overflow: crate::skin_text::TextOverflow::from_i32(text_def.overflow),
        outline_color,
        outline_width: text_def.outline_width,
        shadow,
        font_type,
        ..Default::default()
    };
    apply_destination(&mut text.base, dst);

    Some(text.into())
}

/// Resolves a font ID reference to a FontType.
///
/// Looks up the font definition in the skin data, then determines the type
/// based on the file extension and font_type field.
fn resolve_font_type(data: &JsonSkinData, font_id: &FlexId, skin_path: Option<&Path>) -> FontType {
    let font_def = match data.font.iter().find(|f| f.id == *font_id) {
        Some(f) => f,
        None => return FontType::Default,
    };

    let raw_path = match &font_def.path {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return FontType::Default,
    };

    // Resolve relative path against skin directory
    let full_path = if let Some(sp) = skin_path
        && let Some(parent) = sp.parent()
    {
        let candidate = parent.join(&raw_path);
        candidate.to_string_lossy().to_string()
    } else {
        raw_path.clone()
    };

    // Determine font type by extension
    let lower = raw_path.to_lowercase();
    if lower.ends_with(".fnt") {
        FontType::Bitmap {
            path: full_path,
            bitmap_type: font_def.font_type,
        }
    } else {
        FontType::Ttf(full_path)
    }
}

fn try_build_slider(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<SkinObjectType> {
    let sl_def = data.slider.iter().find(|s| s.id == *dst_id)?;

    let value_id = if let Some(ref val) = sl_def.value {
        val.as_id().unwrap_or(sl_def.slider_type)
    } else {
        sl_def.slider_type
    };

    let mut slider = crate::skin_slider::SkinSlider {
        base: SkinObjectBase::default(),
        direction: crate::skin_slider::SliderDirection::from_i32(sl_def.angle),
        range: sl_def.range,
        ref_id: Some(crate::property_id::FloatId(value_id)),
        changeable: sl_def.changeable,
        ..Default::default()
    };
    apply_destination(&mut slider.base, dst);

    // Resolve source image handle
    if let Some(&handle) = source_images.get(sl_def.src.as_str()) {
        slider.source_images = vec![handle];
    }

    Some(slider.into())
}

fn try_build_graph(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<SkinObjectType> {
    let gr_def = data.graph.iter().find(|g| g.id == *dst_id)?;

    let value_id = if let Some(ref val) = gr_def.value {
        val.as_id().unwrap_or(gr_def.graph_type)
    } else {
        gr_def.graph_type
    };

    // JSON angle 0 = up, non-zero = right
    let direction = if gr_def.angle == 0 {
        crate::skin_graph::GraphDirection::Up
    } else {
        crate::skin_graph::GraphDirection::Right
    };

    let mut graph = crate::skin_graph::SkinGraph {
        base: SkinObjectBase::default(),
        direction,
        ref_id: Some(crate::property_id::FloatId(value_id)),
        ..Default::default()
    };
    apply_destination(&mut graph.base, dst);

    // Resolve source image handle
    if let Some(&handle) = source_images.get(gr_def.src.as_str()) {
        graph.source_images = vec![handle];
    }

    Some(graph.into())
}

fn try_build_bpm_graph(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let bg_def = data.bpmgraph.iter().find(|b| b.id == *dst_id)?;

    let mut bpm_graph = crate::skin_bpm_graph::SkinBpmGraph {
        base: SkinObjectBase::default(),
        delay: bg_def.delay,
        line_width: bg_def.line_width,
        colors: crate::skin_bpm_graph::BpmGraphColors {
            main_bpm: parse_color(&bg_def.main_bpm_color),
            min_bpm: parse_color(&bg_def.min_bpm_color),
            max_bpm: parse_color(&bg_def.max_bpm_color),
            other_bpm: parse_color(&bg_def.other_bpm_color),
            stop: parse_color(&bg_def.stop_line_color),
            transition: parse_color(&bg_def.transition_line_color),
        },
    };
    apply_destination(&mut bpm_graph.base, dst);

    Some(bpm_graph.into())
}

fn try_build_hit_error_visualizer(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let hev_def = data.hiterrorvisualizer.iter().find(|h| h.id == *dst_id)?;

    let mut vis = crate::skin_visualizer::SkinHitErrorVisualizer {
        base: SkinObjectBase::default(),
        width: hev_def.width,
        judge_width_millis: hev_def.judge_width_millis,
        line_width: hev_def.line_width,
        hiterror_mode: hev_def.hiterror_mode != 0,
        color_mode: hev_def.color_mode != 0,
        ema_mode: crate::skin_visualizer::EmaMode::from_i32(hev_def.ema_mode),
        ema_alpha: hev_def.alpha,
        window_length: hev_def.window_length,
        draw_decay: hev_def.draw_decay != 0,
        line_color: parse_color(&hev_def.line_color),
        center_color: parse_color(&hev_def.center_color),
        ema_color: parse_color(&hev_def.ema_color),
        judge_colors: [
            parse_color(&hev_def.pg_color),
            parse_color(&hev_def.gr_color),
            parse_color(&hev_def.gd_color),
            parse_color(&hev_def.bd_color),
            parse_color(&hev_def.pr_color),
        ],
    };
    apply_destination(&mut vis.base, dst);

    Some(vis.into())
}

fn try_build_timing_visualizer(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let tv_def = data.timingvisualizer.iter().find(|t| t.id == *dst_id)?;

    let mut vis = crate::skin_visualizer::SkinTimingVisualizer {
        base: SkinObjectBase::default(),
        width: tv_def.width,
        judge_width_millis: tv_def.judge_width_millis,
        line_width: tv_def.line_width,
        draw_decay: tv_def.draw_decay != 0,
        line_color: parse_color(&tv_def.line_color),
        center_color: parse_color(&tv_def.center_color),
        judge_colors: [
            parse_color(&tv_def.pg_color),
            parse_color(&tv_def.gr_color),
            parse_color(&tv_def.gd_color),
            parse_color(&tv_def.bd_color),
            parse_color(&tv_def.pr_color),
        ],
    };
    apply_destination(&mut vis.base, dst);

    Some(vis.into())
}

fn try_build_timing_distribution(
    data: &JsonSkinData,
    dst: &JsonDestination,
    dst_id: &FlexId,
) -> Option<SkinObjectType> {
    let td_def = data
        .timingdistributiongraph
        .iter()
        .find(|t| t.id == *dst_id)?;

    let mut graph = crate::skin_visualizer::SkinTimingDistributionGraph {
        base: SkinObjectBase::default(),
        graph_width: td_def.width,
        line_width: td_def.line_width,
        draw_average: td_def.draw_average != 0,
        draw_dev: td_def.draw_dev != 0,
        graph_color: parse_color(&td_def.graph_color),
        average_color: parse_color(&td_def.average_color),
        dev_color: parse_color(&td_def.dev_color),
        judge_colors: [
            parse_color(&td_def.pg_color),
            parse_color(&td_def.gr_color),
            parse_color(&td_def.gd_color),
            parse_color(&td_def.bd_color),
            parse_color(&td_def.pr_color),
        ],
    };
    apply_destination(&mut graph.base, dst);

    Some(graph.into())
}

// ---------------------------------------------------------------------------
// Destination processing
// ---------------------------------------------------------------------------

/// Applies a JSON destination to a SkinObjectBase.
///
/// Fills animation keyframes with inheritance (MIN_VALUE → inherit from
/// previous frame or use defaults), sets timer, blend, offsets, etc.
fn apply_destination(base: &mut SkinObjectBase, dst: &JsonDestination) {
    // Set base properties
    base.blend = dst.blend;
    base.filter = dst.filter;
    base.set_center(dst.center);
    base.name = Some(dst.id.as_str().to_string());

    // Timer
    if let Some(ref timer) = dst.timer
        && let Some(id) = timer.as_id()
    {
        base.timer = Some(TimerId(id));
    }

    // Loop
    base.loop_time = dst.loop_time;

    // Draw conditions
    if let Some(ref draw) = dst.draw
        && let Some(id) = draw.as_id()
    {
        base.draw_conditions.push(BooleanId(id));
    } else if !dst.op.is_empty() {
        base.option_conditions = dst.op.clone();
    }

    // Stretch
    if dst.stretch >= 0 {
        base.stretch = StretchType::from_id(dst.stretch).unwrap_or_default();
    }

    // Mouse rect
    if let Some(ref mr) = dst.mouse_rect {
        base.mouse_rect = Some(Rect::new(
            mr.x as f32,
            mr.y as f32,
            mr.w as f32,
            mr.h as f32,
        ));
    }

    // Animation keyframes
    let mut prev: Option<&JsonAnimation> = None;
    for anim in &dst.dst {
        let resolved = resolve_animation(anim, prev);
        base.add_destination(Destination {
            time: resolved.time as i64,
            region: Rect::new(
                resolved.x as f32,
                resolved.y as f32,
                resolved.w as f32,
                resolved.h as f32,
            ),
            color: Color::from_rgba_u8(
                resolved.r as u8,
                resolved.g as u8,
                resolved.b as u8,
                resolved.a as u8,
            ),
            angle: resolved.angle,
            acc: resolved.acc,
        });
        prev = Some(anim);
    }

    // Offsets
    let mut offset_ids: Vec<i32> = dst.offsets.clone();
    offset_ids.push(dst.offset);
    base.set_offset_ids(&offset_ids);
}

/// Resolves animation keyframe values, inheriting from the previous frame
/// or using defaults for the first frame.
///
/// Matches Java's `setDestination()` fill logic exactly.
fn resolve_animation(anim: &JsonAnimation, prev: Option<&JsonAnimation>) -> ResolvedAnimation {
    match prev {
        None => ResolvedAnimation {
            time: if anim.time == i32::MIN { 0 } else { anim.time },
            x: if anim.x == i32::MIN { 0 } else { anim.x },
            y: if anim.y == i32::MIN { 0 } else { anim.y },
            w: if anim.w == i32::MIN { 0 } else { anim.w },
            h: if anim.h == i32::MIN { 0 } else { anim.h },
            acc: if anim.acc == i32::MIN { 0 } else { anim.acc },
            a: if anim.a == i32::MIN { 255 } else { anim.a },
            r: if anim.r == i32::MIN { 255 } else { anim.r },
            g: if anim.g == i32::MIN { 255 } else { anim.g },
            b: if anim.b == i32::MIN { 255 } else { anim.b },
            angle: if anim.angle == i32::MIN {
                0
            } else {
                anim.angle
            },
        },
        Some(p) => {
            // Resolve previous frame values first (for inheritance chain)
            // Note: in the Java code, `prev` is already resolved in-place,
            // so we use the raw prev values here (they were already resolved).
            let prev_resolved = resolve_prev(p);
            ResolvedAnimation {
                time: if anim.time == i32::MIN {
                    prev_resolved.time
                } else {
                    anim.time
                },
                x: if anim.x == i32::MIN {
                    prev_resolved.x
                } else {
                    anim.x
                },
                y: if anim.y == i32::MIN {
                    prev_resolved.y
                } else {
                    anim.y
                },
                w: if anim.w == i32::MIN {
                    prev_resolved.w
                } else {
                    anim.w
                },
                h: if anim.h == i32::MIN {
                    prev_resolved.h
                } else {
                    anim.h
                },
                acc: if anim.acc == i32::MIN {
                    prev_resolved.acc
                } else {
                    anim.acc
                },
                a: if anim.a == i32::MIN {
                    prev_resolved.a
                } else {
                    anim.a
                },
                r: if anim.r == i32::MIN {
                    prev_resolved.r
                } else {
                    anim.r
                },
                g: if anim.g == i32::MIN {
                    prev_resolved.g
                } else {
                    anim.g
                },
                b: if anim.b == i32::MIN {
                    prev_resolved.b
                } else {
                    anim.b
                },
                angle: if anim.angle == i32::MIN {
                    prev_resolved.angle
                } else {
                    anim.angle
                },
            }
        }
    }
}

/// Resolves a previous animation frame's values (for inheritance chain).
///
/// In Java, the animation values are modified in-place during iteration,
/// so the "previous" values are already resolved. We replicate this by
/// treating MIN_VALUE fields as their default values.
fn resolve_prev(anim: &JsonAnimation) -> ResolvedAnimation {
    ResolvedAnimation {
        time: if anim.time == i32::MIN { 0 } else { anim.time },
        x: if anim.x == i32::MIN { 0 } else { anim.x },
        y: if anim.y == i32::MIN { 0 } else { anim.y },
        w: if anim.w == i32::MIN { 0 } else { anim.w },
        h: if anim.h == i32::MIN { 0 } else { anim.h },
        acc: if anim.acc == i32::MIN { 0 } else { anim.acc },
        a: if anim.a == i32::MIN { 255 } else { anim.a },
        r: if anim.r == i32::MIN { 255 } else { anim.r },
        g: if anim.g == i32::MIN { 255 } else { anim.g },
        b: if anim.b == i32::MIN { 255 } else { anim.b },
        angle: if anim.angle == i32::MIN {
            0
        } else {
            anim.angle
        },
    }
}

struct ResolvedAnimation {
    time: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    acc: i32,
    a: i32,
    r: i32,
    g: i32,
    b: i32,
    angle: i32,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::json_skin::PropertyRef;

    // -- Option testing --

    #[test]
    fn test_option_single_enabled() {
        let enabled = HashSet::from([901]);
        assert!(test_option(&serde_json::json!(901), &enabled));
        assert!(!test_option(&serde_json::json!(902), &enabled));
    }

    #[test]
    fn test_option_negation() {
        let enabled = HashSet::from([901]);
        assert!(!test_option(&serde_json::json!(-901), &enabled));
        assert!(test_option(&serde_json::json!(-902), &enabled));
    }

    #[test]
    fn test_option_and() {
        let enabled = HashSet::from([901, 911]);
        assert!(test_option(&serde_json::json!([901, 911]), &enabled));
        assert!(!test_option(&serde_json::json!([901, 912]), &enabled));
    }

    #[test]
    fn test_option_or_and() {
        let enabled = HashSet::from([902, 911]);
        // (901 OR 902) AND 911
        assert!(test_option(&serde_json::json!([[901, 902], 911]), &enabled));
        // (903 OR 904) AND 911
        assert!(!test_option(
            &serde_json::json!([[903, 904], 911]),
            &enabled
        ));
    }

    #[test]
    fn test_option_null() {
        let enabled = HashSet::new();
        assert!(test_option(&Value::Null, &enabled));
    }

    // -- Conditional resolution --

    #[test]
    fn test_resolve_conditional_array() {
        let enabled = HashSet::from([901]);
        let json = serde_json::json!([
            {"if": 901, "value": {"id": "a"}},
            {"if": 902, "value": {"id": "b"}},
            {"id": "c"}
        ]);
        let resolved = resolve_conditionals(json, &enabled);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], "a");
        assert_eq!(arr[1]["id"], "c");
    }

    #[test]
    fn test_resolve_conditional_values() {
        let enabled = HashSet::from([901]);
        let json = serde_json::json!([
            {"if": 901, "values": [{"id": "a"}, {"id": "b"}]}
        ]);
        let resolved = resolve_conditionals(json, &enabled);
        let arr = resolved.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_resolve_conditional_not_matched() {
        let enabled = HashSet::new();
        let json = serde_json::json!([
            {"if": 901, "value": {"id": "a"}}
        ]);
        let resolved = resolve_conditionals(json, &enabled);
        let arr = resolved.as_array().unwrap();
        assert!(arr.is_empty());
    }

    // -- Header loading --

    #[test]
    fn test_load_header_minimal() {
        let json = r#"{
            "type": 6,
            "name": "Test Skin",
            "author": "Test Author",
            "w": 1280,
            "h": 720
        }"#;
        let header = load_header(json).unwrap();
        assert_eq!(header.name, "Test Skin");
        assert_eq!(header.author, "Test Author");
        assert_eq!(header.format, SkinFormat::Beatoraja);
    }

    #[test]
    fn test_load_header_with_options() {
        let json = r#"{
            "type": 0,
            "name": "Play Skin",
            "property": [
                {
                    "name": "BGA Size",
                    "item": [
                        {"name": "Normal", "op": 900},
                        {"name": "Extend", "op": 901}
                    ],
                    "def": "Normal"
                }
            ]
        }"#;
        let header = load_header(json).unwrap();
        assert_eq!(header.options.len(), 1);
        assert_eq!(header.options[0].option_ids, vec![900, 901]);
        assert_eq!(header.options[0].default_label, Some("Normal".to_string()));
    }

    #[test]
    fn test_load_header_play_offsets() {
        let json = r#"{"type": 0, "name": "Play7K"}"#;
        let header = load_header(json).unwrap();
        // Play skins get 4 standard offsets added
        assert_eq!(header.offsets.len(), 4);
        assert_eq!(header.offsets[0].name, "All offset(%)");
        assert_eq!(header.offsets[1].name, "Notes offset");
    }

    #[test]
    fn test_load_header_invalid_type() {
        let json = r#"{"name": "No Type"}"#;
        assert!(load_header(json).is_err());
    }

    // -- Full skin loading --

    #[test]
    fn test_load_skin_minimal() {
        let json = r#"{
            "type": 6,
            "name": "Decide",
            "w": 1280,
            "h": 720,
            "fadeout": 500,
            "scene": 3000,
            "destination": [
                {"id": -100, "dst": [{"x": 0, "y": 0, "w": 1280, "h": 720}]}
            ]
        }"#;
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        assert_eq!(skin.fadeout, 500);
        assert_eq!(skin.scene, 3000);
        assert_eq!(skin.object_count(), 1);
    }

    #[test]
    fn test_load_skin_with_text() {
        let json = r#"{
            "type": 6,
            "name": "Test",
            "text": [
                {"id": "title", "font": 0, "size": 30, "ref": 12}
            ],
            "destination": [
                {"id": "title", "dst": [{"x": 100, "y": 200, "w": 18, "h": 18}]}
            ]
        }"#;
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        assert_eq!(skin.object_count(), 1);
        match &skin.objects[0] {
            SkinObjectType::Text(t) => {
                assert_eq!(t.font_size, 30.0);
                assert_eq!(t.ref_id.unwrap().0, 12);
            }
            _ => panic!("Expected Text object"),
        }
    }

    #[test]
    fn test_load_skin_with_conditionals() {
        let json = r#"{
            "type": 6,
            "name": "Test",
            "image": [
                {"id": "img_a", "src": 0},
                {"id": "img_b", "src": 0}
            ],
            "destination": [
                {"if": 901, "value": {"id": "img_a", "dst": [{"x": 0, "y": 0, "w": 100, "h": 100}]}},
                {"id": "img_b", "dst": [{"x": 0, "y": 0, "w": 200, "h": 200}]}
            ]
        }"#;

        // Without option 901 enabled: only img_b
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        assert_eq!(skin.object_count(), 1);

        // With option 901 enabled: both objects
        let skin = load_skin(json, &HashSet::from([901]), Resolution::Hd, None).unwrap();
        assert_eq!(skin.object_count(), 2);
    }

    #[test]
    fn test_load_skin_custom_events() {
        let json = r#"{
            "type": 6,
            "name": "Test",
            "customEvents": [
                {"id": 1000, "action": 100, "condition": -50, "minInterval": 200}
            ],
            "customTimers": [
                {"id": 10000, "timer": 41},
                {"id": 10001}
            ],
            "destination": []
        }"#;
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        assert_eq!(skin.custom_events.len(), 1);
        assert_eq!(skin.custom_events[0].id, EventId(100));
        assert_eq!(skin.custom_events[0].condition, Some(BooleanId(-50)));
        assert_eq!(skin.custom_events[0].min_interval, 200);
        assert_eq!(skin.custom_timers.len(), 2);
        assert!(!skin.custom_timers[0].is_passive());
        assert!(skin.custom_timers[1].is_passive());
    }

    // -- Animation resolution --

    #[test]
    fn test_animation_first_frame_defaults() {
        let anim = JsonAnimation {
            time: i32::MIN,
            x: 100,
            y: i32::MIN,
            w: 200,
            h: i32::MIN,
            acc: i32::MIN,
            a: i32::MIN,
            r: i32::MIN,
            g: i32::MIN,
            b: i32::MIN,
            angle: i32::MIN,
        };
        let resolved = resolve_animation(&anim, None);
        assert_eq!(resolved.time, 0);
        assert_eq!(resolved.x, 100);
        assert_eq!(resolved.y, 0);
        assert_eq!(resolved.w, 200);
        assert_eq!(resolved.h, 0);
        assert_eq!(resolved.a, 255);
        assert_eq!(resolved.r, 255);
        assert_eq!(resolved.angle, 0);
    }

    #[test]
    fn test_animation_inheritance() {
        let prev = JsonAnimation {
            time: 0,
            x: 100,
            y: 200,
            w: 300,
            h: 400,
            acc: 0,
            a: 255,
            r: 255,
            g: 255,
            b: 255,
            angle: 45,
        };
        let anim = JsonAnimation {
            time: 1000,
            x: i32::MIN, // inherit 100
            y: 500,      // override
            w: i32::MIN, // inherit 300
            h: i32::MIN, // inherit 400
            acc: i32::MIN,
            a: 128,
            r: i32::MIN,
            g: i32::MIN,
            b: i32::MIN,
            angle: i32::MIN, // inherit 45
        };
        let resolved = resolve_animation(&anim, Some(&prev));
        assert_eq!(resolved.time, 1000);
        assert_eq!(resolved.x, 100);
        assert_eq!(resolved.y, 500);
        assert_eq!(resolved.w, 300);
        assert_eq!(resolved.a, 128);
        assert_eq!(resolved.angle, 45);
    }

    // -- Destination processing --

    #[test]
    fn test_apply_destination_basic() {
        let dst = JsonDestination {
            id: FlexId::from("test"),
            blend: 2,
            filter: 1,
            center: 5,
            loop_time: 1000,
            stretch: 1,
            dst: vec![JsonAnimation {
                time: 0,
                x: 10,
                y: 20,
                w: 100,
                h: 50,
                acc: 0,
                a: 200,
                r: 255,
                g: 128,
                b: 64,
                angle: 30,
            }],
            ..Default::default()
        };
        let mut base = SkinObjectBase::default();
        apply_destination(&mut base, &dst);

        assert_eq!(base.blend, 2);
        assert_eq!(base.filter, 1);
        assert_eq!(base.center, 5);
        assert_eq!(base.loop_time, 1000);
        assert_eq!(base.stretch, StretchType::KeepAspectRatioFitInner);
        assert_eq!(base.destinations.len(), 1);
        assert_eq!(base.destinations[0].time, 0);
        assert!((base.destinations[0].region.x - 10.0).abs() < 0.001);
        assert_eq!(base.destinations[0].angle, 30);
    }

    #[test]
    fn test_apply_destination_with_timer_and_draw() {
        let dst = JsonDestination {
            id: FlexId::from("test"),
            timer: Some(PropertyRef::Id(42)),
            draw: Some(PropertyRef::Id(100)),
            dst: vec![JsonAnimation::default()],
            ..Default::default()
        };
        let mut base = SkinObjectBase::default();
        apply_destination(&mut base, &dst);

        assert_eq!(base.timer, Some(TimerId(42)));
        assert_eq!(base.draw_conditions, vec![BooleanId(100)]);
    }

    #[test]
    fn test_apply_destination_offsets() {
        let dst = JsonDestination {
            id: FlexId::from("test"),
            offset: 10,
            offsets: vec![20, 30],
            dst: vec![JsonAnimation::default()],
            ..Default::default()
        };
        let mut base = SkinObjectBase::default();
        apply_destination(&mut base, &dst);

        assert_eq!(base.offset_ids, vec![20, 30, 10]);
    }

    // -- Font resolution --

    #[test]
    fn test_font_resolution_ttf() {
        let json = r#"{
            "type": 6,
            "name": "Test",
            "font": [{"id": "0", "path": "fonts/myfont.ttf", "type": 0}],
            "text": [{"id": "title", "font": 0, "size": 24, "ref": 12}],
            "destination": [
                {"id": "title", "dst": [{"x": 0, "y": 0, "w": 200, "h": 30}]}
            ]
        }"#;
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        match &skin.objects[0] {
            SkinObjectType::Text(t) => {
                assert!(matches!(t.font_type, FontType::Ttf(_)));
                if let FontType::Ttf(path) = &t.font_type {
                    assert_eq!(path, "fonts/myfont.ttf");
                }
            }
            _ => panic!("Expected Text object"),
        }
    }

    #[test]
    fn test_font_resolution_bitmap() {
        let json = r#"{
            "type": 6,
            "name": "Test",
            "font": [{"id": "0", "path": "fonts/bitmap.fnt", "type": 1}],
            "text": [{"id": "title", "font": 0, "size": 24, "ref": 12}],
            "destination": [
                {"id": "title", "dst": [{"x": 0, "y": 0, "w": 200, "h": 30}]}
            ]
        }"#;
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        match &skin.objects[0] {
            SkinObjectType::Text(t) => {
                if let FontType::Bitmap { path, bitmap_type } = &t.font_type {
                    assert_eq!(path, "fonts/bitmap.fnt");
                    assert_eq!(*bitmap_type, 1);
                } else {
                    panic!("Expected Bitmap font type");
                }
            }
            _ => panic!("Expected Text object"),
        }
    }

    #[test]
    fn test_font_resolution_default_when_missing() {
        let json = r#"{
            "type": 6,
            "name": "Test",
            "text": [{"id": "title", "font": 99, "size": 24, "ref": 12}],
            "destination": [
                {"id": "title", "dst": [{"x": 0, "y": 0, "w": 200, "h": 30}]}
            ]
        }"#;
        let skin = load_skin(json, &HashSet::new(), Resolution::Hd, None).unwrap();
        match &skin.objects[0] {
            SkinObjectType::Text(t) => {
                assert!(matches!(t.font_type, FontType::Default));
            }
            _ => panic!("Expected Text object"),
        }
    }

    // -- Real skin loading (ECFN) --

    #[test]
    fn test_load_ecfn_select_skin_no_crash() {
        let skin_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("skins")
            .join("ECFN")
            .join("select")
            .join("select.json");

        if !skin_path.exists() {
            eprintln!("ECFN skin not found, skipping: {}", skin_path.display());
            return;
        }

        let json_str = std::fs::read_to_string(&skin_path).unwrap();

        // load_skin (no images) must not crash even with missing source images
        let skin = load_skin(&json_str, &HashSet::new(), Resolution::Hd, Some(&skin_path)).unwrap();

        assert_eq!(skin.header.name, "beatoraja_default");
        assert!(skin.object_count() > 0);

        // load_skin_with_images with empty map also must not crash
        let skin2 = load_skin_with_images(
            &json_str,
            &HashSet::new(),
            Resolution::Hd,
            Some(&skin_path),
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(skin.object_count(), skin2.object_count());
    }

    #[test]
    fn test_load_skin_with_images_missing_sources_graceful() {
        let json = r#"{
            "type": 6,
            "name": "Test Missing Sources",
            "source": [
                {"id": 0, "path": "nonexistent.png"},
                {"id": 1, "path": "also_missing.png"}
            ],
            "image": [
                {"id": "img_a", "src": 0},
                {"id": "img_b", "src": 1}
            ],
            "slider": [
                {"id": "sl", "src": 0, "angle": 1, "range": 50, "type": 17}
            ],
            "graph": [
                {"id": "gr", "src": 1, "angle": 1, "type": 100}
            ],
            "destination": [
                {"id": "img_a", "dst": [{"x": 0, "y": 0, "w": 100, "h": 100}]},
                {"id": "img_b", "dst": [{"x": 0, "y": 0, "w": 100, "h": 100}]},
                {"id": "sl", "dst": [{"x": 0, "y": 0, "w": 10, "h": 10}]},
                {"id": "gr", "dst": [{"x": 0, "y": 0, "w": 100, "h": 10}]}
            ]
        }"#;

        // Empty source_images map — all sources are "missing"
        let skin =
            load_skin_with_images(json, &HashSet::new(), Resolution::Hd, None, &HashMap::new())
                .unwrap();

        // All 4 objects should still be created (just with empty source images)
        assert_eq!(skin.object_count(), 4);

        // Image objects should have empty sources
        match &skin.objects[0] {
            SkinObjectType::Image(img) => assert!(img.sources.is_empty()),
            _ => panic!("Expected Image"),
        }

        // Slider should have empty source_images
        match &skin.objects[2] {
            SkinObjectType::Slider(sl) => assert!(sl.source_images.is_empty()),
            _ => panic!("Expected Slider"),
        }

        // Graph should have empty source_images
        match &skin.objects[3] {
            SkinObjectType::Graph(gr) => assert!(gr.source_images.is_empty()),
            _ => panic!("Expected Graph"),
        }
    }

    // -- JSON pre-processing --

    #[test]
    fn test_preprocess_trailing_comma() {
        let input = r#"{"a": [1, 2, 3, ], "b": {"x": 1, }}"#;
        let output = preprocess_json(input);
        assert!(serde_json::from_str::<Value>(&output).is_ok());
    }

    #[test]
    fn test_preprocess_missing_comma() {
        let input = r#"[{"id": "a"} {"id": "b"}]"#;
        let output = preprocess_json(input);
        let parsed: Vec<Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_preprocess_both_issues() {
        let input = r#"{"items": [{"x": 1,} {"y": 2}]}"#;
        let output = preprocess_json(input);
        assert!(serde_json::from_str::<Value>(&output).is_ok());
    }

    #[test]
    fn test_preprocess_preserves_strings() {
        let input = r#"{"text": "hello} {world", "x": 1}"#;
        let output = preprocess_json(input);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["text"], "hello} {world");
    }

    #[test]
    fn test_preprocess_valid_json_unchanged() {
        let input = r#"{"a": [1, 2], "b": {"c": 3}}"#;
        let output = preprocess_json(input);
        assert_eq!(
            serde_json::from_str::<Value>(&output).unwrap(),
            serde_json::from_str::<Value>(input).unwrap()
        );
    }
}
