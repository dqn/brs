//! JudgeManager constructor and initialization.

use bms_model::{LaneProperty, LnType, NoteType, PlayMode};

use super::JudgeConfig;
use super::JudgeManager;
use super::lane_state::{LaneState, MultiBadCollector, NOT_SET};
use crate::JUDGE_PR;
use crate::judge_property::JudgeNoteType;
use crate::score_data::ScoreData;

impl JudgeManager {
    /// Initialize the judge manager with chart data and configuration.
    pub fn new(config: &JudgeConfig<'_>) -> Self {
        let play_mode = config.play_mode;
        let key_count = play_mode.key_count();

        // Build per-lane note indices
        let mut lane_notes: Vec<Vec<usize>> = vec![Vec::new(); key_count];
        for (i, note) in config.notes.iter().enumerate() {
            if note.lane < key_count {
                lane_notes[note.lane].push(i);
            }
        }
        // Sort each lane's notes by time
        for lane in &mut lane_notes {
            lane.sort_by_key(|&i| config.notes[i].time_us);
        }

        // Build lane states
        let lane_states: Vec<LaneState> = (0..key_count)
            .map(|lane| LaneState::new(lane, play_mode.is_scratch_key(lane)))
            .collect();

        // Compute scaled judge windows
        let nmjudge = config.judge_property.judge_windows(
            JudgeNoteType::Note,
            config.judge_rank,
            &config.judge_window_rate,
        );
        let smjudge = if config.judge_property.scratch.is_empty() {
            nmjudge.clone()
        } else {
            config.judge_property.judge_windows(
                JudgeNoteType::Scratch,
                config.judge_rank,
                &config.scratch_judge_window_rate,
            )
        };
        let cnendmjudge = config.judge_property.judge_windows(
            JudgeNoteType::LongNoteEnd,
            config.judge_rank,
            &config.judge_window_rate,
        );
        let scnendmjudge = if config.judge_property.longscratch.is_empty() {
            cnendmjudge.clone()
        } else {
            config.judge_property.judge_windows(
                JudgeNoteType::LongScratchEnd,
                config.judge_rank,
                &config.scratch_judge_window_rate,
            )
        };
        let nreleasemargin = config.judge_property.longnote_margin;
        let sreleasemargin = config.judge_property.longscratch_margin;

        // Compute combined window bounds
        let mut mjudge_start: i64 = 0;
        let mut mjudge_end: i64 = 0;
        for w in nmjudge.iter().chain(smjudge.iter()) {
            mjudge_start = mjudge_start.min(w[0]);
            mjudge_end = mjudge_end.max(w[1]);
        }

        // Total playable notes for ghost array
        // Exclude pure LN end notes (not independently judged in pure LN mode)
        let total_notes = config
            .notes
            .iter()
            .filter(|n| {
                if !n.is_playable() {
                    return false;
                }
                // Pure LN end: LongNote type, end_time_us == 0 (end marker), has pair link
                if n.note_type == NoteType::LongNote
                    && n.end_time_us == 0
                    && n.pair_index != usize::MAX
                    && config.ln_type == LnType::LongNote
                {
                    return false;
                }
                true
            })
            .count();

        // Initialize ghost array (default to JUDGE_PR = 4)
        let ghost = vec![JUDGE_PR; total_notes];

        // Note states (0 = unjudged)
        let note_states = vec![0i32; config.notes.len()];

        // Lane property (for physical key → lane mapping and BSS sckey tracking)
        let lane_property = match config.lane_property {
            Some(lp) => lp.clone(),
            None => LaneProperty::new(play_mode),
        };

        // BSS tracking
        let sckey = vec![0i32; lane_property.scratch_count()];

        // Autoplay press times (per physical key)
        let auto_presstime = vec![NOT_SET; lane_property.physical_key_count()];

        // PMS multi-bad enabled
        let is_pms = matches!(play_mode, PlayMode::PopN5K | PlayMode::PopN9K);

        let player_count = play_mode.player_count();

        Self {
            algorithm: config.algorithm,
            miss_condition: config.judge_property.miss,
            ln_type: config.ln_type,
            combo_cond: config.judge_property.combo,
            judge_vanish: config.judge_property.judge_vanish,
            nmjudge,
            smjudge,
            cnendmjudge,
            scnendmjudge,
            nreleasemargin,
            sreleasemargin,
            mjudge_start,
            mjudge_end,
            lane_count: key_count,
            lane_property,
            lane_states,
            lane_notes,
            note_states,
            score: ScoreData::default(),
            combo: 0,
            max_combo: 0,
            course_combo: 0,
            course_max_combo: 0,
            ghost,
            pass_notes: 0,
            recent_judges: vec![NOT_SET; 100],
            recent_index: 0,
            now_judge: vec![0; player_count],
            now_combo: vec![0; player_count],
            sckey,
            autoplay: config.autoplay,
            auto_presstime,
            lane_judge: vec![0i32; key_count],
            judge_timing: vec![0i64; player_count],
            multi_bad: MultiBadCollector::new(is_pms),
            prev_time: 0,
        }
    }
}
