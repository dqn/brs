//! Condition evaluation for beatoraja skins
//!
//! Implements the if/op (operation code) system used by beatoraja
//! for conditional rendering of skin elements.

/// Operation code ranges and meanings
pub mod opcodes {
    // Boolean conditions (0-999)
    // Return true/false based on game state

    /// Always true
    pub const ALWAYS_TRUE: i32 = 0;

    // Play state (1-99)
    /// Auto play enabled
    pub const AUTO_PLAY: i32 = 1;
    /// Replay mode
    pub const REPLAY: i32 = 2;
    /// Practice mode
    pub const PRACTICE: i32 = 3;
    /// Course mode (dan/段位)
    pub const COURSE_MODE: i32 = 4;

    // Gauge type (30-39)
    /// Groove gauge (NORMAL)
    pub const GAUGE_GROOVE: i32 = 30;
    /// Hard gauge
    pub const GAUGE_HARD: i32 = 31;
    /// Ex-hard gauge
    pub const GAUGE_EXHARD: i32 = 32;
    /// Easy gauge
    pub const GAUGE_EASY: i32 = 33;
    /// Assist easy gauge
    pub const GAUGE_ASSIST_EASY: i32 = 34;

    // Gauge state (40-49)
    /// Gauge is in red zone (< 30%)
    pub const GAUGE_RED: i32 = 40;
    /// Gauge is in yellow zone (30-80%)
    pub const GAUGE_YELLOW: i32 = 41;
    /// Gauge is in green zone (>= 80%)
    pub const GAUGE_GREEN: i32 = 42;
    /// Gauge at 100%
    pub const GAUGE_MAX: i32 = 43;

    // Judge result state (50-59)
    /// Last judgment was PGREAT
    pub const JUDGE_PGREAT: i32 = 50;
    /// Last judgment was GREAT
    pub const JUDGE_GREAT: i32 = 51;
    /// Last judgment was GOOD
    pub const JUDGE_GOOD: i32 = 52;
    /// Last judgment was BAD
    pub const JUDGE_BAD: i32 = 53;
    /// Last judgment was POOR
    pub const JUDGE_POOR: i32 = 54;
    /// Last judgment was MISS
    pub const JUDGE_MISS: i32 = 55;

    // Timing state (60-69)
    /// Last note was FAST
    pub const TIMING_FAST: i32 = 60;
    /// Last note was SLOW
    pub const TIMING_SLOW: i32 = 61;
    /// Last note was on time (within PGREAT)
    pub const TIMING_JUST: i32 = 62;

    // Play options (70-99)
    /// Random is enabled
    pub const OPTION_RANDOM: i32 = 70;
    /// Mirror is enabled
    pub const OPTION_MIRROR: i32 = 71;
    /// S-Random is enabled
    pub const OPTION_SRANDOM: i32 = 72;
    /// R-Random is enabled
    pub const OPTION_RRANDOM: i32 = 73;
    /// SUDDEN+ is enabled
    pub const OPTION_SUDDEN: i32 = 80;
    /// HIDDEN+ is enabled
    pub const OPTION_HIDDEN: i32 = 81;
    /// LIFT is enabled
    pub const OPTION_LIFT: i32 = 82;

    // Clear state (100-119)
    /// No play record
    pub const CLEAR_NO_PLAY: i32 = 100;
    /// Failed
    pub const CLEAR_FAILED: i32 = 101;
    /// Assist clear
    pub const CLEAR_ASSIST: i32 = 102;
    /// Easy clear
    pub const CLEAR_EASY: i32 = 103;
    /// Normal clear
    pub const CLEAR_NORMAL: i32 = 104;
    /// Hard clear
    pub const CLEAR_HARD: i32 = 105;
    /// Ex-hard clear
    pub const CLEAR_EXHARD: i32 = 106;
    /// Full combo
    pub const CLEAR_FULLCOMBO: i32 = 107;
    /// Perfect (all PGreat)
    pub const CLEAR_PERFECT: i32 = 108;
    /// Max (100% Perfect)
    pub const CLEAR_MAX: i32 = 109;

    // Rank (120-139)
    /// DJ LEVEL F
    pub const RANK_F: i32 = 120;
    /// DJ LEVEL E
    pub const RANK_E: i32 = 121;
    /// DJ LEVEL D
    pub const RANK_D: i32 = 122;
    /// DJ LEVEL C
    pub const RANK_C: i32 = 123;
    /// DJ LEVEL B
    pub const RANK_B: i32 = 124;
    /// DJ LEVEL A
    pub const RANK_A: i32 = 125;
    /// DJ LEVEL AA
    pub const RANK_AA: i32 = 126;
    /// DJ LEVEL AAA
    pub const RANK_AAA: i32 = 127;

    // Song state (140-159)
    /// BGA is loading
    pub const BGA_LOADING: i32 = 140;
    /// BGA is available
    pub const BGA_AVAILABLE: i32 = 141;
    /// Has BPM changes
    pub const HAS_BPM_CHANGE: i32 = 142;
    /// Has long notes
    pub const HAS_LONGNOTE: i32 = 143;
    /// Has mines
    pub const HAS_MINE: i32 = 144;

    // Key state (200-299)
    /// Key 1 pressed (1P side)
    pub const KEY_1P_1: i32 = 200;
    /// Key 2 pressed (1P side)
    pub const KEY_1P_2: i32 = 201;
    /// Key 3 pressed (1P side)
    pub const KEY_1P_3: i32 = 202;
    /// Key 4 pressed (1P side)
    pub const KEY_1P_4: i32 = 203;
    /// Key 5 pressed (1P side)
    pub const KEY_1P_5: i32 = 204;
    /// Key 6 pressed (1P side)
    pub const KEY_1P_6: i32 = 205;
    /// Key 7 pressed (1P side)
    pub const KEY_1P_7: i32 = 206;
    /// Scratch pressed (1P side)
    pub const KEY_1P_SCRATCH: i32 = 207;

    // LN hold state (300-399)
    /// LN active on lane 1 (1P)
    pub const LN_1P_1: i32 = 300;
    /// LN active on lane 2 (1P)
    pub const LN_1P_2: i32 = 301;
    /// LN active on lane 3 (1P)
    pub const LN_1P_3: i32 = 302;
    /// LN active on lane 4 (1P)
    pub const LN_1P_4: i32 = 303;
    /// LN active on lane 5 (1P)
    pub const LN_1P_5: i32 = 304;
    /// LN active on lane 6 (1P)
    pub const LN_1P_6: i32 = 305;
    /// LN active on lane 7 (1P)
    pub const LN_1P_7: i32 = 306;
    /// LN active on scratch (1P)
    pub const LN_1P_SCRATCH: i32 = 307;

    // Full combo state (400-409)
    /// Currently maintaining full combo
    pub const FULLCOMBO_ONGOING: i32 = 400;
    /// Full combo broken
    pub const FULLCOMBO_BROKEN: i32 = 401;

    // Score update (410-419)
    /// Best score updated
    pub const BEST_UPDATED: i32 = 410;
    /// Target score reached
    pub const TARGET_REACHED: i32 = 411;

    // Custom operations (900-999)
    // These are defined by individual skins through the property system
    pub const CUSTOM_START: i32 = 900;
    pub const CUSTOM_END: i32 = 999;

    // Negation bit
    // When a code has this bit set, the condition is negated
    pub const NEGATE_BIT: i32 = 1000;
}

/// Timer IDs for animation timing
pub mod timers {
    /// Always on (starts at scene load)
    pub const SCENE_START: i32 = 0;
    /// Skin loaded timer
    pub const SKIN_LOADED: i32 = 1;
    /// Play starts timer
    pub const PLAY_START: i32 = 10;
    /// Music starts timer
    pub const MUSIC_START: i32 = 11;
    /// Play ends timer
    pub const PLAY_END: i32 = 12;
    /// Failed timer
    pub const FAILED: i32 = 13;
    /// Result loaded timer
    pub const RESULT_LOADED: i32 = 20;

    // Judge timers (per player, per judge type)
    /// 1P PGREAT timer
    pub const JUDGE_1P_PGREAT: i32 = 50;
    /// 1P GREAT timer
    pub const JUDGE_1P_GREAT: i32 = 51;
    /// 1P GOOD timer
    pub const JUDGE_1P_GOOD: i32 = 52;
    /// 1P BAD timer
    pub const JUDGE_1P_BAD: i32 = 53;
    /// 1P POOR timer
    pub const JUDGE_1P_POOR: i32 = 54;
    /// 1P MISS timer
    pub const JUDGE_1P_MISS: i32 = 55;

    // Key press timers (per lane)
    /// 1P Key 1 press timer
    pub const KEY_1P_1_ON: i32 = 100;
    /// 1P Key 2 press timer
    pub const KEY_1P_2_ON: i32 = 101;
    /// 1P Key 3 press timer
    pub const KEY_1P_3_ON: i32 = 102;
    /// 1P Key 4 press timer
    pub const KEY_1P_4_ON: i32 = 103;
    /// 1P Key 5 press timer
    pub const KEY_1P_5_ON: i32 = 104;
    /// 1P Key 6 press timer
    pub const KEY_1P_6_ON: i32 = 105;
    /// 1P Key 7 press timer
    pub const KEY_1P_7_ON: i32 = 106;
    /// 1P Scratch press timer
    pub const KEY_1P_SCRATCH_ON: i32 = 107;

    // Key release timers
    /// 1P Key 1 release timer
    pub const KEY_1P_1_OFF: i32 = 120;

    // Combo milestone timers
    /// Combo 100 timer
    pub const COMBO_100: i32 = 200;
    /// Combo 500 timer
    pub const COMBO_500: i32 = 201;
    /// Combo 1000 timer
    pub const COMBO_1000: i32 = 202;

    // Custom timers
    pub const CUSTOM_START: i32 = 900;
    pub const CUSTOM_END: i32 = 999;

    // Special: timer not set / disabled
    pub const DISABLED: i32 = -1;
}

/// Value sources for number/slider elements
pub mod values {
    // Score values (0-99)
    /// Current EX Score
    pub const EX_SCORE: i32 = 0;
    /// Target EX Score
    pub const TARGET_SCORE: i32 = 1;
    /// Best EX Score
    pub const BEST_SCORE: i32 = 2;
    /// Score rate (percentage)
    pub const SCORE_RATE: i32 = 3;
    /// Max possible score
    pub const MAX_SCORE: i32 = 4;

    // Combo values (10-19)
    /// Current combo
    pub const COMBO: i32 = 10;
    /// Max combo
    pub const MAX_COMBO: i32 = 11;
    /// Target combo
    pub const TARGET_COMBO: i32 = 12;

    // Judge count values (20-39)
    /// PGREAT count
    pub const PGREAT_COUNT: i32 = 20;
    /// GREAT count
    pub const GREAT_COUNT: i32 = 21;
    /// GOOD count
    pub const GOOD_COUNT: i32 = 22;
    /// BAD count
    pub const BAD_COUNT: i32 = 23;
    /// POOR count
    pub const POOR_COUNT: i32 = 24;
    /// MISS count
    pub const MISS_COUNT: i32 = 25;

    // FAST/SLOW counts (30-39)
    /// FAST count
    pub const FAST_COUNT: i32 = 30;
    /// SLOW count
    pub const SLOW_COUNT: i32 = 31;

    // Gauge values (40-49)
    /// Gauge percentage (0-100)
    pub const GAUGE: i32 = 40;
    /// Groove gauge
    pub const GAUGE_GROOVE: i32 = 41;
    /// Hard gauge
    pub const GAUGE_HARD: i32 = 42;
    /// Ex-hard gauge
    pub const GAUGE_EXHARD: i32 = 43;

    // Progress values (50-59)
    /// Song progress (0-100)
    pub const PROGRESS: i32 = 50;
    /// Loading progress
    pub const LOADING_PROGRESS: i32 = 51;

    // BPM values (60-69)
    /// Current BPM
    pub const BPM_CURRENT: i32 = 60;
    /// Min BPM
    pub const BPM_MIN: i32 = 61;
    /// Max BPM
    pub const BPM_MAX: i32 = 62;

    // Hi-speed values (70-79)
    /// Hi-speed multiplier (x100)
    pub const HISPEED: i32 = 70;
    /// Green number
    pub const GREEN_NUMBER: i32 = 71;
    /// Duration (ms)
    pub const DURATION: i32 = 72;

    // Lane cover values (80-89)
    /// SUDDEN+ percentage
    pub const SUDDEN_PERCENT: i32 = 80;
    /// HIDDEN+ percentage
    pub const HIDDEN_PERCENT: i32 = 81;
    /// LIFT percentage
    pub const LIFT_PERCENT: i32 = 82;

    // Time values (90-99)
    /// Play time in seconds
    pub const PLAY_TIME: i32 = 90;
    /// Play time minutes
    pub const PLAY_TIME_MIN: i32 = 91;
    /// Play time seconds
    pub const PLAY_TIME_SEC: i32 = 92;
    /// Total song time in seconds
    pub const TOTAL_TIME: i32 = 93;

    // Timing values (100-109)
    /// Last timing difference (ms)
    pub const TIMING_MS: i32 = 100;
    /// Average timing (ms)
    pub const TIMING_AVG: i32 = 101;

    // Song info (110-119)
    /// Difficulty level
    pub const LEVEL: i32 = 110;
    /// Total notes
    pub const TOTAL_NOTES: i32 = 111;
    /// Remaining notes
    pub const REMAINING_NOTES: i32 = 112;

    // Rank values (120-129)
    /// Current DJ rank (0-8)
    pub const RANK: i32 = 120;
    /// Target rank
    pub const TARGET_RANK: i32 = 121;
    /// Best rank
    pub const BEST_RANK: i32 = 122;
}

/// Game state for condition evaluation
#[derive(Debug, Clone, Default)]
pub struct GameState {
    // Play options
    pub auto_play: bool,
    pub replay: bool,
    pub practice: bool,
    pub course_mode: bool,

    // Gauge
    pub gauge_type: GaugeType,
    pub gauge_value: f32,

    // Judge
    pub last_judge: Option<JudgeType>,
    pub last_timing: Option<TimingType>,

    // Options
    pub random_enabled: bool,
    pub mirror_enabled: bool,
    pub srandom_enabled: bool,
    pub rrandom_enabled: bool,
    pub sudden_enabled: bool,
    pub hidden_enabled: bool,
    pub lift_enabled: bool,

    // Clear state
    pub clear_type: ClearType,
    pub rank: DjRank,

    // Key state
    pub keys_pressed: [bool; 8],
    pub ln_active: [bool; 8],

    // Full combo state
    pub fullcombo_ongoing: bool,

    // Score state
    pub best_updated: bool,
    pub target_reached: bool,

    // BGA state
    pub bga_loading: bool,
    pub bga_available: bool,

    // Song info
    pub has_bpm_change: bool,
    pub has_longnote: bool,
    pub has_mine: bool,

    // Custom properties (op codes 900-999)
    pub custom_options: std::collections::HashMap<i32, bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GaugeType {
    #[default]
    Groove,
    Hard,
    ExHard,
    Easy,
    AssistEasy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeType {
    PGreat,
    Great,
    Good,
    Bad,
    Poor,
    Miss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingType {
    Fast,
    Slow,
    Just,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClearType {
    #[default]
    NoPlay,
    Failed,
    AssistClear,
    EasyClear,
    NormalClear,
    HardClear,
    ExHardClear,
    FullCombo,
    Perfect,
    Max,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum DjRank {
    #[default]
    F,
    E,
    D,
    C,
    B,
    A,
    AA,
    AAA,
}

/// Evaluate a list of operation conditions
pub fn evaluate_conditions(ops: &[i32], state: &GameState) -> bool {
    // Empty conditions = always show
    if ops.is_empty() {
        return true;
    }

    // All conditions must be true (AND logic)
    ops.iter().all(|&op| evaluate_condition(op, state))
}

/// Evaluate a single operation condition
pub fn evaluate_condition(op: i32, state: &GameState) -> bool {
    // Check for negation
    let negated = op >= opcodes::NEGATE_BIT;
    let base_op = if negated {
        op - opcodes::NEGATE_BIT
    } else {
        op
    };

    let result = evaluate_base_condition(base_op, state);

    if negated { !result } else { result }
}

fn evaluate_base_condition(op: i32, state: &GameState) -> bool {
    use opcodes::*;

    match op {
        ALWAYS_TRUE => true,

        // Play state
        AUTO_PLAY => state.auto_play,
        REPLAY => state.replay,
        PRACTICE => state.practice,
        COURSE_MODE => state.course_mode,

        // Gauge type
        GAUGE_GROOVE => state.gauge_type == GaugeType::Groove,
        GAUGE_HARD => state.gauge_type == GaugeType::Hard,
        GAUGE_EXHARD => state.gauge_type == GaugeType::ExHard,
        GAUGE_EASY => state.gauge_type == GaugeType::Easy,
        GAUGE_ASSIST_EASY => state.gauge_type == GaugeType::AssistEasy,

        // Gauge state
        GAUGE_RED => state.gauge_value < 30.0,
        GAUGE_YELLOW => state.gauge_value >= 30.0 && state.gauge_value < 80.0,
        GAUGE_GREEN => state.gauge_value >= 80.0,
        GAUGE_MAX => state.gauge_value >= 100.0,

        // Judge result
        JUDGE_PGREAT => state.last_judge == Some(JudgeType::PGreat),
        JUDGE_GREAT => state.last_judge == Some(JudgeType::Great),
        JUDGE_GOOD => state.last_judge == Some(JudgeType::Good),
        JUDGE_BAD => state.last_judge == Some(JudgeType::Bad),
        JUDGE_POOR => state.last_judge == Some(JudgeType::Poor),
        JUDGE_MISS => state.last_judge == Some(JudgeType::Miss),

        // Timing
        TIMING_FAST => state.last_timing == Some(TimingType::Fast),
        TIMING_SLOW => state.last_timing == Some(TimingType::Slow),
        TIMING_JUST => state.last_timing == Some(TimingType::Just),

        // Options
        OPTION_RANDOM => state.random_enabled,
        OPTION_MIRROR => state.mirror_enabled,
        OPTION_SRANDOM => state.srandom_enabled,
        OPTION_RRANDOM => state.rrandom_enabled,
        OPTION_SUDDEN => state.sudden_enabled,
        OPTION_HIDDEN => state.hidden_enabled,
        OPTION_LIFT => state.lift_enabled,

        // Clear state
        CLEAR_NO_PLAY => state.clear_type == ClearType::NoPlay,
        CLEAR_FAILED => state.clear_type == ClearType::Failed,
        CLEAR_ASSIST => state.clear_type == ClearType::AssistClear,
        CLEAR_EASY => state.clear_type == ClearType::EasyClear,
        CLEAR_NORMAL => state.clear_type == ClearType::NormalClear,
        CLEAR_HARD => state.clear_type == ClearType::HardClear,
        CLEAR_EXHARD => state.clear_type == ClearType::ExHardClear,
        CLEAR_FULLCOMBO => state.clear_type == ClearType::FullCombo,
        CLEAR_PERFECT => state.clear_type == ClearType::Perfect,
        CLEAR_MAX => state.clear_type == ClearType::Max,

        // Rank
        RANK_F => state.rank == DjRank::F,
        RANK_E => state.rank == DjRank::E,
        RANK_D => state.rank == DjRank::D,
        RANK_C => state.rank == DjRank::C,
        RANK_B => state.rank == DjRank::B,
        RANK_A => state.rank == DjRank::A,
        RANK_AA => state.rank == DjRank::AA,
        RANK_AAA => state.rank == DjRank::AAA,

        // Song state
        BGA_LOADING => state.bga_loading,
        BGA_AVAILABLE => state.bga_available,
        HAS_BPM_CHANGE => state.has_bpm_change,
        HAS_LONGNOTE => state.has_longnote,
        HAS_MINE => state.has_mine,

        // Key state (1P)
        KEY_1P_1 => state.keys_pressed.first().copied().unwrap_or(false),
        KEY_1P_2 => state.keys_pressed.get(1).copied().unwrap_or(false),
        KEY_1P_3 => state.keys_pressed.get(2).copied().unwrap_or(false),
        KEY_1P_4 => state.keys_pressed.get(3).copied().unwrap_or(false),
        KEY_1P_5 => state.keys_pressed.get(4).copied().unwrap_or(false),
        KEY_1P_6 => state.keys_pressed.get(5).copied().unwrap_or(false),
        KEY_1P_7 => state.keys_pressed.get(6).copied().unwrap_or(false),
        KEY_1P_SCRATCH => state.keys_pressed.get(7).copied().unwrap_or(false),

        // LN state (1P)
        LN_1P_1 => state.ln_active.first().copied().unwrap_or(false),
        LN_1P_2 => state.ln_active.get(1).copied().unwrap_or(false),
        LN_1P_3 => state.ln_active.get(2).copied().unwrap_or(false),
        LN_1P_4 => state.ln_active.get(3).copied().unwrap_or(false),
        LN_1P_5 => state.ln_active.get(4).copied().unwrap_or(false),
        LN_1P_6 => state.ln_active.get(5).copied().unwrap_or(false),
        LN_1P_7 => state.ln_active.get(6).copied().unwrap_or(false),
        LN_1P_SCRATCH => state.ln_active.get(7).copied().unwrap_or(false),

        // Full combo state
        FULLCOMBO_ONGOING => state.fullcombo_ongoing,
        FULLCOMBO_BROKEN => !state.fullcombo_ongoing,

        // Score state
        BEST_UPDATED => state.best_updated,
        TARGET_REACHED => state.target_reached,

        // Custom options (900-999)
        op if (CUSTOM_START..=CUSTOM_END).contains(&op) => {
            state.custom_options.get(&op).copied().unwrap_or(false)
        }

        // Unknown op code - default to true
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_empty_conditions() {
        let state = GameState::default();
        assert!(evaluate_conditions(&[], &state));
    }

    #[test]
    fn test_evaluate_always_true() {
        let state = GameState::default();
        assert!(evaluate_conditions(&[opcodes::ALWAYS_TRUE], &state));
    }

    #[test]
    fn test_evaluate_gauge_conditions() {
        let mut state = GameState::default();

        state.gauge_value = 20.0;
        assert!(evaluate_condition(opcodes::GAUGE_RED, &state));
        assert!(!evaluate_condition(opcodes::GAUGE_YELLOW, &state));

        state.gauge_value = 50.0;
        assert!(!evaluate_condition(opcodes::GAUGE_RED, &state));
        assert!(evaluate_condition(opcodes::GAUGE_YELLOW, &state));

        state.gauge_value = 90.0;
        assert!(!evaluate_condition(opcodes::GAUGE_YELLOW, &state));
        assert!(evaluate_condition(opcodes::GAUGE_GREEN, &state));
    }

    #[test]
    fn test_evaluate_negation() {
        let mut state = GameState::default();
        state.auto_play = true;

        // Without negation
        assert!(evaluate_condition(opcodes::AUTO_PLAY, &state));

        // With negation
        assert!(!evaluate_condition(
            opcodes::AUTO_PLAY + opcodes::NEGATE_BIT,
            &state
        ));
    }

    #[test]
    fn test_evaluate_multiple_conditions_and() {
        let mut state = GameState::default();
        state.gauge_type = GaugeType::Hard;
        state.gauge_value = 50.0;

        // Both conditions true
        assert!(evaluate_conditions(
            &[opcodes::GAUGE_HARD, opcodes::GAUGE_YELLOW],
            &state
        ));

        // One condition false
        assert!(!evaluate_conditions(
            &[opcodes::GAUGE_HARD, opcodes::GAUGE_GREEN],
            &state
        ));
    }

    #[test]
    fn test_evaluate_key_state() {
        let mut state = GameState::default();
        state.keys_pressed[0] = true;
        state.keys_pressed[2] = true;

        assert!(evaluate_condition(opcodes::KEY_1P_1, &state));
        assert!(!evaluate_condition(opcodes::KEY_1P_2, &state));
        assert!(evaluate_condition(opcodes::KEY_1P_3, &state));
    }

    #[test]
    fn test_custom_options() {
        let mut state = GameState::default();
        state.custom_options.insert(900, true);
        state.custom_options.insert(901, false);

        assert!(evaluate_condition(900, &state));
        assert!(!evaluate_condition(901, &state));
        assert!(!evaluate_condition(902, &state)); // Not set = false
    }
}
