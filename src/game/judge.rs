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
        if timing_diff_ms > Self::EXACT_THRESHOLD_MS {
            TimingDirection::Fast
        } else if timing_diff_ms < -Self::EXACT_THRESHOLD_MS {
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
    /// Get EX score contribution for a single judgment result.
    /// This is the canonical implementation of EX score values.
    /// See also `ScoreManager::ex_score()` for cumulative score.
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
    /// BAD window for early hits (positive value, player pressed too early)
    pub bad_early_window: f64,
    /// BAD window for late hits (positive value, player pressed too late)
    pub bad_late_window: f64,
}

/// Release timing windows for CN (Charge Note)
/// These are wider than normal note windows
#[derive(Debug, Clone)]
pub struct ReleaseConfig {
    pub pgreat_window: f64,
    pub great_window: f64,
    pub good_window: f64,
    pub bad_early_window: f64,
    pub bad_late_window: f64,
}

impl ReleaseConfig {
    /// Create beatoraja-style release windows
    pub fn beatoraja() -> Self {
        Self {
            pgreat_window: 120.0,
            great_window: 160.0,
            good_window: 200.0,
            bad_early_window: 220.0,
            bad_late_window: 280.0,
        }
    }

    /// Create LR2-style release windows
    /// LR2 uses the same window as normal note's GOOD window
    pub fn lr2(rank: JudgeRank) -> Self {
        let good_window = JudgeConfig::lr2(rank).good_window;
        Self {
            pgreat_window: good_window,
            great_window: good_window,
            good_window,
            bad_early_window: good_window,
            bad_late_window: good_window,
        }
    }

    /// Create release config for the given system and rank
    pub fn for_system(system: JudgeSystemType, rank: JudgeRank) -> Self {
        match system {
            JudgeSystemType::Beatoraja => Self::beatoraja(),
            JudgeSystemType::Lr2 => Self::lr2(rank),
        }
    }

    /// Get the BAD window for a given timing direction
    pub fn bad_window(&self, early: bool) -> f64 {
        if early {
            self.bad_early_window
        } else {
            self.bad_late_window
        }
    }

    /// Create an expanded version of this config (multiply all windows by 1.5x)
    /// Used for EXPAND JUDGE assist option
    #[allow(dead_code)]
    pub fn expand(&self) -> Self {
        const EXPAND_MULTIPLIER: f64 = 1.5;
        Self {
            pgreat_window: self.pgreat_window * EXPAND_MULTIPLIER,
            great_window: self.great_window * EXPAND_MULTIPLIER,
            good_window: self.good_window * EXPAND_MULTIPLIER,
            bad_early_window: self.bad_early_window * EXPAND_MULTIPLIER,
            bad_late_window: self.bad_late_window * EXPAND_MULTIPLIER,
        }
    }
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self::beatoraja()
    }
}

impl JudgeConfig {
    /// Create beatoraja-style timing windows for given rank
    /// beatoraja uses scale-based calculation with asymmetric BAD window
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
            // beatoraja has asymmetric BAD: +220ms (early) / -280ms (late)
            bad_early_window: 220.0 * scale,
            bad_late_window: 280.0 * scale,
        }
    }

    /// Create LR2-style timing windows for given rank
    /// LR2 uses fixed windows per rank with symmetric BAD
    pub fn lr2(rank: JudgeRank) -> Self {
        match rank {
            JudgeRank::VeryEasy | JudgeRank::Easy => Self {
                pgreat_window: 21.0,
                great_window: 60.0,
                good_window: 120.0,
                bad_early_window: 200.0,
                bad_late_window: 200.0,
            },
            JudgeRank::Normal => Self {
                pgreat_window: 18.0,
                great_window: 40.0,
                good_window: 100.0,
                bad_early_window: 200.0,
                bad_late_window: 200.0,
            },
            JudgeRank::Hard => Self {
                pgreat_window: 15.0,
                great_window: 30.0,
                good_window: 60.0,
                bad_early_window: 200.0,
                bad_late_window: 200.0,
            },
            JudgeRank::VeryHard => Self {
                pgreat_window: 8.0,
                great_window: 24.0,
                good_window: 40.0,
                bad_early_window: 200.0,
                bad_late_window: 200.0,
            },
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

    /// Get the BAD window for a given timing direction
    pub fn bad_window(&self, early: bool) -> f64 {
        if early {
            self.bad_early_window
        } else {
            self.bad_late_window
        }
    }

    /// Get the maximum BAD window (for miss detection)
    #[allow(dead_code)]
    pub fn max_bad_window(&self) -> f64 {
        self.bad_early_window.max(self.bad_late_window)
    }

    /// Create an expanded version of this config (multiply all windows by 1.5x)
    /// Used for EXPAND JUDGE assist option
    #[allow(dead_code)]
    pub fn expand(&self) -> Self {
        const EXPAND_MULTIPLIER: f64 = 1.5;
        Self {
            pgreat_window: self.pgreat_window * EXPAND_MULTIPLIER,
            great_window: self.great_window * EXPAND_MULTIPLIER,
            good_window: self.good_window * EXPAND_MULTIPLIER,
            bad_early_window: self.bad_early_window * EXPAND_MULTIPLIER,
            bad_late_window: self.bad_late_window * EXPAND_MULTIPLIER,
        }
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
    bad_early_window: Option<f64>,
    bad_late_window: Option<f64>,
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

    pub fn bad_early_window(mut self, ms: f64) -> Self {
        self.bad_early_window = Some(ms);
        self
    }

    pub fn bad_late_window(mut self, ms: f64) -> Self {
        self.bad_late_window = Some(ms);
        self
    }

    /// Set symmetric BAD window (same for early and late)
    pub fn bad_window(mut self, ms: f64) -> Self {
        self.bad_early_window = Some(ms);
        self.bad_late_window = Some(ms);
        self
    }

    pub fn build(self) -> JudgeConfig {
        let default = JudgeConfig::normal();
        JudgeConfig {
            pgreat_window: self.pgreat_window.unwrap_or(default.pgreat_window),
            great_window: self.great_window.unwrap_or(default.great_window),
            good_window: self.good_window.unwrap_or(default.good_window),
            bad_early_window: self.bad_early_window.unwrap_or(default.bad_early_window),
            bad_late_window: self.bad_late_window.unwrap_or(default.bad_late_window),
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
            release_config: ReleaseConfig::for_system(system_type, rank),
        }
    }

    #[allow(dead_code)]
    pub fn with_release_config(mut self, release_config: ReleaseConfig) -> Self {
        self.release_config = release_config;
        self
    }

    /// Apply EXPAND JUDGE (multiply all windows by 1.5x)
    #[allow(dead_code)]
    pub fn with_expand(mut self) -> Self {
        self.config = self.config.expand();
        self.release_config = self.release_config.expand();
        self
    }

    /// Get the current system type
    #[allow(dead_code)]
    pub fn system_type(&self) -> JudgeSystemType {
        self.system_type
    }

    pub fn judge(&self, time_diff_ms: f64) -> Option<JudgeResult> {
        // time_diff_ms > 0 means player pressed early (note hasn't arrived yet)
        // time_diff_ms < 0 means player pressed late (note has passed)
        let abs_diff = time_diff_ms.abs();

        if abs_diff <= self.config.pgreat_window {
            Some(JudgeResult::PGreat)
        } else if abs_diff <= self.config.great_window {
            Some(JudgeResult::Great)
        } else if abs_diff <= self.config.good_window {
            Some(JudgeResult::Good)
        } else {
            // BAD window check - handle asymmetric windows
            let early = time_diff_ms > 0.0;
            if abs_diff <= self.config.bad_window(early) {
                Some(JudgeResult::Bad)
            } else {
                None
            }
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
        } else {
            // BAD window check - handle asymmetric windows
            let early = time_diff_ms > 0.0;
            if abs_diff <= self.release_config.bad_window(early) {
                Some(JudgeResult::Bad)
            } else {
                None
            }
        }
    }

    /// Check if release is too early (before the release window)
    pub fn is_early_release(&self, time_diff_ms: f64) -> bool {
        time_diff_ms > self.release_config.bad_early_window
    }

    /// Check if a button press should trigger empty POOR
    /// time_diff_ms: current_time - nearest_note_time (positive = note is in the past)
    #[allow(dead_code)]
    pub fn is_empty_poor(&self, time_diff_ms: f64) -> bool {
        let early = time_diff_ms > 0.0;
        let abs_diff = time_diff_ms.abs();
        let bad_window = self.config.bad_window(early);

        match self.system_type {
            JudgeSystemType::Beatoraja => {
                // beatoraja: both early and late presses can trigger empty POOR
                abs_diff > bad_window
            }
            JudgeSystemType::Lr2 => {
                // LR2: only early presses trigger empty POOR (not after)
                time_diff_ms < 0.0 && abs_diff > self.config.bad_late_window
            }
        }
    }

    // Public API for checking if time difference is within any judge window
    #[allow(dead_code)]
    pub fn is_in_window(&self, time_diff_ms: f64) -> bool {
        let early = time_diff_ms > 0.0;
        time_diff_ms.abs() <= self.config.bad_window(early)
    }

    pub fn is_missed(&self, time_diff_ms: f64) -> bool {
        // Note is missed when it's past the late BAD window
        time_diff_ms < -self.config.bad_late_window
    }

    #[allow(dead_code)]
    pub fn release_bad_window(&self) -> f64 {
        // Return the maximum for compatibility
        self.release_config
            .bad_early_window
            .max(self.release_config.bad_late_window)
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
        // beatoraja has asymmetric BAD: +220ms (early) / -280ms (late)
        assert!((config.bad_early_window - 220.0).abs() < 0.01);
        assert!((config.bad_late_window - 280.0).abs() < 0.01);
    }

    #[test]
    fn test_beatoraja_rank_scaling() {
        let easy = JudgeConfig::beatoraja(JudgeRank::Easy);
        let hard = JudgeConfig::beatoraja(JudgeRank::Hard);

        // Hard should be half of Easy
        assert!((hard.pgreat_window - easy.pgreat_window * 0.5).abs() < 0.01);
        assert!((hard.bad_early_window - easy.bad_early_window * 0.5).abs() < 0.01);
        assert!((hard.bad_late_window - easy.bad_late_window * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lr2_timing_windows() {
        let config = JudgeConfig::lr2(JudgeRank::Easy);
        assert!((config.pgreat_window - 21.0).abs() < 0.01);
        assert!((config.great_window - 60.0).abs() < 0.01);
        assert!((config.good_window - 120.0).abs() < 0.01);
        // LR2 has symmetric BAD
        assert!((config.bad_early_window - 200.0).abs() < 0.01);
        assert!((config.bad_late_window - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_lr2_timing_windows_all_ranks() {
        // NORMAL
        let normal = JudgeConfig::lr2(JudgeRank::Normal);
        assert!((normal.pgreat_window - 18.0).abs() < 0.01);
        assert!((normal.great_window - 40.0).abs() < 0.01);
        assert!((normal.good_window - 100.0).abs() < 0.01);

        // HARD
        let hard = JudgeConfig::lr2(JudgeRank::Hard);
        assert!((hard.pgreat_window - 15.0).abs() < 0.01);
        assert!((hard.great_window - 30.0).abs() < 0.01);
        assert!((hard.good_window - 60.0).abs() < 0.01);

        // VERY HARD
        let vhard = JudgeConfig::lr2(JudgeRank::VeryHard);
        assert!((vhard.pgreat_window - 8.0).abs() < 0.01);
        assert!((vhard.great_window - 24.0).abs() < 0.01);
        assert!((vhard.good_window - 40.0).abs() < 0.01);
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
        // Early press (positive time_diff means note hasn't arrived)
        assert!(judge.is_empty_poor(300.0)); // Way too early
        // Late press (negative time_diff means note has passed)
        assert!(judge.is_empty_poor(-300.0)); // Way too late
        assert!(!judge.is_empty_poor(100.0)); // Within window
    }

    #[test]
    fn test_empty_poor_lr2() {
        let judge = JudgeSystem::for_system(JudgeSystemType::Lr2, JudgeRank::Easy);

        // LR2: Only late presses trigger empty POOR (after the note)
        assert!(judge.is_empty_poor(-300.0)); // Late - should be empty POOR
        assert!(!judge.is_empty_poor(300.0)); // Early - should NOT be empty POOR in LR2
    }

    #[test]
    fn test_beatoraja_asymmetric_bad_judgment() {
        let judge = JudgeSystem::for_system(JudgeSystemType::Beatoraja, JudgeRank::Easy);

        // Early hit at 210ms should be BAD (within +220ms window)
        assert_eq!(judge.judge(210.0), Some(JudgeResult::Bad));
        // Early hit at 230ms should be None (outside +220ms window)
        assert_eq!(judge.judge(230.0), None);

        // Late hit at -270ms should be BAD (within -280ms window)
        assert_eq!(judge.judge(-270.0), Some(JudgeResult::Bad));
        // Late hit at -290ms should be None (outside -280ms window)
        assert_eq!(judge.judge(-290.0), None);
    }

    #[test]
    fn test_release_config_beatoraja() {
        let config = ReleaseConfig::beatoraja();
        assert!((config.pgreat_window - 120.0).abs() < 0.01);
        assert!((config.great_window - 160.0).abs() < 0.01);
        assert!((config.good_window - 200.0).abs() < 0.01);
        assert!((config.bad_early_window - 220.0).abs() < 0.01);
        assert!((config.bad_late_window - 280.0).abs() < 0.01);
    }

    #[test]
    fn test_release_config_lr2() {
        let config = ReleaseConfig::lr2(JudgeRank::Easy);
        // LR2 release uses GOOD window for all judgments
        assert!((config.pgreat_window - 120.0).abs() < 0.01);
        assert!((config.great_window - 120.0).abs() < 0.01);
        assert!((config.good_window - 120.0).abs() < 0.01);
        assert!((config.bad_early_window - 120.0).abs() < 0.01);
        assert!((config.bad_late_window - 120.0).abs() < 0.01);
    }
}
