/// Player rule definitions mapping play modes to judge and gauge properties.
use crate::gauge_property::{self, GaugeProperty};
use crate::judge_property::JudgeProperty;

/// Rule set for a play mode, providing judge windows and gauge configuration.
pub struct PlayerRule {
    pub judge: JudgeProperty,
    pub gauge: GaugeProperty,
}

impl PlayerRule {
    /// Get the rule set for a given play mode.
    ///
    /// Currently always returns LR2 rules (matching Java BMSPlayerRuleSet.LR2).
    pub fn lr2() -> Self {
        Self {
            judge: JudgeProperty::lr2(),
            gauge: gauge_property::lr2(),
        }
    }

    /// Calculate the default TOTAL value when not specified in BMS.
    ///
    /// Formula from Java BMSPlayerRule.calculateDefaultTotal:
    /// `160.0 + (n + min(max(n - 400, 0), 200)) * 0.16`
    pub fn default_total(total_notes: usize) -> f64 {
        let n = total_notes as f64;
        160.0 + (n + (n - 400.0).clamp(0.0, 200.0)) * 0.16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lr2_rule_returns_valid_properties() {
        let rule = PlayerRule::lr2();
        // LR2 judge: PG window should be [-21000, 21000]
        assert_eq!(rule.judge.note[0], [-21000, 21000]);
        // LR2 gauge: Normal init should be 20
        let normal = &rule.gauge.elements[2];
        assert!((normal.init - 20.0).abs() < 1e-6);
    }

    #[test]
    fn default_total_zero_notes() {
        let result = PlayerRule::default_total(0);
        assert!((result - 160.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_total_400_notes() {
        let result = PlayerRule::default_total(400);
        assert!((result - 224.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_total_600_notes() {
        let result = PlayerRule::default_total(600);
        assert!((result - 288.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_total_1000_notes() {
        let result = PlayerRule::default_total(1000);
        assert!((result - 352.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_total_monotonically_increasing() {
        let mut prev = PlayerRule::default_total(0);
        for n in (1..=2000).step_by(10) {
            let current = PlayerRule::default_total(n);
            assert!(
                current >= prev,
                "default_total({n}) = {current} < default_total({}) = {prev}",
                n - 10
            );
            prev = current;
        }
    }

    #[test]
    fn default_total_matches_java() {
        let test_cases: &[(usize, f64)] = &[
            (0, 160.0),
            (200, 192.0),
            (400, 224.0),
            (600, 288.0),
            (800, 320.0),
            (1000, 352.0),
        ];
        for &(notes, expected) in test_cases {
            let result = PlayerRule::default_total(notes);
            assert!(
                (result - expected).abs() < f64::EPSILON,
                "default_total({notes}) = {result}, expected {expected}"
            );
        }
    }
}
