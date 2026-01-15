use super::GaugeType;

/// Clear lamp type for display purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClearLamp {
    #[default]
    NoPlay,
    Failed,
    AssistEasy,
    Easy,
    Normal,
    Hard,
    ExHard,
    FullCombo,
}

impl ClearLamp {
    pub fn from_gauge(gauge_type: Option<GaugeType>, is_full_combo: bool) -> Self {
        if is_full_combo {
            return ClearLamp::FullCombo;
        }
        match gauge_type {
            None => ClearLamp::Failed,
            Some(GaugeType::AssistEasy) => ClearLamp::AssistEasy,
            Some(GaugeType::Easy) => ClearLamp::Easy,
            Some(GaugeType::Normal) => ClearLamp::Normal,
            Some(GaugeType::Hard) => ClearLamp::Hard,
            Some(GaugeType::ExHard) | Some(GaugeType::Hazard) => ClearLamp::ExHard,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ClearLamp::NoPlay => "NO PLAY",
            ClearLamp::Failed => "FAILED",
            ClearLamp::AssistEasy => "ASSIST EASY",
            ClearLamp::Easy => "EASY CLEAR",
            ClearLamp::Normal => "CLEAR",
            ClearLamp::Hard => "HARD CLEAR",
            ClearLamp::ExHard => "EX-HARD CLEAR",
            ClearLamp::FullCombo => "FULL COMBO",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayResult {
    pub title: String,
    pub artist: String,
    pub ex_score: u32,
    pub max_combo: u32,
    pub pgreat_count: u32,
    pub great_count: u32,
    pub good_count: u32,
    pub bad_count: u32,
    pub poor_count: u32,
    pub total_notes: u32,
    pub clear_lamp: ClearLamp,
}

impl PlayResult {
    pub fn accuracy(&self) -> f64 {
        if self.total_notes == 0 {
            return 0.0;
        }
        let max_ex = self.total_notes * 2;
        self.ex_score as f64 / max_ex as f64 * 100.0
    }

    pub fn rank(&self) -> &'static str {
        let acc = self.accuracy();
        if acc >= 100.0 {
            "MAX"
        } else if acc >= 94.44 {
            "AAA"
        } else if acc >= 88.88 {
            "AA"
        } else if acc >= 77.77 {
            "A"
        } else if acc >= 66.66 {
            "B"
        } else if acc >= 55.55 {
            "C"
        } else if acc >= 44.44 {
            "D"
        } else if acc >= 33.33 {
            "E"
        } else {
            "F"
        }
    }
}
