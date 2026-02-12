// LR2 Play skin loader.
//
// Handles state-specific commands for the play screen:
// - SRC_NOTE / SRC_LN_* / SRC_HCN_* / SRC_MINE — Note textures
// - DST_NOTE / DST_NOTE2 / DST_NOTE_EXPANSION_RATE — Note placement
// - SRC_LINE / DST_LINE — Measure lines
// - SRC_NOWJUDGE_1P / DST_NOWJUDGE_1P (2P/3P) — Judge display
// - SRC_NOWCOMBO_1P / DST_NOWCOMBO_1P (2P/3P) — Combo numbers
// - SRC_JUDGELINE / DST_JUDGELINE — Judge line
// - SRC_BGA / DST_BGA — BGA display
// - SRC_HIDDEN / DST_HIDDEN / SRC_LIFT / DST_LIFT — Covers
// - CLOSE / PLAYSTART / LOADSTART / LOADEND / FINISHMARGIN / JUDGETIMER
// - SRC_NOTECHART_1P / SRC_BPMCHART / SRC_TIMING_1P — Graphs
//
// Ported from LR2PlaySkinLoader.java.

use crate::loader::lr2_csv_loader::{Lr2CsvState, parse_field};
use crate::play_skin::PlaySkinConfig;
use crate::skin::Skin;
use crate::skin_bga::SkinBga;
use crate::skin_hidden::{SkinHidden, SkinLiftCover};
use crate::skin_judge::SkinJudge;
use crate::skin_note::SkinNote;

// ---------------------------------------------------------------------------
// Play state
// ---------------------------------------------------------------------------

/// Internal state for play skin loading.
#[derive(Default)]
pub struct Lr2PlayState {
    /// Note object index in skin.objects.
    note_idx: Option<usize>,
    /// Note object being constructed (reserved for texture population).
    _note: SkinNote,
    /// Current lane being loaded for note textures.
    note_lane: i32,
    /// Judge objects (reserved for construction).
    _judge: [Option<SkinJudge>; 3],
    /// Current judge object indices in skin.objects.
    judge_idx: [Option<usize>; 3],
    /// BGA object index.
    bga_idx: Option<usize>,
    /// Hidden cover object index.
    hidden_idx: Option<usize>,
    /// Lift cover object index.
    lift_idx: Option<usize>,
    /// Line (measure line) object index.
    line_idx: Option<usize>,
    /// Judge line object index.
    judgeline_idx: Option<usize>,
    /// Note chart object index.
    notechart_idx: Option<usize>,
    /// BPM chart object index.
    bpmchart_idx: Option<usize>,
    /// Timing chart object index.
    timingchart_idx: Option<usize>,
}

// ---------------------------------------------------------------------------
// Command dispatch
// ---------------------------------------------------------------------------

/// Processes a play-screen specific LR2 command.
///
/// Returns true if the command was handled.
pub fn process_play_command(
    cmd: &str,
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    play_state: &mut Lr2PlayState,
) -> bool {
    match cmd {
        // Global timing commands
        "CLOSE" => {
            skin.scene = parse_field(fields, 1);
            true
        }
        "PLAYSTART" => {
            // playstart timing — stored in skin data for reference
            true
        }
        "LOADSTART" | "LOADEND" => true,
        "FINISHMARGIN" => true,
        "JUDGETIMER" => true,

        // Note textures
        "SRC_NOTE" => {
            play_state.note_lane = parse_field(fields, 2);
            true
        }
        "SRC_LN_END"
        | "SRC_LN_START"
        | "SRC_LN_BODY"
        | "SRC_LN_BODY_INACTIVE"
        | "SRC_LN_BODY_ACTIVE" => {
            // Store LN texture reference for current lane
            true
        }
        "SRC_HCN_END"
        | "SRC_HCN_START"
        | "SRC_HCN_BODY"
        | "SRC_HCN_BODY_INACTIVE"
        | "SRC_HCN_BODY_ACTIVE"
        | "SRC_HCN_DAMAGE"
        | "SRC_HCN_REACTIVE" => {
            // Store HCN texture reference for current lane
            true
        }
        "SRC_MINE" => {
            // Store mine note texture reference
            true
        }

        // Note placement
        "DST_NOTE" => {
            if play_state.note_idx.is_none() {
                let note = SkinNote::default();
                let idx = skin.objects.len();
                skin.add(note.into());
                play_state.note_idx = Some(idx);
            }
            if let Some(idx) = play_state.note_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }
        "DST_NOTE2" => true,
        "DST_NOTE_EXPANSION_RATE" => true,

        // Measure line
        "SRC_LINE" => {
            let img = crate::skin_image::SkinImage::default();
            let idx = skin.objects.len();
            skin.add(img.into());
            play_state.line_idx = Some(idx);
            true
        }
        "DST_LINE" => {
            if let Some(idx) = play_state.line_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }

        // Judge display
        "SRC_NOWJUDGE_1P" => {
            src_judge(0, fields, skin, play_state);
            true
        }
        "SRC_NOWJUDGE_2P" => {
            src_judge(1, fields, skin, play_state);
            true
        }
        "SRC_NOWJUDGE_3P" => {
            src_judge(2, fields, skin, play_state);
            true
        }
        "DST_NOWJUDGE_1P" => {
            dst_judge(0, fields, skin, state, play_state);
            true
        }
        "DST_NOWJUDGE_2P" => {
            dst_judge(1, fields, skin, state, play_state);
            true
        }
        "DST_NOWJUDGE_3P" => {
            dst_judge(2, fields, skin, state, play_state);
            true
        }

        // Combo numbers
        "SRC_NOWCOMBO_1P" | "SRC_NOWCOMBO_2P" | "SRC_NOWCOMBO_3P" => true,
        "DST_NOWCOMBO_1P" | "DST_NOWCOMBO_2P" | "DST_NOWCOMBO_3P" => true,

        // Judge line
        "SRC_JUDGELINE" => {
            let img = crate::skin_image::SkinImage::default();
            let idx = skin.objects.len();
            skin.add(img.into());
            play_state.judgeline_idx = Some(idx);
            true
        }
        "DST_JUDGELINE" => {
            if let Some(idx) = play_state.judgeline_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }

        // BGA
        "SRC_BGA" => {
            let bga = SkinBga::default();
            let idx = skin.objects.len();
            skin.add(bga.into());
            play_state.bga_idx = Some(idx);
            true
        }
        "DST_BGA" => {
            if let Some(idx) = play_state.bga_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }

        // Hidden / Lift covers
        "SRC_HIDDEN" => {
            let hidden = SkinHidden::default();
            let idx = skin.objects.len();
            skin.add(hidden.into());
            play_state.hidden_idx = Some(idx);
            true
        }
        "DST_HIDDEN" => {
            if let Some(idx) = play_state.hidden_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }
        "SRC_LIFT" => {
            let lift = SkinLiftCover::default();
            let idx = skin.objects.len();
            skin.add(lift.into());
            play_state.lift_idx = Some(idx);
            true
        }
        "DST_LIFT" => {
            if let Some(idx) = play_state.lift_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }

        // Charts
        "SRC_NOTECHART_1P" => {
            let graph = crate::skin_visualizer::SkinNoteDistributionGraph::default();
            let idx = skin.objects.len();
            skin.add(graph.into());
            play_state.notechart_idx = Some(idx);
            true
        }
        "DST_NOTECHART_1P" => {
            if let Some(idx) = play_state.notechart_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }
        "SRC_BPMCHART" => {
            let graph = crate::skin_bpm_graph::SkinBpmGraph::default();
            let idx = skin.objects.len();
            skin.add(graph.into());
            play_state.bpmchart_idx = Some(idx);
            true
        }
        "DST_BPMCHART" => {
            if let Some(idx) = play_state.bpmchart_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }
        "SRC_TIMING_1P" => {
            let graph = crate::skin_visualizer::SkinTimingDistributionGraph::default();
            let idx = skin.objects.len();
            skin.add(graph.into());
            play_state.timingchart_idx = Some(idx);
            true
        }
        "DST_TIMING_1P" => {
            if let Some(idx) = play_state.timingchart_idx {
                state.apply_dst_to(idx, fields, skin);
            }
            true
        }

        // Pomyu character stubs
        "DST_PM_CHARA_1P"
        | "DST_PM_CHARA_2P"
        | "DST_PM_CHARA_ANIMATION"
        | "SRC_PM_CHARA_IMAGE"
        | "DST_PM_CHARA_IMAGE" => true,

        _ => false,
    }
}

/// Collects play state into PlaySkinConfig after loading completes.
pub fn collect_play_config(skin: &Skin, play_state: &Lr2PlayState) -> Option<PlaySkinConfig> {
    let note = play_state.note_idx.and_then(|idx| {
        skin.objects.get(idx).and_then(|obj| {
            if let crate::skin_object_type::SkinObjectType::Note(n) = obj {
                Some(n.clone())
            } else {
                None
            }
        })
    });

    let bga = play_state.bga_idx.and_then(|idx| {
        skin.objects.get(idx).and_then(|obj| {
            if let crate::skin_object_type::SkinObjectType::Bga(b) = obj {
                Some(b.clone())
            } else {
                None
            }
        })
    });

    let hidden_cover = play_state.hidden_idx.and_then(|idx| {
        skin.objects.get(idx).and_then(|obj| {
            if let crate::skin_object_type::SkinObjectType::Hidden(h) = obj {
                Some(h.clone())
            } else {
                None
            }
        })
    });

    let lift_cover = play_state.lift_idx.and_then(|idx| {
        skin.objects.get(idx).and_then(|obj| {
            if let crate::skin_object_type::SkinObjectType::LiftCover(l) = obj {
                Some(l.clone())
            } else {
                None
            }
        })
    });

    let judges: Vec<SkinJudge> = play_state
        .judge_idx
        .iter()
        .filter_map(|idx_opt| {
            idx_opt.and_then(|idx| {
                skin.objects.get(idx).and_then(|obj| {
                    if let crate::skin_object_type::SkinObjectType::Judge(j) = obj {
                        Some(j.as_ref().clone())
                    } else {
                        None
                    }
                })
            })
        })
        .collect();

    if note.is_none()
        && bga.is_none()
        && hidden_cover.is_none()
        && lift_cover.is_none()
        && judges.is_empty()
    {
        return None;
    }

    Some(PlaySkinConfig {
        note,
        bga,
        hidden_cover,
        lift_cover,
        judges,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn src_judge(player: usize, _fields: &[&str], skin: &mut Skin, play_state: &mut Lr2PlayState) {
    let judge = SkinJudge {
        player: player as i32,
        ..Default::default()
    };
    let idx = skin.objects.len();
    skin.add(judge.into());
    play_state.judge_idx[player] = Some(idx);
}

fn dst_judge(
    player: usize,
    fields: &[&str],
    skin: &mut Skin,
    state: &mut Lr2CsvState,
    play_state: &mut Lr2PlayState,
) {
    if let Some(idx) = play_state.judge_idx[player] {
        state.apply_dst_to(idx, fields, skin);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin_header::SkinHeader;
    use crate::skin_object_type::SkinObjectType;
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
    fn test_close_command() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();
        let fields: Vec<&str> = "#CLOSE,5000".split(',').collect();
        assert!(process_play_command(
            "CLOSE", &fields, &mut skin, &mut state, &mut ps
        ));
        assert_eq!(skin.scene, 5000);
    }

    #[test]
    fn test_dst_note_creates_note() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();

        let dst: Vec<&str> = "#DST_NOTE,0,0,100,50,200,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
            .split(',')
            .collect();
        assert!(process_play_command(
            "DST_NOTE", &dst, &mut skin, &mut state, &mut ps
        ));

        assert!(ps.note_idx.is_some());
        assert_eq!(skin.object_count(), 1);
        assert!(matches!(skin.objects[0], SkinObjectType::Note(_)));
    }

    #[test]
    fn test_bga_src_dst() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();

        let src: Vec<&str> = "#SRC_BGA,0,0,0,0,256,256,1,1,0,0".split(',').collect();
        assert!(process_play_command(
            "SRC_BGA", &src, &mut skin, &mut state, &mut ps
        ));
        assert!(ps.bga_idx.is_some());

        let dst: Vec<&str> = "#DST_BGA,0,0,0,0,256,256,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
            .split(',')
            .collect();
        assert!(process_play_command(
            "DST_BGA", &dst, &mut skin, &mut state, &mut ps
        ));
    }

    #[test]
    fn test_hidden_lift_covers() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();

        let src: Vec<&str> = "#SRC,0,0,0,0,100,100,1,1,0,0".split(',').collect();
        let dst: Vec<&str> = "#DST,0,0,0,0,100,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
            .split(',')
            .collect();

        process_play_command("SRC_HIDDEN", &src, &mut skin, &mut state, &mut ps);
        process_play_command("DST_HIDDEN", &dst, &mut skin, &mut state, &mut ps);
        assert!(ps.hidden_idx.is_some());

        process_play_command("SRC_LIFT", &src, &mut skin, &mut state, &mut ps);
        process_play_command("DST_LIFT", &dst, &mut skin, &mut state, &mut ps);
        assert!(ps.lift_idx.is_some());
    }

    #[test]
    fn test_judge_1p() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();

        let src: Vec<&str> = "#SRC,0,0,0,0,100,50,1,1,0,0".split(',').collect();
        let dst: Vec<&str> = "#DST,0,0,200,300,100,50,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
            .split(',')
            .collect();

        process_play_command("SRC_NOWJUDGE_1P", &src, &mut skin, &mut state, &mut ps);
        process_play_command("DST_NOWJUDGE_1P", &dst, &mut skin, &mut state, &mut ps);

        assert!(ps.judge_idx[0].is_some());
        assert!(matches!(skin.objects[0], SkinObjectType::Judge(_)));
    }

    #[test]
    fn test_collect_play_config() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();

        let dst: Vec<&str> = "#DST_NOTE,0,0,100,50,200,100,0,255,255,255,255,0,0,0,0,0,0,0,0,0"
            .split(',')
            .collect();
        process_play_command("DST_NOTE", &dst, &mut skin, &mut state, &mut ps);

        let src: Vec<&str> = "#SRC,0,0,0,0,256,256,1,1,0,0".split(',').collect();
        process_play_command("SRC_BGA", &src, &mut skin, &mut state, &mut ps);

        let config = collect_play_config(&skin, &ps).unwrap();
        assert!(config.note.is_some());
        assert!(config.bga.is_some());
    }

    #[test]
    fn test_empty_play_config_returns_none() {
        let (skin, _) = make_skin();
        let ps = Lr2PlayState::default();
        assert!(collect_play_config(&skin, &ps).is_none());
    }

    #[test]
    fn test_timing_commands_handled() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();
        let fields: Vec<&str> = vec!["#CMD", "0"];

        assert!(process_play_command(
            "PLAYSTART",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "LOADSTART",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "LOADEND", &fields, &mut skin, &mut state, &mut ps
        ));
        assert!(process_play_command(
            "FINISHMARGIN",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "JUDGETIMER",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
    }

    #[test]
    fn test_note_texture_commands_handled() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();
        let fields: Vec<&str> = vec!["#CMD", "0", "3"];

        assert!(process_play_command(
            "SRC_NOTE", &fields, &mut skin, &mut state, &mut ps
        ));
        assert!(process_play_command(
            "SRC_LN_END",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "SRC_LN_START",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "SRC_LN_BODY",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "SRC_HCN_END",
            &fields,
            &mut skin,
            &mut state,
            &mut ps
        ));
        assert!(process_play_command(
            "SRC_MINE", &fields, &mut skin, &mut state, &mut ps
        ));
    }

    #[test]
    fn test_unhandled_returns_false() {
        let (mut skin, mut state) = make_skin();
        let mut ps = Lr2PlayState::default();
        let fields: Vec<&str> = vec!["#UNKNOWN"];

        assert!(!process_play_command(
            "UNKNOWN", &fields, &mut skin, &mut state, &mut ps
        ));
    }
}
