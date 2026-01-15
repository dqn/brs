use super::JudgeResult;

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
        // LR2 uses similar values but with different modifiers applied elsewhere
        Self::beatoraja(gauge_type)
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
}

impl GaugeManager {
    #[cfg(test)]
    pub fn new(gauge_type: GaugeType, system: GaugeSystem, total_notes: usize) -> Self {
        Self::new_with_gas(gauge_type, system, total_notes, false)
    }

    pub fn new_with_gas(
        gauge_type: GaugeType,
        system: GaugeSystem,
        total_notes: usize,
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
        damage = self.apply_modifiers(gauge_type, current_hp, damage, judgment);

        let state = &mut self.states[idx];
        state.hp = (current_hp + damage).clamp(prop.min, prop.max);

        // Check for failure on survival gauges
        if gauge_type.is_survival() && state.hp <= 0.0 {
            state.failed = true;
        }
    }

    fn apply_modifiers(
        &self,
        gauge_type: GaugeType,
        current_hp: f32,
        damage: f32,
        judgment: JudgeResult,
    ) -> f32 {
        match self.system {
            GaugeSystem::Beatoraja => {
                self.apply_beatoraja_modifiers(gauge_type, current_hp, damage)
            }
            GaugeSystem::Lr2 => self.apply_lr2_modifiers(gauge_type, current_hp, damage, judgment),
        }
    }

    fn apply_beatoraja_modifiers(
        &self,
        gauge_type: GaugeType,
        current_hp: f32,
        damage: f32,
    ) -> f32 {
        // beatoraja: Gradual damage reduction starting from 50%
        if damage < 0.0 && current_hp < 50.0 && !gauge_type.is_survival() {
            let reduction = current_hp / 50.0;
            damage * reduction.max(0.1)
        } else {
            damage
        }
    }

    fn apply_lr2_modifiers(
        &self,
        gauge_type: GaugeType,
        current_hp: f32,
        damage: f32,
        judgment: JudgeResult,
    ) -> f32 {
        let mut modified = damage;

        // LR2: Damage multiplier based on note count for survival gauges
        if gauge_type.is_survival()
            && damage < 0.0
            && matches!(judgment, JudgeResult::Bad | JudgeResult::Poor)
        {
            let multiplier = self.calculate_lr2_damage_multiplier();
            modified *= multiplier;
        }

        // LR2: Binary damage reduction below 32%
        if current_hp < 32.0 && modified < 0.0 {
            modified *= 0.5;
        }

        modified
    }

    fn calculate_lr2_damage_multiplier(&self) -> f32 {
        let n = self.total_notes as f32;

        if n >= 1000.0 {
            1.0
        } else if n >= 500.0 {
            1.0 + (1000.0 - n) * 0.002
        } else if n >= 250.0 {
            2.0 + (500.0 - n) * 0.004
        } else if n >= 125.0 {
            3.0 + (250.0 - n) * 0.008
        } else if n >= 60.0 {
            4.0 + (125.0 - n) / 65.0
        } else if n >= 30.0 {
            5.0 + (60.0 - n) / 15.0
        } else if n >= 20.0 {
            8.0 + (30.0 - n) * 0.2
        } else {
            10.0
        }
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
        let mut gauge =
            GaugeManager::new_with_gas(GaugeType::ExHard, GaugeSystem::Beatoraja, 1000, true);

        assert_eq!(gauge.active_gauge(), GaugeType::ExHard);

        // Fail EX-HARD
        for _ in 0..7 {
            gauge.apply_judgment(JudgeResult::Poor);
        }

        // Should shift to HARD
        assert_eq!(gauge.active_gauge(), GaugeType::Hard);
    }
}
