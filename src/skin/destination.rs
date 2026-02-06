// Destination system for skin objects.
// Implements multi-destination linear interpolation with acceleration modes.

/// A single destination keyframe.
#[derive(Debug, Clone, PartialEq)]
pub struct Destination {
    /// Time in milliseconds.
    pub time: i64,
    /// X position.
    pub x: f32,
    /// Y position.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
    /// Acceleration type (0=linear, 1=ease-in, 2=ease-out, 3=step).
    pub acc: i32,
    /// Alpha (0-255).
    pub a: i32,
    /// Red (0-255).
    pub r: i32,
    /// Green (0-255).
    pub g: i32,
    /// Blue (0-255).
    pub b: i32,
    /// Angle in degrees.
    pub angle: i32,
}

impl Destination {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time: i64,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        acc: i32,
        a: i32,
        r: i32,
        g: i32,
        b: i32,
        angle: i32,
    ) -> Self {
        Self {
            time,
            x,
            y,
            w,
            h,
            acc,
            a,
            r,
            g,
            b,
            angle,
        }
    }
}

/// Interpolated destination result at a given time.
#[derive(Debug, Clone, PartialEq)]
pub struct InterpolatedDst {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub angle: i32,
}

/// Configuration for a skin object's destination animation.
#[derive(Debug, Clone, Default)]
pub struct DestinationSet {
    /// Destination keyframes sorted by time.
    pub destinations: Vec<Destination>,
    /// Timer ID (0 = no timer, uses global time).
    pub timer: i32,
    /// Loop point in milliseconds.
    pub loop_ms: i32,
    /// Blend mode (0=alpha, 2=additive, 9=invert).
    pub blend: i32,
    /// Filter mode (0=nearest, 1=linear).
    pub filter: i32,
    /// Rotation center index (0-9).
    pub center: i32,
    /// Option conditions: positive means "must be true", negative means "must be false".
    pub options: Vec<i32>,
    /// Stretch type.
    pub stretch: i32,
    /// Offset IDs.
    pub offsets: Vec<i32>,
    /// Mouse rect for conditional drawing.
    pub mouse_rect: Option<[f32; 4]>,
}

impl DestinationSet {
    pub fn add_destination(&mut self, dst: Destination) {
        // Insert sorted by time
        let pos = self.destinations.partition_point(|d| d.time <= dst.time);
        self.destinations.insert(pos, dst);
    }

    /// Returns true if there are no destination keyframes.
    pub fn is_empty(&self) -> bool {
        self.destinations.is_empty()
    }

    /// Interpolate the destination at the given time (ms).
    /// Returns None if the object should not be drawn.
    pub fn interpolate(&self, time_ms: i64) -> Option<InterpolatedDst> {
        if self.destinations.is_empty() {
            return None;
        }

        let dsts = &self.destinations;
        let start_time = dsts[0].time;
        let end_time = dsts[dsts.len() - 1].time;

        // Apply loop
        let time = if self.loop_ms == -1 {
            if time_ms > end_time {
                return None;
            }
            time_ms
        } else if end_time > 0 && time_ms > self.loop_ms as i64 {
            let loop_ms = self.loop_ms as i64;
            if end_time == loop_ms {
                loop_ms
            } else {
                (time_ms - loop_ms) % (end_time - loop_ms) + loop_ms
            }
        } else {
            time_ms
        };

        if time < start_time {
            return None;
        }

        // Find the interpolation segment
        if dsts.len() == 1 || time >= end_time {
            let d = &dsts[dsts.len() - 1];
            return Some(InterpolatedDst {
                x: d.x,
                y: d.y,
                w: d.w,
                h: d.h,
                r: d.r as f32 / 255.0,
                g: d.g as f32 / 255.0,
                b: d.b as f32 / 255.0,
                a: d.a as f32 / 255.0,
                angle: d.angle,
            });
        }

        // Binary search for segment
        let mut idx = dsts.len() - 2;
        for i in (0..dsts.len() - 1).rev() {
            if dsts[i].time <= time {
                idx = i;
                break;
            }
        }

        let d1 = &dsts[idx];
        let d2 = &dsts[idx + 1];
        let segment_duration = d2.time - d1.time;

        if segment_duration == 0 {
            return Some(InterpolatedDst {
                x: d1.x,
                y: d1.y,
                w: d1.w,
                h: d1.h,
                r: d1.r as f32 / 255.0,
                g: d1.g as f32 / 255.0,
                b: d1.b as f32 / 255.0,
                a: d1.a as f32 / 255.0,
                angle: d1.angle,
            });
        }

        let raw_rate = (time - d1.time) as f32 / segment_duration as f32;
        let acc = if d1.acc != 0 { d1.acc } else { d2.acc };

        // Apply acceleration
        let rate = match acc {
            1 => raw_rate * raw_rate,                       // ease-in (quadratic)
            2 => 1.0 - (raw_rate - 1.0) * (raw_rate - 1.0), // ease-out
            3 => 0.0,                                       // step (use d1 values)
            _ => raw_rate,                                  // linear
        };

        Some(InterpolatedDst {
            x: d1.x + (d2.x - d1.x) * rate,
            y: d1.y + (d2.y - d1.y) * rate,
            w: d1.w + (d2.w - d1.w) * rate,
            h: d1.h + (d2.h - d1.h) * rate,
            r: (d1.r as f32 + (d2.r - d1.r) as f32 * rate) / 255.0,
            g: (d1.g as f32 + (d2.g - d1.g) as f32 * rate) / 255.0,
            b: (d1.b as f32 + (d2.b - d1.b) as f32 * rate) / 255.0,
            a: (d1.a as f32 + (d2.a - d1.a) as f32 * rate) / 255.0,
            angle: if acc == 3 {
                d1.angle
            } else {
                d1.angle + ((d2.angle - d1.angle) as f32 * rate) as i32
            },
        })
    }
}

/// Rotation center X coordinate lookup (0-9).
pub const CENTER_X: [f32; 10] = [0.5, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0];
/// Rotation center Y coordinate lookup (0-9).
pub const CENTER_Y: [f32; 10] = [0.5, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0];

/// Get rotation center coordinates for a given center index (0-9).
pub fn rotation_center(center: i32) -> (f32, f32) {
    let idx = center.clamp(0, 9) as usize;
    (CENTER_X[idx], CENTER_Y[idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dst(time: i64, x: f32, y: f32, w: f32, h: f32) -> Destination {
        Destination::new(time, x, y, w, h, 0, 255, 255, 255, 255, 0)
    }

    #[test]
    fn single_destination_always_returns_same() {
        let mut set = DestinationSet::default();
        set.add_destination(make_dst(0, 10.0, 20.0, 100.0, 50.0));

        let result = set.interpolate(0).unwrap();
        assert_eq!(result.x, 10.0);
        assert_eq!(result.y, 20.0);

        let result = set.interpolate(1000).unwrap();
        assert_eq!(result.x, 10.0);
    }

    #[test]
    fn linear_interpolation_midpoint() {
        let mut set = DestinationSet::default();
        set.add_destination(make_dst(0, 0.0, 0.0, 100.0, 100.0));
        set.add_destination(make_dst(1000, 200.0, 100.0, 100.0, 100.0));

        let result = set.interpolate(500).unwrap();
        assert!((result.x - 100.0).abs() < 0.01);
        assert!((result.y - 50.0).abs() < 0.01);
    }

    #[test]
    fn before_start_time_returns_none() {
        let mut set = DestinationSet::default();
        set.add_destination(make_dst(100, 0.0, 0.0, 10.0, 10.0));

        assert!(set.interpolate(50).is_none());
    }

    #[test]
    fn ease_in_acceleration() {
        let mut set = DestinationSet::default();
        set.add_destination(Destination::new(
            0, 0.0, 0.0, 100.0, 100.0, 1, 255, 255, 255, 255, 0,
        ));
        set.add_destination(make_dst(1000, 100.0, 0.0, 100.0, 100.0));

        let result = set.interpolate(500).unwrap();
        // ease-in: rate = 0.5^2 = 0.25, so x = 25.0
        assert!((result.x - 25.0).abs() < 0.01);
    }

    #[test]
    fn ease_out_acceleration() {
        let mut set = DestinationSet::default();
        set.add_destination(Destination::new(
            0, 0.0, 0.0, 100.0, 100.0, 2, 255, 255, 255, 255, 0,
        ));
        set.add_destination(make_dst(1000, 100.0, 0.0, 100.0, 100.0));

        let result = set.interpolate(500).unwrap();
        // ease-out: rate = 1 - (0.5 - 1)^2 = 1 - 0.25 = 0.75, so x = 75.0
        assert!((result.x - 75.0).abs() < 0.01);
    }

    #[test]
    fn step_acceleration() {
        let mut set = DestinationSet::default();
        set.add_destination(Destination::new(
            0, 0.0, 0.0, 100.0, 100.0, 3, 255, 255, 255, 255, 0,
        ));
        set.add_destination(make_dst(1000, 100.0, 0.0, 100.0, 100.0));

        let result = set.interpolate(500).unwrap();
        assert!((result.x - 0.0).abs() < 0.01);
    }

    #[test]
    fn loop_wraps_time() {
        let mut set = DestinationSet::default();
        set.loop_ms = 0;
        set.add_destination(make_dst(0, 0.0, 0.0, 100.0, 100.0));
        set.add_destination(make_dst(1000, 100.0, 0.0, 100.0, 100.0));

        // After end_time with loop_ms=0, should loop back
        let result = set.interpolate(1500).unwrap();
        assert!((result.x - 50.0).abs() < 0.01);
    }

    #[test]
    fn loop_minus_one_hides_after_end() {
        let mut set = DestinationSet::default();
        set.loop_ms = -1;
        set.add_destination(make_dst(0, 0.0, 0.0, 100.0, 100.0));
        set.add_destination(make_dst(1000, 100.0, 0.0, 100.0, 100.0));

        assert!(set.interpolate(1500).is_none());
    }

    #[test]
    fn color_interpolation() {
        let mut set = DestinationSet::default();
        set.add_destination(Destination::new(
            0, 0.0, 0.0, 10.0, 10.0, 0, 0, 255, 0, 0, 0,
        ));
        set.add_destination(Destination::new(
            1000, 0.0, 0.0, 10.0, 10.0, 0, 255, 0, 255, 0, 0,
        ));

        let result = set.interpolate(500).unwrap();
        assert!((result.a - 0.5).abs() < 0.01);
        assert!((result.r - 0.5).abs() < 0.01);
        assert!((result.g - 0.5).abs() < 0.01);
    }

    #[test]
    fn rotation_center_lookup() {
        assert_eq!(rotation_center(0), (0.5, 0.5));
        assert_eq!(rotation_center(1), (0.0, 0.0));
        assert_eq!(rotation_center(9), (1.0, 1.0));
    }

    #[test]
    fn add_destination_sorted() {
        let mut set = DestinationSet::default();
        set.add_destination(make_dst(1000, 0.0, 0.0, 10.0, 10.0));
        set.add_destination(make_dst(0, 0.0, 0.0, 10.0, 10.0));
        set.add_destination(make_dst(500, 0.0, 0.0, 10.0, 10.0));

        assert_eq!(set.destinations[0].time, 0);
        assert_eq!(set.destinations[1].time, 500);
        assert_eq!(set.destinations[2].time, 1000);
    }
}
