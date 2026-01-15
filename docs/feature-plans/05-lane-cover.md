# Lane Cover Implementation

## Overview

Lane covers (SUDDEN+, HIDDEN, LIFT) allow players to customize note visibility by hiding portions of the lane. This is essential for adjusting to different scroll speeds and personal preferences.

## Cover Types

### SUDDEN+ (Top Cover)

- Covers the **top** portion of the lane
- Notes appear suddenly when they pass the cover
- Used to reduce visible note travel time
- Most commonly used cover type

### HIDDEN+ (Bottom Cover)

- Covers the **bottom** portion of the lane
- Notes disappear before reaching the judge line
- Purely cosmetic (no gameplay impact)
- Rarely used in competitive play

### LIFT

- Raises the **judge line** from the bottom
- Covers the bottom portion but ALSO moves where you need to hit
- Different from HIDDEN because it affects timing perception
- Useful for players who prefer hitting notes higher on screen

## White Number System

The lane has a total height of **1000 units**.

| Cover | White Number Range | Description |
|-------|-------------------|-------------|
| SUDDEN+ | 0-1000 | How much of top is covered (%) |
| LIFT | 0-1000 | How much of bottom is covered (%) |

**Example:**
- SUDDEN+ 300 = Top 30% covered, notes visible for 70% of lane
- LIFT 200 = Judge line raised 20% from bottom

## Green Number (Visibility Time)

Green number represents how long a note is visible before hitting the judge line.

**beatoraja:** Uses milliseconds directly
- Green 500 = Note visible for 500ms

**IIDX:** Uses frames (at 60fps)
- Green 300 = Note visible for 300 frames = 5000ms

### Relationship with Hi-Speed

```
Green Number = Lane Height × (1 - SUDDEN+ / 1000) / Scroll Speed
```

## Floating Hi-Speed

When adjusting SUDDEN+ or LIFT, the scroll speed automatically adjusts to maintain constant Green Number.

### Behavior

1. Player adjusts SUDDEN+ cover height
2. System calculates required Hi-Speed to keep Green Number constant
3. Notes continue appearing at the same rhythm

### Formula

```
New_HiSpeed = Old_HiSpeed × Old_VisibleRatio / New_VisibleRatio

where:
VisibleRatio = 1 - (SUDDEN + LIFT) / 1000
```

## Implementation

### Data Structures

```rust
#[derive(Clone, Copy)]
pub struct LaneCover {
    /// SUDDEN+ cover amount (0-1000)
    pub sudden: u16,
    /// HIDDEN+ cover amount (0-1000, rarely used)
    pub hidden: u16,
    /// LIFT amount (0-1000)
    pub lift: u16,
}

impl LaneCover {
    pub fn new() -> Self {
        Self {
            sudden: 0,
            hidden: 0,
            lift: 0,
        }
    }

    /// Get visible portion of lane (0.0-1.0)
    pub fn visible_ratio(&self) -> f32 {
        let covered = (self.sudden + self.hidden + self.lift) as f32 / 1000.0;
        (1.0 - covered).max(0.1)  // Minimum 10% visible
    }

    /// Get top edge of visible area (0.0-1.0 from bottom)
    pub fn visible_top(&self) -> f32 {
        1.0 - (self.sudden as f32 / 1000.0)
    }

    /// Get bottom edge of visible area (0.0-1.0 from bottom)
    pub fn visible_bottom(&self) -> f32 {
        (self.lift as f32 / 1000.0)
    }

    /// Get judge line position (0.0-1.0 from bottom)
    pub fn judge_line_position(&self) -> f32 {
        self.lift as f32 / 1000.0
    }
}

#[derive(Clone, Copy)]
pub struct ScrollSettings {
    pub hi_speed: f32,        // Scroll speed multiplier (1.0 - 10.0)
    pub lane_cover: LaneCover,
    pub floating_hs: bool,    // Enable floating hi-speed
    pub target_green: f32,    // Target green number (ms)
}

impl ScrollSettings {
    pub fn new() -> Self {
        Self {
            hi_speed: 1.0,
            lane_cover: LaneCover::new(),
            floating_hs: true,
            target_green: 500.0,
        }
    }

    /// Calculate current green number in milliseconds
    pub fn green_number(&self, base_scroll_time_ms: f32) -> f32 {
        let visible = self.lane_cover.visible_ratio();
        base_scroll_time_ms * visible / self.hi_speed
    }

    /// Adjust hi-speed to maintain target green number
    pub fn apply_floating_hs(&mut self, base_scroll_time_ms: f32) {
        if self.floating_hs {
            let visible = self.lane_cover.visible_ratio();
            self.hi_speed = base_scroll_time_ms * visible / self.target_green;
            self.hi_speed = self.hi_speed.clamp(0.5, 10.0);
        }
    }

    /// Adjust SUDDEN+ and recalculate hi-speed if floating
    pub fn adjust_sudden(&mut self, delta: i16, base_scroll_time_ms: f32) {
        let old_visible = self.lane_cover.visible_ratio();

        self.lane_cover.sudden = (self.lane_cover.sudden as i16 + delta)
            .clamp(0, 900) as u16;

        if self.floating_hs {
            let new_visible = self.lane_cover.visible_ratio();
            self.hi_speed = self.hi_speed * old_visible / new_visible;
            self.hi_speed = self.hi_speed.clamp(0.5, 10.0);
        }
    }

    /// Adjust LIFT and recalculate hi-speed if floating
    pub fn adjust_lift(&mut self, delta: i16, base_scroll_time_ms: f32) {
        let old_visible = self.lane_cover.visible_ratio();

        self.lane_cover.lift = (self.lane_cover.lift as i16 + delta)
            .clamp(0, 500) as u16;  // Limit LIFT to 50%

        if self.floating_hs {
            let new_visible = self.lane_cover.visible_ratio();
            self.hi_speed = self.hi_speed * old_visible / new_visible;
            self.hi_speed = self.hi_speed.clamp(0.5, 10.0);
        }
    }
}
```

### Rendering

```rust
pub struct LaneRenderer {
    lane_height: f32,
    lane_bottom_y: f32,
    settings: ScrollSettings,
}

impl LaneRenderer {
    /// Convert note time to Y position on screen
    pub fn note_y_position(&self, note_time_ms: i64, current_time_ms: i64) -> f32 {
        let time_diff = (note_time_ms - current_time_ms) as f32;
        let base_scroll_per_ms = self.lane_height / 1000.0;  // Example base speed
        let scroll_per_ms = base_scroll_per_ms * self.settings.hi_speed;

        let judge_y = self.lane_bottom_y +
            self.settings.lane_cover.judge_line_position() * self.lane_height;

        judge_y + time_diff * scroll_per_ms
    }

    /// Check if a note at given Y position is visible
    pub fn is_note_visible(&self, y_position: f32) -> bool {
        let visible_top = self.lane_bottom_y +
            self.settings.lane_cover.visible_top() * self.lane_height;
        let visible_bottom = self.lane_bottom_y +
            self.settings.lane_cover.visible_bottom() * self.lane_height;

        y_position >= visible_bottom && y_position <= visible_top
    }

    /// Draw lane cover overlays
    pub fn draw_covers(&self) {
        let cover = &self.settings.lane_cover;

        // Draw SUDDEN+ cover (top)
        if cover.sudden > 0 {
            let cover_height = (cover.sudden as f32 / 1000.0) * self.lane_height;
            let cover_y = self.lane_bottom_y + self.lane_height - cover_height;
            draw_rectangle(
                0.0, cover_y,
                LANE_WIDTH, cover_height,
                COVER_COLOR,
            );
        }

        // Draw LIFT cover (bottom, with judge line)
        if cover.lift > 0 {
            let cover_height = (cover.lift as f32 / 1000.0) * self.lane_height;
            draw_rectangle(
                0.0, self.lane_bottom_y,
                LANE_WIDTH, cover_height,
                COVER_COLOR,
            );
        }

        // Draw judge line at adjusted position
        let judge_y = self.lane_bottom_y +
            cover.judge_line_position() * self.lane_height;
        draw_line(
            0.0, judge_y,
            LANE_WIDTH, judge_y,
            JUDGE_LINE_COLOR,
        );
    }
}
```

### Input Handling

Common control schemes:

| Input | Action |
|-------|--------|
| START + Turntable | Adjust SUDDEN+ |
| Double-tap START | Toggle SUDDEN+ on/off |
| START + SELECT + Turntable | Adjust Green Number |
| (SUDDEN off) START + Turntable | Adjust LIFT |

```rust
pub fn handle_cover_input(
    settings: &mut ScrollSettings,
    input: &Input,
    base_scroll_time_ms: f32,
) {
    if input.is_held(Key::Start) {
        let delta = input.turntable_delta() as i16 * 5;

        if input.is_held(Key::Select) {
            // Adjust green number directly
            settings.target_green = (settings.target_green + delta as f32)
                .clamp(100.0, 2000.0);
            settings.apply_floating_hs(base_scroll_time_ms);
        } else {
            // Adjust SUDDEN+
            settings.adjust_sudden(delta, base_scroll_time_ms);
        }
    }

    if input.double_tap(Key::Start) {
        // Toggle SUDDEN+
        if settings.lane_cover.sudden > 0 {
            settings.lane_cover.sudden = 0;
        } else {
            settings.lane_cover.sudden = 300;  // Default SUDDEN+ value
        }
        settings.apply_floating_hs(base_scroll_time_ms);
    }
}
```

## Display Information

Show current settings on screen:

```
HI-SPEED: 3.50
GREEN: 573
SUDDEN+: 287
LIFT: 0
```

## Persistence

Save lane cover settings per-user:

```rust
pub struct UserSettings {
    pub default_sudden: u16,
    pub default_lift: u16,
    pub default_hi_speed: f32,
    pub floating_hs_enabled: bool,
    pub target_green: f32,
}
```

## Reference Links

- [Lane Covers Guide](https://iidx.org/customize/lanecover)
- [Floating Hi-Speed](https://iidx.org/beginner/floating)
- [In-game Controls](https://iidx.org/beginner/ingame)
- [beatoraja Scroll Configuration](https://github.com/wcko87/beatoraja-english-guide/wiki/Scroll-Speed-and-Green-Number)
