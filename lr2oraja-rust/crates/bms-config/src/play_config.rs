use serde::{Deserialize, Serialize};

pub const FIX_HISPEED_OFF: i32 = 0;
pub const FIX_HISPEED_STARTBPM: i32 = 1;
pub const FIX_HISPEED_MAXBPM: i32 = 2;
pub const FIX_HISPEED_MAINBPM: i32 = 3;
pub const FIX_HISPEED_MINBPM: i32 = 4;

pub const HISPEED_MAX: f32 = 20.0;
pub const HISPEED_MIN: f32 = 0.01;
pub const DURATION_MAX: i32 = 10000;
pub const DURATION_MIN: i32 = 1;
pub const CONSTANT_FADEIN_MAX: i32 = 1000;
pub const CONSTANT_FADEIN_MIN: i32 = -1000;
pub const HISPEEDMARGIN_MAX: f32 = 10.0;
pub const HISPEEDMARGIN_MIN: f32 = 0.0;

const VALID_JUDGE_TYPES: [&str; 4] = ["Combo", "Duration", "Lowest", "Score"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PlayConfig {
    pub hispeed: f32,
    pub duration: i32,
    pub enable_constant: bool,
    pub constant_fadein_time: i32,
    pub fixhispeed: i32,
    pub hispeedmargin: f32,
    pub lanecover: f32,
    pub enablelanecover: bool,
    pub lift: f32,
    pub enablelift: bool,
    pub hidden: f32,
    pub enablehidden: bool,
    pub lanecovermarginlow: f32,
    pub lanecovermarginhigh: f32,
    pub lanecoverswitchduration: i32,
    pub hispeedautoadjust: bool,
    pub judgetype: String,
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            hispeed: 1.0,
            duration: 500,
            enable_constant: false,
            constant_fadein_time: 100,
            fixhispeed: FIX_HISPEED_MAINBPM,
            hispeedmargin: 0.25,
            lanecover: 0.2,
            enablelanecover: true,
            lift: 0.1,
            enablelift: false,
            hidden: 0.1,
            enablehidden: false,
            lanecovermarginlow: 0.001,
            lanecovermarginhigh: 0.01,
            lanecoverswitchduration: 500,
            hispeedautoadjust: false,
            judgetype: "Combo".to_string(),
        }
    }
}

impl PlayConfig {
    pub fn validate(&mut self) {
        self.hispeed = self.hispeed.clamp(HISPEED_MIN, HISPEED_MAX);
        self.duration = self.duration.clamp(DURATION_MIN, DURATION_MAX);
        self.constant_fadein_time = self
            .constant_fadein_time
            .clamp(CONSTANT_FADEIN_MIN, CONSTANT_FADEIN_MAX);
        self.hispeedmargin = self
            .hispeedmargin
            .clamp(HISPEEDMARGIN_MIN, HISPEEDMARGIN_MAX);
        self.fixhispeed = self.fixhispeed.clamp(0, FIX_HISPEED_MINBPM);
        self.lanecover = self.lanecover.clamp(0.0, 1.0);
        self.lift = self.lift.clamp(0.0, 1.0);
        self.hidden = self.hidden.clamp(0.0, 1.0);
        self.lanecovermarginlow = self.lanecovermarginlow.clamp(0.0, 1.0);
        self.lanecovermarginhigh = self.lanecovermarginhigh.clamp(0.0, 1.0);
        self.lanecoverswitchduration = self.lanecoverswitchduration.clamp(0, 1000000);
        if !VALID_JUDGE_TYPES.contains(&self.judgetype.as_str()) {
            self.judgetype = "Combo".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = PlayConfig::default();
        assert_eq!(config.hispeed, 1.0);
        assert_eq!(config.duration, 500);
        assert!(!config.enable_constant);
        assert_eq!(config.constant_fadein_time, 100);
        assert_eq!(config.fixhispeed, FIX_HISPEED_MAINBPM);
        assert_eq!(config.hispeedmargin, 0.25);
        assert_eq!(config.lanecover, 0.2);
        assert!(config.enablelanecover);
        assert_eq!(config.lift, 0.1);
        assert!(!config.enablelift);
        assert_eq!(config.hidden, 0.1);
        assert!(!config.enablehidden);
        assert_eq!(config.lanecovermarginlow, 0.001);
        assert_eq!(config.lanecovermarginhigh, 0.01);
        assert_eq!(config.lanecoverswitchduration, 500);
        assert!(!config.hispeedautoadjust);
        assert_eq!(config.judgetype, "Combo");
    }

    #[test]
    fn test_validate_clamps_values() {
        let mut config = PlayConfig {
            hispeed: 100.0,
            duration: -5,
            constant_fadein_time: 5000,
            hispeedmargin: -1.0,
            fixhispeed: 10,
            lanecover: 2.0,
            lift: -0.5,
            hidden: 1.5,
            lanecovermarginlow: -1.0,
            lanecovermarginhigh: 5.0,
            lanecoverswitchduration: 2000000,
            judgetype: "Invalid".to_string(),
            ..Default::default()
        };
        config.validate();

        assert_eq!(config.hispeed, HISPEED_MAX);
        assert_eq!(config.duration, DURATION_MIN);
        assert_eq!(config.constant_fadein_time, CONSTANT_FADEIN_MAX);
        assert_eq!(config.hispeedmargin, HISPEEDMARGIN_MIN);
        assert_eq!(config.fixhispeed, FIX_HISPEED_MINBPM);
        assert_eq!(config.lanecover, 1.0);
        assert_eq!(config.lift, 0.0);
        assert_eq!(config.hidden, 1.0);
        assert_eq!(config.lanecovermarginlow, 0.0);
        assert_eq!(config.lanecovermarginhigh, 1.0);
        assert_eq!(config.lanecoverswitchduration, 1000000);
        assert_eq!(config.judgetype, "Combo");
    }

    #[test]
    fn test_validate_accepts_valid_judge_types() {
        for judge_type in &VALID_JUDGE_TYPES {
            let mut config = PlayConfig {
                judgetype: judge_type.to_string(),
                ..Default::default()
            };
            config.validate();
            assert_eq!(config.judgetype, *judge_type);
        }
    }

    #[test]
    fn test_serde_round_trip() {
        let config = PlayConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PlayConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.hispeed, deserialized.hispeed);
        assert_eq!(config.duration, deserialized.duration);
        assert_eq!(config.enable_constant, deserialized.enable_constant);
        assert_eq!(config.fixhispeed, deserialized.fixhispeed);
        assert_eq!(config.judgetype, deserialized.judgetype);
        assert_eq!(config.enablelanecover, deserialized.enablelanecover);
    }

    #[test]
    fn test_serde_uses_camel_case() {
        let config = PlayConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("\"enableConstant\""));
        assert!(json.contains("\"constantFadeinTime\""));
        assert!(!json.contains("\"enable_constant\""));
        assert!(!json.contains("\"constant_fadein_time\""));
    }

    #[test]
    fn test_serde_default_fills_missing_fields() {
        let json = r#"{"hispeed": 2.5}"#;
        let config: PlayConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.hispeed, 2.5);
        assert_eq!(config.duration, 500);
        assert_eq!(config.judgetype, "Combo");
    }
}
