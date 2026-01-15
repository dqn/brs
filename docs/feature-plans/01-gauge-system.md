# Gauge System Implementation

## Overview

The gauge (HP/life bar) system determines whether a player passes or fails a song. Different gauge types provide varying difficulty levels, from beginner-friendly to extremely punishing.

## Gauge Types

### beatoraja Gauges

| Gauge | Start | Pass | Min | Description |
|-------|------:|-----:|----:|-------------|
| ASSIST EASY | 20% | 60% | 2% | Most lenient, for beginners |
| EASY | 20% | 80% | 2% | Reduced damage |
| NORMAL (GROOVE) | 20% | 80% | 2% | Standard gauge |
| HARD | 100% | >0% | 0% | Survival, fail at 0% |
| EX-HARD | 100% | >0% | 0% | Double HARD damage |
| HAZARD | 100% | >0% | 0% | Instant fail on combo break |

### Damage/Recovery Table (beatoraja 7KEY)

Values represent percentage change per judgment.

| Gauge | PGREAT | GREAT | GOOD | BAD | POOR | Empty POOR |
|-------|-------:|------:|-----:|----:|-----:|-----------:|
| ASSIST EASY | +1.0 | +1.0 | +0.5 | -1.5 | -3.0 | -0.5 |
| EASY | +1.0 | +1.0 | +0.5 | -1.5 | -4.5 | -1.0 |
| NORMAL | +1.0 | +1.0 | +0.5 | -3.0 | -6.0 | -2.0 |
| HARD | +0.15 | +0.12 | +0.03 | -5.0 | -10.0 | -5.0 |
| EX-HARD | +0.15 | +0.06 | 0.0 | -8.0 | -16.0 | -8.0 |

## LR2 vs beatoraja Differences

### Damage Reduction at Low HP

**beatoraja:**
- Gradual reduction starting from 50% gauge
- Smooth curve to prevent sudden deaths

**LR2:**
- Damage halved when gauge drops below 32%
- Binary threshold (full damage above, half below)
- Fail when gauge drops below 2%

### TOTAL-based Scaling

**beatoraja:**
- When TOTAL < 250: Recovery rate decreases
- Charts under 100 notes: No gauge increase at all

**LR2 HARD Gauge Damage Multiplier:**
```
Notes 1000+: 1.0x (default)
Notes 999-500: +0.002x per note below 1000
Notes 499-250: +0.004x per note below 500
Notes 249-125: +0.008x per note below 250
Notes 124-60: +(1/65)x per note below 125
Notes 59-30: +(1/15)x per note below 60
Notes 29-20: +0.2x per note below 30
Notes 1-20: BAD = -60%, Empty POOR = -20% (fixed)

Results:
- 1000 notes: 1.0x damage
- 500 notes: 2.0x damage
- 250 notes: 3.0x damage
- 125 notes: 4.0x damage
- 60 notes: 5.0x damage
- 30 notes: 8.0x damage
- 20 notes: 10.0x damage
```

### TOTAL Formula

**beatoraja:**
Recovery per note = `TOTAL / note_count` (clamped)

**LR2oraja:**
```
TOTAL = 160 + (N + clamp(N - 400, 0, 200)) * 0.16
where N = total_notes
```

## Gauge Auto Shift (GAS)

### Overview

GAS calculates all gauge types simultaneously, awarding the highest clear the player achieves in a single play.

### Behavior

1. Start with EX-HARD gauge display
2. If EX-HARD fails (HP = 0) → Switch display to HARD
3. If HARD fails → Switch to NORMAL/EASY/ASSIST EASY
4. Record the highest clear achieved

### Configuration Options

- **Bottom Shift Gauge**: Minimum gauge to shift to (prevents flooding results with ASSIST clears)
- Example: Set to EASY to never display ASSIST EASY

### Visual Behavior

- Only the currently active gauge is displayed
- On result screen, player can view gauge graph for all types

## Implementation

### Rust Data Structures

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GaugeType {
    AssistEasy,
    Easy,
    Normal,
    Hard,
    ExHard,
    Hazard,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GaugeSystem {
    Beatoraja,
    LR2,
}

pub struct GaugeConfig {
    pub gauge_type: GaugeType,
    pub system: GaugeSystem,
    pub auto_shift: bool,
    pub bottom_shift_gauge: GaugeType,
}

pub struct GaugeState {
    /// Current HP value (0.0 - 100.0)
    pub hp: f32,
    /// Whether this gauge has failed
    pub failed: bool,
}

pub struct GaugeProperty {
    pub initial: f32,
    pub border: f32,    // Pass threshold
    pub min: f32,
    pub max: f32,
    pub damage: GaugeDamage,
}

pub struct GaugeDamage {
    pub pgreat: f32,
    pub great: f32,
    pub good: f32,
    pub bad: f32,
    pub poor: f32,
    pub empty_poor: f32,
}
```

### Gauge Manager

```rust
pub struct GaugeManager {
    config: GaugeConfig,
    /// All gauge states (for GAS)
    states: HashMap<GaugeType, GaugeState>,
    /// Currently displayed gauge
    active_gauge: GaugeType,
    /// Chart metadata
    total_notes: usize,
    total_value: f32,
}

impl GaugeManager {
    pub fn new(config: GaugeConfig, total_notes: usize, total_value: f32) -> Self {
        let mut states = HashMap::new();

        if config.auto_shift {
            // Initialize all gauge types for GAS
            for gauge_type in GaugeType::all() {
                let prop = Self::get_property(gauge_type, config.system);
                states.insert(gauge_type, GaugeState {
                    hp: prop.initial,
                    failed: false,
                });
            }
        } else {
            let prop = Self::get_property(config.gauge_type, config.system);
            states.insert(config.gauge_type, GaugeState {
                hp: prop.initial,
                failed: false,
            });
        }

        Self {
            config,
            states,
            active_gauge: config.gauge_type,
            total_notes,
            total_value,
        }
    }

    pub fn apply_judgment(&mut self, judgment: Judgment) {
        for (gauge_type, state) in &mut self.states {
            if state.failed {
                continue;
            }

            let prop = Self::get_property(*gauge_type, self.config.system);
            let damage = self.calculate_damage(*gauge_type, judgment);

            state.hp = (state.hp + damage).clamp(prop.min, prop.max);

            // Check for failure
            if Self::is_survival_gauge(*gauge_type) && state.hp <= 0.0 {
                state.failed = true;
            }
        }

        // Update active gauge for GAS
        if self.config.auto_shift {
            self.update_active_gauge();
        }
    }

    fn calculate_damage(&self, gauge_type: GaugeType, judgment: Judgment) -> f32 {
        let base = Self::get_base_damage(gauge_type, judgment, self.config.system);

        match self.config.system {
            GaugeSystem::Beatoraja => {
                self.apply_beatoraja_modifiers(gauge_type, base, judgment)
            }
            GaugeSystem::LR2 => {
                self.apply_lr2_modifiers(gauge_type, base, judgment)
            }
        }
    }

    fn apply_lr2_modifiers(&self, gauge_type: GaugeType, base: f32, judgment: Judgment) -> f32 {
        let mut damage = base;

        // LR2 damage multiplier based on note count
        if Self::is_survival_gauge(gauge_type) && judgment.is_miss() {
            let multiplier = self.calculate_lr2_damage_multiplier();
            damage *= multiplier;
        }

        // LR2 damage reduction below 32%
        let state = &self.states[&gauge_type];
        if state.hp < 32.0 && damage < 0.0 {
            damage *= 0.5;
        }

        damage
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
            10.0  // Fixed values for 1-20 notes
        }
    }

    fn is_survival_gauge(gauge_type: GaugeType) -> bool {
        matches!(gauge_type, GaugeType::Hard | GaugeType::ExHard | GaugeType::Hazard)
    }

    pub fn is_cleared(&self) -> bool {
        let state = &self.states[&self.active_gauge];
        let prop = Self::get_property(self.active_gauge, self.config.system);

        if Self::is_survival_gauge(self.active_gauge) {
            !state.failed
        } else {
            state.hp >= prop.border
        }
    }

    pub fn get_best_clear(&self) -> Option<GaugeType> {
        // Return highest non-failed gauge type
        for gauge_type in GaugeType::all_by_difficulty().iter().rev() {
            if let Some(state) = self.states.get(gauge_type) {
                if !state.failed && self.check_clear(*gauge_type, state) {
                    return Some(*gauge_type);
                }
            }
        }
        None
    }
}
```

## Clear Lamps

The gauge type affects the clear lamp recorded:

| Lamp | Condition |
|------|-----------|
| FAILED | No gauge cleared |
| ASSIST EASY | ASSIST EASY cleared |
| EASY | EASY cleared |
| NORMAL | NORMAL cleared |
| HARD | HARD cleared |
| EX-HARD | EX-HARD cleared |
| FULL COMBO | Any gauge + no BAD/POOR |
| PERFECT | Any gauge + no GOOD/BAD/POOR |
| MAX | 100% PGREAT |

## Reference Links

- [beatoraja GaugeProperty.java](https://github.com/exch-bms2/beatoraja/blob/master/src/bms/player/beatoraja/play/GaugeProperty.java)
- [beatoraja GrooveGauge.java](https://github.com/exch-bms2/beatoraja/blob/master/src/bms/player/beatoraja/play/GrooveGauge.java)
- [LR2oraja Gauge Implementation](https://github.com/wcko87/lr2oraja)
- [LR2GAS for Lunatic Rave 2](https://github.com/MatVeiQaaa/LR2GAS)
- [IIDX/LR2/beatoraja Differences](https://iidx.org/misc/iidx_lr2_beatoraja_diff)
