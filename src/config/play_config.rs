use serde::{Deserialize, Serialize};

/// Play-specific configuration (per-chart adjustments).
/// Controls visual and timing settings during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayConfig {
    /// Hi-speed multiplier (scroll speed).
    #[serde(default = "default_hispeed")]
    pub hispeed: f64,
    /// Judge timing adjustment in milliseconds (negative = early, positive = late).
    #[serde(default)]
    pub judge_timing: i32,
    /// Lane cover percentage (0.0 - 1.0, top-of-screen cover).
    #[serde(default)]
    pub lane_cover: f64,
    /// Lift percentage (0.0 - 1.0, bottom-of-screen lift).
    #[serde(default)]
    pub lift: f64,
    /// Enable hidden+ (top cover toggleable).
    #[serde(default)]
    pub hidden_plus: bool,
    /// Enable sudden+ (bottom cover toggleable).
    #[serde(default)]
    pub sudden_plus: bool,
    /// Duration-based green number (ms).
    /// When set, hi-speed is auto-calculated from this.
    #[serde(default)]
    pub duration: Option<u32>,
}

fn default_hispeed() -> f64 {
    1.0
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self {
            hispeed: default_hispeed(),
            judge_timing: 0,
            lane_cover: 0.0,
            lift: 0.0,
            hidden_plus: false,
            sudden_plus: false,
            duration: None,
        }
    }
}

impl PlayConfig {
    /// Calculate effective visible area ratio (0.0 - 1.0).
    pub fn visible_ratio(&self) -> f64 {
        (1.0 - self.lane_cover - self.lift).clamp(0.0, 1.0)
    }

    /// Calculate effective hi-speed considering duration.
    /// `base_bpm`: the base BPM of the chart.
    /// `lane_height_px`: the pixel height of the lane.
    pub fn effective_hispeed(&self, base_bpm: f64, lane_height_px: f64) -> f64 {
        match self.duration {
            Some(dur) if dur > 0 => {
                // duration = lane_height_px / (bpm / 60 * hispeed * beat_height_ratio)
                // Simplified: hispeed ≈ lane_height_px * 60000.0 / (bpm * dur * visible)
                let visible = self.visible_ratio().max(0.1);
                let effective_lane = lane_height_px * visible;
                effective_lane * 60_000.0 / (base_bpm * dur as f64 * lane_height_px)
            }
            _ => self.hispeed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let config = PlayConfig::default();
        assert!((config.hispeed - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.judge_timing, 0);
        assert!((config.lane_cover - 0.0).abs() < f64::EPSILON);
        assert!(!config.hidden_plus);
        assert!(!config.sudden_plus);
        assert!(config.duration.is_none());
    }

    #[test]
    fn visible_ratio_no_cover() {
        let config = PlayConfig::default();
        assert!((config.visible_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn visible_ratio_with_cover() {
        let config = PlayConfig {
            lane_cover: 0.3,
            lift: 0.1,
            ..Default::default()
        };
        assert!((config.visible_ratio() - 0.6).abs() < 0.001);
    }

    #[test]
    fn visible_ratio_clamp() {
        let config = PlayConfig {
            lane_cover: 0.8,
            lift: 0.5,
            ..Default::default()
        };
        assert!((config.visible_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_hispeed_no_duration() {
        let config = PlayConfig {
            hispeed: 2.5,
            ..Default::default()
        };
        assert!((config.effective_hispeed(150.0, 800.0) - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_hispeed_with_duration() {
        let config = PlayConfig {
            duration: Some(300),
            ..Default::default()
        };
        let hs = config.effective_hispeed(150.0, 800.0);
        // Should compute a reasonable value > 0
        assert!(hs > 0.0);
    }

    #[test]
    fn serialization_round_trip() {
        let config = PlayConfig {
            hispeed: 3.5,
            judge_timing: -5,
            lane_cover: 0.25,
            lift: 0.05,
            hidden_plus: true,
            sudden_plus: false,
            duration: Some(350),
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: PlayConfig = serde_json::from_str(&json).unwrap();
        assert!((restored.hispeed - 3.5).abs() < f64::EPSILON);
        assert_eq!(restored.judge_timing, -5);
        assert!((restored.lane_cover - 0.25).abs() < f64::EPSILON);
        assert!(restored.hidden_plus);
        assert_eq!(restored.duration, Some(350));
    }
}
