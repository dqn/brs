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

impl JudgeConfig {
    pub fn normal() -> Self {
        Self {
            pgreat_window: 20.0,
            great_window: 60.0,
            good_window: 150.0,
            bad_window: 280.0,
        }
    }

    #[allow(dead_code)]
    pub fn easy() -> Self {
        Self {
            pgreat_window: 25.0,
            great_window: 75.0,
            good_window: 200.0,
            bad_window: 350.0,
        }
    }

    #[allow(dead_code)]
    pub fn hard() -> Self {
        Self {
            pgreat_window: 15.0,
            great_window: 45.0,
            good_window: 100.0,
            bad_window: 200.0,
        }
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
    config: JudgeConfig,
}

impl JudgeSystem {
    pub fn new(config: JudgeConfig) -> Self {
        Self { config }
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

    // Public API for checking if time difference is within any judge window
    #[allow(dead_code)]
    pub fn is_in_window(&self, time_diff_ms: f64) -> bool {
        time_diff_ms.abs() <= self.config.bad_window
    }

    pub fn is_missed(&self, time_diff_ms: f64) -> bool {
        time_diff_ms < -self.config.bad_window
    }
}

impl Default for JudgeSystem {
    fn default() -> Self {
        Self::new(JudgeConfig::default())
    }
}
