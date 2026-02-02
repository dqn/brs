use serde::{Deserialize, Serialize};

/// Audio system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume (0.0 - 1.0).
    pub master_volume: f64,
    /// Keysound volume multiplier (0.0 - 1.0).
    pub keysound_volume: f64,
    /// BGM volume multiplier (0.0 - 1.0).
    pub bgm_volume: f64,
    /// Maximum number of concurrent sounds.
    pub sound_capacity: usize,
    /// Maximum memory usage for sound cache in MB.
    pub max_memory_mb: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            keysound_volume: 1.0,
            bgm_volume: 1.0,
            sound_capacity: 512,
            max_memory_mb: 512,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AudioConfig::default();
        assert!((config.master_volume - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.sound_capacity, 512);
        assert_eq!(config.max_memory_mb, 512);
    }

    #[test]
    fn test_serialization() {
        let config = AudioConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AudioConfig = serde_json::from_str(&json).unwrap();
        assert!((deserialized.master_volume - config.master_volume).abs() < f64::EPSILON);
    }
}
