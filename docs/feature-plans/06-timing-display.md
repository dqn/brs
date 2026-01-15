# Timing Display Implementation

## Overview

Timing feedback helps players understand and improve their accuracy. This includes FAST/SLOW indicators, millisecond offset display, and visual timing guides.

## FAST/SLOW Display

### Behavior

- Shown for every judgment except PGREAT
- FAST = pressed too early (negative timing difference)
- SLOW = pressed too late (positive timing difference)

### Timing Threshold

```rust
const EXACT_THRESHOLD_MS: i64 = 1;  // Within ±1ms is considered exact

pub enum TimingDirection {
    Fast,
    Exact,
    Slow,
}

pub fn get_timing_direction(timing_diff_ms: i64) -> TimingDirection {
    if timing_diff_ms < -EXACT_THRESHOLD_MS {
        TimingDirection::Fast
    } else if timing_diff_ms > EXACT_THRESHOLD_MS {
        TimingDirection::Slow
    } else {
        TimingDirection::Exact
    }
}
```

### Display Options

| Option | Description |
|--------|-------------|
| OFF | No FAST/SLOW display |
| ON | Show FAST or SLOW text |
| DETAIL | Show ±milliseconds value |

## Millisecond Display

### Format

```
PGREAT     (no text, or just the judgment)
GREAT -8   (8ms early)
GOOD +15   (15ms late)
BAD -42    (42ms early)
```

### Implementation

```rust
pub struct TimingDisplay {
    pub enabled: bool,
    pub show_ms: bool,
    pub current_timing: Option<TimingInfo>,
    pub display_duration_ms: u32,
    pub timer: f32,
}

pub struct TimingInfo {
    pub judgment: Judgment,
    pub direction: TimingDirection,
    pub offset_ms: i64,
}

impl TimingDisplay {
    pub fn new() -> Self {
        Self {
            enabled: true,
            show_ms: true,
            current_timing: None,
            display_duration_ms: 500,
            timer: 0.0,
        }
    }

    pub fn show(&mut self, judgment: Judgment, timing_diff_ms: i64) {
        if !self.enabled {
            return;
        }

        // Don't show timing for PGREAT (it's perfect)
        if judgment == Judgment::PGreat {
            self.current_timing = None;
            return;
        }

        self.current_timing = Some(TimingInfo {
            judgment,
            direction: get_timing_direction(timing_diff_ms),
            offset_ms: timing_diff_ms,
        });
        self.timer = self.display_duration_ms as f32;
    }

    pub fn update(&mut self, delta_ms: f32) {
        if self.timer > 0.0 {
            self.timer -= delta_ms;
            if self.timer <= 0.0 {
                self.current_timing = None;
            }
        }
    }

    pub fn get_display_text(&self) -> Option<String> {
        self.current_timing.as_ref().map(|info| {
            if self.show_ms {
                let sign = if info.offset_ms >= 0 { "+" } else { "" };
                format!("{} {}{}",
                    match info.direction {
                        TimingDirection::Fast => "FAST",
                        TimingDirection::Exact => "",
                        TimingDirection::Slow => "SLOW",
                    },
                    sign,
                    info.offset_ms
                )
            } else {
                match info.direction {
                    TimingDirection::Fast => "FAST".to_string(),
                    TimingDirection::Exact => String::new(),
                    TimingDirection::Slow => "SLOW".to_string(),
                }
            }
        })
    }
}
```

## FAST/SLOW Counter

Track cumulative FAST/SLOW counts during a song.

```rust
pub struct TimingStats {
    pub fast_count: u32,
    pub slow_count: u32,
    pub exact_count: u32,  // PGREAT
    pub total_notes: u32,
}

impl TimingStats {
    pub fn new() -> Self {
        Self {
            fast_count: 0,
            slow_count: 0,
            exact_count: 0,
            total_notes: 0,
        }
    }

    pub fn record(&mut self, judgment: Judgment, timing_diff_ms: i64) {
        self.total_notes += 1;

        if judgment == Judgment::PGreat {
            self.exact_count += 1;
        } else if timing_diff_ms < 0 {
            self.fast_count += 1;
        } else {
            self.slow_count += 1;
        }
    }

    /// Get ratio of FAST:SLOW (ideal is close to 1.0)
    pub fn fast_slow_ratio(&self) -> f32 {
        if self.slow_count == 0 {
            f32::INFINITY
        } else {
            self.fast_count as f32 / self.slow_count as f32
        }
    }

    pub fn get_display(&self) -> String {
        format!("FAST:{} / SLOW:{}", self.fast_count, self.slow_count)
    }
}
```

## Green Number Display

### What is Green Number?

The time (in milliseconds or frames) that a note is visible before reaching the judge line.

### beatoraja Green Number

Uses milliseconds directly:
- 573 = Note visible for 573ms

### IIDX Green Number

Uses frames at 60fps:
- 300 = 300 frames = 5000ms

### Calculation

```rust
pub fn calculate_green_number(
    lane_height: f32,
    visible_ratio: f32,  // After SUDDEN+/LIFT
    hi_speed: f32,
    base_bpm: f32,
) -> f32 {
    // Base scroll time for one lane height at 150 BPM, 1.0x speed
    let base_time_ms = 60000.0 / 150.0 * 4.0;  // 4 beats visible

    // Adjust for actual BPM and speed
    let scroll_time = base_time_ms * 150.0 / base_bpm / hi_speed;

    // Apply visibility ratio
    scroll_time * visible_ratio
}
```

### Display

```rust
pub struct GreenNumberDisplay {
    pub current_green: f32,
    pub target_green: f32,
    pub show_both: bool,  // Show current and target
}

impl GreenNumberDisplay {
    pub fn get_text(&self) -> String {
        if self.show_both {
            format!("GREEN: {:.0} (→{:.0})", self.current_green, self.target_green)
        } else {
            format!("GREEN: {:.0}", self.current_green)
        }
    }
}
```

## Input Offset Adjustment

### Purpose

Compensate for input/display latency.

### Settings

```rust
pub struct OffsetSettings {
    /// Judge offset in milliseconds (negative = hit earlier, positive = hit later)
    pub judge_offset_ms: i32,
    /// Visual offset in milliseconds (negative = notes appear earlier)
    pub visual_offset_ms: i32,
}

impl Default for OffsetSettings {
    fn default() -> Self {
        Self {
            judge_offset_ms: 0,
            visual_offset_ms: 0,
        }
    }
}
```

### Auto Adjust

Automatically adjust offset based on FAST/SLOW ratio:

```rust
pub struct AutoAdjust {
    enabled: bool,
    adjustment_rate: f32,  // ms per adjustment
    samples: Vec<i64>,     // Recent timing differences
    sample_count: usize,
}

impl AutoAdjust {
    pub fn new() -> Self {
        Self {
            enabled: false,
            adjustment_rate: 1.0,
            samples: Vec::with_capacity(50),
            sample_count: 50,
        }
    }

    pub fn record_timing(&mut self, timing_diff_ms: i64) {
        if !self.enabled {
            return;
        }

        self.samples.push(timing_diff_ms);
        if self.samples.len() > self.sample_count {
            self.samples.remove(0);
        }
    }

    pub fn calculate_adjustment(&self) -> i32 {
        if self.samples.len() < 10 {
            return 0;
        }

        let avg: f64 = self.samples.iter().map(|&x| x as f64).sum::<f64>()
            / self.samples.len() as f64;

        // Only adjust if significantly off-center
        if avg.abs() > 5.0 {
            (avg * self.adjustment_rate as f64) as i32
        } else {
            0
        }
    }
}
```

## Visual Feedback Colors

```rust
pub struct TimingColors {
    pub pgreat: Color,
    pub great: Color,
    pub good: Color,
    pub bad: Color,
    pub poor: Color,
    pub fast: Color,
    pub slow: Color,
}

impl Default for TimingColors {
    fn default() -> Self {
        Self {
            pgreat: Color::new(1.0, 1.0, 0.0, 1.0),    // Yellow
            great: Color::new(1.0, 0.8, 0.0, 1.0),    // Orange
            good: Color::new(0.0, 1.0, 0.0, 1.0),     // Green
            bad: Color::new(0.0, 0.5, 1.0, 1.0),      // Blue
            poor: Color::new(1.0, 0.0, 0.0, 1.0),     // Red
            fast: Color::new(0.0, 0.8, 1.0, 1.0),     // Cyan
            slow: Color::new(1.0, 0.5, 0.0, 1.0),     // Orange
        }
    }
}
```

## Calibration Guide

For setting up offsets:

1. **FAST:SLOW ratio should be ~1:1**
   - Too many FASTs → Decrease green number (faster scroll)
   - Too many SLOWs → Increase green number (slower scroll)

2. **Judge offset**
   - If consistently hitting early → Increase judge offset (positive)
   - If consistently hitting late → Decrease judge offset (negative)

3. **Audio latency**
   - Bluetooth headphones: +50ms to +150ms
   - HDMI audio: +20ms to +50ms
   - Dedicated audio: ~0ms

## Reference Links

- [Timing Guide (Intermediate)](https://iidx.org/intermediate/timing)
- [Timing Guide (Advanced)](https://iidx.org/advanced/timing)
- [beatoraja Configuration](https://github.com/wcko87/beatoraja-english-guide/wiki/Configuration)
