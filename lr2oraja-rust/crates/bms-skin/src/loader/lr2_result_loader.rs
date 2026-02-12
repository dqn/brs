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
use crate::skin_bpm_graph::SkinBpmGraph;
use crate::skin_gauge_graph::SkinGaugeGraph;
use crate::skin_object::Rect;
use crate::skin_visualizer::{SkinNoteDistributionGraph, SkinTimingDistributionGraph};

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
    let _values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let graph = SkinGaugeGraph::default();
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.gauge_obj_idx = Some(idx);
}

fn src_note_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let _values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let graph = SkinNoteDistributionGraph::default();
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.note_obj_idx = Some(idx);
}

fn src_bpm_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let _values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let graph = SkinBpmGraph::default();
    let idx = skin.objects.len();
    skin.add(graph.into());
    result_state.bpm_obj_idx = Some(idx);
}

fn src_timing_chart(fields: &[&str], skin: &mut Skin, result_state: &mut Lr2ResultState) {
    let _values = crate::loader::lr2_csv_loader::parse_int_pub(fields);
    let graph = SkinTimingDistributionGraph::default();
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
}
