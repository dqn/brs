use serde::{Deserialize, Serialize};

/// Random option for note placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RandomOption {
    #[default]
    Off,
    Mirror,
    Random,
    RRandom,
    SRandom,
    Spiral,
    HRandom,
    AllScr,
    RandomPlus,
    SRandomPlus,
}

/// Long note mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LnMode {
    #[default]
    LongNote,
    ChargeNote,
    HellChargeNote,
}

/// Gauge option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GaugeOption {
    AssistEasy,
    Easy,
    #[default]
    Normal,
    Hard,
    ExHard,
    Hazard,
}

/// Per-player configuration.
/// Stores personal preferences and play options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// Player display name.
    #[serde(default = "default_name")]
    pub name: String,
    /// Random option for 1P side.
    #[serde(default)]
    pub random: RandomOption,
    /// Random option for 2P side (DP mode).
    #[serde(default)]
    pub random_2p: RandomOption,
    /// Gauge option.
    #[serde(default)]
    pub gauge: GaugeOption,
    /// Long note mode.
    #[serde(default)]
    pub ln_mode: LnMode,
    /// Enable assist options (auto scratch, etc.).
    #[serde(default)]
    pub auto_scratch: bool,
}

fn default_name() -> String {
    "Player".to_string()
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            random: RandomOption::Off,
            random_2p: RandomOption::Off,
            gauge: GaugeOption::Normal,
            ln_mode: LnMode::LongNote,
            auto_scratch: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let config = PlayerConfig::default();
        assert_eq!(config.name, "Player");
        assert_eq!(config.random, RandomOption::Off);
        assert_eq!(config.gauge, GaugeOption::Normal);
        assert_eq!(config.ln_mode, LnMode::LongNote);
        assert!(!config.auto_scratch);
    }

    #[test]
    fn serialization_round_trip() {
        let config = PlayerConfig {
            name: "TestPlayer".to_string(),
            random: RandomOption::Mirror,
            random_2p: RandomOption::Random,
            gauge: GaugeOption::Hard,
            ln_mode: LnMode::ChargeNote,
            auto_scratch: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: PlayerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "TestPlayer");
        assert_eq!(restored.random, RandomOption::Mirror);
        assert_eq!(restored.random_2p, RandomOption::Random);
        assert_eq!(restored.gauge, GaugeOption::Hard);
        assert_eq!(restored.ln_mode, LnMode::ChargeNote);
        assert!(restored.auto_scratch);
    }

    #[test]
    fn deserialization_with_defaults() {
        let json = r#"{"name": "Hello"}"#;
        let config: PlayerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "Hello");
        assert_eq!(config.random, RandomOption::Off);
        assert_eq!(config.gauge, GaugeOption::Normal);
    }
}
