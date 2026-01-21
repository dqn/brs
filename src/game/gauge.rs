use super::JudgeResult;

// ============================================================================
// Gauge Constants
// ============================================================================

/// HP threshold for beatoraja damage reduction (gradual reduction starts here)
const BEATORAJA_DAMAGE_REDUCTION_THRESHOLD: f32 = 50.0;

/// Minimum damage multiplier when HP is very low (beatoraja)
const BEATORAJA_MIN_DAMAGE_MULTIPLIER: f32 = 0.1;

/// LR2 low-life damage reduction threshold (Hard gauge)
const LR2_LOW_HP_REDUCTION_THRESHOLD: f32 = 30.0;
/// LR2 low-life damage reduction multiplier (Hard gauge)
const LR2_LOW_HP_REDUCTION_MULTIPLIER: f32 = 0.6;

// ============================================================================
// Gauge Types
// ============================================================================

/// Gauge type determining difficulty and pass conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GaugeType {
    AssistEasy,
    Easy,
    Normal,
    Hard,
    ExHard,
    Hazard,
}

impl GaugeType {
    /// Returns all gauge types in order of difficulty (easiest to hardest)
    pub fn all() -> &'static [GaugeType] {
        &[
            GaugeType::AssistEasy,
            GaugeType::Easy,
            GaugeType::Normal,
            GaugeType::Hard,
            GaugeType::ExHard,
            GaugeType::Hazard,
        ]
    }

    /// Check if this is a survival gauge (fails at 0%)
    pub fn is_survival(&self) -> bool {
        matches!(
            self,
            GaugeType::Hard | GaugeType::ExHard | GaugeType::Hazard
        )
    }
}

/// Gauge system flavor (affects damage calculation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GaugeSystem {
    #[default]
    Beatoraja,
    #[allow(dead_code)]
    Lr2,
}

/// State of a single gauge
#[derive(Debug, Clone, Copy)]
pub struct GaugeState {
    /// Current HP value (0.0 - 100.0)
    pub hp: f32,
    /// Whether this gauge has failed
    pub failed: bool,
}

/// Damage/recovery values for each judgment
#[derive(Debug, Clone, Copy)]
struct GaugeDamage {
    pgreat: f32,
    great: f32,
    good: f32,
    bad: f32,
    poor: f32,
    empty_poor: f32,
}

/// Properties for a gauge type
#[derive(Debug, Clone, Copy)]
struct GaugeProperty {
    initial: f32,
    #[allow(dead_code)]
    border: f32,
    min: f32,
    max: f32,
    damage: GaugeDamage,
}

impl GaugeProperty {
    fn get(gauge_type: GaugeType, system: GaugeSystem) -> Self {
        match system {
            GaugeSystem::Beatoraja => Self::beatoraja(gauge_type),
            GaugeSystem::Lr2 => Self::lr2(gauge_type),
        }
    }

    fn beatoraja(gauge_type: GaugeType) -> Self {
        match gauge_type {
            GaugeType::AssistEasy => Self {
                initial: 20.0,
                border: 60.0,
                min: 2.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 1.0,
                    great: 1.0,
                    good: 0.5,
                    bad: -1.5,
                    poor: -3.0,
                    empty_poor: -0.5,
                },
            },
            GaugeType::Easy => Self {
                initial: 20.0,
                border: 80.0,
                min: 2.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 1.0,
                    great: 1.0,
                    good: 0.5,
                    bad: -1.5,
                    poor: -4.5,
                    empty_poor: -1.0,
                },
            },
            GaugeType::Normal => Self {
                initial: 20.0,
                border: 80.0,
                min: 2.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 1.0,
                    great: 1.0,
                    good: 0.5,
                    bad: -3.0,
                    poor: -6.0,
                    empty_poor: -2.0,
                },
            },
            GaugeType::Hard => Self {
                initial: 100.0,
                border: 0.0,
                min: 0.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 0.15,
                    great: 0.12,
                    good: 0.03,
                    bad: -5.0,
                    poor: -10.0,
                    empty_poor: -5.0,
                },
            },
            GaugeType::ExHard => Self {
                initial: 100.0,
                border: 0.0,
                min: 0.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 0.15,
                    great: 0.06,
                    good: 0.0,
                    bad: -8.0,
                    poor: -16.0,
                    empty_poor: -8.0,
                },
            },
            GaugeType::Hazard => Self {
                initial: 100.0,
                border: 0.0,
                min: 0.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 0.0,
                    great: 0.0,
                    good: 0.0,
                    bad: -100.0,
                    poor: -100.0,
                    empty_poor: 0.0,
                },
            },
        }
    }

    fn lr2(gauge_type: GaugeType) -> Self {
        match gauge_type {
            GaugeType::AssistEasy | GaugeType::Easy => Self {
                initial: 20.0,
                border: 80.0,
                min: 2.0,
                max: 100.0,
                damage: GaugeDamage {
                    // LR2 EASY: (T/n)*1.2, (T/n)*0.6, -3.2/-4.8/-1.6
                    pgreat: 1.2,
                    great: 1.2,
                    good: 0.6,
                    bad: -3.2,
                    poor: -4.8,
                    empty_poor: -1.6,
                },
            },
            GaugeType::Normal => Self {
                initial: 20.0,
                border: 80.0,
                min: 2.0,
                max: 100.0,
                damage: GaugeDamage {
                    // LR2 GROOVE: (T/n)*1.0, (T/n)*0.5, -4/-6/-2
                    pgreat: 1.0,
                    great: 1.0,
                    good: 0.5,
                    bad: -4.0,
                    poor: -6.0,
                    empty_poor: -2.0,
                },
            },
            GaugeType::Hard => Self {
                initial: 100.0,
                border: 0.0,
                min: 0.0,
                max: 100.0,
                damage: GaugeDamage {
                    // LR2 HARD: fixed recovery +0.1/+0.1/+0.05, -6/-10/-2
                    pgreat: 0.1,
                    great: 0.1,
                    good: 0.05,
                    bad: -6.0,
                    poor: -10.0,
                    empty_poor: -2.0,
                },
            },
            GaugeType::ExHard => Self {
                // LR2にはEX-HARDがないため HARD 相当として扱う
                initial: 100.0,
                border: 0.0,
                min: 0.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 0.1,
                    great: 0.1,
                    good: 0.05,
                    bad: -6.0,
                    poor: -10.0,
                    empty_poor: -2.0,
                },
            },
            GaugeType::Hazard => Self {
                // LR2 DEATH相当: BAD/POORで即失敗、空POORは無効
                initial: 100.0,
                border: 0.0,
                min: 0.0,
                max: 100.0,
                damage: GaugeDamage {
                    pgreat: 0.0,
                    great: 0.0,
                    good: 0.0,
                    bad: -100.0,
                    poor: -100.0,
                    empty_poor: 0.0,
                },
            },
        }
    }
}

/// Manages gauge state during gameplay
pub struct GaugeManager {
    system: GaugeSystem,
    /// All gauge states (index corresponds to GaugeType::all())
    states: Vec<GaugeState>,
    /// Currently displayed gauge type
    active_gauge: GaugeType,
    /// Whether auto shift (GAS) is enabled
    auto_shift: bool,
    /// Total notes in chart (for LR2 damage multiplier)
    total_notes: usize,
    /// #TOTAL value from BMS (for LR2 recovery scaling)
    total_value: f64,
}

impl GaugeManager {
    /// Create a GaugeManager for testing purposes.
    /// Use `new_with_gas()` for production code.
    #[cfg(test)]
    pub fn new(gauge_type: GaugeType, system: GaugeSystem, total_notes: usize) -> Self {
        Self::new_with_gas(gauge_type, system, total_notes, 160.0, false)
    }

    pub fn new_with_gas(
        gauge_type: GaugeType,
        system: GaugeSystem,
        total_notes: usize,
        total_value: f64,
        auto_shift: bool,
    ) -> Self {
        let states = if auto_shift {
            // Initialize all gauge types for GAS
            GaugeType::all()
                .iter()
                .map(|gt| {
                    let prop = GaugeProperty::get(*gt, system);
                    GaugeState {
                        hp: prop.initial,
                        failed: false,
                    }
                })
                .collect()
        } else {
            // Only initialize the selected gauge
            let prop = GaugeProperty::get(gauge_type, system);
            vec![GaugeState {
                hp: prop.initial,
                failed: false,
            }]
        };

        Self {
            system,
            states,
            active_gauge: gauge_type,
            auto_shift,
            total_notes,
            total_value,
        }
    }

    /// Apply a judgment to all active gauges
    pub fn apply_judgment(&mut self, judgment: JudgeResult) {
        if self.auto_shift {
            for (i, gauge_type) in GaugeType::all().iter().enumerate() {
                self.apply_to_gauge(i, *gauge_type, judgment, false);
            }
            self.update_active_gauge();
        } else {
            let gauge_idx = self.gauge_index(self.active_gauge);
            self.apply_to_gauge(gauge_idx, self.active_gauge, judgment, false);
        }
    }

    /// Apply empty POOR (wrong key press without note)
    #[allow(dead_code)]
    pub fn apply_empty_poor(&mut self) {
        if self.auto_shift {
            for (i, gauge_type) in GaugeType::all().iter().enumerate() {
                self.apply_to_gauge(i, *gauge_type, JudgeResult::Poor, true);
            }
            self.update_active_gauge();
        } else {
            let gauge_idx = self.gauge_index(self.active_gauge);
            self.apply_to_gauge(gauge_idx, self.active_gauge, JudgeResult::Poor, true);
        }
    }

    fn apply_to_gauge(
        &mut self,
        idx: usize,
        gauge_type: GaugeType,
        judgment: JudgeResult,
        is_empty_poor: bool,
    ) {
        if self.states[idx].failed {
            return;
        }

        let current_hp = self.states[idx].hp;
        let prop = GaugeProperty::get(gauge_type, self.system);
        let mut damage = if is_empty_poor {
            prop.damage.empty_poor
        } else {
            match judgment {
                JudgeResult::PGreat => prop.damage.pgreat,
                JudgeResult::Great => prop.damage.great,
                JudgeResult::Good => prop.damage.good,
                JudgeResult::Bad => prop.damage.bad,
                JudgeResult::Poor => prop.damage.poor,
            }
        };

        // Apply system-specific modifiers
        damage = self.apply_modifiers(gauge_type, current_hp, damage);

        let state = &mut self.states[idx];
        state.hp = (current_hp + damage).clamp(prop.min, prop.max);

        // Check for failure on survival gauges
        if gauge_type.is_survival() && state.hp <= 0.0 {
            state.failed = true;
        }
    }

    fn apply_modifiers(&self, gauge_type: GaugeType, current_hp: f32, damage: f32) -> f32 {
        match self.system {
            GaugeSystem::Beatoraja => {
                self.apply_beatoraja_modifiers(gauge_type, current_hp, damage)
            }
            GaugeSystem::Lr2 => self.apply_lr2_modifiers(gauge_type, current_hp, damage),
        }
    }

    fn apply_beatoraja_modifiers(
        &self,
        gauge_type: GaugeType,
        current_hp: f32,
        damage: f32,
    ) -> f32 {
        // beatoraja: Gradual damage reduction starting from threshold
        if damage < 0.0
            && current_hp < BEATORAJA_DAMAGE_REDUCTION_THRESHOLD
            && !gauge_type.is_survival()
        {
            let reduction = current_hp / BEATORAJA_DAMAGE_REDUCTION_THRESHOLD;
            damage * reduction.max(BEATORAJA_MIN_DAMAGE_MULTIPLIER)
        } else {
            damage
        }
    }

    fn apply_lr2_modifiers(&self, gauge_type: GaugeType, current_hp: f32, damage: f32) -> f32 {
        let mut modified = damage;

        // LR2: groove/easyはTOTAL/ノーツ数で回復量をスケール
        if !gauge_type.is_survival() && modified > 0.0 && self.total_notes > 0 {
            modified *= (self.total_value / self.total_notes as f64) as f32;
        }

        // LR2: HARD系はTOTAL/ノーツ数でダメージ増幅
        if matches!(gauge_type, GaugeType::Hard | GaugeType::ExHard) && modified < 0.0 {
            modified *= self.lr2_damage_multiplier();
            if current_hp <= LR2_LOW_HP_REDUCTION_THRESHOLD {
                modified *= LR2_LOW_HP_REDUCTION_MULTIPLIER;
            }
        }

        modified
    }

    /// LR2 HARD/EXHARD向けのダメージ倍率
    /// TOTAL値が低い譜面/ノート数が少ない譜面でダメージが増加する
    fn lr2_damage_multiplier(&self) -> f32 {
        let fix1total = [
            240.0f32, 230.0, 210.0, 200.0, 180.0, 160.0, 150.0, 130.0, 120.0, 0.0,
        ];
        let fix1table = [1.0f32, 1.11, 1.25, 1.5, 1.666, 2.0, 2.5, 3.333, 5.0, 10.0];

        let total = self.total_value as f32;
        let mut i = 0usize;
        while i < fix1total.len() - 1 && total < fix1total[i] {
            i += 1;
        }

        let total_notes = self.total_notes as i32;
        let mut fix2 = 1.0f32;
        let mut note = 1000i32;
        let mut step = 0.002f32;
        while note > total_notes || note > 1 {
            let clamp = total_notes.max(note / 2);
            fix2 += step * (note - clamp) as f32;
            note /= 2;
            step *= 2.0;
        }

        fix1table[i].max(fix2)
    }

    fn update_active_gauge(&mut self) {
        // Find the highest non-failed gauge
        for (i, gauge_type) in GaugeType::all().iter().enumerate().rev() {
            if !self.states[i].failed {
                self.active_gauge = *gauge_type;
                return;
            }
        }
    }

    fn gauge_index(&self, gauge_type: GaugeType) -> usize {
        if self.auto_shift {
            GaugeType::all()
                .iter()
                .position(|gt| *gt == gauge_type)
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Get current HP percentage
    pub fn hp(&self) -> f32 {
        let idx = self.gauge_index(self.active_gauge);
        self.states[idx].hp
    }

    /// Get the currently active gauge type
    pub fn active_gauge(&self) -> GaugeType {
        self.active_gauge
    }

    /// Check if the current gauge is cleared
    pub fn is_cleared(&self) -> bool {
        let idx = self.gauge_index(self.active_gauge);
        let state = &self.states[idx];
        let prop = GaugeProperty::get(self.active_gauge, self.system);

        if self.active_gauge.is_survival() {
            !state.failed
        } else {
            state.hp >= prop.border
        }
    }

    /// Get the best clear lamp achieved (for GAS)
    pub fn best_clear(&self) -> Option<GaugeType> {
        if !self.auto_shift {
            return if self.is_cleared() {
                Some(self.active_gauge)
            } else {
                None
            };
        }

        // Find highest cleared gauge
        for (i, gauge_type) in GaugeType::all().iter().enumerate().rev() {
            let state = &self.states[i];
            let prop = GaugeProperty::get(*gauge_type, self.system);

            let cleared = if gauge_type.is_survival() {
                !state.failed
            } else {
                state.hp >= prop.border
            };

            if cleared {
                return Some(*gauge_type);
            }
        }

        None
    }

    /// Check if the player has failed (all gauges failed)
    pub fn is_failed(&self) -> bool {
        if self.auto_shift {
            self.states.iter().all(|s| s.failed)
        } else {
            self.states[0].failed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_gauge_recovery() {
        let mut gauge = GaugeManager::new(GaugeType::Normal, GaugeSystem::Beatoraja, 1000);

        assert!((gauge.hp() - 20.0).abs() < 0.01);

        gauge.apply_judgment(JudgeResult::PGreat);
        assert!((gauge.hp() - 21.0).abs() < 0.01);
    }

    #[test]
    fn test_hard_gauge_failure() {
        let mut gauge = GaugeManager::new(GaugeType::Hard, GaugeSystem::Beatoraja, 1000);

        assert!((gauge.hp() - 100.0).abs() < 0.01);

        // Apply many POORs to fail
        for _ in 0..10 {
            gauge.apply_judgment(JudgeResult::Poor);
        }

        assert!(gauge.is_failed());
    }

    #[test]
    fn test_gas_shifts_gauge() {
        let mut gauge = GaugeManager::new_with_gas(
            GaugeType::ExHard,
            GaugeSystem::Beatoraja,
            1000,
            160.0,
            true,
        );

        assert_eq!(gauge.active_gauge(), GaugeType::ExHard);

        // Fail EX-HARD
        for _ in 0..7 {
            gauge.apply_judgment(JudgeResult::Poor);
        }

        // Should shift to HARD
        assert_eq!(gauge.active_gauge(), GaugeType::Hard);
    }

    #[test]
    fn test_lr2_normal_gauge_values() {
        let mut gauge =
            GaugeManager::new_with_gas(GaugeType::Normal, GaugeSystem::Lr2, 200, 200.0, false);

        assert!((gauge.hp() - 20.0).abs() < 0.01);

        gauge.apply_judgment(JudgeResult::PGreat);
        assert!((gauge.hp() - 21.0).abs() < 0.01);

        gauge.apply_judgment(JudgeResult::Good);
        assert!((gauge.hp() - 21.5).abs() < 0.01);

        gauge.apply_judgment(JudgeResult::Bad);
        assert!((gauge.hp() - 17.5).abs() < 0.01);
    }

    #[test]
    fn test_lr2_easy_gauge_values() {
        let mut gauge =
            GaugeManager::new_with_gas(GaugeType::Easy, GaugeSystem::Lr2, 200, 200.0, false);

        gauge.apply_judgment(JudgeResult::PGreat);
        assert!((gauge.hp() - 21.2).abs() < 0.01);

        gauge.apply_judgment(JudgeResult::Good);
        assert!((gauge.hp() - 21.8).abs() < 0.01);

        gauge.apply_judgment(JudgeResult::Bad);
        assert!((gauge.hp() - 18.6).abs() < 0.01);
    }

    #[test]
    fn test_lr2_damage_multiplier_from_total() {
        let gauge =
            GaugeManager::new_with_gas(GaugeType::Hard, GaugeSystem::Lr2, 1000, 120.0, false);

        let multiplier = gauge.lr2_damage_multiplier();
        assert!((multiplier - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_lr2_low_hp_damage_reduction() {
        let gauge =
            GaugeManager::new_with_gas(GaugeType::Hard, GaugeSystem::Lr2, 1000, 240.0, false);

        let high_hp = gauge.apply_lr2_modifiers(GaugeType::Hard, 50.0, -6.0);
        assert!((high_hp + 6.0).abs() < 0.01);

        let low_hp = gauge.apply_lr2_modifiers(GaugeType::Hard, 25.0, -6.0);
        assert!((low_hp + 3.6).abs() < 0.01);
    }
}
