# Judge Timing Implementation

## Overview

The timing window system determines how accurately a player must hit notes. Different systems (beatoraja, LR2, IIDX) have slightly different timing windows that affect gameplay difficulty.

## Timing Windows Comparison

### 7-Key EASY Judge (default for most BMS)

| Judgment | beatoraja | LR2 | IIDX |
|----------|----------:|----:|-----:|
| PGREAT | ±20ms | ±21ms | ±16.67ms |
| GREAT | ±60ms | ±60ms | ±33.33ms |
| GOOD | ±150ms | ±120ms | - |
| BAD | ±220-280ms | ±200ms | - |

### beatoraja Full Timing Table (7KEY)

All values in milliseconds.

| Judgment | VERY EASY | EASY | NORMAL | HARD | VERY HARD |
|----------|----------:|-----:|-------:|-----:|----------:|
| PGREAT | ±25 | ±20 | ±15 | ±10 | ±5 |
| GREAT | ±75 | ±60 | ±45 | ±30 | ±15 |
| GOOD | ±188 | ±150 | ±113 | ±75 | ±38 |
| BAD (early) | -275 | -220 | -165 | -110 | -55 |
| BAD (late) | +350 | +280 | +210 | +140 | +70 |

**Scaling formula:** EASY is baseline (1.0x), other ranks scale proportionally:
- VERY EASY: 1.25x
- NORMAL: 0.75x
- HARD: 0.50x
- VERY HARD: 0.25x

### LR2 Timing Windows (PGREAT only)

| Judge Rank | PGREAT Window |
|------------|---------------|
| VERY EASY | ±21ms (same as EASY) |
| EASY | ±21ms |
| NORMAL | ±18ms |
| HARD | ±15ms |
| VERY HARD | ±8ms |

**Note:** LR2 uses fixed windows per rank, not scaling.

### Scratch Notes

beatoraja adds ±10ms to all scratch timing windows.

### Long Note Release Windows

beatoraja uses significantly wider windows for LN release:
- ±120ms (PGREAT equivalent)
- ±160ms (GREAT equivalent)
- ±200ms (GOOD equivalent)
- -220ms/+280ms (BAD equivalent)

## Empty POOR (空POOR)

An Empty POOR occurs when the player presses a button with no note nearby.

### beatoraja

- Can occur both before AND after a note
- Counts as a miss in damage calculation
- Uses the POOR window timing

### LR2

- **Can only occur BEFORE a note** (not after)
- This makes LR2 slightly more lenient for survival gauges
- After hitting a note, pressing the button again within timing won't give Empty POOR

### Implementation Difference

```rust
// beatoraja style: check both directions
fn is_empty_poor_beatoraja(press_time: i64, note_time: i64, window: i64) -> bool {
    (press_time - note_time).abs() > window
}

// LR2 style: only check early presses
fn is_empty_poor_lr2(press_time: i64, note_time: i64, window: i64) -> bool {
    press_time < note_time - window
}
```

## BMS #RANK Command

The chart's `#RANK` header specifies the default judge difficulty:

| Value | Judge Rank |
|------:|------------|
| 0 | VERY HARD |
| 1 | HARD |
| 2 | NORMAL |
| 3 | EASY (default) |

## Implementation

### Rust Data Structures

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JudgeSystem {
    Beatoraja,
    LR2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JudgeRank {
    VeryEasy,
    Easy,
    Normal,
    Hard,
    VeryHard,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Judgment {
    PGreat,
    Great,
    Good,
    Bad,
    Poor,       // Hitting wrong timing
    EmptyPoor,  // Pressing with no note
}

/// Timing window for a single judgment
#[derive(Clone, Copy)]
pub struct TimingWindow {
    /// Early timing threshold (negative value)
    pub early: i64,
    /// Late timing threshold (positive value)
    pub late: i64,
}

impl TimingWindow {
    pub fn symmetric(ms: i64) -> Self {
        Self { early: -ms, late: ms }
    }

    pub fn asymmetric(early: i64, late: i64) -> Self {
        Self { early: -early, late }
    }

    pub fn contains(&self, timing_diff: i64) -> bool {
        timing_diff >= self.early && timing_diff <= self.late
    }
}

/// Complete timing configuration for a note type
pub struct JudgeProperty {
    pub pgreat: TimingWindow,
    pub great: TimingWindow,
    pub good: TimingWindow,
    pub bad: TimingWindow,
    pub poor: TimingWindow,
}
```

### Judge Property Factory

```rust
impl JudgeProperty {
    pub fn beatoraja_7key(rank: JudgeRank) -> Self {
        let scale = match rank {
            JudgeRank::VeryEasy => 1.25,
            JudgeRank::Easy => 1.0,
            JudgeRank::Normal => 0.75,
            JudgeRank::Hard => 0.50,
            JudgeRank::VeryHard => 0.25,
        };

        Self {
            pgreat: TimingWindow::symmetric((20.0 * scale) as i64),
            great: TimingWindow::symmetric((60.0 * scale) as i64),
            good: TimingWindow::symmetric((150.0 * scale) as i64),
            bad: TimingWindow::asymmetric(
                (220.0 * scale) as i64,
                (280.0 * scale) as i64,
            ),
            poor: TimingWindow::asymmetric(
                (150.0 * scale) as i64,
                (500.0 * scale) as i64,
            ),
        }
    }

    pub fn beatoraja_scratch(rank: JudgeRank) -> Self {
        let mut prop = Self::beatoraja_7key(rank);
        // Add ±10ms to all scratch windows
        prop.pgreat.early -= 10;
        prop.pgreat.late += 10;
        prop.great.early -= 10;
        prop.great.late += 10;
        prop.good.early -= 10;
        prop.good.late += 10;
        prop.bad.early -= 10;
        prop.bad.late += 10;
        prop
    }

    pub fn lr2(rank: JudgeRank) -> Self {
        // LR2 uses fixed windows per rank
        let pgreat_ms = match rank {
            JudgeRank::VeryEasy | JudgeRank::Easy => 21,
            JudgeRank::Normal => 18,
            JudgeRank::Hard => 15,
            JudgeRank::VeryHard => 8,
        };

        // Other windows scale similarly
        let great_ms = pgreat_ms * 3;    // ~60ms for EASY
        let good_ms = pgreat_ms * 6;     // ~120ms for EASY
        let bad_ms = pgreat_ms * 10;     // ~200ms for EASY

        Self {
            pgreat: TimingWindow::symmetric(pgreat_ms),
            great: TimingWindow::symmetric(great_ms),
            good: TimingWindow::symmetric(good_ms),
            bad: TimingWindow::symmetric(bad_ms),
            poor: TimingWindow::symmetric(bad_ms + 50),
        }
    }
}
```

### Judgment Evaluation

```rust
pub struct JudgeManager {
    system: JudgeSystem,
    property: JudgeProperty,
    scratch_property: JudgeProperty,
}

impl JudgeManager {
    pub fn judge_note(
        &self,
        timing_diff_ms: i64,
        is_scratch: bool,
    ) -> Judgment {
        let prop = if is_scratch {
            &self.scratch_property
        } else {
            &self.property
        };

        if prop.pgreat.contains(timing_diff_ms) {
            Judgment::PGreat
        } else if prop.great.contains(timing_diff_ms) {
            Judgment::Great
        } else if prop.good.contains(timing_diff_ms) {
            Judgment::Good
        } else if prop.bad.contains(timing_diff_ms) {
            Judgment::Bad
        } else if prop.poor.contains(timing_diff_ms) {
            Judgment::Poor
        } else {
            Judgment::EmptyPoor
        }
    }

    /// Check if a button press should trigger empty POOR
    pub fn is_empty_poor(
        &self,
        press_time_ms: i64,
        nearest_note_time_ms: i64,
    ) -> bool {
        let diff = press_time_ms - nearest_note_time_ms;

        match self.system {
            JudgeSystem::Beatoraja => {
                // beatoraja: both early and late
                diff.abs() > self.property.poor.late
            }
            JudgeSystem::LR2 => {
                // LR2: only early presses count as empty POOR
                diff < self.property.poor.early
            }
        }
    }
}
```

### Timing Direction (FAST/SLOW)

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimingDirection {
    Fast,   // Pressed too early
    Exact,  // Perfect timing (PGREAT)
    Slow,   // Pressed too late
}

impl JudgeManager {
    pub fn get_timing_direction(&self, timing_diff_ms: i64) -> TimingDirection {
        if timing_diff_ms < -1 {
            TimingDirection::Fast
        } else if timing_diff_ms > 1 {
            TimingDirection::Slow
        } else {
            TimingDirection::Exact
        }
    }
}
```

## Scoring Impact

### EX Score

beatoraja's slightly narrower PGREAT window (±20ms vs LR2's ±21ms) means:
- EX score is marginally harder to maximize in beatoraja
- GREAT windows are identical, so combo difficulty is similar

### Comparison Summary

| Aspect | beatoraja | LR2 |
|--------|-----------|-----|
| PGREAT difficulty | Slightly harder | Slightly easier |
| Empty POOR | Both directions | Early only |
| Survival gauges | Slightly easier (late E.POOR safe) | Slightly harder |
| Window scaling | Linear multiplication | Fixed per rank |

## Reference Links

- [beatoraja JudgeProperty.java](https://github.com/exch-bms2/beatoraja/blob/master/src/bms/player/beatoraja/play/JudgeProperty.java)
- [IIDX/LR2/beatoraja Differences](https://iidx.org/misc/iidx_lr2_beatoraja_diff)
- [Gauge Calculation and Timing Windows](https://iidx.org/compendium/gauges_and_timing)
- [LR2oraja Judge Implementation](https://github.com/wcko87/lr2oraja)
