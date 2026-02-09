use serde::{Deserialize, Serialize};

/// Audio driver type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DriverType {
    #[default]
    OpenAL,
    PortAudio,
}

/// Audio frequency processing type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum FrequencyType {
    /// No audio processing for speed changes.
    Unprocessed,
    /// Adjust frequency (pitch changes with speed).
    #[default]
    Frequency,
}

/// Audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct AudioConfig {
    pub driver: DriverType,
    pub driver_name: Option<String>,
    pub device_buffer_size: i32,
    pub device_simultaneous_sources: i32,
    pub sample_rate: i32,
    pub freq_option: FrequencyType,
    pub fast_forward: FrequencyType,
    pub systemvolume: f32,
    pub keyvolume: f32,
    pub bgvolume: f32,
    pub normalize_volume: bool,
    pub is_loop_result_sound: bool,
    pub is_loop_course_result_sound: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            driver: DriverType::OpenAL,
            driver_name: None,
            device_buffer_size: 384,
            device_simultaneous_sources: 128,
            sample_rate: 0,
            freq_option: FrequencyType::Frequency,
            fast_forward: FrequencyType::Frequency,
            systemvolume: 0.5,
            keyvolume: 0.5,
            bgvolume: 0.5,
            normalize_volume: false,
            is_loop_result_sound: false,
            is_loop_course_result_sound: false,
        }
    }
}

impl AudioConfig {
    pub fn validate(&mut self) {
        self.device_buffer_size = self.device_buffer_size.clamp(4, 4096);
        self.device_simultaneous_sources = self.device_simultaneous_sources.clamp(16, 1024);
        self.systemvolume = self.systemvolume.clamp(0.0, 1.0);
        self.keyvolume = self.keyvolume.clamp(0.0, 1.0);
        self.bgvolume = self.bgvolume.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let ac = AudioConfig::default();
        assert_eq!(ac.driver, DriverType::OpenAL);
        assert_eq!(ac.device_buffer_size, 384);
        assert_eq!(ac.device_simultaneous_sources, 128);
        assert_eq!(ac.sample_rate, 0);
        assert_eq!(ac.freq_option, FrequencyType::Frequency);
        assert_eq!(ac.fast_forward, FrequencyType::Frequency);
        assert!((ac.systemvolume - 0.5).abs() < f32::EPSILON);
        assert!((ac.keyvolume - 0.5).abs() < f32::EPSILON);
        assert!((ac.bgvolume - 0.5).abs() < f32::EPSILON);
        assert!(!ac.normalize_volume);
        assert!(!ac.is_loop_result_sound);
        assert!(!ac.is_loop_course_result_sound);
    }

    #[test]
    fn test_validate_clamps() {
        let mut ac = AudioConfig {
            device_buffer_size: 1,
            device_simultaneous_sources: 5000,
            systemvolume: -0.5,
            keyvolume: 2.0,
            bgvolume: 0.5,
            ..Default::default()
        };
        ac.validate();
        assert_eq!(ac.device_buffer_size, 4);
        assert_eq!(ac.device_simultaneous_sources, 1024);
        assert!((ac.systemvolume - 0.0).abs() < f32::EPSILON);
        assert!((ac.keyvolume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_serde_round_trip() {
        let ac = AudioConfig::default();
        let json = serde_json::to_string(&ac).unwrap();
        let back: AudioConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.driver, ac.driver);
        assert_eq!(back.device_buffer_size, ac.device_buffer_size);
    }

    #[test]
    fn test_driver_type_serde() {
        let json = serde_json::to_string(&DriverType::PortAudio).unwrap();
        assert_eq!(json, "\"PortAudio\"");
        let back: DriverType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DriverType::PortAudio);
    }

    #[test]
    fn test_frequency_type_serde() {
        let json = serde_json::to_string(&FrequencyType::Unprocessed).unwrap();
        assert_eq!(json, "\"UNPROCESSED\"");
        let json2 = serde_json::to_string(&FrequencyType::Frequency).unwrap();
        assert_eq!(json2, "\"FREQUENCY\"");
    }

    #[test]
    fn test_deserialize_from_empty() {
        let json = "{}";
        let ac: AudioConfig = serde_json::from_str(json).unwrap();
        assert_eq!(ac.driver, DriverType::OpenAL);
        assert_eq!(ac.device_buffer_size, 384);
    }
}
