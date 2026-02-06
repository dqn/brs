// Skin property IDs matching beatoraja SkinProperty.java exactly.
// These constants define the protocol between skin files and the game engine.

// ============================================================================
// Image IDs
// ============================================================================

pub const IMAGE_STAGEFILE: i32 = 100;
pub const IMAGE_BACKBMP: i32 = 101;
pub const IMAGE_BANNER: i32 = 102;
pub const IMAGE_BLACK: i32 = 110;
pub const IMAGE_WHITE: i32 = 111;

// ============================================================================
// Timer IDs
// ============================================================================

pub const TIMER_STARTINPUT: i32 = 1;
pub const TIMER_FADEOUT: i32 = 2;
pub const TIMER_FAILED: i32 = 3;

pub const TIMER_SONGBAR_MOVE: i32 = 10;
pub const TIMER_SONGBAR_CHANGE: i32 = 11;
pub const TIMER_SONGBAR_MOVE_UP: i32 = 12;
pub const TIMER_SONGBAR_MOVE_DOWN: i32 = 13;
pub const TIMER_SONGBAR_STOP: i32 = 14;
pub const TIMER_README_BEGIN: i32 = 15;
pub const TIMER_README_END: i32 = 16;

pub const TIMER_PANEL1_ON: i32 = 21;
pub const TIMER_PANEL2_ON: i32 = 22;
pub const TIMER_PANEL3_ON: i32 = 23;
pub const TIMER_PANEL4_ON: i32 = 24;
pub const TIMER_PANEL5_ON: i32 = 25;
pub const TIMER_PANEL6_ON: i32 = 26;
pub const TIMER_PANEL1_OFF: i32 = 31;
pub const TIMER_PANEL2_OFF: i32 = 32;
pub const TIMER_PANEL3_OFF: i32 = 33;
pub const TIMER_PANEL4_OFF: i32 = 34;
pub const TIMER_PANEL5_OFF: i32 = 35;
pub const TIMER_PANEL6_OFF: i32 = 36;

pub const TIMER_READY: i32 = 40;
pub const TIMER_PLAY: i32 = 41;
pub const TIMER_GAUGE_INCREASE_1P: i32 = 42;
pub const TIMER_GAUGE_INCREASE_2P: i32 = 43;
pub const TIMER_GAUGE_MAX_1P: i32 = 44;
pub const TIMER_GAUGE_MAX_2P: i32 = 45;
pub const TIMER_JUDGE_1P: i32 = 46;
pub const TIMER_JUDGE_2P: i32 = 47;
pub const TIMER_JUDGE_3P: i32 = 247;
pub const TIMER_COMBO_1P: i32 = 446;
pub const TIMER_COMBO_2P: i32 = 447;
pub const TIMER_COMBO_3P: i32 = 448;
pub const TIMER_FULLCOMBO_1P: i32 = 48;
pub const TIMER_SCORE_A: i32 = 348;
pub const TIMER_SCORE_AA: i32 = 349;
pub const TIMER_SCORE_AAA: i32 = 350;
pub const TIMER_SCORE_BEST: i32 = 351;
pub const TIMER_SCORE_TARGET: i32 = 352;
pub const TIMER_FULLCOMBO_2P: i32 = 49;

pub const TIMER_BOMB_1P_SCRATCH: i32 = 50;
pub const TIMER_BOMB_1P_KEY1: i32 = 51;
pub const TIMER_BOMB_1P_KEY2: i32 = 52;
pub const TIMER_BOMB_1P_KEY3: i32 = 53;
pub const TIMER_BOMB_1P_KEY4: i32 = 54;
pub const TIMER_BOMB_1P_KEY5: i32 = 55;
pub const TIMER_BOMB_1P_KEY6: i32 = 56;
pub const TIMER_BOMB_1P_KEY7: i32 = 57;
pub const TIMER_BOMB_1P_KEY8: i32 = 58;
pub const TIMER_BOMB_1P_KEY9: i32 = 59;
pub const TIMER_BOMB_2P_SCRATCH: i32 = 60;
pub const TIMER_BOMB_2P_KEY1: i32 = 61;
pub const TIMER_BOMB_2P_KEY2: i32 = 62;
pub const TIMER_BOMB_2P_KEY3: i32 = 63;
pub const TIMER_BOMB_2P_KEY4: i32 = 64;
pub const TIMER_BOMB_2P_KEY5: i32 = 65;
pub const TIMER_BOMB_2P_KEY6: i32 = 66;
pub const TIMER_BOMB_2P_KEY7: i32 = 67;
pub const TIMER_BOMB_2P_KEY8: i32 = 68;
pub const TIMER_BOMB_2P_KEY9: i32 = 69;

pub const TIMER_HOLD_1P_SCRATCH: i32 = 70;
pub const TIMER_HOLD_1P_KEY1: i32 = 71;
pub const TIMER_HOLD_2P_SCRATCH: i32 = 80;
pub const TIMER_HOLD_2P_KEY1: i32 = 81;

pub const TIMER_HCN_ACTIVE_1P_SCRATCH: i32 = 250;
pub const TIMER_HCN_ACTIVE_1P_KEY1: i32 = 251;
pub const TIMER_HCN_DAMAGE_1P_SCRATCH: i32 = 270;
pub const TIMER_HCN_DAMAGE_1P_KEY1: i32 = 271;

pub const TIMER_KEYON_1P_SCRATCH: i32 = 100;
pub const TIMER_KEYON_1P_KEY1: i32 = 101;
pub const TIMER_KEYON_1P_KEY2: i32 = 102;
pub const TIMER_KEYON_1P_KEY3: i32 = 103;
pub const TIMER_KEYON_1P_KEY4: i32 = 104;
pub const TIMER_KEYON_1P_KEY5: i32 = 105;
pub const TIMER_KEYON_1P_KEY6: i32 = 106;
pub const TIMER_KEYON_1P_KEY7: i32 = 107;
pub const TIMER_KEYON_1P_KEY8: i32 = 108;
pub const TIMER_KEYON_1P_KEY9: i32 = 109;
pub const TIMER_KEYON_2P_SCRATCH: i32 = 110;
pub const TIMER_KEYON_2P_KEY1: i32 = 111;
pub const TIMER_KEYON_2P_KEY2: i32 = 112;
pub const TIMER_KEYON_2P_KEY3: i32 = 113;
pub const TIMER_KEYON_2P_KEY4: i32 = 114;
pub const TIMER_KEYON_2P_KEY5: i32 = 115;
pub const TIMER_KEYON_2P_KEY6: i32 = 116;
pub const TIMER_KEYON_2P_KEY7: i32 = 117;
pub const TIMER_KEYON_2P_KEY8: i32 = 118;
pub const TIMER_KEYON_2P_KEY9: i32 = 119;

pub const TIMER_KEYOFF_1P_SCRATCH: i32 = 120;
pub const TIMER_KEYOFF_1P_KEY1: i32 = 121;
pub const TIMER_KEYOFF_1P_KEY2: i32 = 122;
pub const TIMER_KEYOFF_1P_KEY3: i32 = 123;
pub const TIMER_KEYOFF_1P_KEY4: i32 = 124;
pub const TIMER_KEYOFF_1P_KEY5: i32 = 125;
pub const TIMER_KEYOFF_1P_KEY6: i32 = 126;
pub const TIMER_KEYOFF_1P_KEY7: i32 = 127;
pub const TIMER_KEYOFF_1P_KEY8: i32 = 128;
pub const TIMER_KEYOFF_1P_KEY9: i32 = 129;
pub const TIMER_KEYOFF_2P_SCRATCH: i32 = 130;
pub const TIMER_KEYOFF_2P_KEY1: i32 = 131;
pub const TIMER_KEYOFF_2P_KEY2: i32 = 132;
pub const TIMER_KEYOFF_2P_KEY3: i32 = 133;
pub const TIMER_KEYOFF_2P_KEY4: i32 = 134;
pub const TIMER_KEYOFF_2P_KEY5: i32 = 135;
pub const TIMER_KEYOFF_2P_KEY6: i32 = 136;
pub const TIMER_KEYOFF_2P_KEY7: i32 = 137;
pub const TIMER_KEYOFF_2P_KEY8: i32 = 138;
pub const TIMER_KEYOFF_2P_KEY9: i32 = 139;

pub const TIMER_RHYTHM: i32 = 140;
pub const TIMER_ENDOFNOTE_1P: i32 = 143;
pub const TIMER_ENDOFNOTE_2P: i32 = 144;

pub const TIMER_RESULTGRAPH_BEGIN: i32 = 150;
pub const TIMER_RESULTGRAPH_END: i32 = 151;
pub const TIMER_RESULT_UPDATESCORE: i32 = 152;

pub const TIMER_IR_CONNECT_BEGIN: i32 = 172;
pub const TIMER_IR_CONNECT_SUCCESS: i32 = 173;
pub const TIMER_IR_CONNECT_FAIL: i32 = 174;

pub const TIMER_PM_CHARA_1P_NEUTRAL: i32 = 900;
pub const TIMER_PM_CHARA_1P_FEVER: i32 = 901;
pub const TIMER_PM_CHARA_1P_GREAT: i32 = 902;
pub const TIMER_PM_CHARA_1P_GOOD: i32 = 903;
pub const TIMER_PM_CHARA_1P_BAD: i32 = 904;
pub const TIMER_PM_CHARA_2P_NEUTRAL: i32 = 905;
pub const TIMER_PM_CHARA_2P_GREAT: i32 = 906;
pub const TIMER_PM_CHARA_2P_BAD: i32 = 907;
pub const TIMER_MUSIC_END: i32 = 908;
pub const TIMER_PM_CHARA_DANCE: i32 = 909;

// Extended timers
pub const TIMER_BOMB_1P_KEY10: i32 = 1010;
pub const TIMER_BOMB_2P_KEY10: i32 = 1110;
pub const TIMER_HOLD_1P_KEY10: i32 = 1210;
pub const TIMER_HOLD_2P_KEY10: i32 = 1310;
pub const TIMER_KEYON_1P_KEY10: i32 = 1410;
pub const TIMER_KEYON_2P_KEY10: i32 = 1510;
pub const TIMER_KEYOFF_1P_KEY10: i32 = 1610;
pub const TIMER_KEYOFF_2P_KEY10: i32 = 1710;
pub const TIMER_HCN_ACTIVE_1P_KEY10: i32 = 1810;
pub const TIMER_HCN_ACTIVE_2P_KEY10: i32 = 1910;
pub const TIMER_HCN_DAMAGE_1P_KEY10: i32 = 2010;
pub const TIMER_HCN_DAMAGE_2P_KEY10: i32 = 2110;

pub const TIMER_MAX: i32 = 2999;

pub const TIMER_CUSTOM_BEGIN: i32 = 10000;
pub const TIMER_CUSTOM_END: i32 = 19999;

/// Timer OFF sentinel value (matches Long.MIN_VALUE in Java).
pub const TIMER_OFF_VALUE: i64 = i64::MIN;

// ============================================================================
// Slider IDs
// ============================================================================

pub const SLIDER_MUSICSELECT_POSITION: i32 = 1;
pub const SLIDER_LANECOVER: i32 = 4;
pub const SLIDER_LANECOVER2: i32 = 5;
pub const SLIDER_MUSIC_PROGRESS: i32 = 6;
pub const SLIDER_SKINSELECT_POSITION: i32 = 7;
pub const SLIDER_MASTER_VOLUME: i32 = 17;
pub const SLIDER_KEY_VOLUME: i32 = 18;
pub const SLIDER_BGM_VOLUME: i32 = 19;

// ============================================================================
// Bargraph IDs
// ============================================================================

pub const BARGRAPH_MUSIC_PROGRESS: i32 = 101;
pub const BARGRAPH_LOAD_PROGRESS: i32 = 102;
pub const BARGRAPH_LEVEL: i32 = 103;
pub const BARGRAPH_LEVEL_BEGINNER: i32 = 105;
pub const BARGRAPH_LEVEL_NORMAL: i32 = 106;
pub const BARGRAPH_LEVEL_HYPER: i32 = 107;
pub const BARGRAPH_LEVEL_ANOTHER: i32 = 108;
pub const BARGRAPH_LEVEL_INSANE: i32 = 109;
pub const BARGRAPH_SCORERATE: i32 = 110;
pub const BARGRAPH_SCORERATE_FINAL: i32 = 111;
pub const BARGRAPH_BESTSCORERATE_NOW: i32 = 112;
pub const BARGRAPH_BESTSCORERATE: i32 = 113;
pub const BARGRAPH_TARGETSCORERATE_NOW: i32 = 114;
pub const BARGRAPH_TARGETSCORERATE: i32 = 115;
pub const BARGRAPH_RATE_PGREAT: i32 = 140;
pub const BARGRAPH_RATE_GREAT: i32 = 141;
pub const BARGRAPH_RATE_GOOD: i32 = 142;
pub const BARGRAPH_RATE_BAD: i32 = 143;
pub const BARGRAPH_RATE_POOR: i32 = 144;
pub const BARGRAPH_RATE_MAXCOMBO: i32 = 145;
pub const BARGRAPH_RATE_SCORE: i32 = 146;
pub const BARGRAPH_RATE_EXSCORE: i32 = 147;

// ============================================================================
// String IDs
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
pub const STRING_SEARCHWORD: i32 = 30;
pub const STRING_SKIN_NAME: i32 = 50;
pub const STRING_SKIN_AUTHOR: i32 = 51;
pub const STRING_SKIN_CUSTOMIZE_CATEGORY1: i32 = 100;
pub const STRING_SKIN_CUSTOMIZE_CATEGORY10: i32 = 109;
pub const STRING_SKIN_CUSTOMIZE_ITEM1: i32 = 110;
pub const STRING_SKIN_CUSTOMIZE_ITEM10: i32 = 119;
pub const STRING_RANKING1_NAME: i32 = 120;
pub const STRING_RANKING10_NAME: i32 = 129;
pub const STRING_COURSE1_TITLE: i32 = 150;
pub const STRING_COURSE10_TITLE: i32 = 159;
pub const STRING_DIRECTORY: i32 = 1000;
pub const STRING_TABLE_NAME: i32 = 1001;
pub const STRING_TABLE_LEVEL: i32 = 1002;
pub const STRING_TABLE_FULL: i32 = 1003;
pub const STRING_VERSION: i32 = 1010;
pub const STRING_IR_NAME: i32 = 1020;
pub const STRING_IR_USER_NAME: i32 = 1021;
pub const STRING_SONG_HASH_MD5: i32 = 1030;
pub const STRING_SONG_HASH_SHA256: i32 = 1031;

// ============================================================================
// Number IDs
// ============================================================================

pub const NUMBER_HISPEED_LR2: i32 = 10;
pub const NUMBER_HISPEED: i32 = 310;
pub const NUMBER_HISPEED_AFTERDOT: i32 = 311;
pub const NUMBER_DURATION: i32 = 312;
pub const NUMBER_DURATION_GREEN: i32 = 313;
pub const NUMBER_JUDGETIMING: i32 = 12;
pub const NUMBER_LANECOVER1: i32 = 14;
pub const NUMBER_LIFT1: i32 = 314;
pub const NUMBER_HIDDEN1: i32 = 315;
pub const NUMBER_LANECOVER2: i32 = 316;

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
pub const NUMBER_POINT: i32 = 100;
pub const NUMBER_SCORE2: i32 = 101;
pub const NUMBER_SCORE_RATE: i32 = 102;
pub const NUMBER_SCORE_RATE_AFTERDOT: i32 = 103;
pub const NUMBER_COMBO: i32 = 104;
pub const NUMBER_MAXCOMBO2: i32 = 105;
pub const NUMBER_TOTALNOTES2: i32 = 106;
pub const NUMBER_GROOVEGAUGE: i32 = 107;
pub const NUMBER_GROOVEGAUGE_AFTERDOT: i32 = 407;
pub const NUMBER_DIFF_EXSCORE: i32 = 108;
pub const NUMBER_PERFECT: i32 = 110;
pub const NUMBER_EARLY_PERFECT: i32 = 410;
pub const NUMBER_LATE_PERFECT: i32 = 411;
pub const NUMBER_GREAT: i32 = 111;
pub const NUMBER_EARLY_GREAT: i32 = 412;
pub const NUMBER_LATE_GREAT: i32 = 413;
pub const NUMBER_GOOD: i32 = 112;
pub const NUMBER_EARLY_GOOD: i32 = 414;
pub const NUMBER_LATE_GOOD: i32 = 415;
pub const NUMBER_BAD: i32 = 113;
pub const NUMBER_EARLY_BAD: i32 = 416;
pub const NUMBER_LATE_BAD: i32 = 417;
pub const NUMBER_POOR: i32 = 114;
pub const NUMBER_EARLY_POOR: i32 = 418;
pub const NUMBER_LATE_POOR: i32 = 419;
pub const NUMBER_MISS: i32 = 420;
pub const NUMBER_EARLY_MISS: i32 = 421;
pub const NUMBER_LATE_MISS: i32 = 422;
pub const NUMBER_TOTALEARLY: i32 = 423;
pub const NUMBER_TOTALLATE: i32 = 424;
pub const NUMBER_COMBOBREAK: i32 = 425;
pub const NUMBER_TOTAL_RATE: i32 = 115;
pub const NUMBER_TOTAL_RATE_AFTERDOT: i32 = 116;
pub const NUMBER_MAXBPM: i32 = 90;
pub const NUMBER_MINBPM: i32 = 91;
pub const NUMBER_MAINBPM: i32 = 92;
pub const NUMBER_PLAYLEVEL: i32 = 96;
pub const NUMBER_NOWBPM: i32 = 160;
pub const NUMBER_PLAYTIME_MINUTE: i32 = 161;
pub const NUMBER_PLAYTIME_SECOND: i32 = 162;
pub const NUMBER_TIMELEFT_MINUTE: i32 = 163;
pub const NUMBER_TIMELEFT_SECOND: i32 = 164;
pub const NUMBER_LOADING_PROGRESS: i32 = 165;

pub const NUMBER_TARGET_SCORE: i32 = 121;
pub const NUMBER_TARGET_SCORE_RATE: i32 = 122;
pub const NUMBER_TARGET_SCORE_RATE_AFTERDOT: i32 = 123;
pub const NUMBER_DIFF_EXSCORE2: i32 = 128;

pub const NUMBER_HIGHSCORE: i32 = 150;
pub const NUMBER_TARGET_SCORE2: i32 = 151;
pub const NUMBER_DIFF_HIGHSCORE: i32 = 152;
pub const NUMBER_DIFF_TARGETSCORE: i32 = 153;

pub const NUMBER_JUDGERANK: i32 = 400;

// ============================================================================
// Rate (Float) IDs
// ============================================================================

pub const RATE_MUSICSELECT_POSITION: i32 = 1;
pub const RATE_LANECOVER: i32 = 4;
pub const RATE_LANECOVER2: i32 = 5;
pub const RATE_MUSIC_PROGRESS: i32 = 6;
pub const RATE_SCORE: i32 = 110;
pub const RATE_SCORE_FINAL: i32 = 111;
pub const RATE_BESTSCORE_NOW: i32 = 112;
pub const RATE_BESTSCORE: i32 = 113;
pub const RATE_TARGETSCORE_NOW: i32 = 114;
pub const RATE_TARGETSCORE: i32 = 115;
pub const RATE_PGREAT: i32 = 140;
pub const RATE_GREAT: i32 = 141;
pub const RATE_GOOD: i32 = 142;
pub const RATE_BAD: i32 = 143;
pub const RATE_POOR: i32 = 144;
pub const RATE_MAXCOMBO: i32 = 145;
pub const RATE_EXSCORE: i32 = 147;
pub const RATE_LOAD_PROGRESS: i32 = 102;

// ============================================================================
// Value/Judge IDs
// ============================================================================

pub const VALUE_JUDGE_1P_SCRATCH: i32 = 500;
pub const VALUE_JUDGE_1P_KEY1: i32 = 501;
pub const VALUE_JUDGE_1P_KEY9: i32 = 509;
pub const VALUE_JUDGE_2P_SCRATCH: i32 = 510;
pub const VALUE_JUDGE_2P_KEY1: i32 = 511;
pub const VALUE_JUDGE_2P_KEY9: i32 = 519;
pub const VALUE_JUDGE_1P: i32 = 520;
pub const VALUE_JUDGE_2P: i32 = 521;
pub const VALUE_JUDGE_3P: i32 = 522;
pub const VALUE_JUDGE_1P_DURATION: i32 = 525;
pub const VALUE_JUDGE_2P_DURATION: i32 = 526;
pub const VALUE_JUDGE_3P_DURATION: i32 = 527;

// ============================================================================
// Option IDs
// ============================================================================

pub const OPTION_RANDOM_VALUE: i32 = -1;
pub const OPTION_FOLDERBAR: i32 = 1;
pub const OPTION_SONGBAR: i32 = 2;
pub const OPTION_GRADEBAR: i32 = 3;

pub const OPTION_BGANORMAL: i32 = 30;
pub const OPTION_BGAEXTEND: i32 = 31;
pub const OPTION_AUTOPLAYOFF: i32 = 32;
pub const OPTION_AUTOPLAYON: i32 = 33;

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

pub const OPTION_7KEYSONG: i32 = 160;
pub const OPTION_5KEYSONG: i32 = 161;
pub const OPTION_14KEYSONG: i32 = 162;
pub const OPTION_10KEYSONG: i32 = 163;
pub const OPTION_9KEYSONG: i32 = 164;

pub const OPTION_NO_STAGEFILE: i32 = 190;
pub const OPTION_STAGEFILE: i32 = 191;
pub const OPTION_NO_BANNER: i32 = 192;
pub const OPTION_BANNER: i32 = 193;
pub const OPTION_NO_BACKBMP: i32 = 194;
pub const OPTION_BACKBMP: i32 = 195;

pub const OPTION_1P_PERFECT: i32 = 241;
pub const OPTION_1P_GREAT: i32 = 242;
pub const OPTION_1P_EARLY: i32 = 1242;
pub const OPTION_1P_GOOD: i32 = 243;
pub const OPTION_1P_LATE: i32 = 1243;
pub const OPTION_1P_BAD: i32 = 244;
pub const OPTION_1P_POOR: i32 = 245;
pub const OPTION_1P_MISS: i32 = 246;

pub const OPTION_1P_AAA: i32 = 200;
pub const OPTION_1P_AA: i32 = 201;
pub const OPTION_1P_A: i32 = 202;
pub const OPTION_1P_B: i32 = 203;
pub const OPTION_1P_C: i32 = 204;
pub const OPTION_1P_D: i32 = 205;
pub const OPTION_1P_E: i32 = 206;
pub const OPTION_1P_F: i32 = 207;

pub const OPTION_1P_100: i32 = 240;
pub const OPTION_1P_BORDER_OR_MORE: i32 = 1240;

pub const OPTION_CONSTANT: i32 = 400;

// ============================================================================
// Offset IDs
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
pub const OFFSET_MAX: i32 = 199;

// ============================================================================
// Button/Event IDs
// ============================================================================

pub const BUTTON_GAUGE_1P: i32 = 40;
pub const BUTTON_RANDOM_1P: i32 = 42;
pub const BUTTON_RANDOM_2P: i32 = 43;
pub const BUTTON_BGA: i32 = 72;
pub const BUTTON_TARGET: i32 = 77;

pub const EVENT_CUSTOM_BEGIN: i32 = 1000;
pub const EVENT_CUSTOM_END: i32 = 1999;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_bomb_keys_are_sequential() {
        assert_eq!(TIMER_BOMB_1P_SCRATCH, 50);
        assert_eq!(TIMER_BOMB_1P_KEY1, 51);
        assert_eq!(TIMER_BOMB_1P_KEY9, 59);
        assert_eq!(TIMER_BOMB_2P_SCRATCH, 60);
    }

    #[test]
    fn timer_keyon_keys_are_sequential() {
        assert_eq!(TIMER_KEYON_1P_SCRATCH, 100);
        assert_eq!(TIMER_KEYON_1P_KEY9, 109);
        assert_eq!(TIMER_KEYON_2P_SCRATCH, 110);
        assert_eq!(TIMER_KEYON_2P_KEY9, 119);
    }

    #[test]
    fn timer_keyoff_keys_are_sequential() {
        assert_eq!(TIMER_KEYOFF_1P_SCRATCH, 120);
        assert_eq!(TIMER_KEYOFF_1P_KEY9, 129);
        assert_eq!(TIMER_KEYOFF_2P_SCRATCH, 130);
        assert_eq!(TIMER_KEYOFF_2P_KEY9, 139);
    }

    #[test]
    fn option_judge_ids_match_beatoraja() {
        assert_eq!(OPTION_1P_PERFECT, 241);
        assert_eq!(OPTION_1P_GREAT, 242);
        assert_eq!(OPTION_1P_GOOD, 243);
        assert_eq!(OPTION_1P_BAD, 244);
        assert_eq!(OPTION_1P_POOR, 245);
        assert_eq!(OPTION_1P_MISS, 246);
    }

    #[test]
    fn value_judge_ids_match_beatoraja() {
        assert_eq!(VALUE_JUDGE_1P_SCRATCH, 500);
        assert_eq!(VALUE_JUDGE_1P_KEY1, 501);
        assert_eq!(VALUE_JUDGE_1P, 520);
    }
}
