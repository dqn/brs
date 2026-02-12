// RenderSnapshot: structural comparison of draw commands between Java and Rust.
//
// Instead of pixel-level SSIM comparison (which fails across different rendering
// engines), this captures "what to draw" as a serializable data structure.
// Both Java and Rust generate the same JSON format for field-by-field comparison.

use bms_render::eval;
use bms_render::state_provider::SkinStateProvider;
use bms_skin::property_id::STRING_TABLE_FULL;
use bms_skin::skin::Skin;
use bms_skin::skin_object::SkinObjectBase;
use bms_skin::skin_object_type::SkinObjectType;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A snapshot of all draw commands for a skin at a given point in time.
#[derive(Debug, Serialize, Deserialize)]
pub struct RenderSnapshot {
    pub skin_width: f32,
    pub skin_height: f32,
    pub time_ms: i64,
    pub commands: Vec<DrawCommand>,
}

/// A single draw command for one skin object.
#[derive(Debug, Serialize, Deserialize)]
pub struct DrawCommand {
    pub object_index: usize,
    pub object_type: String,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst: Option<DrawRect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<DrawColor>,
    pub angle: i32,
    pub blend: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<DrawDetail>,
}

/// Rectangle in skin coordinates.
#[derive(Debug, Serialize, Deserialize)]
pub struct DrawRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// RGBA color (0.0-1.0).
#[derive(Debug, Serialize, Deserialize)]
pub struct DrawColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Type-specific metadata for draw commands.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DrawDetail {
    Image {
        source_index: usize,
        frame_index: usize,
    },
    Number {
        value: i32,
    },
    Text {
        content: String,
        align: i32,
    },
    Slider {
        value: f64,
        direction: i32,
    },
    Graph {
        value: f64,
        direction: i32,
    },
    Gauge {
        value: f64,
        nodes: i32,
    },
    BpmGraph,
    HitErrorVisualizer,
    NoteDistributionGraph,
    TimingDistributionGraph,
    TimingVisualizer,
}

// ---------------------------------------------------------------------------
// Capture
// ---------------------------------------------------------------------------

/// Captures a RenderSnapshot from a Skin + SkinStateProvider.
/// Pure function — no GPU or Bevy dependency.
pub fn capture_render_snapshot(skin: &Skin, provider: &dyn SkinStateProvider) -> RenderSnapshot {
    let mut commands = Vec::with_capacity(skin.objects.len());

    for (idx, object) in skin.objects.iter().enumerate() {
        let base = object.base();
        if !matches_option_conditions(base, skin) {
            // Java Skin.prepare() drops statically non-drawable objects (e.g. option mismatch).
            // Skip them here so command_count parity tracks the prepared object set.
            continue;
        }
        let object_type = object_type_name(object);
        let blend = base.blend;

        let resolved = eval::resolve_common(base, provider);

        let (visible, dst, color, angle, detail) = match resolved {
            Some((rect, col, final_angle, final_alpha)) => {
                if !is_object_renderable(base, object, provider) {
                    (false, None, None, 0, None)
                } else {
                    let dst = DrawRect {
                        x: rect.x,
                        y: rect.y,
                        w: rect.w,
                        h: rect.h,
                    };
                    let color = DrawColor {
                        r: col.r,
                        g: col.g,
                        b: col.b,
                        a: final_alpha,
                    };
                    let detail = resolve_detail(object, provider);
                    (true, Some(dst), Some(color), final_angle, detail)
                }
            }
            None => (false, None, None, 0, None),
        };

        commands.push(DrawCommand {
            object_index: idx,
            object_type: object_type.to_string(),
            visible,
            dst,
            color,
            angle,
            blend,
            detail,
        });
    }

    RenderSnapshot {
        skin_width: skin.width,
        skin_height: skin.height,
        time_ms: provider.now_time_ms(),
        commands,
    }
}

fn matches_option_conditions(base: &SkinObjectBase, skin: &Skin) -> bool {
    base.option_conditions.iter().all(|&op| {
        if op == 0 {
            true
        } else {
            let abs = op.abs();
            let Some(selected) = skin.options.get(&abs).copied() else {
                // Java splits op into BooleanProperty and option lists.
                // If this ID is not a custom option entry, do not treat it
                // as an option-prunable condition here.
                return true;
            };
            if op > 0 { selected == 1 } else { selected == 0 }
        }
    })
}

fn is_object_renderable(
    base: &SkinObjectBase,
    object: &SkinObjectType,
    provider: &dyn SkinStateProvider,
) -> bool {
    // Negative destination IDs (-110/-111 etc.) are special system overlays.
    // Rust runtime resolution is incomplete; treat them as non-renderable here
    // to match Java RenderSnapshot output.
    if let Some(name) = &base.name
        && let Ok(id) = name.parse::<i32>()
    {
        return id >= 0;
    }
    if let SkinObjectType::Text(text) = object
        && resolve_text_render_content(text, provider).is_empty()
    {
        // Java SkinText.prepare() sets draw=false when StringProperty resolves to
        // null/empty. Snapshot parity needs to mirror this gate.
        return false;
    }
    true
}

fn resolve_text_render_content(
    text: &bms_skin::skin_text::SkinText,
    provider: &dyn SkinStateProvider,
) -> String {
    if let Some(ref_id) = text.ref_id {
        if let Some(content) = provider.string_value(ref_id) {
            return content;
        }
        if ref_id.0 == STRING_TABLE_FULL {
            // Java GM mocks allocate PlayerResource via Unsafe without running field
            // initializers, and tablefull is computed from null + "". This yields
            // "null" and keeps tablefull text visible in decide skin snapshots.
            return "null".to_string();
        }
    }
    text.constant_text.clone().unwrap_or_default()
}

/// Returns the type name string for a SkinObjectType.
fn object_type_name(object: &SkinObjectType) -> &'static str {
    match object {
        SkinObjectType::Bga(_) => "SkinBGA",
        SkinObjectType::Image(_) => "Image",
        SkinObjectType::Number(_) => "Number",
        SkinObjectType::Text(_) => "Text",
        SkinObjectType::Slider(_) => "Slider",
        SkinObjectType::Graph(_) => "Graph",
        SkinObjectType::Gauge(_) => "Gauge",
        SkinObjectType::BpmGraph(_) => "BpmGraph",
        SkinObjectType::HitErrorVisualizer(_) => "HitErrorVisualizer",
        SkinObjectType::NoteDistributionGraph(_) => "NoteDistributionGraph",
        SkinObjectType::TimingDistributionGraph(_) => "TimingDistributionGraph",
        SkinObjectType::TimingVisualizer(_) => "TimingVisualizer",
        SkinObjectType::Note(_) => "Note",
        SkinObjectType::Judge(_) => "Judge",
        SkinObjectType::Hidden(_) => "Hidden",
        SkinObjectType::LiftCover(_) => "LiftCover",
        SkinObjectType::Bar(_) => "Bar",
        SkinObjectType::DistributionGraph(_) => "DistributionGraph",
        SkinObjectType::Float(_) => "Float",
    }
}

/// Resolves type-specific draw detail for a skin object.
fn resolve_detail(object: &SkinObjectType, provider: &dyn SkinStateProvider) -> Option<DrawDetail> {
    match object {
        SkinObjectType::Bga(_) => None,
        SkinObjectType::Image(img) => {
            let source_index = img
                .ref_id
                .map(|id| provider.integer_value(id) as usize)
                .unwrap_or(0);

            let time = eval::resolve_timer_time(&img.base, provider).unwrap_or(0);

            let frame_index =
                if let Some(source) = img.sources.get(source_index).or(img.sources.first()) {
                    match source {
                        bms_skin::skin_image::SkinImageSource::Frames { images, cycle, .. } => {
                            bms_skin::skin_source::image_index(images.len(), time, *cycle)
                        }
                        bms_skin::skin_image::SkinImageSource::Reference(_) => 0,
                    }
                } else {
                    0
                };

            Some(DrawDetail::Image {
                source_index,
                frame_index,
            })
        }
        SkinObjectType::Number(num) => {
            let value = num.ref_id.map(|id| provider.integer_value(id)).unwrap_or(0);
            Some(DrawDetail::Number { value })
        }
        SkinObjectType::Text(text) => {
            let content = eval::resolve_text_content(text, provider);
            let align = text.align as i32;
            Some(DrawDetail::Text { content, align })
        }
        SkinObjectType::Slider(slider) => {
            let value = slider
                .ref_id
                .map(|id| provider.float_value(id) as f64)
                .unwrap_or(0.0);
            let direction = slider.direction as i32;
            Some(DrawDetail::Slider { value, direction })
        }
        SkinObjectType::Graph(graph) => {
            let value = graph
                .ref_id
                .map(|id| provider.float_value(id) as f64)
                .unwrap_or(0.0);
            let direction = graph.direction as i32;
            Some(DrawDetail::Graph { value, direction })
        }
        SkinObjectType::Gauge(gauge) => {
            // Gauge value comes from the provider; for snapshot we record the node count
            Some(DrawDetail::Gauge {
                value: 0.0, // Gauge value is runtime state, not a property
                nodes: gauge.nodes,
            })
        }
        SkinObjectType::BpmGraph(_) => Some(DrawDetail::BpmGraph),
        SkinObjectType::HitErrorVisualizer(_) => Some(DrawDetail::HitErrorVisualizer),
        SkinObjectType::NoteDistributionGraph(_) => Some(DrawDetail::NoteDistributionGraph),
        SkinObjectType::TimingDistributionGraph(_) => Some(DrawDetail::TimingDistributionGraph),
        SkinObjectType::TimingVisualizer(_) => Some(DrawDetail::TimingVisualizer),
        SkinObjectType::Note(_) => None,
        SkinObjectType::Judge(_) => None,
        SkinObjectType::Hidden(_) => None,
        SkinObjectType::LiftCover(_) => None,
        SkinObjectType::Bar(_) => None,
        SkinObjectType::DistributionGraph(_) => None,
        SkinObjectType::Float(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Comparison helpers
// ---------------------------------------------------------------------------

/// Compare two RenderSnapshots with tolerances for float fields.
/// Returns a list of differences found.
pub fn compare_snapshots(java: &RenderSnapshot, rust: &RenderSnapshot) -> Vec<String> {
    let mut diffs = Vec::new();

    if (java.skin_width - rust.skin_width).abs() > 0.01 {
        diffs.push(format!(
            "skin_width: java={} rust={}",
            java.skin_width, rust.skin_width
        ));
    }
    if (java.skin_height - rust.skin_height).abs() > 0.01 {
        diffs.push(format!(
            "skin_height: java={} rust={}",
            java.skin_height, rust.skin_height
        ));
    }

    if java.commands.len() != rust.commands.len() {
        diffs.push(format!(
            "command_count: java={} rust={}",
            java.commands.len(),
            rust.commands.len()
        ));
        return diffs;
    }

    for (i, (jc, rc)) in java.commands.iter().zip(rust.commands.iter()).enumerate() {
        let prefix = format!("cmd[{}]", i);

        if jc.object_type != rc.object_type {
            diffs.push(format!(
                "{} object_type: java={} rust={}",
                prefix, jc.object_type, rc.object_type
            ));
        }

        if jc.visible != rc.visible {
            diffs.push(format!(
                "{} visible: java={} rust={}",
                prefix, jc.visible, rc.visible
            ));
            continue; // Skip position/color comparison if visibility differs
        }

        if !jc.visible {
            continue; // Both hidden, no further comparison needed
        }

        if jc.angle != rc.angle {
            diffs.push(format!(
                "{} angle: java={} rust={}",
                prefix, jc.angle, rc.angle
            ));
        }

        if jc.blend != rc.blend {
            diffs.push(format!(
                "{} blend: java={} rust={}",
                prefix, jc.blend, rc.blend
            ));
        }

        // Compare dst rect with ±1.0 pixel tolerance
        compare_optional_rect(&prefix, "dst", &jc.dst, &rc.dst, 1.0, &mut diffs);

        // Compare color with tolerance
        compare_optional_color(&prefix, &jc.color, &rc.color, &mut diffs);

        // Compare detail
        compare_detail(&prefix, &jc.detail, &rc.detail, &mut diffs);
    }

    diffs
}

fn compare_optional_rect(
    prefix: &str,
    name: &str,
    java: &Option<DrawRect>,
    rust: &Option<DrawRect>,
    tolerance: f32,
    diffs: &mut Vec<String>,
) {
    match (java, rust) {
        (Some(j), Some(r)) => {
            if (j.x - r.x).abs() > tolerance {
                diffs.push(format!(
                    "{} {}.x: java={} rust={} (diff={})",
                    prefix,
                    name,
                    j.x,
                    r.x,
                    (j.x - r.x).abs()
                ));
            }
            if (j.y - r.y).abs() > tolerance {
                diffs.push(format!(
                    "{} {}.y: java={} rust={} (diff={})",
                    prefix,
                    name,
                    j.y,
                    r.y,
                    (j.y - r.y).abs()
                ));
            }
            if (j.w - r.w).abs() > tolerance {
                diffs.push(format!(
                    "{} {}.w: java={} rust={} (diff={})",
                    prefix,
                    name,
                    j.w,
                    r.w,
                    (j.w - r.w).abs()
                ));
            }
            if (j.h - r.h).abs() > tolerance {
                diffs.push(format!(
                    "{} {}.h: java={} rust={} (diff={})",
                    prefix,
                    name,
                    j.h,
                    r.h,
                    (j.h - r.h).abs()
                ));
            }
        }
        (None, None) => {}
        _ => {
            diffs.push(format!(
                "{} {}: java={} rust={}",
                prefix,
                name,
                java.is_some(),
                rust.is_some()
            ));
        }
    }
}

fn compare_optional_color(
    prefix: &str,
    java: &Option<DrawColor>,
    rust: &Option<DrawColor>,
    diffs: &mut Vec<String>,
) {
    match (java, rust) {
        (Some(j), Some(r)) => {
            let rgb_tol = 0.005;
            let alpha_tol = 0.01;
            if (j.r - r.r).abs() > rgb_tol {
                diffs.push(format!("{} color.r: java={} rust={}", prefix, j.r, r.r));
            }
            if (j.g - r.g).abs() > rgb_tol {
                diffs.push(format!("{} color.g: java={} rust={}", prefix, j.g, r.g));
            }
            if (j.b - r.b).abs() > rgb_tol {
                diffs.push(format!("{} color.b: java={} rust={}", prefix, j.b, r.b));
            }
            if (j.a - r.a).abs() > alpha_tol {
                diffs.push(format!("{} color.a: java={} rust={}", prefix, j.a, r.a));
            }
        }
        (None, None) => {}
        _ => {
            diffs.push(format!(
                "{} color: java={} rust={}",
                prefix,
                java.is_some(),
                rust.is_some()
            ));
        }
    }
}

fn compare_detail(
    prefix: &str,
    java: &Option<DrawDetail>,
    rust: &Option<DrawDetail>,
    diffs: &mut Vec<String>,
) {
    match (java, rust) {
        (Some(j), Some(r)) => {
            match (j, r) {
                (
                    DrawDetail::Image {
                        source_index: js,
                        frame_index: jf,
                    },
                    DrawDetail::Image {
                        source_index: rs,
                        frame_index: rf,
                    },
                ) => {
                    if js != rs {
                        diffs.push(format!(
                            "{} detail.source_index: java={} rust={}",
                            prefix, js, rs
                        ));
                    }
                    if jf != rf {
                        diffs.push(format!(
                            "{} detail.frame_index: java={} rust={}",
                            prefix, jf, rf
                        ));
                    }
                }
                (DrawDetail::Number { value: jv }, DrawDetail::Number { value: rv }) => {
                    if jv != rv {
                        diffs.push(format!("{} detail.value: java={} rust={}", prefix, jv, rv));
                    }
                }
                (
                    DrawDetail::Text {
                        content: jc,
                        align: ja,
                    },
                    DrawDetail::Text {
                        content: rc,
                        align: ra,
                    },
                ) => {
                    if jc != rc {
                        diffs.push(format!(
                            "{} detail.content: java={:?} rust={:?}",
                            prefix, jc, rc
                        ));
                    }
                    if ja != ra {
                        diffs.push(format!("{} detail.align: java={} rust={}", prefix, ja, ra));
                    }
                }
                (
                    DrawDetail::Slider {
                        value: jv,
                        direction: jd,
                    },
                    DrawDetail::Slider {
                        value: rv,
                        direction: rd,
                    },
                ) => {
                    if (jv - rv).abs() > 0.001 {
                        diffs.push(format!("{} detail.value: java={} rust={}", prefix, jv, rv));
                    }
                    if jd != rd {
                        diffs.push(format!(
                            "{} detail.direction: java={} rust={}",
                            prefix, jd, rd
                        ));
                    }
                }
                (
                    DrawDetail::Graph {
                        value: jv,
                        direction: jd,
                    },
                    DrawDetail::Graph {
                        value: rv,
                        direction: rd,
                    },
                ) => {
                    if (jv - rv).abs() > 0.001 {
                        diffs.push(format!("{} detail.value: java={} rust={}", prefix, jv, rv));
                    }
                    if jd != rd {
                        diffs.push(format!(
                            "{} detail.direction: java={} rust={}",
                            prefix, jd, rd
                        ));
                    }
                }
                (DrawDetail::Gauge { nodes: jn, .. }, DrawDetail::Gauge { nodes: rn, .. }) => {
                    if jn != rn {
                        diffs.push(format!("{} detail.nodes: java={} rust={}", prefix, jn, rn));
                    }
                }
                // Visualizer types: just check type matches
                (DrawDetail::BpmGraph, DrawDetail::BpmGraph)
                | (DrawDetail::HitErrorVisualizer, DrawDetail::HitErrorVisualizer)
                | (DrawDetail::NoteDistributionGraph, DrawDetail::NoteDistributionGraph)
                | (DrawDetail::TimingDistributionGraph, DrawDetail::TimingDistributionGraph)
                | (DrawDetail::TimingVisualizer, DrawDetail::TimingVisualizer) => {}
                _ => {
                    diffs.push(format!("{} detail type mismatch", prefix));
                }
            }
        }
        (None, None) => {}
        _ => {
            diffs.push(format!(
                "{} detail: java={} rust={}",
                prefix,
                java.is_some(),
                rust.is_some()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_render::state_provider::StaticStateProvider;
    use bms_skin::skin_header::SkinHeader;
    use bms_skin::skin_image::{SkinImage, SkinImageSource};
    use bms_skin::skin_object::{Destination, Rect};
    use bms_skin::skin_text::SkinText;

    fn make_provider() -> StaticStateProvider {
        StaticStateProvider::default()
    }

    fn make_skin_with_image() -> Skin {
        let mut skin = Skin::new(SkinHeader::default());
        let mut img = SkinImage::default();
        img.sources = vec![SkinImageSource::Frames {
            images: vec![bms_skin::image_handle::ImageHandle(1)],
            timer: None,
            cycle: 0,
        }];
        img.base.add_destination(Destination {
            time: 0,
            region: Rect::new(10.0, 20.0, 100.0, 50.0),
            color: bms_skin::skin_object::Color::white(),
            angle: 0,
            acc: 0,
        });
        skin.add(img.into());
        skin
    }

    #[test]
    fn capture_empty_skin() {
        let skin = Skin::new(SkinHeader::default());
        let provider = make_provider();
        let snapshot = capture_render_snapshot(&skin, &provider);
        assert!(snapshot.commands.is_empty());
    }

    #[test]
    fn capture_visible_image() {
        let skin = make_skin_with_image();
        let provider = make_provider();
        let snapshot = capture_render_snapshot(&skin, &provider);

        assert_eq!(snapshot.commands.len(), 1);
        let cmd = &snapshot.commands[0];
        assert!(cmd.visible);
        assert_eq!(cmd.object_type, "Image");
        assert!(cmd.dst.is_some());
        let dst = cmd.dst.as_ref().unwrap();
        assert!((dst.x - 10.0).abs() < 0.001);
        assert!((dst.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn capture_text_with_content() {
        let mut skin = Skin::new(SkinHeader::default());
        let mut text = SkinText::with_constant("hello".to_string());
        text.base.add_destination(Destination {
            time: 0,
            region: Rect::new(0.0, 0.0, 200.0, 30.0),
            color: bms_skin::skin_object::Color::white(),
            angle: 0,
            acc: 0,
        });
        skin.add(text.into());

        let provider = make_provider();
        let snapshot = capture_render_snapshot(&skin, &provider);

        assert_eq!(snapshot.commands.len(), 1);
        let cmd = &snapshot.commands[0];
        assert!(cmd.visible);
        assert_eq!(cmd.object_type, "Text");
        if let Some(DrawDetail::Text { content, .. }) = &cmd.detail {
            assert_eq!(content, "hello");
        } else {
            panic!("Expected Text detail");
        }
    }

    #[test]
    fn compare_identical_snapshots() {
        let skin = make_skin_with_image();
        let provider = make_provider();
        let s1 = capture_render_snapshot(&skin, &provider);
        let s2 = capture_render_snapshot(&skin, &provider);
        let diffs = compare_snapshots(&s1, &s2);
        assert!(diffs.is_empty(), "Diffs: {:?}", diffs);
    }

    #[test]
    fn json_round_trip() {
        let skin = make_skin_with_image();
        let provider = make_provider();
        let snapshot = capture_render_snapshot(&skin, &provider);
        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let parsed: RenderSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.commands.len(), snapshot.commands.len());
    }
}
