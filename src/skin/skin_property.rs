// Skin property IDs ported from beatoraja's SkinProperty.java.
// Only commonly used IDs are included.

// ============================================================================
// TIMER IDs
// ============================================================================

pub const TIMER_STARTINPUT: i32 = 1;
pub const TIMER_FADEOUT: i32 = 2;
pub const TIMER_FAILED: i32 = 3;

pub const TIMER_SONGBAR_MOVE: i32 = 10;
pub const TIMER_SONGBAR_CHANGE: i32 = 11;

pub const TIMER_PANEL1_ON: i32 = 21;
pub const TIMER_PANEL2_ON: i32 = 22;
pub const TIMER_PANEL3_ON: i32 = 23;
pub const TIMER_PANEL1_OFF: i32 = 31;
pub const TIMER_PANEL2_OFF: i32 = 32;
pub const TIMER_PANEL3_OFF: i32 = 33;

pub const TIMER_READY: i32 = 40;
pub const TIMER_PLAY: i32 = 41;
pub const TIMER_GAUGE_INCREASE_1P: i32 = 42;
pub const TIMER_GAUGE_INCREASE_2P: i32 = 43;
pub const TIMER_GAUGE_MAX_1P: i32 = 44;
pub const TIMER_GAUGE_MAX_2P: i32 = 45;
pub const TIMER_JUDGE_1P: i32 = 46;
pub const TIMER_JUDGE_2P: i32 = 47;
pub const TIMER_FULLCOMBO_1P: i32 = 48;
pub const TIMER_FULLCOMBO_2P: i32 = 49;

// Bomb timers (1P)
pub const TIMER_BOMB_1P_SCRATCH: i32 = 50;
pub const TIMER_BOMB_1P_KEY1: i32 = 51;
pub const TIMER_BOMB_1P_KEY2: i32 = 52;
pub const TIMER_BOMB_1P_KEY3: i32 = 53;
pub const TIMER_BOMB_1P_KEY4: i32 = 54;
pub const TIMER_BOMB_1P_KEY5: i32 = 55;
pub const TIMER_BOMB_1P_KEY6: i32 = 56;
pub const TIMER_BOMB_1P_KEY7: i32 = 57;

// Bomb timers (2P)
pub const TIMER_BOMB_2P_SCRATCH: i32 = 60;
pub const TIMER_BOMB_2P_KEY1: i32 = 61;

// Hold timers (1P)
pub const TIMER_HOLD_1P_SCRATCH: i32 = 70;
pub const TIMER_HOLD_1P_KEY1: i32 = 71;
pub const TIMER_HOLD_1P_KEY2: i32 = 72;
pub const TIMER_HOLD_1P_KEY3: i32 = 73;
pub const TIMER_HOLD_1P_KEY4: i32 = 74;
pub const TIMER_HOLD_1P_KEY5: i32 = 75;
pub const TIMER_HOLD_1P_KEY6: i32 = 76;
pub const TIMER_HOLD_1P_KEY7: i32 = 77;

// Hold timers (2P)
pub const TIMER_HOLD_2P_SCRATCH: i32 = 80;
pub const TIMER_HOLD_2P_KEY1: i32 = 81;

// Key on timers (1P)
pub const TIMER_KEYON_1P_SCRATCH: i32 = 100;
pub const TIMER_KEYON_1P_KEY1: i32 = 101;
pub const TIMER_KEYON_1P_KEY2: i32 = 102;
pub const TIMER_KEYON_1P_KEY3: i32 = 103;
pub const TIMER_KEYON_1P_KEY4: i32 = 104;
pub const TIMER_KEYON_1P_KEY5: i32 = 105;
pub const TIMER_KEYON_1P_KEY6: i32 = 106;
pub const TIMER_KEYON_1P_KEY7: i32 = 107;

// Key on timers (2P)
pub const TIMER_KEYON_2P_SCRATCH: i32 = 110;
pub const TIMER_KEYON_2P_KEY1: i32 = 111;

// Key off timers (1P)
pub const TIMER_KEYOFF_1P_SCRATCH: i32 = 120;
pub const TIMER_KEYOFF_1P_KEY1: i32 = 121;
pub const TIMER_KEYOFF_1P_KEY2: i32 = 122;
pub const TIMER_KEYOFF_1P_KEY3: i32 = 123;
pub const TIMER_KEYOFF_1P_KEY4: i32 = 124;
pub const TIMER_KEYOFF_1P_KEY5: i32 = 125;
pub const TIMER_KEYOFF_1P_KEY6: i32 = 126;
pub const TIMER_KEYOFF_1P_KEY7: i32 = 127;

// Key off timers (2P)
pub const TIMER_KEYOFF_2P_SCRATCH: i32 = 130;
pub const TIMER_KEYOFF_2P_KEY1: i32 = 131;

pub const TIMER_RHYTHM: i32 = 140;
pub const TIMER_ENDOFNOTE_1P: i32 = 143;
pub const TIMER_ENDOFNOTE_2P: i32 = 144;

pub const TIMER_RESULTGRAPH_BEGIN: i32 = 150;
pub const TIMER_RESULTGRAPH_END: i32 = 151;
pub const TIMER_RESULT_UPDATESCORE: i32 = 152;

// Combo timer
pub const TIMER_COMBO_1P: i32 = 446;
pub const TIMER_COMBO_2P: i32 = 447;

// Timer off value
pub const TIMER_OFF_VALUE: i64 = i64::MIN;

// ============================================================================
// NUMBER IDs
// ============================================================================

pub const NUMBER_HISPEED_LR2: i32 = 10;
pub const NUMBER_JUDGETIMING: i32 = 12;
pub const NUMBER_LANECOVER1: i32 = 14;

pub const NUMBER_TOTALPLAYTIME_HOUR: i32 = 17;
pub const NUMBER_TOTALPLAYTIME_MINUTE: i32 = 18;
pub const NUMBER_TOTALPLAYTIME_SECOND: i32 = 19;
pub const NUMBER_CURRENT_FPS: i32 = 20;

pub const NUMBER_TIME_YEAR: i32 = 21;
pub const NUMBER_TIME_MONTH: i32 = 22;
pub const NUMBER_TIME_DAY: i32 = 23;
pub const NUMBER_TIME_HOUR: i32 = 24;
pub const NUMBER_TIME_MINUTE: i32 = 25;
pub const NUMBER_TIME_SECOND: i32 = 26;

pub const NUMBER_SCORE: i32 = 71;
pub const NUMBER_MAXSCORE: i32 = 72;
pub const NUMBER_TOTALNOTES: i32 = 74;
pub const NUMBER_MAXCOMBO: i32 = 75;
pub const NUMBER_MISSCOUNT: i32 = 76;

pub const NUMBER_MAXBPM: i32 = 90;
pub const NUMBER_MINBPM: i32 = 91;
pub const NUMBER_MAINBPM: i32 = 92;
pub const NUMBER_PLAYLEVEL: i32 = 96;

pub const NUMBER_SCORE2: i32 = 101;
pub const NUMBER_SCORE_RATE: i32 = 102;
pub const NUMBER_SCORE_RATE_AFTERDOT: i32 = 103;
pub const NUMBER_COMBO: i32 = 104;
pub const NUMBER_MAXCOMBO2: i32 = 105;
pub const NUMBER_TOTALNOTES2: i32 = 106;
pub const NUMBER_GROOVEGAUGE: i32 = 107;
pub const NUMBER_DIFF_EXSCORE: i32 = 108;

pub const NUMBER_PERFECT: i32 = 110;
pub const NUMBER_GREAT: i32 = 111;
pub const NUMBER_GOOD: i32 = 112;
pub const NUMBER_BAD: i32 = 113;
pub const NUMBER_POOR: i32 = 114;
pub const NUMBER_TOTAL_RATE: i32 = 115;
pub const NUMBER_TOTAL_RATE_AFTERDOT: i32 = 116;

pub const NUMBER_NOWBPM: i32 = 160;
pub const NUMBER_PLAYTIME_MINUTE: i32 = 161;
pub const NUMBER_PLAYTIME_SECOND: i32 = 162;
pub const NUMBER_TIMELEFT_MINUTE: i32 = 163;
pub const NUMBER_TIMELEFT_SECOND: i32 = 164;
pub const NUMBER_LOADING_PROGRESS: i32 = 165;

pub const NUMBER_HISPEED: i32 = 310;
pub const NUMBER_HISPEED_AFTERDOT: i32 = 311;
pub const NUMBER_DURATION: i32 = 312;
pub const NUMBER_DURATION_GREEN: i32 = 313;
pub const NUMBER_LIFT1: i32 = 314;
pub const NUMBER_HIDDEN1: i32 = 315;

pub const NUMBER_GROOVEGAUGE_AFTERDOT: i32 = 407;

// Early/Late counts
pub const NUMBER_EARLY_PERFECT: i32 = 410;
pub const NUMBER_LATE_PERFECT: i32 = 411;
pub const NUMBER_EARLY_GREAT: i32 = 412;
pub const NUMBER_LATE_GREAT: i32 = 413;
pub const NUMBER_EARLY_GOOD: i32 = 414;
pub const NUMBER_LATE_GOOD: i32 = 415;
pub const NUMBER_EARLY_BAD: i32 = 416;
pub const NUMBER_LATE_BAD: i32 = 417;
pub const NUMBER_EARLY_POOR: i32 = 418;
pub const NUMBER_LATE_POOR: i32 = 419;
pub const NUMBER_MISS: i32 = 420;
pub const NUMBER_EARLY_MISS: i32 = 421;
pub const NUMBER_LATE_MISS: i32 = 422;
pub const NUMBER_TOTALEARLY: i32 = 423;
pub const NUMBER_TOTALLATE: i32 = 424;
pub const NUMBER_COMBOBREAK: i32 = 425;

// ============================================================================
// OPTION IDs
// ============================================================================

pub const OPTION_RANDOM_VALUE: i32 = -1;

pub const OPTION_FOLDERBAR: i32 = 1;
pub const OPTION_SONGBAR: i32 = 2;
pub const OPTION_GRADEBAR: i32 = 3;
pub const OPTION_PLAYABLEBAR: i32 = 5;

pub const OPTION_PANEL1: i32 = 21;
pub const OPTION_PANEL2: i32 = 22;
pub const OPTION_PANEL3: i32 = 23;

pub const OPTION_AUTOPLAYOFF: i32 = 32;
pub const OPTION_AUTOPLAYON: i32 = 33;
pub const OPTION_SCOREGRAPHOFF: i32 = 38;
pub const OPTION_SCOREGRAPHON: i32 = 39;

pub const OPTION_BGAOFF: i32 = 40;
pub const OPTION_BGAON: i32 = 41;
pub const OPTION_GAUGE_GROOVE: i32 = 42;
pub const OPTION_GAUGE_HARD: i32 = 43;
pub const OPTION_GAUGE_EX: i32 = 1046;

pub const OPTION_NOW_LOADING: i32 = 80;
pub const OPTION_LOADED: i32 = 81;

pub const OPTION_REPLAY_OFF: i32 = 82;
pub const OPTION_REPLAY_RECORDING: i32 = 83;
pub const OPTION_REPLAY_PLAYING: i32 = 84;

pub const OPTION_RESULT_CLEAR: i32 = 90;
pub const OPTION_RESULT_FAIL: i32 = 91;

// Difficulty options
pub const OPTION_DIFFICULTY0: i32 = 150;
pub const OPTION_DIFFICULTY1: i32 = 151;
pub const OPTION_DIFFICULTY2: i32 = 152;
pub const OPTION_DIFFICULTY3: i32 = 153;
pub const OPTION_DIFFICULTY4: i32 = 154;
pub const OPTION_DIFFICULTY5: i32 = 155;

pub const OPTION_7KEYSONG: i32 = 160;
pub const OPTION_5KEYSONG: i32 = 161;
pub const OPTION_14KEYSONG: i32 = 162;
pub const OPTION_10KEYSONG: i32 = 163;
pub const OPTION_9KEYSONG: i32 = 164;

pub const OPTION_NO_LN: i32 = 172;
pub const OPTION_LN: i32 = 173;
pub const OPTION_NO_BPMCHANGE: i32 = 176;
pub const OPTION_BPMCHANGE: i32 = 177;

// Rank options (1P)
pub const OPTION_1P_AAA: i32 = 200;
pub const OPTION_1P_AA: i32 = 201;
pub const OPTION_1P_A: i32 = 202;
pub const OPTION_1P_B: i32 = 203;
pub const OPTION_1P_C: i32 = 204;
pub const OPTION_1P_D: i32 = 205;
pub const OPTION_1P_E: i32 = 206;
pub const OPTION_1P_F: i32 = 207;

// Gauge range options
pub const OPTION_1P_0_9: i32 = 230;
pub const OPTION_1P_10_19: i32 = 231;
pub const OPTION_1P_20_29: i32 = 232;
pub const OPTION_1P_30_39: i32 = 233;
pub const OPTION_1P_40_49: i32 = 234;
pub const OPTION_1P_50_59: i32 = 235;
pub const OPTION_1P_60_69: i32 = 236;
pub const OPTION_1P_70_79: i32 = 237;
pub const OPTION_1P_80_89: i32 = 238;
pub const OPTION_1P_90_99: i32 = 239;
pub const OPTION_1P_100: i32 = 240;
pub const OPTION_1P_BORDER_OR_MORE: i32 = 1240;

// Judge options (1P)
pub const OPTION_1P_PERFECT: i32 = 241;
pub const OPTION_1P_GREAT: i32 = 242;
pub const OPTION_1P_EARLY: i32 = 1242;
pub const OPTION_1P_GOOD: i32 = 243;
pub const OPTION_1P_LATE: i32 = 1243;
pub const OPTION_1P_BAD: i32 = 244;
pub const OPTION_1P_POOR: i32 = 245;
pub const OPTION_1P_MISS: i32 = 246;

// Judge options (2P)
pub const OPTION_2P_PERFECT: i32 = 261;
pub const OPTION_2P_GREAT: i32 = 262;
pub const OPTION_2P_EARLY: i32 = 1262;
pub const OPTION_2P_GOOD: i32 = 263;
pub const OPTION_2P_LATE: i32 = 1263;

// Lane cover options
pub const OPTION_LANECOVER1_CHANGING: i32 = 270;
pub const OPTION_LANECOVER1_ON: i32 = 271;
pub const OPTION_LIFT1_ON: i32 = 272;
pub const OPTION_HIDDEN1_ON: i32 = 273;

// Course stage options
pub const OPTION_COURSE_STAGE1: i32 = 280;
pub const OPTION_COURSE_STAGE2: i32 = 281;
pub const OPTION_COURSE_STAGE3: i32 = 282;
pub const OPTION_COURSE_STAGE4: i32 = 283;
pub const OPTION_COURSE_STAGE_FINAL: i32 = 289;

// Result rank options (1P)
pub const OPTION_RESULT_AAA_1P: i32 = 300;
pub const OPTION_RESULT_AA_1P: i32 = 301;
pub const OPTION_RESULT_A_1P: i32 = 302;
pub const OPTION_RESULT_B_1P: i32 = 303;
pub const OPTION_RESULT_C_1P: i32 = 304;
pub const OPTION_RESULT_D_1P: i32 = 305;
pub const OPTION_RESULT_E_1P: i32 = 306;
pub const OPTION_RESULT_F_1P: i32 = 307;

// Update options
pub const OPTION_UPDATE_SCORE: i32 = 330;
pub const OPTION_UPDATE_MAXCOMBO: i32 = 331;
pub const OPTION_UPDATE_MISSCOUNT: i32 = 332;

pub const OPTION_CONSTANT: i32 = 400;

// ============================================================================
// SLIDER / BARGRAPH IDs
// ============================================================================

pub const SLIDER_MUSICSELECT_POSITION: i32 = 1;
pub const SLIDER_LANECOVER: i32 = 4;
pub const SLIDER_LANECOVER2: i32 = 5;
pub const SLIDER_MUSIC_PROGRESS: i32 = 6;

pub const BARGRAPH_MUSIC_PROGRESS: i32 = 101;
pub const BARGRAPH_LOAD_PROGRESS: i32 = 102;
pub const BARGRAPH_SCORERATE: i32 = 110;
pub const BARGRAPH_BESTSCORERATE_NOW: i32 = 112;
pub const BARGRAPH_TARGETSCORERATE_NOW: i32 = 114;

// ============================================================================
// STRING IDs
// ============================================================================

pub const STRING_RIVAL: i32 = 1;
pub const STRING_PLAYER: i32 = 2;
pub const STRING_TITLE: i32 = 10;
pub const STRING_SUBTITLE: i32 = 11;
pub const STRING_FULLTITLE: i32 = 12;
pub const STRING_GENRE: i32 = 13;
pub const STRING_ARTIST: i32 = 14;
pub const STRING_SUBARTIST: i32 = 15;
pub const STRING_FULLARTIST: i32 = 16;

// ============================================================================
// OFFSET IDs
// ============================================================================

pub const OFFSET_SCRATCHANGLE_1P: i32 = 1;
pub const OFFSET_SCRATCHANGLE_2P: i32 = 2;
pub const OFFSET_LIFT: i32 = 3;
pub const OFFSET_LANECOVER: i32 = 4;
pub const OFFSET_HIDDEN_COVER: i32 = 5;
pub const OFFSET_ALL: i32 = 10;
pub const OFFSET_NOTES_1P: i32 = 30;
pub const OFFSET_JUDGE_1P: i32 = 32;
pub const OFFSET_JUDGEDETAIL_1P: i32 = 33;

// ============================================================================
// BUTTON IDs
// ============================================================================

pub const BUTTON_MODE: i32 = 11;
pub const BUTTON_SORT: i32 = 12;
pub const BUTTON_KEYCONFIG: i32 = 13;
pub const BUTTON_SKINSELECT: i32 = 14;
pub const BUTTON_PLAY: i32 = 15;
pub const BUTTON_AUTOPLAY: i32 = 16;
pub const BUTTON_GAUGE_1P: i32 = 40;
pub const BUTTON_RANDOM_1P: i32 = 42;
pub const BUTTON_BGA: i32 = 72;
pub const BUTTON_TARGET: i32 = 77;

// ============================================================================
// VALUE IDs (for judge display)
// ============================================================================

pub const VALUE_JUDGE_1P_SCRATCH: i32 = 500;
pub const VALUE_JUDGE_1P_KEY1: i32 = 501;
pub const VALUE_JUDGE_1P_KEY2: i32 = 502;
pub const VALUE_JUDGE_1P_KEY3: i32 = 503;
pub const VALUE_JUDGE_1P_KEY4: i32 = 504;
pub const VALUE_JUDGE_1P_KEY5: i32 = 505;
pub const VALUE_JUDGE_1P_KEY6: i32 = 506;
pub const VALUE_JUDGE_1P_KEY7: i32 = 507;
pub const VALUE_JUDGE_1P: i32 = 520;
pub const VALUE_JUDGE_2P: i32 = 521;
pub const VALUE_JUDGE_1P_DURATION: i32 = 525;
pub const VALUE_JUDGE_2P_DURATION: i32 = 526;
