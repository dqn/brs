use serde::{Deserialize, Serialize};

/// Gauge modifier type that determines how gauge values are adjusted based on model parameters.
/// Corresponds to GrooveGauge.GaugeModifier in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GaugeModifier {
    /// Recovery amount uses TOTAL value: f > 0 ? f * total / total_notes : f
    Total,
    /// Recovery capped by TOTAL: pg = max(min(0.15, (2*total - 320) / total_notes), 0); f > 0 ? f * pg / 0.15 : f
    LimitIncrement,
    /// Damage increased based on TOTAL and total_notes.
    ModifyDamage,
}

impl GaugeModifier {
    /// Apply the modifier to a gauge value.
    /// `f`: the base gauge value for a judge level.
    /// `total`: BMS model TOTAL value.
    /// `total_notes`: total playable notes in the chart.
    pub fn modify(self, f: f32, total: f64, total_notes: usize) -> f32 {
        match self {
            Self::Total => {
                if f > 0.0 {
                    f * (total as f32) / (total_notes as f32)
                } else {
                    f
                }
            }
            Self::LimitIncrement => {
                let pg = (0.15f32)
                    .min(((2.0 * total - 320.0) / total_notes as f64) as f32)
                    .max(0.0);
                if f > 0.0 { f * pg / 0.15 } else { f }
            }
            Self::ModifyDamage => {
                if f < 0.0 {
                    let mut fix2 = 1.0f32;
                    let fix1total: [f64; 10] = [
                        240.0, 230.0, 210.0, 200.0, 180.0, 160.0, 150.0, 130.0, 120.0, 0.0,
                    ];
                    let fix1table: [f32; 10] =
                        [1.0, 1.11, 1.25, 1.5, 1.666, 2.0, 2.5, 3.333, 5.0, 10.0];

                    let mut i = 0;
                    while i < fix1total.len() - 1 && total < fix1total[i] {
                        i += 1;
                    }

                    let mut note = 1000u32;
                    let mut modv = 0.002f32;
                    while note > total_notes as u32 || note > 1 {
                        fix2 += modv * (note as f32 - (total_notes as u32).max(note / 2) as f32);
                        note /= 2;
                        modv *= 2.0;
                    }
                    f * fix1table[i].max(fix2)
                } else {
                    f
                }
            }
        }
    }
}

/// A single gauge element property defining one gauge type's behavior.
/// Corresponds to GaugeProperty.GaugeElementProperty in beatoraja.
#[derive(Debug, Clone)]
pub struct GaugeElementProperty {
    /// Gauge modifier type (None for fixed gauges like HAZARD/CLASS).
    pub modifier: Option<GaugeModifier>,
    /// Minimum gauge value.
    pub min: f32,
    /// Maximum gauge value.
    pub max: f32,
    /// Initial gauge value.
    pub init: f32,
    /// Border value for clear condition.
    pub border: f32,
    /// Gauge change values per judge level [PG, GR, GD, BD, PR, MS].
    pub value: [f32; 6],
    /// Guts table: when value < guts[i][0], damage is multiplied by guts[i][1].
    pub guts: &'static [[f32; 2]],
}

// =========================================================================
// FIVEKEYS gauge element properties
// =========================================================================

pub const ASSIST_EASY_5: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 50.0,
    value: [1.0, 1.0, 0.5, -1.5, -3.0, -0.5],
    guts: &[],
};

pub const EASY_5: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 75.0,
    value: [1.0, 1.0, 0.5, -1.5, -4.5, -1.0],
    guts: &[],
};

pub const NORMAL_5: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 75.0,
    value: [1.0, 1.0, 0.5, -3.0, -6.0, -2.0],
    guts: &[],
};

pub const HARD_5: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.0, 0.0, 0.0, -5.0, -10.0, -5.0],
    guts: &[],
};

pub const EXHARD_5: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::ModifyDamage),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.0, 0.0, 0.0, -10.0, -20.0, -10.0],
    guts: &[],
};

pub const HAZARD_5: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.0, 0.0, 0.0, -100.0, -100.0, -100.0],
    guts: &[],
};

pub const CLASS_5: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.01, 0.01, 0.0, -0.5, -1.0, -0.5],
    guts: &[],
};

pub const EXCLASS_5: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.01, 0.01, 0.0, -1.0, -2.0, -1.0],
    guts: &[],
};

pub const EXHARDCLASS_5: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.01, 0.01, 0.0, -2.5, -5.0, -2.5],
    guts: &[],
};

// =========================================================================
// SEVENKEYS gauge element properties
// =========================================================================

pub const ASSIST_EASY: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 60.0,
    value: [1.0, 1.0, 0.5, -1.5, -3.0, -0.5],
    guts: &[],
};

pub const EASY: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 80.0,
    value: [1.0, 1.0, 0.5, -1.5, -4.5, -1.0],
    guts: &[],
};

pub const NORMAL: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 80.0,
    value: [1.0, 1.0, 0.5, -3.0, -6.0, -2.0],
    guts: &[],
};

pub const HARD: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.12, 0.03, -5.0, -10.0, -5.0],
    guts: &[
        [10.0, 0.4],
        [20.0, 0.5],
        [30.0, 0.6],
        [40.0, 0.7],
        [50.0, 0.8],
    ],
};

pub const EXHARD: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -8.0, -16.0, -8.0],
    guts: &[],
};

pub const HAZARD: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -100.0, -100.0, -10.0],
    guts: &[],
};

pub const CLASS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.12, 0.06, -1.5, -3.0, -1.5],
    guts: &[
        [5.0, 0.4],
        [10.0, 0.5],
        [15.0, 0.6],
        [20.0, 0.7],
        [25.0, 0.8],
    ],
};

pub const EXCLASS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.12, 0.03, -3.0, -6.0, -3.0],
    guts: &[],
};

pub const EXHARDCLASS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -5.0, -10.0, -5.0],
    guts: &[],
};

// =========================================================================
// PMS gauge element properties
// =========================================================================

pub const ASSIST_EASY_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 120.0,
    init: 30.0,
    border: 65.0,
    value: [1.0, 1.0, 0.5, -1.0, -2.0, -2.0],
    guts: &[],
};

pub const EASY_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 120.0,
    init: 30.0,
    border: 85.0,
    value: [1.0, 1.0, 0.5, -1.0, -3.0, -3.0],
    guts: &[],
};

pub const NORMAL_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 120.0,
    init: 30.0,
    border: 85.0,
    value: [1.0, 1.0, 0.5, -2.0, -6.0, -6.0],
    guts: &[],
};

pub const HARD_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.12, 0.03, -5.0, -10.0, -10.0],
    guts: &[
        [10.0, 0.4],
        [20.0, 0.5],
        [30.0, 0.6],
        [40.0, 0.7],
        [50.0, 0.8],
    ],
};

pub const EXHARD_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -10.0, -15.0, -15.0],
    guts: &[],
};

pub const HAZARD_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -100.0, -100.0, -100.0],
    guts: &[],
};

pub const CLASS_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.12, 0.06, -1.5, -3.0, -3.0],
    guts: &[
        [5.0, 0.4],
        [10.0, 0.5],
        [15.0, 0.6],
        [20.0, 0.7],
        [25.0, 0.8],
    ],
};

pub const EXCLASS_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.12, 0.03, -3.0, -6.0, -6.0],
    guts: &[],
};

pub const EXHARDCLASS_PMS: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -5.0, -10.0, -10.0],
    guts: &[],
};

// =========================================================================
// KEYBOARD gauge element properties
// =========================================================================

pub const ASSIST_EASY_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 30.0,
    border: 50.0,
    value: [1.0, 1.0, 0.5, -1.0, -2.0, -1.0],
    guts: &[],
};

pub const EASY_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 70.0,
    value: [1.0, 1.0, 0.5, -1.0, -3.0, -1.0],
    guts: &[],
};

pub const NORMAL_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 70.0,
    value: [1.0, 1.0, 0.5, -2.0, -4.0, -2.0],
    guts: &[],
};

pub const HARD_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.2, 0.2, 0.1, -4.0, -8.0, -4.0],
    guts: &[
        [10.0, 0.4],
        [20.0, 0.5],
        [30.0, 0.6],
        [40.0, 0.7],
        [50.0, 0.8],
    ],
};

pub const EXHARD_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::LimitIncrement),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.2, 0.1, 0.0, -6.0, -12.0, -6.0],
    guts: &[],
};

pub const HAZARD_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.2, 0.1, 0.0, -100.0, -100.0, -100.0],
    guts: &[],
};

pub const CLASS_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.2, 0.2, 0.1, -1.5, -3.0, -1.5],
    guts: &[
        [5.0, 0.4],
        [10.0, 0.5],
        [15.0, 0.6],
        [20.0, 0.7],
        [25.0, 0.8],
    ],
};

pub const EXCLASS_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.2, 0.2, 0.1, -3.0, -6.0, -3.0],
    guts: &[],
};

pub const EXHARDCLASS_KB: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.2, 0.1, 0.0, -5.0, -10.0, -5.0],
    guts: &[],
};

// =========================================================================
// LR2 gauge element properties
// =========================================================================

pub const ASSIST_EASY_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 60.0,
    value: [1.2, 1.2, 0.6, -3.2, -4.8, -1.6],
    guts: &[],
};

pub const EASY_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 80.0,
    value: [1.2, 1.2, 0.6, -3.2, -4.8, -1.6],
    guts: &[],
};

pub const NORMAL_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::Total),
    min: 2.0,
    max: 100.0,
    init: 20.0,
    border: 80.0,
    value: [1.0, 1.0, 0.5, -4.0, -6.0, -2.0],
    guts: &[],
};

pub const HARD_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::ModifyDamage),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.1, 0.1, 0.05, -6.0, -10.0, -2.0],
    guts: &[[30.0, 0.6]],
};

pub const EXHARD_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: Some(GaugeModifier::ModifyDamage),
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.1, 0.1, 0.05, -12.0, -20.0, -2.0],
    guts: &[],
};

pub const HAZARD_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.15, 0.06, 0.0, -100.0, -100.0, -10.0],
    guts: &[],
};

pub const CLASS_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.10, 0.10, 0.05, -2.0, -3.0, -2.0],
    guts: &[[30.0, 0.6]],
};

pub const EXCLASS_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.10, 0.10, 0.05, -6.0, -10.0, -2.0],
    guts: &[[30.0, 0.6]],
};

pub const EXHARDCLASS_LR2: GaugeElementProperty = GaugeElementProperty {
    modifier: None,
    min: 0.0,
    max: 100.0,
    init: 100.0,
    border: 0.0,
    value: [0.10, 0.10, 0.05, -12.0, -20.0, -2.0],
    guts: &[],
};

// =========================================================================
// GaugeProperty: grouping per mode
// =========================================================================

/// Gauge type indices.
pub const GAUGE_ASSIST_EASY: usize = 0;
pub const GAUGE_EASY: usize = 1;
pub const GAUGE_NORMAL: usize = 2;
pub const GAUGE_HARD: usize = 3;
pub const GAUGE_EXHARD: usize = 4;
pub const GAUGE_HAZARD: usize = 5;
pub const GAUGE_CLASS: usize = 6;
pub const GAUGE_EXCLASS: usize = 7;
pub const GAUGE_EXHARDCLASS: usize = 8;

/// Gauge property set for a mode.
/// Corresponds to GaugeProperty enum in beatoraja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GaugePropertyType {
    FiveKeys,
    SevenKeys,
    Pms,
    Keyboard,
    Lr2,
}

impl GaugePropertyType {
    /// Get gauge element properties for this mode.
    /// Returns 9 elements: [ASSIST_EASY, EASY, NORMAL, HARD, EXHARD, HAZARD, CLASS, EXCLASS, EXHARDCLASS].
    pub fn elements(self) -> [&'static GaugeElementProperty; 9] {
        match self {
            Self::FiveKeys => [
                &ASSIST_EASY_5,
                &EASY_5,
                &NORMAL_5,
                &HARD_5,
                &EXHARD_5,
                &HAZARD_5,
                &CLASS_5,
                &EXCLASS_5,
                &EXHARDCLASS_5,
            ],
            Self::SevenKeys => [
                &ASSIST_EASY,
                &EASY,
                &NORMAL,
                &HARD,
                &EXHARD,
                &HAZARD,
                &CLASS,
                &EXCLASS,
                &EXHARDCLASS,
            ],
            Self::Pms => [
                &ASSIST_EASY_PMS,
                &EASY_PMS,
                &NORMAL_PMS,
                &HARD_PMS,
                &EXHARD_PMS,
                &HAZARD_PMS,
                &CLASS_PMS,
                &EXCLASS_PMS,
                &EXHARDCLASS_PMS,
            ],
            Self::Keyboard => [
                &ASSIST_EASY_KB,
                &EASY_KB,
                &NORMAL_KB,
                &HARD_KB,
                &EXHARD_KB,
                &HAZARD_KB,
                &CLASS_KB,
                &EXCLASS_KB,
                &EXHARDCLASS_KB,
            ],
            Self::Lr2 => [
                &ASSIST_EASY_LR2,
                &EASY_LR2,
                &NORMAL_LR2,
                &HARD_LR2,
                &EXHARD_LR2,
                &HAZARD_LR2,
                &CLASS_LR2,
                &EXCLASS_LR2,
                &EXHARDCLASS_LR2,
            ],
        }
    }

    /// Get gauge property type from play mode.
    pub fn from_mode(mode: crate::model::note::PlayMode) -> Self {
        match mode {
            crate::model::note::PlayMode::Beat5K | crate::model::note::PlayMode::Beat10K => {
                Self::FiveKeys
            }
            crate::model::note::PlayMode::Beat7K | crate::model::note::PlayMode::Beat14K => {
                Self::SevenKeys
            }
            crate::model::note::PlayMode::PopN9K => Self::Pms,
            crate::model::note::PlayMode::Keyboard24K
            | crate::model::note::PlayMode::Keyboard24KDouble => Self::Keyboard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GaugeModifier tests
    // =========================================================================

    #[test]
    fn total_modifier_positive() {
        // total=300, total_notes=1000, f=1.0 -> 1.0 * 300 / 1000 = 0.3
        let result = GaugeModifier::Total.modify(1.0, 300.0, 1000);
        assert!((result - 0.3).abs() < 0.001);
    }

    #[test]
    fn total_modifier_negative_unchanged() {
        let result = GaugeModifier::Total.modify(-5.0, 300.0, 1000);
        assert!((result - (-5.0)).abs() < 0.001);
    }

    #[test]
    fn limit_increment_modifier() {
        // total=300, total_notes=1000
        // pg = min(0.15, (2*300 - 320) / 1000) = min(0.15, 0.28) = 0.15
        // f=0.15 -> 0.15 * 0.15 / 0.15 = 0.15
        let result = GaugeModifier::LimitIncrement.modify(0.15, 300.0, 1000);
        assert!((result - 0.15).abs() < 0.001);
    }

    #[test]
    fn limit_increment_modifier_low_total() {
        // total=170, total_notes=1000
        // pg = min(0.15, (2*170 - 320) / 1000) = min(0.15, 0.02) = 0.02
        // f=0.15 -> 0.15 * 0.02 / 0.15 = 0.02
        let result = GaugeModifier::LimitIncrement.modify(0.15, 170.0, 1000);
        assert!((result - 0.02).abs() < 0.001);
    }

    #[test]
    fn limit_increment_modifier_negative_unchanged() {
        let result = GaugeModifier::LimitIncrement.modify(-5.0, 300.0, 1000);
        assert!((result - (-5.0)).abs() < 0.001);
    }

    #[test]
    fn modify_damage_positive_unchanged() {
        let result = GaugeModifier::ModifyDamage.modify(0.1, 300.0, 1000);
        assert!((result - 0.1).abs() < 0.001);
    }

    #[test]
    fn modify_damage_high_total_high_notes() {
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 300.0, 1000);
        assert!(result < 0.0);
        assert!(result <= -6.0);
    }

    #[test]
    fn modify_damage_low_total_low_notes() {
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 100.0, 100);
        assert!(result < -6.0); // More damage due to multiplier > 1
    }

    // =========================================================================
    // GaugeElementProperty value tests
    // =========================================================================

    #[test]
    fn sevenkeys_normal_values() {
        assert_eq!(NORMAL.value, [1.0, 1.0, 0.5, -3.0, -6.0, -2.0]);
        assert!((NORMAL.min - 2.0).abs() < 0.001);
        assert!((NORMAL.max - 100.0).abs() < 0.001);
        assert!((NORMAL.init - 20.0).abs() < 0.001);
        assert!((NORMAL.border - 80.0).abs() < 0.001);
        assert_eq!(NORMAL.modifier, Some(GaugeModifier::Total));
    }

    #[test]
    fn sevenkeys_hard_values() {
        assert_eq!(HARD.value, [0.15, 0.12, 0.03, -5.0, -10.0, -5.0]);
        assert!((HARD.min - 0.0).abs() < 0.001);
        assert!((HARD.init - 100.0).abs() < 0.001);
        assert!((HARD.border - 0.0).abs() < 0.001);
        assert_eq!(HARD.modifier, Some(GaugeModifier::LimitIncrement));
        assert_eq!(
            HARD.guts,
            &[
                [10.0, 0.4],
                [20.0, 0.5],
                [30.0, 0.6],
                [40.0, 0.7],
                [50.0, 0.8]
            ]
        );
    }

    #[test]
    fn sevenkeys_hazard_values() {
        assert_eq!(HAZARD.value, [0.15, 0.06, 0.0, -100.0, -100.0, -10.0]);
        assert_eq!(HAZARD.modifier, None);
    }

    #[test]
    fn pms_normal_values() {
        assert_eq!(NORMAL_PMS.value, [1.0, 1.0, 0.5, -2.0, -6.0, -6.0]);
        assert!((NORMAL_PMS.max - 120.0).abs() < 0.001);
        assert!((NORMAL_PMS.init - 30.0).abs() < 0.001);
        assert!((NORMAL_PMS.border - 85.0).abs() < 0.001);
    }

    #[test]
    fn lr2_hard_values() {
        assert_eq!(HARD_LR2.value, [0.1, 0.1, 0.05, -6.0, -10.0, -2.0]);
        assert_eq!(HARD_LR2.modifier, Some(GaugeModifier::ModifyDamage));
        assert_eq!(HARD_LR2.guts, &[[30.0, 0.6]]);
    }

    #[test]
    fn keyboard_hard_values() {
        assert_eq!(HARD_KB.value, [0.2, 0.2, 0.1, -4.0, -8.0, -4.0]);
        assert_eq!(HARD_KB.modifier, Some(GaugeModifier::LimitIncrement));
    }

    // =========================================================================
    // GaugePropertyType tests
    // =========================================================================

    #[test]
    fn gauge_property_type_elements_count() {
        assert_eq!(GaugePropertyType::FiveKeys.elements().len(), 9);
        assert_eq!(GaugePropertyType::SevenKeys.elements().len(), 9);
        assert_eq!(GaugePropertyType::Pms.elements().len(), 9);
        assert_eq!(GaugePropertyType::Keyboard.elements().len(), 9);
        assert_eq!(GaugePropertyType::Lr2.elements().len(), 9);
    }

    #[test]
    fn gauge_property_type_from_mode() {
        use crate::model::note::PlayMode;
        assert_eq!(
            GaugePropertyType::from_mode(PlayMode::Beat5K),
            GaugePropertyType::FiveKeys
        );
        assert_eq!(
            GaugePropertyType::from_mode(PlayMode::Beat7K),
            GaugePropertyType::SevenKeys
        );
        assert_eq!(
            GaugePropertyType::from_mode(PlayMode::PopN9K),
            GaugePropertyType::Pms
        );
        assert_eq!(
            GaugePropertyType::from_mode(PlayMode::Keyboard24K),
            GaugePropertyType::Keyboard
        );
    }

    #[test]
    fn sevenkeys_elements_order() {
        let elems = GaugePropertyType::SevenKeys.elements();
        assert!((elems[GAUGE_ASSIST_EASY].border - 60.0).abs() < 0.001);
        assert!((elems[GAUGE_EASY].border - 80.0).abs() < 0.001);
        assert!((elems[GAUGE_NORMAL].border - 80.0).abs() < 0.001);
        assert!((elems[GAUGE_HARD].border - 0.0).abs() < 0.001);
        assert!((elems[GAUGE_HAZARD].init - 100.0).abs() < 0.001);
    }

    #[test]
    fn all_45_gauges_have_6_values() {
        for gtype in [
            GaugePropertyType::FiveKeys,
            GaugePropertyType::SevenKeys,
            GaugePropertyType::Pms,
            GaugePropertyType::Keyboard,
            GaugePropertyType::Lr2,
        ] {
            for elem in gtype.elements() {
                assert_eq!(
                    elem.value.len(),
                    6,
                    "Gauge {:?} has wrong value count",
                    gtype
                );
            }
        }
    }

    #[test]
    fn five_keys_assist_easy_border() {
        assert!((ASSIST_EASY_5.border - 50.0).abs() < 0.001);
    }

    #[test]
    fn pms_max_120_for_easy_gauges() {
        assert!((ASSIST_EASY_PMS.max - 120.0).abs() < 0.001);
        assert!((EASY_PMS.max - 120.0).abs() < 0.001);
        assert!((NORMAL_PMS.max - 120.0).abs() < 0.001);
    }
}
