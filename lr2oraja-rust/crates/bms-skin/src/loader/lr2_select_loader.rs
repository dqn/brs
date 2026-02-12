// LR2 Select skin loader.
//
// Handles state-specific commands for the music selection screen:
// - SRC_BAR_BODY / DST_BAR_BODY_OFF / DST_BAR_BODY_ON — Bar body images
// - BAR_CENTER / BAR_AVAILABLE — Bar positioning
// - SRC_BAR_LAMP / DST_BAR_LAMP — Clear lamps
// - SRC_BAR_MY_LAMP / DST_BAR_MY_LAMP — Player lamps
// - SRC_BAR_RIVAL_LAMP / DST_BAR_RIVAL_LAMP — Rival lamps
// - SRC_BAR_LEVEL / DST_BAR_LEVEL — Level display
// - SRC_BAR_TROPHY / DST_BAR_TROPHY — Trophy icons
// - SRC_BAR_LABEL / DST_BAR_LABEL — Label images
// - SRC_BAR_TITLE / DST_BAR_TITLE — Title text
// - SRC_NOTECHART / DST_NOTECHART — Note distribution chart
// - SRC_BPMCHART / DST_BPMCHART — BPM chart
// - SRC_BAR_FLASH / SRC_BAR_RANK / SRC_README — Stubs
//
// Ported from LR2SelectSkinLoader.java.

use crate::loader::lr2_csv_loader::{Lr2CsvState, parse_field};
use crate::music_select_skin::MusicSelectSkinConfig;
use crate::skin::Skin;
use crate::skin_bar::SkinBar;
use crate::skin_bpm_graph::SkinBpmGraph;
use crate::skin_distribution_graph::SkinDistributionGraph;

// ---------------------------------------------------------------------------
// Select state
// ---------------------------------------------------------------------------

/// Internal state for select skin loading.
#[derive(Default)]
pub struct Lr2SelectState {
    /// The song bar being constructed.
    pub skinbar: SkinBar,
    /// Current bar image source index being loaded.
    bar_src_idx: i32,
    /// Current lamp source index.
    lamp_src_idx: i32,
    /// Current player lamp source index.
    my_lamp_src_idx: i32,
    /// Current rival lamp source index.
    rival_lamp_src_idx: i32,
    /// Current level source index.
    level_src_idx: i32,
    /// Current trophy source index.
    trophy_src_idx: i32,
    /// Current label source index.
    label_src_idx: i32,
    /// Note chart object index.
    note_chart_idx: Option<usize>,
    /// BPM chart object index.
    bpm_chart_idx: Option<usize>,
}

// ---------------------------------------------------------------------------
// Command dispatch
// ---------------------------------------------------------------------------

/// Processes a select-screen specific LR2 command.
///
/// Returns true if the command was handled.
pub fn process_select_command(
    cmd: &str,
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    select_state: &mut Lr2SelectState,
) -> bool {
    match cmd {
        // Bar body
        "SRC_BAR_BODY" => {
            select_state.bar_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_BODY_OFF" => {
            let pos = parse_field(fields, 1);
            if (0..60).contains(&pos) {
                // Store position in bar
                select_state.skinbar.position = parse_field(fields, 1);
            }
            true
        }
        "DST_BAR_BODY_ON" => {
            // On-state bar body positioning (stored as-is)
            true
        }
        "BAR_CENTER" => {
            select_state.skinbar.position = parse_field(fields, 1);
            true
        }
        "BAR_AVAILABLE" => {
            // Bar availability count — informational only
            true
        }

        // Lamps
        "SRC_BAR_LAMP" => {
            select_state.lamp_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_LAMP" => true,
        "SRC_BAR_MY_LAMP" => {
            select_state.my_lamp_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_MY_LAMP" => true,
        "SRC_BAR_RIVAL_LAMP" => {
            select_state.rival_lamp_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_RIVAL_LAMP" => true,

        // Level
        "SRC_BAR_LEVEL" => {
            select_state.level_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_LEVEL" => true,

        // Trophy
        "SRC_BAR_TROPHY" => {
            select_state.trophy_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_TROPHY" => true,

        // Label
        "SRC_BAR_LABEL" => {
            select_state.label_src_idx = parse_field(fields, 2);
            true
        }
        "DST_BAR_LABEL" => true,

        // Title text
        "SRC_BAR_TITLE" => true,
        "DST_BAR_TITLE" => true,

        // Charts
        "SRC_NOTECHART" => {
            let graph = SkinDistributionGraph::default();
            let idx = skin.objects.len();
            skin.add(graph.into());
            select_state.note_chart_idx = Some(idx);
            true
        }
        "DST_NOTECHART" => {
            if let Some(idx) = select_state.note_chart_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }
        "SRC_BPMCHART" => {
            let graph = SkinBpmGraph::default();
            let idx = skin.objects.len();
            skin.add(graph.into());
            select_state.bpm_chart_idx = Some(idx);
            true
        }
        "DST_BPMCHART" => {
            if let Some(idx) = select_state.bpm_chart_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }

        // Stubs (Java also has empty implementations)
        "SRC_BAR_FLASH" | "SRC_BAR_RANK" | "DST_BAR_RANK" | "SRC_README" | "DST_README" => true,

        _ => false,
    }
}

/// Collects select state into MusicSelectSkinConfig after loading completes.
pub fn collect_select_config(select_state: &Lr2SelectState) -> Option<MusicSelectSkinConfig> {
    Some(MusicSelectSkinConfig {
        bar: Some(select_state.skinbar.clone()),
        distribution_graph: None,
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
    fn test_bar_center() {
        let (mut skin, mut state) = make_skin();
        let mut ss = Lr2SelectState::default();

        let fields: Vec<&str> = "#BAR_CENTER,12".split(',').collect();
        assert!(process_select_command(
            "BAR_CENTER",
            &fields,
            &mut skin,
            &mut state,
            &mut ss
        ));
        assert_eq!(ss.skinbar.position, 12);
    }

    #[test]
    fn test_bar_body_src_dst() {
        let (mut skin, mut state) = make_skin();
        let mut ss = Lr2SelectState::default();

        let src: Vec<&str> = "#SRC_BAR_BODY,0,5,0,0,200,30,1,1,0,0".split(',').collect();
        assert!(process_select_command(
            "SRC_BAR_BODY",
            &src,
            &mut skin,
            &mut state,
            &mut ss
        ));
        assert_eq!(ss.bar_src_idx, 5);

        let dst: Vec<&str> = "#DST_BAR_BODY_OFF,3,0,100,50,200,30,0,255,255,255,255"
            .split(',')
            .collect();
        assert!(process_select_command(
            "DST_BAR_BODY_OFF",
            &dst,
            &mut skin,
            &mut state,
            &mut ss
        ));
    }

    #[test]
    fn test_note_chart() {
        let (mut skin, mut state) = make_skin();
        let mut ss = Lr2SelectState::default();

        let src: Vec<&str> = "#SRC_NOTECHART,0,0,0,0,200,100,1,1,0,0"
            .split(',')
            .collect();
        assert!(process_select_command(
            "SRC_NOTECHART",
            &src,
            &mut skin,
            &mut state,
            &mut ss
        ));
        assert!(ss.note_chart_idx.is_some());
        assert_eq!(skin.object_count(), 1);

        let dst: Vec<&str> =
            "#DST_NOTECHART,0,0,100,50,200,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
                .split(',')
                .collect();
        assert!(process_select_command(
            "DST_NOTECHART",
            &dst,
            &mut skin,
            &mut state,
            &mut ss
        ));
    }

    #[test]
    fn test_bpm_chart() {
        let (mut skin, mut state) = make_skin();
        let mut ss = Lr2SelectState::default();

        let src: Vec<&str> = "#SRC_BPMCHART,0,0,0,0,200,100,1,1,0,0".split(',').collect();
        assert!(process_select_command(
            "SRC_BPMCHART",
            &src,
            &mut skin,
            &mut state,
            &mut ss
        ));
        assert!(ss.bpm_chart_idx.is_some());
    }

    #[test]
    fn test_stubs_return_true() {
        let (mut skin, mut state) = make_skin();
        let mut ss = Lr2SelectState::default();
        let fields: Vec<&str> = vec!["#CMD"];

        assert!(process_select_command(
            "SRC_BAR_FLASH",
            &fields,
            &mut skin,
            &mut state,
            &mut ss
        ));
        assert!(process_select_command(
            "SRC_BAR_RANK",
            &fields,
            &mut skin,
            &mut state,
            &mut ss
        ));
        assert!(process_select_command(
            "SRC_README",
            &fields,
            &mut skin,
            &mut state,
            &mut ss
        ));
    }

    #[test]
    fn test_unhandled_returns_false() {
        let (mut skin, mut state) = make_skin();
        let mut ss = Lr2SelectState::default();
        let fields: Vec<&str> = vec!["#UNKNOWN"];

        assert!(!process_select_command(
            "UNKNOWN", &fields, &mut skin, &mut state, &mut ss
        ));
    }

    #[test]
    fn test_collect_select_config() {
        let mut ss = Lr2SelectState::default();
        ss.skinbar.position = 7;

        let config = collect_select_config(&ss).unwrap();
        assert_eq!(config.bar.as_ref().unwrap().position, 7);
    }
}
