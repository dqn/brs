// Target property — score target for play screen comparison.
//
// Ported from Java `TargetProperty.java`.
// Computes a target EX score for real-time score difference display.

/// Score target for play screen comparison.
///
/// Three categories exist in Java:
/// 1. StaticRate — fixed percentage targets (AAA, AA, MAX, etc.)
/// 2. NextRank — the next rank threshold above the player's old score
/// 3. Rival/IR — rival or internet ranking targets (not yet implemented)
///
/// Rival/IR variants fall back to MAX since the rival data accessor is not
/// yet ported.
pub enum TargetProperty {
    /// Fixed score rate target (e.g., AAA = 24/27).
    StaticRate { name: String, rate: f32 },
    /// Next rank above the player's current best score.
    NextRank,
}

impl TargetProperty {
    /// Resolve a target ID string into a TargetProperty.
    ///
    /// Matches the Java `TargetProperty.getTargetProperty()` dispatch:
    /// StaticTargetProperty -> RivalTargetProperty -> IRTargetProperty -> NextRank -> MAX fallback
    pub fn resolve(id: &str) -> Self {
        // Static rate targets
        match id {
            "RATE_A-" => {
                return Self::static_rate("RANK A-", 100.0 * 17.0 / 27.0);
            }
            "RATE_A" => {
                return Self::static_rate("RANK A", 100.0 * 18.0 / 27.0);
            }
            "RATE_A+" => {
                return Self::static_rate("RANK A+", 100.0 * 19.0 / 27.0);
            }
            "RATE_AA-" => {
                return Self::static_rate("RANK AA-", 100.0 * 20.0 / 27.0);
            }
            "RATE_AA" => {
                return Self::static_rate("RANK AA", 100.0 * 21.0 / 27.0);
            }
            "RATE_AA+" => {
                return Self::static_rate("RANK AA+", 100.0 * 22.0 / 27.0);
            }
            "RATE_AAA-" => {
                return Self::static_rate("RANK AAA-", 100.0 * 23.0 / 27.0);
            }
            "RATE_AAA" => {
                return Self::static_rate("RANK AAA", 100.0 * 24.0 / 27.0);
            }
            "RATE_AAA+" => {
                return Self::static_rate("RANK AAA+", 100.0 * 25.0 / 27.0);
            }
            "RATE_MAX-" => {
                return Self::static_rate("RANK MAX-", 100.0 * 26.0 / 27.0);
            }
            "MAX" => {
                return Self::static_rate("MAX", 100.0);
            }
            "RANK_NEXT" => {
                return Self::NextRank;
            }
            _ => {}
        }

        // Custom rate: RATE_<float>
        if let Some(suffix) = id.strip_prefix("RATE_")
            && let Ok(rate) = suffix.parse::<f32>()
            && (0.0..=100.0).contains(&rate)
        {
            return Self::StaticRate {
                name: format!("SCORE RATE {rate}%"),
                rate,
            };
        }

        // Rival/IR targets — not yet implemented, fall back to MAX
        if id.starts_with("RIVAL_") || id.starts_with("IR_") {
            return Self::static_rate("MAX", 100.0);
        }

        // Default fallback: MAX
        Self::static_rate("MAX", 100.0)
    }

    /// Compute the target EX score.
    ///
    /// For `StaticRate`: `ceil(total_notes * 2 * rate / 100)`
    /// For `NextRank`: scans rank thresholds 15/27..26/27 to find the first
    /// one exceeding `current_exscore`, falling back to MAX.
    ///
    /// Returns `(target_exscore, target_name)`.
    pub fn compute_target(&self, total_notes: i32, current_exscore: i32) -> (i32, String) {
        match self {
            Self::StaticRate { name, rate } => {
                let score = (total_notes as f32 * 2.0 * rate / 100.0).ceil() as i32;
                (score, name.clone())
            }
            Self::NextRank => {
                let max = total_notes * 2;
                let mut target_score = max;
                for i in 15..27 {
                    let target = (max as f32 * i as f32 / 27.0).ceil() as i32;
                    if current_exscore < target {
                        target_score = target;
                        break;
                    }
                }
                (target_score, "NEXT RANK".to_string())
            }
        }
    }

    fn static_rate(name: &str, rate: f32) -> Self {
        Self::StaticRate {
            name: name.to_string(),
            rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_known_ids() {
        assert!(matches!(
            TargetProperty::resolve("MAX"),
            TargetProperty::StaticRate { rate, .. } if (rate - 100.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            TargetProperty::resolve("RATE_AAA"),
            TargetProperty::StaticRate { rate, .. } if (rate - 100.0 * 24.0 / 27.0).abs() < 0.01
        ));
        assert!(matches!(
            TargetProperty::resolve("RANK_NEXT"),
            TargetProperty::NextRank
        ));
    }

    #[test]
    fn resolve_custom_rate() {
        let tp = TargetProperty::resolve("RATE_55.5");
        assert!(matches!(
            tp,
            TargetProperty::StaticRate { rate, .. } if (rate - 55.5).abs() < 0.01
        ));
    }

    #[test]
    fn resolve_out_of_range_rate_falls_back_to_max() {
        let tp = TargetProperty::resolve("RATE_150.0");
        assert!(matches!(
            tp,
            TargetProperty::StaticRate { rate, .. } if (rate - 100.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn resolve_rival_ir_falls_back() {
        // Rival/IR not implemented, should fall back to MAX
        let tp = TargetProperty::resolve("RIVAL_RANK_1");
        assert!(matches!(
            tp,
            TargetProperty::StaticRate { rate, .. } if (rate - 100.0).abs() < f32::EPSILON
        ));
        let tp = TargetProperty::resolve("IR_NEXT_3");
        assert!(matches!(
            tp,
            TargetProperty::StaticRate { rate, .. } if (rate - 100.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn resolve_unknown_falls_back_to_max() {
        let tp = TargetProperty::resolve("SOMETHING_UNKNOWN");
        assert!(matches!(
            tp,
            TargetProperty::StaticRate { rate, .. } if (rate - 100.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn compute_static_max() {
        let tp = TargetProperty::resolve("MAX");
        let (score, name) = tp.compute_target(500, 0);
        // 500 * 2 * 100 / 100 = 1000
        assert_eq!(score, 1000);
        assert_eq!(name, "MAX");
    }

    #[test]
    fn compute_static_aaa() {
        let tp = TargetProperty::resolve("RATE_AAA");
        let (score, _) = tp.compute_target(500, 0);
        // Java: ceil(500 * 2 * (100*24/27) / 100) = ceil(1000 * 24/27) = ceil(888.888...) = 889
        let expected = (500.0f64 * 2.0 * 100.0 * 24.0 / 27.0 / 100.0).ceil() as i32;
        assert_eq!(score, expected);
    }

    #[test]
    fn compute_next_rank_below_b() {
        // total 500 notes, max = 1000
        // current exscore = 400 (40%)
        // Rank thresholds: 15/27 = 555.5 -> ceil = 556
        // 400 < 556, so target = 556
        let tp = TargetProperty::resolve("RANK_NEXT");
        let (score, name) = tp.compute_target(500, 400);
        let expected = (1000.0f32 * 15.0 / 27.0).ceil() as i32;
        assert_eq!(score, expected);
        assert_eq!(name, "NEXT RANK");
    }

    #[test]
    fn compute_next_rank_above_all() {
        // current exscore = 999, exceeds all thresholds -> falls back to max (1000)
        let tp = TargetProperty::resolve("RANK_NEXT");
        let (score, _) = tp.compute_target(500, 999);
        assert_eq!(score, 1000);
    }

    #[test]
    fn compute_next_rank_at_threshold() {
        // current exscore exactly equals the B threshold (15/27 * 1000)
        // Java: ceil(1000 * 15 / 27) = 556
        // 556 < 556 is false, so move to next: 16/27 * 1000 = ceil(592.59) = 593
        let tp = TargetProperty::resolve("RANK_NEXT");
        let b_threshold = (1000.0f32 * 15.0 / 27.0).ceil() as i32;
        let (score, _) = tp.compute_target(500, b_threshold);
        let expected = (1000.0f32 * 16.0 / 27.0).ceil() as i32;
        assert_eq!(score, expected);
    }
}
