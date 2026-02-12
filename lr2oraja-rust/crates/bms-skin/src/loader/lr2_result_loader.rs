// LR2 Result / CourseResult skin loader.
//
// Handles state-specific commands for result and course result screens:
// - SRC_GAUGECHART_1P / DST_GAUGECHART_1P — Gauge history graph
// - SRC_NOTECHART_1P / DST_NOTECHART_1P — Note distribution graph
// - SRC_BPMCHART / DST_BPMCHART — BPM graph
// - SRC_TIMINGCHART_1P / DST_TIMINGCHART_1P — Timing distribution graph
//
// Ported from LR2ResultSkinLoader.java and LR2CourseResultSkinLoader.java.

use crate::loader::lr2_csv_loader::Lr2CsvState;
use crate::result_skin::{CourseResultSkinConfig, ResultSkinConfig};
use crate::skin::Skin;
use crate::skin_bpm_graph::{BpmGraphColors, SkinBpmGraph, parse_hex_color};
use crate::skin_gauge_graph::SkinGaugeGraph;
use crate::skin_object::Rect;
use crate::skin_visualizer::{
    NoteDistributionType, SkinNoteDistributionGraph, SkinTimingDistributionGraph, parse_color,
};

// ---------------------------------------------------------------------------
// Result state
// ---------------------------------------------------------------------------

/// Internal state for result skin loading.
#[derive(Default)]
pub struct Lr2ResultState {
    /// Gauge graph rectangle from SRC command (reserved for future use).
    _gauge_rect: Option<Rect>,
    /// Index of gauge graph object in skin.objects.
    gauge_obj_idx: Option<usize>,
    /// Index of note chart object in skin.objects.
    note_obj_idx: Option<usize>,
    /// Index of BPM chart object in skin.objects.
    bpm_obj_idx: Option<usize>,
    /// Index of timing chart object in skin.objects.
    timing_obj_idx: Option<usize>,
}

// ---------------------------------------------------------------------------
// Result commands
// ---------------------------------------------------------------------------

/// Processes a result-screen specific LR2 command.
///
/// Returns true if the command was handled.
pub fn process_result_command(
    cmd: &str,
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    result_state: &mut Lr2ResultState,
) -> bool {
    match cmd {
        "SRC_GAUGECHART_1P" => {
            src_gauge_chart(fields, skin, result_state);
            true
        }
        "DST_GAUGECHART_1P" => {
            dst_gauge_chart(fields, skin, state, result_state);
            true
        }
        "SRC_NOTECHART_1P" => {
            src_note_chart(fields, skin, result_state);
            true
        }
        "DST_NOTECHART_1P" => {
            dst_note_chart(fields, skin, state, result_state);
            true
        }
        "SRC_BPMCHART" => {
            src_bpm_chart(fields, skin, result_state);
            true
        }
        "DST_BPMCHART" => {
            dst_bpm_chart(fields, skin, state, result_state);
            true
        }
        "SRC_TIMINGCHART_1P" => {
            src_timing_chart(fields, skin, result_state);
            true
        }
        "DST_TIMINGCHART_1P" => {
            dst_timing_chart(fields, skin, state, result_state);
            true
        }
        _ => false,
    }
}

/// Processes a course-result-screen specific LR2 command.
///
/// Returns true if the command was handled.
pub fn process_course_result_command(
    cmd: &str,
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    result_state: &mut Lr2ResultState,
) -> bool {
    match cmd {
        "SRC_GAUGECHART_1P" => {
            src_gauge_chart(fields, skin, result_state);
            true
        }
        "DST_GAUGECHART_1P" => {
            dst_gauge_chart(fields, skin, state, result_state);
            true
        }
        "SRC_NOTECHART_1P" => {
            src_note_chart(fields, skin, result_state);
            true
        }
        "DST_NOTECHART_1P" => {
            dst_note_chart(fields, skin, state, result_state);
            true
        }
        _ => false,
    }
}

/// Collects result state into ResultSkinConfig after loading completes.
pub fn collect_result_config(
    skin: &Skin,
    result_state: &Lr2ResultState,
) -> Option<ResultSkinConfig> {
    let gauge_graph = result_state
        .gauge_obj_idx
        .and_then(|idx| extract_gauge_graph(skin, idx));
    let note_graph = result_state
        .note_obj_idx
        .and_then(|idx| extract_note_graph(skin, idx));
    let bpm_graph = result_state
        .bpm_obj_idx
        .and_then(|idx| extract_bpm_graph(skin, idx));
    let timing_graph = result_state
        .timing_obj_idx
        .and_then(|idx| extract_timing_graph(skin, idx));

    if gauge_graph.is_none()
        && note_graph.is_none()
        && bpm_graph.is_none()
        && timing_graph.is_none()
    {
        return None;
    }

    Some(ResultSkinConfig {
        gauge_graph,
        note_graph,
        bpm_graph,
        timing_graph,
    })
}

/// Collects course result state into CourseResultSkinConfig after loading completes.
pub fn collect_course_result_config(
    skin: &Skin,
    result_state: &Lr2ResultState,
) -> Option<CourseResultSkinConfig> {
    let gauge_graph = result_state
        .gauge_obj_idx
        .and_then(|idx| extract_gauge_graph(skin, idx));
    let note_graph = result_state
        .note_obj_idx
        .and_then(|idx| extract_note_graph(skin, idx));

    if gauge_graph.is_none() && note_graph.is_none() {
        return None;
    }

    Some(CourseResultSkinConfig {
        gauge_graph,
        note_graph,
    })
}

// ---------------------------------------------------------------------------
// SRC handlers
// ---------------------------------------------------------------------------

fn src_gauge_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let line_width = if values[6] != 0 { values[6] } else { 2 };
    let start = values[13];
    let end = if values[14] != 0 { values[14] } else { 1500 };
    let graph = SkinGaugeGraph {
        line_width,
        delay: end - start,
        ..Default::default()
    };
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.gauge_obj_idx = Some(idx);
}

fn src_note_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let graph = SkinNoteDistributionGraph {
        graph_type: NoteDistributionType::from_i32(values[1]),
        delay: if values[15] != 0 { values[15] } else { 500 },
        back_tex_off: values[16] != 0,
        order_reverse: values[17] != 0,
        no_gap: values[18] != 0,
        no_gap_x: values[19] != 0,
        ..Default::default()
    };
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.note_obj_idx = Some(idx);
}

fn src_bpm_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let mut colors = BpmGraphColors::default();
    if let Some(c) = fields.get(5).and_then(|s| parse_hex_color(s)) {
        colors.main_bpm = c;
    }
    if let Some(c) = fields.get(6).and_then(|s| parse_hex_color(s)) {
        colors.min_bpm = c;
    }
    if let Some(c) = fields.get(7).and_then(|s| parse_hex_color(s)) {
        colors.max_bpm = c;
    }
    if let Some(c) = fields.get(8).and_then(|s| parse_hex_color(s)) {
        colors.other_bpm = c;
    }
    if let Some(c) = fields.get(9).and_then(|s| parse_hex_color(s)) {
        colors.stop = c;
    }
    if let Some(c) = fields.get(10).and_then(|s| parse_hex_color(s)) {
        colors.transition = c;
    }
    let graph = SkinBpmGraph::new(values[3], values[4], colors);
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.bpm_obj_idx = Some(idx);
}

fn src_timing_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let graph_color = fields.get(7).map(|s| parse_color(s)).unwrap_or_default();
    let average_color = fields.get(8).map(|s| parse_color(s)).unwrap_or_default();
    let dev_color = fields.get(9).map(|s| parse_color(s)).unwrap_or_default();
    let pg = fields.get(10).map(|s| parse_color(s)).unwrap_or_default();
    let gr = fields.get(11).map(|s| parse_color(s)).unwrap_or_default();
    let gd = fields.get(12).map(|s| parse_color(s)).unwrap_or_default();
    let bd = fields.get(13).map(|s| parse_color(s)).unwrap_or_default();
    let pr = fields.get(14).map(|s| parse_color(s)).unwrap_or_default();
    let graph = SkinTimingDistributionGraph {
        graph_width: values[4],
        line_width: values[6],
        draw_average: values[15] != 0,
        draw_dev: values[16] != 0,
        graph_color,
        average_color,
        dev_color,
        judge_colors: [pg, gr, gd, bd, pr],
        ..Default::default()
    };
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.timing_obj_idx = Some(idx);
}

// ---------------------------------------------------------------------------
// DST handlers
// ---------------------------------------------------------------------------

fn dst_gauge_chart(
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    result_state: &mut Lr2ResultState,
) {
    if let Some(idx) = result_state.gauge_obj_idx {
        state.apply_dst_to(idx, fields, skin);
    }
}

fn dst_note_chart(
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    result_state: &mut Lr2ResultState,
) {
    if let Some(idx) = result_state.note_obj_idx {
        state.apply_dst_to(idx, fields, skin);
    }
}

fn dst_bpm_chart(
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    result_state: &mut Lr2ResultState,
) {
    if let Some(idx) = result_state.bpm_obj_idx {
        state.apply_dst_to(idx, fields, skin);
    }
}

fn dst_timing_chart(
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    result_state: &mut Lr2ResultState,
) {
    if let Some(idx) = result_state.timing_obj_idx {
        state.apply_dst_to(idx, fields, skin);
    }
}

// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

fn extract_gauge_graph(skin: &Skin, idx: usize) -> Option<SkinGaugeGraph> {
    skin.objects.get(idx).and_then(|obj| {
        if let crate::skin_object_type::SkinObjectType::GaugeGraph(g) = obj {
            Some(g.clone())
        } else {
            None
        }
    })
}

fn extract_note_graph(skin: &Skin, idx: usize) -> Option<SkinNoteDistributionGraph> {
    skin.objects.get(idx).and_then(|obj| {
        if let crate::skin_object_type::SkinObjectType::NoteDistributionGraph(g) = obj {
            Some(g.clone())
        } else {
            None
        }
    })
}

fn extract_bpm_graph(skin: &Skin, idx: usize) -> Option<SkinBpmGraph> {
    skin.objects.get(idx).and_then(|obj| {
        if let crate::skin_object_type::SkinObjectType::BpmGraph(g) = obj {
            Some(g.clone())
        } else {
            None
        }
    })
}

fn extract_timing_graph(skin: &Skin, idx: usize) -> Option<SkinTimingDistributionGraph> {
    skin.objects.get(idx).and_then(|obj| {
        if let crate::skin_object_type::SkinObjectType::TimingDistributionGraph(g) = obj {
            Some(g.clone())
        } else {
            None
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin_header::SkinHeader;
    use bms_config::resolution::Resolution;
    use std::collections::HashMap;

    fn make_skin() -> (Skin, Lr2CsvState) {
        let mut header = SkinHeader::default();
        header.resolution = Resolution::Sd;
        header.source_resolution = Some(Resolution::Sd);
        header.destination_resolution = Some(Resolution::Hd);
        let skin = Skin::new(header);
        let state = Lr2CsvState::new(Resolution::Sd, Resolution::Hd, &HashMap::new());
        (skin, state)
    }

    #[test]
    fn test_result_gauge_chart_src_dst() {
        let (mut skin, mut state) = make_skin();
        let mut rs = Lr2ResultState::default();

        let src_fields: Vec<&str> = "#SRC_GAUGECHART_1P,0,0,0,0,200,100,1,1,0,0"
            .split(',')
            .collect();
        let dst_fields: Vec<&str> =
            "#DST_GAUGECHART_1P,0,0,100,50,200,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
                .split(',')
                .collect();

        assert!(process_result_command(
            "SRC_GAUGECHART_1P",
            &src_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));
        assert!(rs.gauge_obj_idx.is_some());

        assert!(process_result_command(
            "DST_GAUGECHART_1P",
            &dst_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));

        let config = collect_result_config(&skin, &rs);
        assert!(config.is_some());
        assert!(config.unwrap().gauge_graph.is_some());
    }

    #[test]
    fn test_result_all_charts() {
        let (mut skin, mut state) = make_skin();
        let mut rs = Lr2ResultState::default();

        let src = "#SRC,0,0,0,0,200,100,1,1,0,0";
        let dst = "#DST,0,0,100,50,200,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0";
        let src_fields: Vec<&str> = src.split(',').collect();
        let dst_fields: Vec<&str> = dst.split(',').collect();

        for cmd in [
            "SRC_GAUGECHART_1P",
            "SRC_NOTECHART_1P",
            "SRC_BPMCHART",
            "SRC_TIMINGCHART_1P",
        ] {
            process_result_command(cmd, &src_fields, &mut skin, &mut state, &mut rs);
        }
        for cmd in [
            "DST_GAUGECHART_1P",
            "DST_NOTECHART_1P",
            "DST_BPMCHART",
            "DST_TIMINGCHART_1P",
        ] {
            process_result_command(cmd, &dst_fields, &mut skin, &mut state, &mut rs);
        }

        let config = collect_result_config(&skin, &rs).unwrap();
        assert!(config.gauge_graph.is_some());
        assert!(config.note_graph.is_some());
        assert!(config.bpm_graph.is_some());
        assert!(config.timing_graph.is_some());
    }

    #[test]
    fn test_course_result_commands() {
        let (mut skin, mut state) = make_skin();
        let mut rs = Lr2ResultState::default();

        let src_fields: Vec<&str> = "#SRC,0,0,0,0,200,100,1,1,0,0".split(',').collect();
        let dst_fields: Vec<&str> = "#DST,0,0,100,50,200,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
            .split(',')
            .collect();

        // Course result only handles gauge and note charts
        assert!(process_course_result_command(
            "SRC_GAUGECHART_1P",
            &src_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));
        assert!(process_course_result_command(
            "DST_GAUGECHART_1P",
            &dst_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));
        assert!(process_course_result_command(
            "SRC_NOTECHART_1P",
            &src_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));
        assert!(process_course_result_command(
            "DST_NOTECHART_1P",
            &dst_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));

        // BPM and timing charts are not handled
        assert!(!process_course_result_command(
            "SRC_BPMCHART",
            &src_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));
        assert!(!process_course_result_command(
            "SRC_TIMINGCHART_1P",
            &src_fields,
            &mut skin,
            &mut state,
            &mut rs
        ));

        let config = collect_course_result_config(&skin, &rs).unwrap();
        assert!(config.gauge_graph.is_some());
        assert!(config.note_graph.is_some());
    }

    #[test]
    fn test_unhandled_command_returns_false() {
        let (mut skin, mut state) = make_skin();
        let mut rs = Lr2ResultState::default();
        let fields: Vec<&str> = vec!["#UNKNOWN"];

        assert!(!process_result_command(
            "UNKNOWN", &fields, &mut skin, &mut state, &mut rs
        ));
        assert!(!process_course_result_command(
            "UNKNOWN", &fields, &mut skin, &mut state, &mut rs
        ));
    }

    #[test]
    fn test_empty_result_returns_none() {
        let (skin, _) = make_skin();
        let rs = Lr2ResultState::default();
        assert!(collect_result_config(&skin, &rs).is_none());
        assert!(collect_course_result_config(&skin, &rs).is_none());
    }

    // -- Field parsing tests --

    #[test]
    fn test_gauge_chart_fields() {
        let (mut skin, _) = make_skin();
        let mut rs = Lr2ResultState::default();

        // values[6]=3 (lineWidth), values[13]=100 (start), values[14]=1600 (end)
        let fields: Vec<&str> = "#SRC,0,0,0,0,0,3,0,0,0,0,0,0,100,1600,0,0,0,0,0,0,0"
            .split(',')
            .collect();
        src_gauge_chart(&fields, &mut skin, &mut rs);

        let graph = extract_gauge_graph(&skin, rs.gauge_obj_idx.unwrap()).unwrap();
        assert_eq!(graph.line_width, 3);
        assert_eq!(graph.delay, 1500); // 1600 - 100
    }

    #[test]
    fn test_gauge_chart_defaults() {
        let (mut skin, _) = make_skin();
        let mut rs = Lr2ResultState::default();

        // Minimal fields — line_width=0 defaults to 2, end=0 defaults to 1500
        let fields: Vec<&str> = "#SRC,0,0,0,0,0,0,0,0,0,0".split(',').collect();
        src_gauge_chart(&fields, &mut skin, &mut rs);

        let graph = extract_gauge_graph(&skin, rs.gauge_obj_idx.unwrap()).unwrap();
        assert_eq!(graph.line_width, 2);
        assert_eq!(graph.delay, 1500);
    }

    #[test]
    fn test_note_chart_fields() {
        let (mut skin, _) = make_skin();
        let mut rs = Lr2ResultState::default();

        // values[1]=1 (Judge type), values[15]=800 (delay), values[16]=1, values[17]=1,
        // values[18]=1, values[19]=1
        let fields: Vec<&str> = "#SRC,1,0,0,0,0,0,0,0,0,0,0,0,0,0,800,1,1,1,1,0,0"
            .split(',')
            .collect();
        src_note_chart(&fields, &mut skin, &mut rs);

        let graph = extract_note_graph(&skin, rs.note_obj_idx.unwrap()).unwrap();
        assert_eq!(graph.graph_type, NoteDistributionType::Judge);
        assert_eq!(graph.delay, 800);
        assert!(graph.back_tex_off);
        assert!(graph.order_reverse);
        assert!(graph.no_gap);
        assert!(graph.no_gap_x);
    }

    #[test]
    fn test_bpm_chart_fields() {
        let (mut skin, _) = make_skin();
        let mut rs = Lr2ResultState::default();

        // values[3]=500 (delay), values[4]=3 (lineWidth)
        // str[5..10] = hex colors
        let fields: Vec<&str> = "#SRC,0,0,500,3,00FF00,0000FF,FF0000,FFFF00,FF00FF,7F7F7F"
            .split(',')
            .collect();
        src_bpm_chart(&fields, &mut skin, &mut rs);

        let graph = extract_bpm_graph(&skin, rs.bpm_obj_idx.unwrap()).unwrap();
        assert_eq!(graph.delay, 500);
        assert_eq!(graph.line_width, 3);
        // main_bpm should be green
        assert!((graph.colors.main_bpm.g - 1.0).abs() < 0.01);
        // min_bpm should be blue
        assert!((graph.colors.min_bpm.b - 1.0).abs() < 0.01);
        // max_bpm should be red
        assert!((graph.colors.max_bpm.r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_timing_chart_fields() {
        let (mut skin, _) = make_skin();
        let mut rs = Lr2ResultState::default();

        // values[4]=200 (width), values[6]=2 (lineWidth),
        // str[7..14] = colors, values[15]=1 (drawAverage), values[16]=1 (drawDev)
        let fields: Vec<&str> =
            "#SRC,0,0,0,200,0,2,FFFFFF,00FF00,0000FF,FF0000,00FF00,FFFF00,FF00FF,7F7F7F,1,1"
                .split(',')
                .collect();
        src_timing_chart(&fields, &mut skin, &mut rs);

        let graph = extract_timing_graph(&skin, rs.timing_obj_idx.unwrap()).unwrap();
        assert_eq!(graph.graph_width, 200);
        assert_eq!(graph.line_width, 2);
        assert!(graph.draw_average);
        assert!(graph.draw_dev);
        // graph_color should be white
        assert!((graph.graph_color.r - 1.0).abs() < 0.01);
        assert!((graph.graph_color.g - 1.0).abs() < 0.01);
        // PG color should be red
        assert!((graph.judge_colors[0].r - 1.0).abs() < 0.01);
    }
}
