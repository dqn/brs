use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ir_config::IRConfig;
use crate::play_mode_config::PlayModeConfig;
use crate::skin_config::SkinConfig;
use crate::skin_type::SkinType;

pub const JUDGETIMING_MAX: i32 = 500;
pub const JUDGETIMING_MIN: i32 = -500;

pub const GAUGEAUTOSHIFT_NONE: i32 = 0;
pub const GAUGEAUTOSHIFT_CONTINUE: i32 = 1;
pub const GAUGEAUTOSHIFT_SURVIVAL_TO_GROOVE: i32 = 2;
pub const GAUGEAUTOSHIFT_BESTCLEAR: i32 = 3;
pub const GAUGEAUTOSHIFT_SELECT_TO_UNDER: i32 = 4;

/// Default target list (matches Java PlayerConfig.targetlist).
fn default_targetlist() -> Vec<String> {
    [
        "RATE_A-",
        "RATE_A",
        "RATE_A+",
        "RATE_AA-",
        "RATE_AA",
        "RATE_AA+",
        "RATE_AAA-",
        "RATE_AAA",
        "RATE_AAA+",
        "RATE_MAX-",
        "MAX",
        "RANK_NEXT",
        "IR_NEXT_1",
        "IR_NEXT_2",
        "IR_NEXT_3",
        "IR_NEXT_4",
        "IR_NEXT_5",
        "IR_NEXT_10",
        "IR_RANK_1",
        "IR_RANK_5",
        "IR_RANK_10",
        "IR_RANK_20",
        "IR_RANK_30",
        "IR_RANK_40",
        "IR_RANK_50",
        "IR_RANKRATE_5",
        "IR_RANKRATE_10",
        "IR_RANKRATE_15",
        "IR_RANKRATE_20",
        "IR_RANKRATE_25",
        "IR_RANKRATE_30",
        "IR_RANKRATE_35",
        "IR_RANKRATE_40",
        "IR_RANKRATE_45",
        "IR_RANKRATE_50",
        "RIVAL_RANK_1",
        "RIVAL_RANK_2",
        "RIVAL_RANK_3",
        "RIVAL_NEXT_1",
        "RIVAL_NEXT_2",
        "RIVAL_NEXT_3",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Per-player configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PlayerConfig {
    pub id: Option<String>,
    pub name: String,

    // Gauge & pattern options
    pub gauge: i32,
    pub random: i32,
    pub random2: i32,
    pub doubleoption: i32,
    pub chart_replication_mode: String,

    // Target
    pub targetid: String,
    pub targetlist: Vec<String>,

    // Judge
    pub judgetiming: i32,
    pub notes_display_timing_auto_adjust: bool,

    // Mode filter (stored as string to avoid bms-model dependency)
    pub mode: Option<String>,

    // Misslayer
    pub misslayer_duration: i32,

    // LN mode
    pub lnmode: i32,
    pub forcedcnendings: bool,

    // Scroll modifier
    pub scroll_mode: i32,
    pub scroll_section: i32,
    pub scroll_rate: f64,

    // LongNote modifier
    pub longnote_mode: i32,
    pub longnote_rate: f64,

    // Custom judge
    pub custom_judge: bool,
    pub key_judge_window_rate_perfect_great: i32,
    pub key_judge_window_rate_great: i32,
    pub key_judge_window_rate_good: i32,
    pub scratch_judge_window_rate_perfect_great: i32,
    pub scratch_judge_window_rate_great: i32,
    pub scratch_judge_window_rate_good: i32,

    // Mine
    pub mine_mode: i32,

    // Assist
    pub bpmguide: bool,
    pub extranote_type: i32,
    pub extranote_depth: i32,
    pub extranote_scratch: bool,

    // Display
    pub showjudgearea: bool,
    pub markprocessednote: bool,

    // H-RANDOM threshold
    pub hran_threshold_bpm: i32,

    // Gauge auto shift
    pub gauge_auto_shift: i32,
    pub bottom_shiftable_gauge: i32,

    // Auto-save replay
    pub autosavereplay: Option<Vec<i32>>,

    // 7to9
    pub seven_to_nine_pattern: i32,
    pub seven_to_nine_type: i32,

    // Exit
    pub exit_press_duration: i32,

    // Misc flags
    pub is_guide_se: bool,
    pub is_window_hold: bool,
    pub is_random_select: bool,

    // Skin configs (length = SkinType::max_id() + 1 = 19)
    pub skin: Vec<SkinConfig>,
    pub skin_history: Option<Vec<SkinConfig>>,

    // Play mode configs
    pub mode5: PlayModeConfig,
    pub mode7: PlayModeConfig,
    pub mode10: PlayModeConfig,
    pub mode14: PlayModeConfig,
    pub mode9: PlayModeConfig,
    pub mode24: PlayModeConfig,
    pub mode24double: PlayModeConfig,

    // Hidden/past note display
    pub showhiddennote: bool,
    pub showpastnote: bool,

    // Chart preview
    pub chart_preview: bool,

    // Sort
    pub sort: i32,
    pub sortid: Option<String>,

    // Input
    pub musicselectinput: i32,

    // IR configs
    pub irconfig: Option<Vec<IRConfig>>,

    // Twitter (legacy)
    pub twitter_consumer_key: Option<String>,
    pub twitter_consumer_secret: Option<String>,
    pub twitter_access_token: Option<String>,
    pub twitter_access_token_secret: Option<String>,

    // Stream
    pub enable_request: bool,
    pub notify_request: bool,
    pub max_request_count: i32,

    // Event mode
    pub event_mode: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            id: None,
            name: "NO NAME".to_string(),
            gauge: 0,
            random: 0,
            random2: 0,
            doubleoption: 0,
            chart_replication_mode: "RIVALCHART".to_string(),
            targetid: "MAX".to_string(),
            targetlist: default_targetlist(),
            judgetiming: 0,
            notes_display_timing_auto_adjust: false,
            mode: None,
            misslayer_duration: 500,
            lnmode: 0,
            forcedcnendings: false,
            scroll_mode: 0,
            scroll_section: 4,
            scroll_rate: 0.5,
            longnote_mode: 0,
            longnote_rate: 1.0,
            custom_judge: false,
            key_judge_window_rate_perfect_great: 400,
            key_judge_window_rate_great: 400,
            key_judge_window_rate_good: 100,
            scratch_judge_window_rate_perfect_great: 400,
            scratch_judge_window_rate_great: 400,
            scratch_judge_window_rate_good: 100,
            mine_mode: 0,
            bpmguide: false,
            extranote_type: 0,
            extranote_depth: 0,
            extranote_scratch: false,
            showjudgearea: false,
            markprocessednote: false,
            hran_threshold_bpm: 120,
            gauge_auto_shift: GAUGEAUTOSHIFT_NONE,
            bottom_shiftable_gauge: 0,
            autosavereplay: None,
            seven_to_nine_pattern: 0,
            seven_to_nine_type: 0,
            exit_press_duration: 1000,
            is_guide_se: false,
            is_window_hold: false,
            is_random_select: false,
            skin: Vec::new(),
            skin_history: None,
            mode5: PlayModeConfig::default(),
            mode7: PlayModeConfig::default(),
            mode10: PlayModeConfig::default(),
            mode14: PlayModeConfig::default(),
            mode9: PlayModeConfig::default(),
            mode24: PlayModeConfig::default(),
            mode24double: PlayModeConfig::default(),
            showhiddennote: false,
            showpastnote: false,
            chart_preview: true,
            sort: 0,
            sortid: None,
            musicselectinput: 0,
            irconfig: None,
            twitter_consumer_key: None,
            twitter_consumer_secret: None,
            twitter_access_token: None,
            twitter_access_token_secret: None,
            enable_request: false,
            notify_request: false,
            max_request_count: 30,
            event_mode: false,
        }
    }
}

impl PlayerConfig {
    /// Read player config from a JSON file.
    pub fn read(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let mut config: PlayerConfig = serde_json::from_str(&data)?;
        config.validate();
        Ok(config)
    }

    /// Write player config to a JSON file.
    pub fn write(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Returns the play mode config for the given mode ID.
    pub fn play_config(&self, mode_id: i32) -> &PlayModeConfig {
        match mode_id {
            5 => &self.mode5,
            7 => &self.mode7,
            10 => &self.mode10,
            14 => &self.mode14,
            9 => &self.mode9,
            25 => &self.mode24,
            50 => &self.mode24double,
            _ => &self.mode7,
        }
    }

    /// Returns a mutable reference to the play mode config for the given mode ID.
    pub fn play_config_mut(&mut self, mode_id: i32) -> &mut PlayModeConfig {
        match mode_id {
            5 => &mut self.mode5,
            7 => &mut self.mode7,
            10 => &mut self.mode10,
            14 => &mut self.mode14,
            9 => &mut self.mode9,
            25 => &mut self.mode24,
            50 => &mut self.mode24double,
            _ => &mut self.mode7,
        }
    }

    pub fn validate(&mut self) {
        let skin_count = (SkinType::max_id() + 1) as usize;

        // Normalize skin array
        if self.skin.len() != skin_count {
            self.skin.resize_with(skin_count, SkinConfig::default);
        }
        for i in 0..self.skin.len() {
            if self.skin[i].path.is_none()
                || self.skin[i].path.as_ref().is_some_and(|p| p.is_empty())
            {
                self.skin[i] = SkinConfig::get_default(i as i32);
            }
            self.skin[i].validate();
        }

        if self.skin_history.is_none() {
            self.skin_history = Some(Vec::new());
        }

        // Validate mode configs with proper key counts
        self.mode5.validate(7);
        self.mode7.validate(9);
        self.mode10.validate(14);
        self.mode14.validate(18);
        self.mode9.validate(9);
        self.mode24.validate(26);
        self.mode24double.validate(52);

        // Clamp sort (Java: BarSorter.defaultSorter.length - 1 = 7)
        self.sort = self.sort.clamp(0, 7);

        // Clamp core options
        self.gauge = self.gauge.clamp(0, 5);
        self.random = self.random.clamp(0, 9);
        self.random2 = self.random2.clamp(0, 9);
        self.doubleoption = self.doubleoption.clamp(0, 3);

        if self.chart_replication_mode.is_empty() {
            self.chart_replication_mode = "NONE".to_string();
        }
        if self.targetid.is_empty() {
            self.targetid = "MAX".to_string();
        }

        self.judgetiming = self.judgetiming.clamp(JUDGETIMING_MIN, JUDGETIMING_MAX);
        self.misslayer_duration = self.misslayer_duration.clamp(0, 5000);
        self.lnmode = self.lnmode.clamp(0, 2);

        // Custom judge window rates
        self.key_judge_window_rate_perfect_great =
            self.key_judge_window_rate_perfect_great.clamp(25, 400);
        self.key_judge_window_rate_great = self.key_judge_window_rate_great.clamp(0, 400);
        self.key_judge_window_rate_good = self.key_judge_window_rate_good.clamp(0, 400);
        self.scratch_judge_window_rate_perfect_great =
            self.scratch_judge_window_rate_perfect_great.clamp(25, 400);
        self.scratch_judge_window_rate_great = self.scratch_judge_window_rate_great.clamp(0, 400);
        self.scratch_judge_window_rate_good = self.scratch_judge_window_rate_good.clamp(0, 400);

        self.hran_threshold_bpm = self.hran_threshold_bpm.clamp(1, 1000);

        // Auto-save replay: normalize to length 4
        match &mut self.autosavereplay {
            None => self.autosavereplay = Some(vec![0; 4]),
            Some(v) => {
                if v.len() != 4 {
                    v.resize(4, 0);
                }
            }
        }

        self.seven_to_nine_pattern = self.seven_to_nine_pattern.clamp(0, 6);
        self.seven_to_nine_type = self.seven_to_nine_type.clamp(0, 2);
        self.exit_press_duration = self.exit_press_duration.clamp(0, 100000);

        // Scroll/longnote/mine mode clamps
        // Java: ScrollSpeedModifier.Mode.values().length = 3 (modes 0..2)
        self.scroll_mode = self.scroll_mode.clamp(0, 2);
        self.scroll_section = self.scroll_section.clamp(1, 1024);
        self.scroll_rate = self.scroll_rate.clamp(0.0, 1.0);
        // Java: LongNoteModifier.Mode.values().length = 6 (modes 0..5)
        self.longnote_mode = self.longnote_mode.clamp(0, 5);
        self.longnote_rate = self.longnote_rate.clamp(0.0, 1.0);
        // Java: MineNoteModifier.Mode.values().length = 5 (modes 0..4)
        self.mine_mode = self.mine_mode.clamp(0, 4);
        self.extranote_depth = self.extranote_depth.clamp(0, 100);

        // IR config: remove duplicates and invalids
        if let Some(ref mut configs) = self.irconfig {
            // Remove duplicates by irname
            let mut seen = Vec::new();
            for config in configs.iter_mut() {
                if config.irname.is_empty() || seen.contains(&config.irname) {
                    config.irname.clear();
                } else {
                    seen.push(config.irname.clone());
                }
            }
            configs.retain(|c| c.validate());
        }

        // Stream
        self.max_request_count = self.max_request_count.clamp(0, 100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let pc = PlayerConfig::default();
        assert_eq!(pc.name, "NO NAME");
        assert_eq!(pc.gauge, 0);
        assert_eq!(pc.random, 0);
        assert_eq!(pc.targetid, "MAX");
        assert!(pc.chart_preview);
        assert_eq!(pc.max_request_count, 30);
        assert_eq!(pc.exit_press_duration, 1000);
        assert!(!pc.targetlist.is_empty());
    }

    #[test]
    fn test_validate_normalizes_skin_array() {
        let mut pc = PlayerConfig::default();
        pc.skin = Vec::new();
        pc.validate();
        assert_eq!(pc.skin.len(), 19);
    }

    #[test]
    fn test_validate_clamps() {
        let mut pc = PlayerConfig {
            gauge: 100,
            random: -1,
            random2: 20,
            doubleoption: 10,
            judgetiming: 1000,
            misslayer_duration: -100,
            lnmode: 5,
            sort: 100,
            hran_threshold_bpm: 0,
            max_request_count: 500,
            ..Default::default()
        };
        pc.validate();
        assert_eq!(pc.gauge, 5);
        assert_eq!(pc.random, 0);
        assert_eq!(pc.random2, 9);
        assert_eq!(pc.doubleoption, 3);
        assert_eq!(pc.judgetiming, JUDGETIMING_MAX);
        assert_eq!(pc.misslayer_duration, 0);
        assert_eq!(pc.lnmode, 2);
        assert_eq!(pc.sort, 7);
        assert_eq!(pc.hran_threshold_bpm, 1);
        assert_eq!(pc.max_request_count, 100);
    }

    #[test]
    fn test_validate_autosavereplay() {
        let mut pc = PlayerConfig::default();
        pc.autosavereplay = None;
        pc.validate();
        assert_eq!(pc.autosavereplay.as_ref().unwrap().len(), 4);

        pc.autosavereplay = Some(vec![1, 2]);
        pc.validate();
        assert_eq!(pc.autosavereplay.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn test_validate_ir_dedup() {
        let mut pc = PlayerConfig {
            irconfig: Some(vec![
                IRConfig {
                    irname: "LR2IR".to_string(),
                    ..Default::default()
                },
                IRConfig {
                    irname: "LR2IR".to_string(), // duplicate
                    ..Default::default()
                },
                IRConfig {
                    irname: "BeatorajaIR".to_string(),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        pc.validate();
        let configs = pc.irconfig.as_ref().unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].irname, "LR2IR");
        assert_eq!(configs[1].irname, "BeatorajaIR");
    }

    #[test]
    fn test_play_config_dispatch() {
        let pc = PlayerConfig::default();
        // All should return references without panicking
        let _ = pc.play_config(5);
        let _ = pc.play_config(7);
        let _ = pc.play_config(9);
        let _ = pc.play_config(10);
        let _ = pc.play_config(14);
        let _ = pc.play_config(25);
        let _ = pc.play_config(50);
        let _ = pc.play_config(99); // defaults to mode7
    }

    #[test]
    fn test_serde_round_trip() {
        let pc = PlayerConfig::default();
        let json = serde_json::to_string(&pc).unwrap();
        let back: PlayerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "NO NAME");
        assert_eq!(back.gauge, 0);
        assert_eq!(back.targetid, "MAX");
    }

    #[test]
    fn test_deserialize_from_empty() {
        let pc: PlayerConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(pc.name, "NO NAME");
        assert_eq!(pc.gauge, 0);
    }

    #[test]
    fn test_read_write_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config_player.json");

        let mut pc = PlayerConfig::default();
        pc.name = "TestPlayer".to_string();
        pc.gauge = 3;
        pc.judgetiming = 42;
        pc.write(&path).unwrap();

        let loaded = PlayerConfig::read(&path).unwrap();
        assert_eq!(loaded.name, "TestPlayer");
        assert_eq!(loaded.gauge, 3);
        assert_eq!(loaded.judgetiming, 42);
    }

    #[test]
    fn test_read_nonexistent_returns_error() {
        let result = PlayerConfig::read(std::path::Path::new("/nonexistent/config_player.json"));
        assert!(result.is_err());
    }
}
