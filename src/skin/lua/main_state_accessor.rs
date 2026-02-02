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

    // BPM
    pub current_bpm: f64,
    pub min_bpm: f64,
    pub max_bpm: f64,

    // Play time
    pub current_time_ms: f64,
    pub total_time_ms: f64,

    // Total notes
    pub total_notes: u32,

    // Hi-speed
    pub hi_speed: f32,

    // Play state
    pub is_playing: bool,
    pub is_ready: bool,
    pub is_finished: bool,
    pub is_clear: bool,
    pub is_failed: bool,

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
    pub player_name: String,
    pub rival_name: String,
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

    /// Get number value by ID.
    pub fn number(&self, id: i32) -> i32 {
        match id {
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

            NUMBER_NOWBPM => self.current_bpm as i32,
            NUMBER_MINBPM => self.min_bpm as i32,
            NUMBER_MAXBPM => self.max_bpm as i32,

            NUMBER_TOTALNOTES | NUMBER_TOTALNOTES2 => self.total_notes as i32,

            NUMBER_HISPEED => (self.hi_speed * 100.0) as i32 / 100,
            NUMBER_HISPEED_AFTERDOT => (self.hi_speed * 100.0) as i32 % 100,

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

            OPTION_NOW_LOADING => !self.is_ready && !self.is_playing && !self.is_finished,
            OPTION_LOADED => self.is_ready || self.is_playing || self.is_finished,

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
            _ => String::new(),
        }
    }

    /// Get float value by ID (for sliders/bargraphs).
    pub fn float_number(&self, id: i32) -> f64 {
        match id {
            BARGRAPH_SCORERATE => self.score_rate / 100.0,
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
