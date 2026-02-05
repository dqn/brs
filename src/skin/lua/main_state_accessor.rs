use std::collections::{HashMap, HashSet};

use crate::skin::SkinOffset;
use crate::skin::skin_property::*;

/// Snapshot of the play state for skin rendering.
/// This is populated each frame from PlayState and used by the skin system.
#[derive(Debug, Clone, Default)]
pub struct MainState {
    // Judge counts
    pub pg_count: u32,
    pub gr_count: u32,
    pub gd_count: u32,
    pub bd_count: u32,
    pub pr_count: u32,
    pub ms_count: u32,

    // Early/Late counts
    pub early_pg_count: u32,
    pub late_pg_count: u32,
    pub early_gr_count: u32,
    pub late_gr_count: u32,
    pub early_gd_count: u32,
    pub late_gd_count: u32,
    pub early_bd_count: u32,
    pub late_bd_count: u32,
    pub early_pr_count: u32,
    pub late_pr_count: u32,
    pub early_ms_count: u32,
    pub late_ms_count: u32,

    // Combo
    pub combo: u32,
    pub max_combo: u32,

    // Gauge
    pub gauge_value: f64,
    pub gauge_type: i32,

    // Score
    pub ex_score: u32,
    pub score_rate: f64,
    pub score_diff_ex: i32,
    pub total_rate: f64,
    pub miss_count: u32,
    pub combo_break: u32,

    // BPM
    pub current_bpm: f64,
    pub min_bpm: f64,
    pub max_bpm: f64,
    pub bpm_change: bool,

    // Chart info
    pub play_level: i32,
    pub difficulty: i32,
    pub has_long_note: bool,
    pub has_bga: bool,
    pub has_stagefile: bool,
    pub has_backbmp: bool,
    pub has_banner: bool,

    // Key mode flags
    pub is_7key: bool,
    pub is_5key: bool,
    pub is_14key: bool,
    pub is_10key: bool,
    pub is_9key: bool,

    // Play time
    pub current_time_ms: f64,
    pub total_time_ms: f64,

    // Total notes
    pub total_notes: u32,

    // Hi-speed
    pub hi_speed: f32,

    // Lane cover settings
    pub lane_cover_sudden: f32,
    pub lane_cover_hidden: f32,
    pub lane_cover_lift: f32,

    // Play state
    pub is_playing: bool,
    pub is_ready: bool,
    pub is_finished: bool,
    pub is_clear: bool,
    pub is_failed: bool,
    pub is_replay: bool,
    pub is_replay_recording: bool,
    /// BGA enabled flag.
    pub bga_enabled: bool,
    /// Autoplay enabled flag.
    pub autoplay_enabled: bool,

    // Last judge
    pub last_judge: Option<LastJudge>,

    // Timers (microseconds, i64::MIN = off)
    pub timers: MainStateTimers,

    // String properties for text display
    pub song_title: String,
    pub song_subtitle: String,
    pub full_title: String,
    pub genre: String,
    pub artist: String,
    pub subartist: String,
    pub full_artist: String,
    pub song_folder: String,
    pub player_name: String,
    pub rival_name: String,

    // Skin config option IDs that should be treated as enabled.
    // 表示判定で有効扱いするスキン設定のオプションID。
    pub skin_options: HashSet<i32>,

    // Offset values for skin rendering.
    // スキン描画用のオフセット値。
    pub skin_offsets: HashMap<i32, SkinOffset>,
}

/// Last judge information.
#[derive(Debug, Clone, Copy)]
pub struct LastJudge {
    pub rank: JudgeType,
    pub is_early: bool,
    pub time_ms: f64,
}

/// Judge type for options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeType {
    Perfect,
    Great,
    Good,
    Bad,
    Poor,
    Miss,
}

/// Timer values for skin rendering.
#[derive(Debug, Clone, Default)]
pub struct MainStateTimers {
    /// Start input timer (microseconds).
    pub start_input: i64,
    /// Fadeout timer.
    pub fadeout: i64,
    /// Ready timer.
    pub ready: i64,
    /// Play timer.
    pub play: i64,
    /// Judge timer (1P).
    pub judge_1p: i64,
    /// Combo timer (1P).
    pub combo_1p: i64,
    /// Bomb timers for each lane [scratch, key1-7].
    pub bomb_1p: [i64; 8],
    /// Hold timers for each lane [scratch, key1-7].
    pub hold_1p: [i64; 8],
    /// Key on timers for each lane [scratch, key1-7].
    pub keyon_1p: [i64; 8],
    /// Key off timers for each lane [scratch, key1-7].
    pub keyoff_1p: [i64; 8],
}

impl MainStateTimers {
    pub fn new() -> Self {
        Self {
            start_input: TIMER_OFF_VALUE,
            fadeout: TIMER_OFF_VALUE,
            ready: TIMER_OFF_VALUE,
            play: TIMER_OFF_VALUE,
            judge_1p: TIMER_OFF_VALUE,
            combo_1p: TIMER_OFF_VALUE,
            bomb_1p: [TIMER_OFF_VALUE; 8],
            hold_1p: [TIMER_OFF_VALUE; 8],
            keyon_1p: [TIMER_OFF_VALUE; 8],
            keyoff_1p: [TIMER_OFF_VALUE; 8],
        }
    }
}

impl MainState {
    /// Create a new empty main state.
    pub fn new() -> Self {
        Self {
            timers: MainStateTimers::new(),
            ..Default::default()
        }
    }

    /// Apply skin config option IDs used for visibility checks.
    /// 表示判定で使用するスキン設定のオプションIDを反映する。
    pub fn set_skin_options(&mut self, options: &HashSet<i32>) {
        self.skin_options = options.clone();
    }

    /// Apply skin offset values used for position adjustments.
    /// 位置補正に使うスキンオフセット値を反映する。
    pub fn set_skin_offsets(&mut self, offsets: &HashMap<i32, SkinOffset>) {
        self.skin_offsets = offsets.clone();
    }

    /// Get offset values by ID.
    pub fn offset(&self, id: i32) -> SkinOffset {
        self.skin_offsets.get(&id).copied().unwrap_or_default()
    }

    /// Get number value by ID.
    pub fn number(&self, id: i32) -> i32 {
        match id {
            NUMBER_SCORE => self.ex_score as i32,
            NUMBER_MAXSCORE => (self.total_notes * 2) as i32,
            NUMBER_MISSCOUNT => self.miss_count as i32,

            NUMBER_PERFECT => self.pg_count as i32,
            NUMBER_GREAT => self.gr_count as i32,
            NUMBER_GOOD => self.gd_count as i32,
            NUMBER_BAD => self.bd_count as i32,
            NUMBER_POOR => self.pr_count as i32,
            NUMBER_MISS => self.ms_count as i32,

            NUMBER_EARLY_PERFECT => self.early_pg_count as i32,
            NUMBER_LATE_PERFECT => self.late_pg_count as i32,
            NUMBER_EARLY_GREAT => self.early_gr_count as i32,
            NUMBER_LATE_GREAT => self.late_gr_count as i32,
            NUMBER_EARLY_GOOD => self.early_gd_count as i32,
            NUMBER_LATE_GOOD => self.late_gd_count as i32,
            NUMBER_EARLY_BAD => self.early_bd_count as i32,
            NUMBER_LATE_BAD => self.late_bd_count as i32,
            NUMBER_EARLY_POOR => self.early_pr_count as i32,
            NUMBER_LATE_POOR => self.late_pr_count as i32,
            NUMBER_EARLY_MISS => self.early_ms_count as i32,
            NUMBER_LATE_MISS => self.late_ms_count as i32,

            NUMBER_COMBO => self.combo as i32,
            NUMBER_MAXCOMBO | NUMBER_MAXCOMBO2 => self.max_combo as i32,

            NUMBER_GROOVEGAUGE => self.gauge_value as i32,
            NUMBER_GROOVEGAUGE_AFTERDOT => ((self.gauge_value * 10.0) as i32) % 10,

            NUMBER_SCORE2 => self.ex_score as i32,
            NUMBER_SCORE_RATE => self.score_rate as i32,
            NUMBER_SCORE_RATE_AFTERDOT => ((self.score_rate * 100.0) as i32) % 100,
            NUMBER_DIFF_EXSCORE => self.score_diff_ex,
            NUMBER_TOTAL_RATE => self.total_rate as i32,
            NUMBER_TOTAL_RATE_AFTERDOT => ((self.total_rate * 100.0) as i32) % 100,

            NUMBER_MAINBPM => self.current_bpm as i32,
            NUMBER_NOWBPM => self.current_bpm as i32,
            NUMBER_MINBPM => self.min_bpm as i32,
            NUMBER_MAXBPM => self.max_bpm as i32,
            NUMBER_PLAYLEVEL => self.play_level,

            NUMBER_TOTALNOTES | NUMBER_TOTALNOTES2 => self.total_notes as i32,

            NUMBER_HISPEED => (self.hi_speed * 100.0) as i32 / 100,
            NUMBER_HISPEED_AFTERDOT => (self.hi_speed * 100.0) as i32 % 100,
            NUMBER_LANECOVER1 => (self.lane_cover_sudden * 100.0).round() as i32,
            NUMBER_LIFT1 => (self.lane_cover_lift * 100.0).round() as i32,
            NUMBER_HIDDEN1 => (self.lane_cover_hidden * 100.0).round() as i32,

            NUMBER_PLAYTIME_MINUTE => (self.current_time_ms / 60000.0).max(0.0) as i32,
            NUMBER_PLAYTIME_SECOND => ((self.current_time_ms / 1000.0).max(0.0) as i32) % 60,

            NUMBER_TIMELEFT_MINUTE => {
                let left = (self.total_time_ms - self.current_time_ms).max(0.0);
                (left / 60000.0) as i32
            }
            NUMBER_TIMELEFT_SECOND => {
                let left = (self.total_time_ms - self.current_time_ms).max(0.0);
                ((left / 1000.0) as i32) % 60
            }

            _ => 0,
        }
    }

    /// Get option value by ID.
    pub fn option(&self, id: i32) -> bool {
        if self.skin_options.contains(&id) {
            return true;
        }
        let rate = self.score_rate;
        let grade = if rate >= 88.89 {
            0
        } else if rate >= 77.78 {
            1
        } else if rate >= 66.67 {
            2
        } else if rate >= 55.56 {
            3
        } else if rate >= 44.45 {
            4
        } else if rate >= 33.34 {
            5
        } else if rate >= 22.23 {
            6
        } else {
            7
        };
        match id {
            OPTION_RESULT_CLEAR => self.is_clear,
            OPTION_RESULT_FAIL => self.is_failed,

            OPTION_1P_PERFECT => matches!(
                self.last_judge,
                Some(LastJudge {
                    rank: JudgeType::Perfect,
                    ..
                })
            ),
            OPTION_1P_GREAT => matches!(
                self.last_judge,
                Some(LastJudge {
                    rank: JudgeType::Great,
                    ..
                })
            ),
            OPTION_1P_GOOD => matches!(
                self.last_judge,
                Some(LastJudge {
                    rank: JudgeType::Good,
                    ..
                })
            ),
            OPTION_1P_BAD => matches!(
                self.last_judge,
                Some(LastJudge {
                    rank: JudgeType::Bad,
                    ..
                })
            ),
            OPTION_1P_POOR => matches!(
                self.last_judge,
                Some(LastJudge {
                    rank: JudgeType::Poor,
                    ..
                })
            ),
            OPTION_1P_MISS => matches!(
                self.last_judge,
                Some(LastJudge {
                    rank: JudgeType::Miss,
                    ..
                })
            ),

            OPTION_1P_EARLY => matches!(self.last_judge, Some(LastJudge { is_early: true, .. })),
            OPTION_1P_LATE => matches!(
                self.last_judge,
                Some(LastJudge {
                    is_early: false,
                    ..
                })
            ),

            OPTION_GAUGE_GROOVE => self.gauge_type == 0 || self.gauge_type == 1,
            OPTION_GAUGE_HARD => self.gauge_type == 2,
            OPTION_GAUGE_EX => self.gauge_type == 3,
            OPTION_BGAON => self.bga_enabled,
            OPTION_BGAOFF => !self.bga_enabled,
            OPTION_AUTOPLAYOFF => !self.autoplay_enabled,
            OPTION_AUTOPLAYON => self.autoplay_enabled,
            OPTION_REPLAY_OFF => !self.is_replay && !self.is_replay_recording,
            OPTION_REPLAY_RECORDING => self.is_replay_recording,
            OPTION_REPLAY_PLAYING => self.is_replay,

            OPTION_NOW_LOADING => !self.is_ready && !self.is_playing && !self.is_finished,
            OPTION_LOADED => self.is_ready || self.is_playing || self.is_finished,
            OPTION_STAGEFILE => self.has_stagefile,
            OPTION_NO_STAGEFILE => !self.has_stagefile,
            OPTION_BACKBMP => self.has_backbmp,
            OPTION_NO_BACKBMP => !self.has_backbmp,
            OPTION_BANNER => self.has_banner,
            OPTION_NO_BANNER => !self.has_banner,
            OPTION_NO_LN => !self.has_long_note,
            OPTION_LN => self.has_long_note,
            OPTION_BPMCHANGE => self.bpm_change,
            OPTION_NO_BPMCHANGE => !self.bpm_change,

            OPTION_7KEYSONG => self.is_7key,
            OPTION_5KEYSONG => self.is_5key,
            OPTION_14KEYSONG => self.is_14key,
            OPTION_10KEYSONG => self.is_10key,
            OPTION_9KEYSONG => self.is_9key,

            // Gauge range options
            OPTION_1P_0_9 => self.gauge_value < 10.0,
            OPTION_1P_10_19 => (10.0..20.0).contains(&self.gauge_value),
            OPTION_1P_20_29 => (20.0..30.0).contains(&self.gauge_value),
            OPTION_1P_30_39 => (30.0..40.0).contains(&self.gauge_value),
            OPTION_1P_40_49 => (40.0..50.0).contains(&self.gauge_value),
            OPTION_1P_50_59 => (50.0..60.0).contains(&self.gauge_value),
            OPTION_1P_60_69 => (60.0..70.0).contains(&self.gauge_value),
            OPTION_1P_70_79 => (70.0..80.0).contains(&self.gauge_value),
            OPTION_1P_80_89 => (80.0..90.0).contains(&self.gauge_value),
            OPTION_1P_90_99 => (90.0..100.0).contains(&self.gauge_value),
            OPTION_1P_100 => self.gauge_value >= 100.0,
            OPTION_1P_BORDER_OR_MORE => self.gauge_value >= 80.0,

            OPTION_LANECOVER1_CHANGING => {
                self.lane_cover_sudden > 0.0
                    || self.lane_cover_hidden > 0.0
                    || self.lane_cover_lift > 0.0
            }
            OPTION_LANECOVER1_ON => self.lane_cover_sudden > 0.0,
            OPTION_LIFT1_ON => self.lane_cover_lift > 0.0,
            OPTION_HIDDEN1_ON => self.lane_cover_hidden > 0.0,

            OPTION_1P_AAA => grade == 0,
            OPTION_1P_AA => grade == 1,
            OPTION_1P_A => grade == 2,
            OPTION_1P_B => grade == 3,
            OPTION_1P_C => grade == 4,
            OPTION_1P_D => grade == 5,
            OPTION_1P_E => grade == 6,
            OPTION_1P_F => grade == 7,

            OPTION_RESULT_AAA_1P => self.is_finished && grade == 0,
            OPTION_RESULT_AA_1P => self.is_finished && grade == 1,
            OPTION_RESULT_A_1P => self.is_finished && grade == 2,
            OPTION_RESULT_B_1P => self.is_finished && grade == 3,
            OPTION_RESULT_C_1P => self.is_finished && grade == 4,
            OPTION_RESULT_D_1P => self.is_finished && grade == 5,
            OPTION_RESULT_E_1P => self.is_finished && grade == 6,
            OPTION_RESULT_F_1P => self.is_finished && grade == 7,

            OPTION_CONSTANT => true,

            _ => false,
        }
    }

    /// Get timer value by ID (microseconds).
    pub fn timer(&self, id: i32) -> i64 {
        match id {
            TIMER_STARTINPUT => self.timers.start_input,
            TIMER_FADEOUT => self.timers.fadeout,
            TIMER_READY => self.timers.ready,
            TIMER_PLAY => self.timers.play,
            TIMER_JUDGE_1P => self.timers.judge_1p,
            TIMER_COMBO_1P => self.timers.combo_1p,

            // Bomb timers
            TIMER_BOMB_1P_SCRATCH => self.timers.bomb_1p[0],
            TIMER_BOMB_1P_KEY1 => self.timers.bomb_1p[1],
            TIMER_BOMB_1P_KEY2 => self.timers.bomb_1p[2],
            TIMER_BOMB_1P_KEY3 => self.timers.bomb_1p[3],
            TIMER_BOMB_1P_KEY4 => self.timers.bomb_1p[4],
            TIMER_BOMB_1P_KEY5 => self.timers.bomb_1p[5],
            TIMER_BOMB_1P_KEY6 => self.timers.bomb_1p[6],
            TIMER_BOMB_1P_KEY7 => self.timers.bomb_1p[7],

            // Hold timers
            TIMER_HOLD_1P_SCRATCH => self.timers.hold_1p[0],
            TIMER_HOLD_1P_KEY1 => self.timers.hold_1p[1],
            TIMER_HOLD_1P_KEY2 => self.timers.hold_1p[2],
            TIMER_HOLD_1P_KEY3 => self.timers.hold_1p[3],
            TIMER_HOLD_1P_KEY4 => self.timers.hold_1p[4],
            TIMER_HOLD_1P_KEY5 => self.timers.hold_1p[5],
            TIMER_HOLD_1P_KEY6 => self.timers.hold_1p[6],
            TIMER_HOLD_1P_KEY7 => self.timers.hold_1p[7],

            // Key on timers
            TIMER_KEYON_1P_SCRATCH => self.timers.keyon_1p[0],
            TIMER_KEYON_1P_KEY1 => self.timers.keyon_1p[1],
            TIMER_KEYON_1P_KEY2 => self.timers.keyon_1p[2],
            TIMER_KEYON_1P_KEY3 => self.timers.keyon_1p[3],
            TIMER_KEYON_1P_KEY4 => self.timers.keyon_1p[4],
            TIMER_KEYON_1P_KEY5 => self.timers.keyon_1p[5],
            TIMER_KEYON_1P_KEY6 => self.timers.keyon_1p[6],
            TIMER_KEYON_1P_KEY7 => self.timers.keyon_1p[7],

            // Key off timers
            TIMER_KEYOFF_1P_SCRATCH => self.timers.keyoff_1p[0],
            TIMER_KEYOFF_1P_KEY1 => self.timers.keyoff_1p[1],
            TIMER_KEYOFF_1P_KEY2 => self.timers.keyoff_1p[2],
            TIMER_KEYOFF_1P_KEY3 => self.timers.keyoff_1p[3],
            TIMER_KEYOFF_1P_KEY4 => self.timers.keyoff_1p[4],
            TIMER_KEYOFF_1P_KEY5 => self.timers.keyoff_1p[5],
            TIMER_KEYOFF_1P_KEY6 => self.timers.keyoff_1p[6],
            TIMER_KEYOFF_1P_KEY7 => self.timers.keyoff_1p[7],

            _ => TIMER_OFF_VALUE,
        }
    }

    /// Get text value by ID.
    pub fn text(&self, id: i32) -> String {
        match id {
            STRING_RIVAL => self.rival_name.clone(),
            STRING_PLAYER => self.player_name.clone(),
            STRING_TITLE => self.song_title.clone(),
            STRING_SUBTITLE => self.song_subtitle.clone(),
            STRING_FULLTITLE => self.full_title.clone(),
            STRING_GENRE => self.genre.clone(),
            STRING_ARTIST => self.artist.clone(),
            STRING_SUBARTIST => self.subartist.clone(),
            STRING_FULLARTIST => self.full_artist.clone(),
            STRING_DIRECTORY | STRING_TABLE_FULL | STRING_TABLE_NAME => self.song_folder.clone(),
            STRING_TABLE_LEVEL => self.play_level.to_string(),
            _ => String::new(),
        }
    }

    /// Get float value by ID (for sliders/bargraphs).
    pub fn float_number(&self, id: i32) -> f64 {
        match id {
            BARGRAPH_SCORERATE => self.score_rate / 100.0,
            SLIDER_LANECOVER | SLIDER_LANECOVER2 => {
                let mut lane = self.lane_cover_sudden;
                if self.lane_cover_lift > 0.0 {
                    lane *= 1.0 - self.lane_cover_lift;
                }
                lane as f64
            }
            SLIDER_MUSIC_PROGRESS | BARGRAPH_MUSIC_PROGRESS => {
                if self.total_time_ms > 0.0 {
                    (self.current_time_ms / self.total_time_ms).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    /// Get gauge value (0.0 - 100.0).
    pub fn gauge(&self) -> f64 {
        self.gauge_value
    }

    /// Get gauge type (0=Normal, 1=Easy, 2=Hard, 3=ExHard).
    pub fn gauge_type(&self) -> i32 {
        self.gauge_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_state_number() {
        let mut state = MainState::new();
        state.pg_count = 100;
        state.gr_count = 50;
        state.combo = 25;
        state.gauge_value = 85.5;

        assert_eq!(state.number(NUMBER_PERFECT), 100);
        assert_eq!(state.number(NUMBER_GREAT), 50);
        assert_eq!(state.number(NUMBER_COMBO), 25);
        assert_eq!(state.number(NUMBER_GROOVEGAUGE), 85);
        assert_eq!(state.number(NUMBER_GROOVEGAUGE_AFTERDOT), 5);
    }

    #[test]
    fn test_main_state_option() {
        let mut state = MainState::new();
        state.gauge_value = 85.0;
        state.is_clear = true;

        assert!(state.option(OPTION_1P_80_89));
        assert!(!state.option(OPTION_1P_90_99));
        assert!(state.option(OPTION_1P_BORDER_OR_MORE));
        assert!(state.option(OPTION_RESULT_CLEAR));
    }

    #[test]
    fn test_main_state_timer() {
        let mut state = MainState::new();
        state.timers.play = 1000000; // 1 second in microseconds

        assert_eq!(state.timer(TIMER_PLAY), 1000000);
        assert_eq!(state.timer(TIMER_READY), TIMER_OFF_VALUE);
    }
}
