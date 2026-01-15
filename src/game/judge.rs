/// Timing direction for FAST/SLOW display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingDirection {
    Fast,
    Exact,
    Slow,
}

impl TimingDirection {
    const EXACT_THRESHOLD_MS: f64 = 1.0;

    pub fn from_timing_diff(timing_diff_ms: f64) -> Self {
        if timing_diff_ms < -Self::EXACT_THRESHOLD_MS {
            TimingDirection::Fast
        } else if timing_diff_ms > Self::EXACT_THRESHOLD_MS {
            TimingDirection::Slow
        } else {
            TimingDirection::Exact
        }
    }
}

/// Cumulative FAST/SLOW statistics during gameplay
#[derive(Debug, Clone, Default)]
pub struct TimingStats {
    pub fast_count: u32,
    pub slow_count: u32,
}

impl TimingStats {
    pub fn record(&mut self, judge: JudgeResult, timing_diff_ms: f64) {
        // Don't count PGREAT (it's within exact threshold)
        if judge == JudgeResult::PGreat {
            return;
        }

        let direction = TimingDirection::from_timing_diff(timing_diff_ms);
        match direction {
            TimingDirection::Fast => self.fast_count += 1,
            TimingDirection::Slow => self.slow_count += 1,
            TimingDirection::Exact => {}
        }
    }
}

/// Judge system type (affects timing windows and empty POOR behavior)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum JudgeSystemType {
    #[default]
    Beatoraja,
    #[allow(dead_code)] // Planned feature for LR2 compatibility mode
    Lr2,
}

/// Judge difficulty rank (affects timing window size)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JudgeRank {
    VeryEasy,
    #[default]
    Easy,
    Normal,
    Hard,
    VeryHard,
}

impl JudgeRank {
    /// Convert from BMS #RANK value
    pub fn from_bms_rank(rank: u32) -> Self {
        match rank {
            0 => JudgeRank::VeryHard,
            1 => JudgeRank::Hard,
            2 => JudgeRank::Normal,
            3 => JudgeRank::Easy,
            _ => JudgeRank::Easy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeResult {
    PGreat,
    Great,
    Good,
    Bad,
    Poor,
}

impl JudgeResult {
    // Public API for calculating EX score contribution
    #[allow(dead_code)]
    pub fn ex_score(&self) -> u32 {
        match self {
            Self::PGreat => 2,
            Self::Great => 1,
            Self::Good | Self::Bad | Self::Poor => 0,
        }
    }

    // Public API for checking if this result breaks combo
    #[allow(dead_code)]
    pub fn is_combo_break(&self) -> bool {
        matches!(self, Self::Bad | Self::Poor)
    }
}

#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub pgreat_window: f64,
    pub great_window: f64,
    pub good_window: f64,
    pub bad_window: f64,
}

/// Release timing windows for CN (Charge Note)
/// These are wider than normal note windows
#[derive(Debug, Clone)]
pub struct ReleaseConfig {
    pub pgreat_window: f64,
    pub great_window: f64,
    pub good_window: f64,
    pub bad_window: f64,
}

impl ReleaseConfig {
    pub fn normal() -> Self {
        Self {
            pgreat_window: 120.0,
            great_window: 160.0,
            good_window: 200.0,
            bad_window: 280.0,
        }
    }
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self::normal()
    }
}

impl JudgeConfig {
    /// Create beatoraja-style timing windows for given rank
    pub fn beatoraja(rank: JudgeRank) -> Self {
        let scale = match rank {
            JudgeRank::VeryEasy => 1.25,
            JudgeRank::Easy => 1.0,
            JudgeRank::Normal => 0.75,
            JudgeRank::Hard => 0.50,
            JudgeRank::VeryHard => 0.25,
        };

        Self {
            pgreat_window: 20.0 * scale,
            great_window: 60.0 * scale,
            good_window: 150.0 * scale,
            bad_window: 280.0 * scale,
        }
    }

    /// Create LR2-style timing windows for given rank
    pub fn lr2(rank: JudgeRank) -> Self {
        // LR2 uses fixed windows per rank (not scaled)
        let pgreat_ms = match rank {
            JudgeRank::VeryEasy | JudgeRank::Easy => 21.0,
            JudgeRank::Normal => 18.0,
            JudgeRank::Hard => 15.0,
            JudgeRank::VeryHard => 8.0,
        };

        // Other windows scale proportionally
        Self {
            pgreat_window: pgreat_ms,
            great_window: pgreat_ms * 3.0, // ~60ms for EASY
            good_window: pgreat_ms * 6.0,  // ~120ms for EASY
            bad_window: pgreat_ms * 10.0,  // ~200ms for EASY
        }
    }

    /// Create timing config for the given system and rank
    pub fn for_system(system: JudgeSystemType, rank: JudgeRank) -> Self {
        match system {
            JudgeSystemType::Beatoraja => Self::beatoraja(rank),
            JudgeSystemType::Lr2 => Self::lr2(rank),
        }
    }

    pub fn normal() -> Self {
        Self::beatoraja(JudgeRank::Easy)
    }

    #[allow(dead_code)]
    pub fn easy() -> Self {
        Self::beatoraja(JudgeRank::VeryEasy)
    }

    #[allow(dead_code)]
    pub fn hard() -> Self {
        Self::beatoraja(JudgeRank::Hard)
    }

    #[allow(dead_code)]
    pub fn builder() -> JudgeConfigBuilder {
        JudgeConfigBuilder::default()
    }
}

impl Default for JudgeConfig {
    fn default() -> Self {
        Self::normal()
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct JudgeConfigBuilder {
    pgreat_window: Option<f64>,
    great_window: Option<f64>,
    good_window: Option<f64>,
    bad_window: Option<f64>,
}

#[allow(dead_code)]
impl JudgeConfigBuilder {
    pub fn pgreat_window(mut self, ms: f64) -> Self {
        self.pgreat_window = Some(ms);
        self
    }

    pub fn great_window(mut self, ms: f64) -> Self {
        self.great_window = Some(ms);
        self
    }

    pub fn good_window(mut self, ms: f64) -> Self {
        self.good_window = Some(ms);
        self
    }

    pub fn bad_window(mut self, ms: f64) -> Self {
        self.bad_window = Some(ms);
        self
    }

    pub fn build(self) -> JudgeConfig {
        let default = JudgeConfig::normal();
        JudgeConfig {
            pgreat_window: self.pgreat_window.unwrap_or(default.pgreat_window),
            great_window: self.great_window.unwrap_or(default.great_window),
            good_window: self.good_window.unwrap_or(default.good_window),
            bad_window: self.bad_window.unwrap_or(default.bad_window),
        }
    }
}

pub struct JudgeSystem {
    system_type: JudgeSystemType,
    #[allow(dead_code)]
    rank: JudgeRank,
    config: JudgeConfig,
    release_config: ReleaseConfig,
}

impl JudgeSystem {
    #[allow(dead_code)] // Alternative constructor for custom judge configs
    pub fn new(config: JudgeConfig) -> Self {
        Self {
            system_type: JudgeSystemType::Beatoraja,
            rank: JudgeRank::Easy,
            config,
            release_config: ReleaseConfig::default(),
        }
    }

    /// Create a JudgeSystem for the specified system type and rank
    pub fn for_system(system_type: JudgeSystemType, rank: JudgeRank) -> Self {
        Self {
            system_type,
            rank,
            config: JudgeConfig::for_system(system_type, rank),
            release_config: ReleaseConfig::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_release_config(mut self, release_config: ReleaseConfig) -> Self {
        self.release_config = release_config;
        self
    }

    /// Get the current system type
    #[allow(dead_code)]
    pub fn system_type(&self) -> JudgeSystemType {
        self.system_type
    }

    pub fn judge(&self, time_diff_ms: f64) -> Option<JudgeResult> {
        let abs_diff = time_diff_ms.abs();

        if abs_diff <= self.config.pgreat_window {
            Some(JudgeResult::PGreat)
        } else if abs_diff <= self.config.great_window {
            Some(JudgeResult::Great)
        } else if abs_diff <= self.config.good_window {
            Some(JudgeResult::Good)
        } else if abs_diff <= self.config.bad_window {
            Some(JudgeResult::Bad)
        } else {
            None
        }
    }

    /// Judge release timing for CN (Charge Note)
    pub fn judge_release(&self, time_diff_ms: f64) -> Option<JudgeResult> {
        let abs_diff = time_diff_ms.abs();

        if abs_diff <= self.release_config.pgreat_window {
            Some(JudgeResult::PGreat)
        } else if abs_diff <= self.release_config.great_window {
            Some(JudgeResult::Great)
        } else if abs_diff <= self.release_config.good_window {
            Some(JudgeResult::Good)
        } else if abs_diff <= self.release_config.bad_window {
            Some(JudgeResult::Bad)
        } else {
            None
        }
    }

    /// Check if release is too early (before the release window)
    pub fn is_early_release(&self, time_diff_ms: f64) -> bool {
        time_diff_ms > self.release_config.bad_window
    }

    /// Check if a button press should trigger empty POOR
    /// time_diff_ms: current_time - nearest_note_time (positive = note is in the past)
    #[allow(dead_code)]
    pub fn is_empty_poor(&self, time_diff_ms: f64) -> bool {
        match self.system_type {
            JudgeSystemType::Beatoraja => {
                // beatoraja: both early and late presses can trigger empty POOR
                time_diff_ms.abs() > self.config.bad_window
            }
            JudgeSystemType::Lr2 => {
                // LR2: only early presses trigger empty POOR (not after)
                time_diff_ms < -self.config.bad_window
            }
        }
    }

    // Public API for checking if time difference is within any judge window
    #[allow(dead_code)]
    pub fn is_in_window(&self, time_diff_ms: f64) -> bool {
        time_diff_ms.abs() <= self.config.bad_window
    }

    pub fn is_missed(&self, time_diff_ms: f64) -> bool {
        time_diff_ms < -self.config.bad_window
    }

    #[allow(dead_code)]
    pub fn release_bad_window(&self) -> f64 {
        self.release_config.bad_window
    }
}

impl Default for JudgeSystem {
    fn default() -> Self {
        Self::for_system(JudgeSystemType::Beatoraja, JudgeRank::Easy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beatoraja_timing_windows() {
        let config = JudgeConfig::beatoraja(JudgeRank::Easy);
        assert!((config.pgreat_window - 20.0).abs() < 0.01);
        assert!((config.great_window - 60.0).abs() < 0.01);
        assert!((config.good_window - 150.0).abs() < 0.01);
        assert!((config.bad_window - 280.0).abs() < 0.01);
    }

    #[test]
    fn test_beatoraja_rank_scaling() {
        let easy = JudgeConfig::beatoraja(JudgeRank::Easy);
        let hard = JudgeConfig::beatoraja(JudgeRank::Hard);

        // Hard should be half of Easy
        assert!((hard.pgreat_window - easy.pgreat_window * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lr2_timing_windows() {
        let config = JudgeConfig::lr2(JudgeRank::Easy);
        assert!((config.pgreat_window - 21.0).abs() < 0.01);
    }

    #[test]
    fn test_judge_rank_from_bms() {
        assert_eq!(JudgeRank::from_bms_rank(0), JudgeRank::VeryHard);
        assert_eq!(JudgeRank::from_bms_rank(1), JudgeRank::Hard);
        assert_eq!(JudgeRank::from_bms_rank(2), JudgeRank::Normal);
        assert_eq!(JudgeRank::from_bms_rank(3), JudgeRank::Easy);
    }

    #[test]
    fn test_empty_poor_beatoraja() {
        let judge = JudgeSystem::for_system(JudgeSystemType::Beatoraja, JudgeRank::Easy);

        // Outside window (both directions) should be empty POOR
        assert!(judge.is_empty_poor(300.0)); // Late
        assert!(judge.is_empty_poor(-300.0)); // Early
        assert!(!judge.is_empty_poor(100.0)); // Within window
    }

    #[test]
    fn test_empty_poor_lr2() {
        let judge = JudgeSystem::for_system(JudgeSystemType::Lr2, JudgeRank::Easy);

        // LR2: Only early presses trigger empty POOR
        assert!(judge.is_empty_poor(-300.0)); // Early - should be empty POOR
        assert!(!judge.is_empty_poor(300.0)); // Late - should NOT be empty POOR
    }
}
