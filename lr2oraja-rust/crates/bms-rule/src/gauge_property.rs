/// Gauge property definitions for all play modes and gauge types.
///
/// Ported from Java: `GaugeProperty.java` and `GrooveGauge.GaugeModifier`.
///
/// Each play mode (FIVEKEYS, SEVENKEYS, PMS, KEYBOARD, LR2) defines 9 gauge
/// element configurations (one per GaugeType). The GaugeModifier determines
/// how the base gauge increment/decrement values are adjusted based on the
/// chart's TOTAL value and total note count.
use crate::JUDGE_COUNT;

/// Number of gauge types.
pub const GAUGE_TYPE_COUNT: usize = 9;

/// Gauge type indices matching Java constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum GaugeType {
    AssistEasy = 0,
    Easy = 1,
    Normal = 2,
    Hard = 3,
    ExHard = 4,
    Hazard = 5,
    Class = 6,
    ExClass = 7,
    ExHardClass = 8,
}

impl GaugeType {
    pub const ALL: [GaugeType; GAUGE_TYPE_COUNT] = [
        GaugeType::AssistEasy,
        GaugeType::Easy,
        GaugeType::Normal,
        GaugeType::Hard,
        GaugeType::ExHard,
        GaugeType::Hazard,
        GaugeType::Class,
        GaugeType::ExClass,
        GaugeType::ExHardClass,
    ];

    pub fn is_course_gauge(self) -> bool {
        matches!(self, Self::Class | Self::ExClass | Self::ExHardClass)
    }
}

impl TryFrom<usize> for GaugeType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::ALL.get(value).copied().ok_or(())
    }
}

/// Modifier that adjusts gauge increment/decrement values based on chart properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GaugeModifier {
    /// Recovery amount is scaled by TOTAL / total_notes.
    Total,
    /// Recovery amount is capped based on TOTAL and total_notes.
    LimitIncrement,
    /// Damage amount is increased when TOTAL is low or note count is low.
    ModifyDamage,
}

impl GaugeModifier {
    /// Apply the modifier to a base gauge value.
    ///
    /// # Arguments
    /// - `value`: Base gauge increment/decrement value
    /// - `total`: Chart's TOTAL value (from #TOTAL header)
    /// - `total_notes`: Total number of playable notes in the chart
    pub fn modify(self, value: f32, total: f64, total_notes: usize) -> f32 {
        match self {
            Self::Total => modify_total(value, total, total_notes),
            Self::LimitIncrement => modify_limit_increment(value, total, total_notes),
            Self::ModifyDamage => modify_damage(value, total, total_notes),
        }
    }
}

/// TOTAL modifier: `if value > 0 { value * total / total_notes } else { value }`
fn modify_total(value: f32, total: f64, total_notes: usize) -> f32 {
    if value > 0.0 {
        (f64::from(value) * total / total_notes as f64) as f32
    } else {
        value
    }
}

/// LimitIncrement modifier: caps recovery based on (2*total - 320) / total_notes.
fn modify_limit_increment(value: f32, total: f64, total_notes: usize) -> f32 {
    let pg = ((2.0 * total - 320.0) / total_notes as f64).clamp(0.0, 0.15) as f32;
    if value > 0.0 {
        value * pg / 0.15
    } else {
        value
    }
}

/// ModifyDamage modifier: increases damage when TOTAL is low or note count is low.
///
/// Only applies to negative (damage) values. Two correction factors are computed:
/// - fix1: based on TOTAL value
/// - fix2: piecewise linear function based on total_notes
///
/// The larger correction factor is applied.
fn modify_damage(value: f32, total: f64, total_notes: usize) -> f32 {
    if value < 0.0 {
        // TOTAL correction (<240)
        // Java: (float)(10.0 / Math.min(10.0, Math.max(1.0, Math.floor(total/16.0) - 5.0)))
        let fix1 = 10.0_f32 / ((total / 16.0).floor() as f32 - 5.0).clamp(1.0, 10.0);

        // Note count correction (<1000)
        let n = total_notes;
        let fix2 = if n <= 20 {
            10.0_f32
        } else if n < 30 {
            8.0 + 0.2 * (30 - n) as f32
        } else if n < 60 {
            5.0 + 0.2 * (60 - n) as f32 / 3.0
        } else if n < 125 {
            4.0 + (125 - n) as f32 / 65.0
        } else if n < 250 {
            3.0 + 0.008 * (250 - n) as f32
        } else if n < 500 {
            2.0 + 0.004 * (500 - n) as f32
        } else if n < 1000 {
            1.0 + 0.002 * (1000 - n) as f32
        } else {
            1.0
        };

        value * fix1.max(fix2)
    } else {
        value
    }
}

/// Guts damage reduction entry.
///
/// When gauge value is below `threshold`, damage is multiplied by `multiplier`.
/// The guts table is checked in order; the first matching entry is used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutsEntry {
    pub threshold: f32,
    pub multiplier: f32,
}

/// Configuration for a single gauge type within a play mode.
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeElementProperty {
    /// Modifier type for adjusting gauge values (None = no adjustment)
    pub modifier: Option<GaugeModifier>,
    /// Minimum gauge value
    pub min: f32,
    /// Maximum gauge value
    pub max: f32,
    /// Initial gauge value
    pub init: f32,
    /// Border value required for clear
    pub border: f32,
    /// Death threshold (gauge goes to 0 if below this)
    pub death: f32,
    /// Base gauge change values: [PG, GR, GD, BD, PR, MS]
    pub values: [f32; JUDGE_COUNT],
    /// Guts damage reduction table
    pub guts: Vec<GutsEntry>,
}

/// Gauge property set containing all 9 gauge element configurations for a play mode.
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeProperty {
    pub elements: [GaugeElementProperty; GAUGE_TYPE_COUNT],
}

impl GaugeProperty {
    pub fn element(&self, gauge_type: GaugeType) -> &GaugeElementProperty {
        &self.elements[gauge_type as usize]
    }
}

// -- Helper constructors for concise constant definitions --

#[allow(clippy::too_many_arguments)]
fn elem(
    modifier: Option<GaugeModifier>,
    min: f32,
    max: f32,
    init: f32,
    border: f32,
    death: f32,
    values: [f32; JUDGE_COUNT],
    guts: Vec<GutsEntry>,
) -> GaugeElementProperty {
    GaugeElementProperty {
        modifier,
        min,
        max,
        init,
        border,
        death,
        values,
        guts,
    }
}

fn guts(threshold: f32, multiplier: f32) -> GutsEntry {
    GutsEntry {
        threshold,
        multiplier,
    }
}

// ============================================================
// Gauge property constant constructors for each play mode.
// Values are taken directly from Java GaugeProperty.GaugeElementProperty.
//
// Order: AssistEasy, Easy, Normal, Hard, ExHard, Hazard, Class, ExClass, ExHardClass
// Values array: [PG, GR, GD, BD, PR, MS]
// ============================================================

/// FIVEKEYS gauge properties.
pub fn fivekeys() -> GaugeProperty {
    GaugeProperty {
        elements: [
            // ASSIST_EASY_5
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                50.0,
                0.0,
                [1.0, 1.0, 0.5, -1.5, -3.0, -0.5],
                vec![],
            ),
            // EASY_5
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                75.0,
                0.0,
                [1.0, 1.0, 0.5, -1.5, -4.5, -1.0],
                vec![],
            ),
            // NORMAL_5
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                75.0,
                0.0,
                [1.0, 1.0, 0.5, -3.0, -6.0, -2.0],
                vec![],
            ),
            // HARD_5
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.0, 0.0, 0.0, -5.0, -10.0, -5.0],
                vec![],
            ),
            // EXHARD_5
            elem(
                Some(GaugeModifier::ModifyDamage),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.0, 0.0, 0.0, -10.0, -20.0, -10.0],
                vec![],
            ),
            // HAZARD_5
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.0, 0.0, 0.0, -100.0, -100.0, -100.0],
                vec![],
            ),
            // CLASS_5
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.01, 0.01, 0.0, -0.5, -1.0, -0.5],
                vec![],
            ),
            // EXCLASS_5
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.01, 0.01, 0.0, -1.0, -2.0, -1.0],
                vec![],
            ),
            // EXHARDCLASS_5
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.01, 0.01, 0.0, -2.5, -5.0, -2.5],
                vec![],
            ),
        ],
    }
}

/// SEVENKEYS gauge properties.
pub fn sevenkeys() -> GaugeProperty {
    GaugeProperty {
        elements: [
            // ASSIST_EASY
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                60.0,
                0.0,
                [1.0, 1.0, 0.5, -1.5, -3.0, -0.5],
                vec![],
            ),
            // EASY
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                80.0,
                0.0,
                [1.0, 1.0, 0.5, -1.5, -4.5, -1.0],
                vec![],
            ),
            // NORMAL
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                80.0,
                0.0,
                [1.0, 1.0, 0.5, -3.0, -6.0, -2.0],
                vec![],
            ),
            // HARD
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.12, 0.03, -5.0, -10.0, -5.0],
                vec![
                    guts(10.0, 0.4),
                    guts(20.0, 0.5),
                    guts(30.0, 0.6),
                    guts(40.0, 0.7),
                    guts(50.0, 0.8),
                ],
            ),
            // EXHARD
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.06, 0.0, -8.0, -16.0, -8.0],
                vec![],
            ),
            // HAZARD
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.06, 0.0, -100.0, -100.0, -10.0],
                vec![],
            ),
            // CLASS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.12, 0.06, -1.5, -3.0, -1.5],
                vec![
                    guts(5.0, 0.4),
                    guts(10.0, 0.5),
                    guts(15.0, 0.6),
                    guts(20.0, 0.7),
                    guts(25.0, 0.8),
                ],
            ),
            // EXCLASS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.12, 0.03, -3.0, -6.0, -3.0],
                vec![],
            ),
            // EXHARDCLASS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.06, 0.0, -5.0, -10.0, -5.0],
                vec![],
            ),
        ],
    }
}

/// PMS gauge properties.
pub fn pms() -> GaugeProperty {
    GaugeProperty {
        elements: [
            // ASSIST_EASY_PMS
            elem(
                Some(GaugeModifier::Total),
                2.0,
                120.0,
                30.0,
                65.0,
                0.0,
                [1.0, 1.0, 0.5, -1.0, -2.0, -2.0],
                vec![],
            ),
            // EASY_PMS
            elem(
                Some(GaugeModifier::Total),
                2.0,
                120.0,
                30.0,
                85.0,
                0.0,
                [1.0, 1.0, 0.5, -1.0, -3.0, -3.0],
                vec![],
            ),
            // NORMAL_PMS
            elem(
                Some(GaugeModifier::Total),
                2.0,
                120.0,
                30.0,
                85.0,
                0.0,
                [1.0, 1.0, 0.5, -2.0, -6.0, -6.0],
                vec![],
            ),
            // HARD_PMS
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.12, 0.03, -5.0, -10.0, -10.0],
                vec![
                    guts(10.0, 0.4),
                    guts(20.0, 0.5),
                    guts(30.0, 0.6),
                    guts(40.0, 0.7),
                    guts(50.0, 0.8),
                ],
            ),
            // EXHARD_PMS
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.06, 0.0, -10.0, -15.0, -15.0],
                vec![],
            ),
            // HAZARD_PMS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.06, 0.0, -100.0, -100.0, -100.0],
                vec![],
            ),
            // CLASS_PMS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.12, 0.06, -1.5, -3.0, -3.0],
                vec![
                    guts(5.0, 0.4),
                    guts(10.0, 0.5),
                    guts(15.0, 0.6),
                    guts(20.0, 0.7),
                    guts(25.0, 0.8),
                ],
            ),
            // EXCLASS_PMS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.12, 0.03, -3.0, -6.0, -6.0],
                vec![],
            ),
            // EXHARDCLASS_PMS
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.15, 0.06, 0.0, -5.0, -10.0, -10.0],
                vec![],
            ),
        ],
    }
}

/// KEYBOARD gauge properties.
pub fn keyboard() -> GaugeProperty {
    GaugeProperty {
        elements: [
            // ASSIST_EASY_KB
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                30.0,
                50.0,
                0.0,
                [1.0, 1.0, 0.5, -1.0, -2.0, -1.0],
                vec![],
            ),
            // EASY_KB
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                70.0,
                0.0,
                [1.0, 1.0, 0.5, -1.0, -3.0, -1.0],
                vec![],
            ),
            // NORMAL_KB
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                70.0,
                0.0,
                [1.0, 1.0, 0.5, -2.0, -4.0, -2.0],
                vec![],
            ),
            // HARD_KB
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.2, 0.2, 0.1, -4.0, -8.0, -4.0],
                vec![
                    guts(10.0, 0.4),
                    guts(20.0, 0.5),
                    guts(30.0, 0.6),
                    guts(40.0, 0.7),
                    guts(50.0, 0.8),
                ],
            ),
            // EXHARD_KB
            elem(
                Some(GaugeModifier::LimitIncrement),
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.2, 0.1, 0.0, -6.0, -12.0, -6.0],
                vec![],
            ),
            // HAZARD_KB
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.2, 0.1, 0.0, -100.0, -100.0, -100.0],
                vec![],
            ),
            // CLASS_KB
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.2, 0.2, 0.1, -1.5, -3.0, -1.5],
                vec![
                    guts(5.0, 0.4),
                    guts(10.0, 0.5),
                    guts(15.0, 0.6),
                    guts(20.0, 0.7),
                    guts(25.0, 0.8),
                ],
            ),
            // EXCLASS_KB
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.2, 0.2, 0.1, -3.0, -6.0, -3.0],
                vec![],
            ),
            // EXHARDCLASS_KB
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                0.0,
                [0.2, 0.1, 0.0, -5.0, -10.0, -5.0],
                vec![],
            ),
        ],
    }
}

/// LR2 gauge properties.
pub fn lr2() -> GaugeProperty {
    GaugeProperty {
        elements: [
            // ASSIST_EASY_LR2
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                60.0,
                0.0,
                [1.2, 1.2, 0.6, -3.2, -4.8, -1.6],
                vec![],
            ),
            // EASY_LR2
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                80.0,
                0.0,
                [1.2, 1.2, 0.6, -3.2, -4.8, -1.6],
                vec![],
            ),
            // NORMAL_LR2
            elem(
                Some(GaugeModifier::Total),
                2.0,
                100.0,
                20.0,
                80.0,
                0.0,
                [1.0, 1.0, 0.5, -4.0, -6.0, -2.0],
                vec![],
            ),
            // HARD_LR2
            elem(
                Some(GaugeModifier::ModifyDamage),
                0.0,
                100.0,
                100.0,
                0.0,
                2.0,
                [0.1, 0.1, 0.05, -6.0, -10.0, -2.0],
                vec![guts(32.0, 0.6)],
            ),
            // EXHARD_LR2
            elem(
                Some(GaugeModifier::ModifyDamage),
                0.0,
                100.0,
                100.0,
                0.0,
                2.0,
                [0.1, 0.1, 0.05, -12.0, -20.0, -2.0],
                vec![],
            ),
            // HAZARD_LR2
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                2.0,
                [0.15, 0.06, 0.0, -100.0, -100.0, -10.0],
                vec![],
            ),
            // CLASS_LR2
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                2.0,
                [0.10, 0.10, 0.05, -2.0, -3.0, -2.0],
                vec![guts(32.0, 0.6)],
            ),
            // EXCLASS_LR2
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                2.0,
                [0.10, 0.10, 0.05, -6.0, -10.0, -2.0],
                vec![guts(32.0, 0.6)],
            ),
            // EXHARDCLASS_LR2
            elem(
                None,
                0.0,
                100.0,
                100.0,
                0.0,
                2.0,
                [0.10, 0.10, 0.05, -12.0, -20.0, -2.0],
                vec![],
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JUDGE_BD, JUDGE_GD, JUDGE_GR, JUDGE_PG, JUDGE_PR};

    // -- GaugeModifier::Total tests --

    #[test]
    fn test_modify_total_positive_value() {
        // total=200, notes=100: value * 200/100 = 1.0 * 2.0 = 2.0
        let result = GaugeModifier::Total.modify(1.0, 200.0, 100);
        assert!((result - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_modify_total_negative_value_unchanged() {
        // Negative values are not modified by Total
        let result = GaugeModifier::Total.modify(-3.0, 200.0, 100);
        assert!((result - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn test_modify_total_zero_value_unchanged() {
        let result = GaugeModifier::Total.modify(0.0, 200.0, 100);
        assert!(result.abs() < 1e-6);
    }

    #[test]
    fn test_modify_total_typical_chart() {
        // Typical chart: total=300, notes=1000
        // PG value: 1.0 * 300/1000 = 0.3
        let result = GaugeModifier::Total.modify(1.0, 300.0, 1000);
        assert!((result - 0.3).abs() < 1e-5);
    }

    // -- GaugeModifier::LimitIncrement tests --

    #[test]
    fn test_modify_limit_increment_high_total() {
        // total=300, notes=1000: pg = (600-320)/1000 = 0.28 clamped to 0.15
        // value * 0.15/0.15 = value
        let result = GaugeModifier::LimitIncrement.modify(0.15, 300.0, 1000);
        assert!((result - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_modify_limit_increment_low_total() {
        // total=160, notes=1000: pg = (320-320)/1000 = 0.0 clamped to 0.0
        // value * 0.0/0.15 = 0.0
        let result = GaugeModifier::LimitIncrement.modify(0.15, 160.0, 1000);
        assert!(result.abs() < 1e-6);
    }

    #[test]
    fn test_modify_limit_increment_negative_unchanged() {
        // Negative values are not modified by LimitIncrement
        let result = GaugeModifier::LimitIncrement.modify(-5.0, 300.0, 1000);
        assert!((result - (-5.0)).abs() < 1e-6);
    }

    #[test]
    fn test_modify_limit_increment_mid_total() {
        // total=200, notes=500: pg = (400-320)/500 = 0.16 clamped to 0.15
        // value * 0.15/0.15 = value
        let result = GaugeModifier::LimitIncrement.modify(0.12, 200.0, 500);
        assert!((result - 0.12).abs() < 1e-6);
    }

    #[test]
    fn test_modify_limit_increment_partial() {
        // total=180, notes=1000: pg = (360-320)/1000 = 0.04
        // value * 0.04/0.15 = 0.15 * 0.04/0.15 = 0.04
        let result = GaugeModifier::LimitIncrement.modify(0.15, 180.0, 1000);
        assert!((result - 0.04).abs() < 1e-5);
    }

    // -- GaugeModifier::ModifyDamage tests --

    #[test]
    fn test_modify_damage_positive_unchanged() {
        // Positive values are not modified by ModifyDamage
        let result = GaugeModifier::ModifyDamage.modify(0.1, 300.0, 1000);
        assert!((result - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_modify_damage_high_total_high_notes() {
        // total=300, notes=1000: both fix factors should be ~1.0
        // fix1 = 10/(floor(300/16) - 5) = 10/(18-5) = 10/13 ≈ 0.769
        // fix2 = 1.0 (notes >= 1000)
        // max(0.769, 1.0) = 1.0
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 300.0, 1000);
        assert!((result - (-6.0)).abs() < 1e-5);
    }

    #[test]
    fn test_modify_damage_low_total() {
        // total=80, notes=1000: fix1 = 10/(floor(80/16)-5) = 10/(5-5) = 10/1 = 10.0
        // (clamped max(1,0) → 1 since 5-5=0 → clamped to 1)
        // fix2 = 1.0
        // max(10, 1) = 10
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 80.0, 1000);
        assert!((result - (-60.0)).abs() < 1e-4);
    }

    #[test]
    fn test_modify_damage_low_notes() {
        // total=300, notes=10: fix1 = 10/(18-5)=10/13≈0.769, fix2=10.0
        // max(0.769, 10) = 10
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 300.0, 10);
        assert!((result - (-60.0)).abs() < 1e-4);
    }

    #[test]
    fn test_modify_damage_mid_notes_range() {
        // total=300, notes=50: fix1=10/13≈0.769
        // fix2 = 5.0 + 0.2*(60-50)/3.0 = 5.0 + 0.667 ≈ 5.667
        // max(0.769, 5.667) = 5.667
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 300.0, 50);
        let expected_fix2 = 5.0_f32 + 0.2 * (60.0 - 50.0) / 3.0;
        let expected = -6.0 * expected_fix2;
        assert!((result - expected).abs() < 1e-4);
    }

    #[test]
    fn test_modify_damage_500_notes() {
        // total=300, notes=300: fix1=10/13≈0.769
        // fix2 = 2.0 + 0.004*(500-300) = 2.0 + 0.8 = 2.8
        // max(0.769, 2.8) = 2.8
        let result = GaugeModifier::ModifyDamage.modify(-6.0, 300.0, 300);
        let expected = -6.0_f32 * 2.8;
        assert!((result - expected).abs() < 1e-4);
    }

    // -- GaugeProperty construction tests --

    #[test]
    fn test_fivekeys_element_count() {
        let prop = fivekeys();
        assert_eq!(prop.elements.len(), GAUGE_TYPE_COUNT);
    }

    #[test]
    fn test_sevenkeys_assist_easy() {
        let prop = sevenkeys();
        let ae = &prop.elements[GaugeType::AssistEasy as usize];
        assert_eq!(ae.modifier, Some(GaugeModifier::Total));
        assert!((ae.min - 2.0).abs() < 1e-6);
        assert!((ae.max - 100.0).abs() < 1e-6);
        assert!((ae.init - 20.0).abs() < 1e-6);
        assert!((ae.border - 60.0).abs() < 1e-6);
        assert!((ae.death - 0.0).abs() < 1e-6);
        assert!((ae.values[JUDGE_PG] - 1.0).abs() < 1e-6);
        assert!((ae.values[JUDGE_BD] - (-1.5)).abs() < 1e-6);
        assert!(ae.guts.is_empty());
    }

    #[test]
    fn test_sevenkeys_hard_guts() {
        let prop = sevenkeys();
        let hard = &prop.elements[GaugeType::Hard as usize];
        assert_eq!(hard.modifier, Some(GaugeModifier::LimitIncrement));
        assert_eq!(hard.guts.len(), 5);
        assert!((hard.guts[0].threshold - 10.0).abs() < 1e-6);
        assert!((hard.guts[0].multiplier - 0.4).abs() < 1e-6);
        assert!((hard.guts[4].threshold - 50.0).abs() < 1e-6);
        assert!((hard.guts[4].multiplier - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_lr2_hard_death_threshold() {
        let prop = lr2();
        let hard = &prop.elements[GaugeType::Hard as usize];
        assert_eq!(hard.modifier, Some(GaugeModifier::ModifyDamage));
        assert!((hard.death - 2.0).abs() < 1e-6);
        assert_eq!(hard.guts.len(), 1);
        assert!((hard.guts[0].threshold - 32.0).abs() < 1e-6);
        assert!((hard.guts[0].multiplier - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_pms_max_is_120() {
        let prop = pms();
        let ae = &prop.elements[GaugeType::AssistEasy as usize];
        assert!((ae.max - 120.0).abs() < 1e-6);
        let easy = &prop.elements[GaugeType::Easy as usize];
        assert!((easy.max - 120.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyboard_hard_values() {
        let prop = keyboard();
        let hard = &prop.elements[GaugeType::Hard as usize];
        assert!((hard.values[JUDGE_PG] - 0.2).abs() < 1e-6);
        assert!((hard.values[JUDGE_GR] - 0.2).abs() < 1e-6);
        assert!((hard.values[JUDGE_GD] - 0.1).abs() < 1e-6);
        assert!((hard.values[JUDGE_BD] - (-4.0)).abs() < 1e-6);
    }

    #[test]
    fn test_gauge_type_from_index() {
        for (i, expected) in GaugeType::ALL.iter().enumerate() {
            let gt: GaugeType = i.try_into().unwrap();
            assert_eq!(gt, *expected);
        }
        assert!(GaugeType::try_from(9).is_err());
    }

    #[test]
    fn test_gauge_type_is_course() {
        assert!(!GaugeType::AssistEasy.is_course_gauge());
        assert!(!GaugeType::Hard.is_course_gauge());
        assert!(GaugeType::Class.is_course_gauge());
        assert!(GaugeType::ExClass.is_course_gauge());
        assert!(GaugeType::ExHardClass.is_course_gauge());
    }

    #[test]
    fn test_all_gauge_properties_have_9_elements() {
        assert_eq!(fivekeys().elements.len(), 9);
        assert_eq!(sevenkeys().elements.len(), 9);
        assert_eq!(pms().elements.len(), 9);
        assert_eq!(keyboard().elements.len(), 9);
        assert_eq!(lr2().elements.len(), 9);
    }

    #[test]
    fn test_fivekeys_hard_no_recovery() {
        // HARD_5 has 0 recovery for all positive judges
        let prop = fivekeys();
        let hard = &prop.elements[GaugeType::Hard as usize];
        assert!(hard.values[JUDGE_PG].abs() < 1e-6);
        assert!(hard.values[JUDGE_GR].abs() < 1e-6);
        assert!(hard.values[JUDGE_GD].abs() < 1e-6);
    }

    #[test]
    fn test_hazard_instant_death_values() {
        // All HAZARD variants should have -100 for BD and PR
        let check_hazard = |prop: &GaugeProperty| {
            let h = &prop.elements[GaugeType::Hazard as usize];
            assert!((h.values[JUDGE_BD] - (-100.0)).abs() < 1e-6);
            assert!((h.values[JUDGE_PR] - (-100.0)).abs() < 1e-6);
        };
        check_hazard(&fivekeys());
        check_hazard(&sevenkeys());
        check_hazard(&pms());
        check_hazard(&keyboard());
    }
}
