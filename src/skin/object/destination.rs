use crate::skin::Destination;

/// Interpolated destination values.
#[derive(Debug, Clone)]
pub struct InterpolatedDest {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub a: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub angle: f32,
}

impl Default for InterpolatedDest {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            a: 255.0,
            r: 255.0,
            g: 255.0,
            b: 255.0,
            angle: 0.0,
        }
    }
}

/// Interpolate between destination keyframes.
pub fn interpolate_destinations(
    destinations: &[Destination],
    elapsed_ms: i64,
    loop_count: i32,
) -> Option<InterpolatedDest> {
    if destinations.is_empty() {
        return None;
    }

    // Single keyframe - no interpolation needed
    if destinations.len() == 1 {
        let dst = &destinations[0];
        return Some(InterpolatedDest {
            x: dst.x,
            y: dst.y,
            w: dst.w,
            h: dst.h,
            a: dst.a,
            r: dst.r,
            g: dst.g,
            b: dst.b,
            angle: dst.angle,
        });
    }

    // Calculate total duration
    let total_duration = destinations.last()?.time as i64;
    if total_duration <= 0 {
        let dst = &destinations[0];
        return Some(InterpolatedDest {
            x: dst.x,
            y: dst.y,
            w: dst.w,
            h: dst.h,
            a: dst.a,
            r: dst.r,
            g: dst.g,
            b: dst.b,
            angle: dst.angle,
        });
    }

    // Handle looping
    let effective_time = if loop_count < 0 {
        // Infinite loop
        elapsed_ms % total_duration
    } else if loop_count == 0 {
        // No loop - clamp to end
        elapsed_ms.min(total_duration)
    } else {
        // Finite loops
        let max_time = total_duration * (loop_count as i64 + 1);
        if elapsed_ms >= max_time {
            // Animation ended
            return None;
        }
        elapsed_ms % total_duration
    };

    // Find the two keyframes to interpolate between
    let mut prev_idx = 0;
    let mut next_idx = 0;

    for (i, dst) in destinations.iter().enumerate() {
        if (dst.time as i64) <= effective_time {
            prev_idx = i;
        }
        if (dst.time as i64) >= effective_time {
            next_idx = i;
            break;
        }
    }

    let prev = &destinations[prev_idx];
    let next = &destinations[next_idx];

    // Calculate interpolation factor
    let t = if prev_idx == next_idx {
        0.0
    } else {
        let prev_time = prev.time as f32;
        let next_time = next.time as f32;
        let duration = next_time - prev_time;
        if duration <= 0.0 {
            0.0
        } else {
            let t = (effective_time as f32 - prev_time) / duration;
            apply_acceleration(t, prev.acc)
        }
    };

    Some(InterpolatedDest {
        x: lerp(prev.x, next.x, t),
        y: lerp(prev.y, next.y, t),
        w: lerp(prev.w, next.w, t),
        h: lerp(prev.h, next.h, t),
        a: lerp(prev.a, next.a, t),
        r: lerp(prev.r, next.r, t),
        g: lerp(prev.g, next.g, t),
        b: lerp(prev.b, next.b, t),
        angle: lerp(prev.angle, next.angle, t),
    })
}

/// Linear interpolation.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Apply acceleration curve to interpolation.
fn apply_acceleration(t: f32, acc: i32) -> f32 {
    match acc {
        0 => t,             // Linear
        1 => t * t,         // Ease in (decelerate)
        2 => t * (2.0 - t), // Ease out (accelerate)
        3 => {
            // Ease in-out
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        _ => t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_single_keyframe() {
        let destinations = vec![Destination {
            time: 0,
            x: 100.0,
            y: 200.0,
            w: 50.0,
            h: 50.0,
            ..Default::default()
        }];

        let result = interpolate_destinations(&destinations, 1000, 0);
        assert!(result.is_some());
        let dst = result.unwrap();
        assert_eq!(dst.x, 100.0);
        assert_eq!(dst.y, 200.0);
    }

    #[test]
    fn test_interpolate_two_keyframes() {
        let destinations = vec![
            Destination {
                time: 0,
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
                ..Default::default()
            },
            Destination {
                time: 1000,
                x: 100.0,
                y: 100.0,
                w: 100.0,
                h: 100.0,
                ..Default::default()
            },
        ];

        // At t=500ms, should be halfway
        let result = interpolate_destinations(&destinations, 500, 0);
        assert!(result.is_some());
        let dst = result.unwrap();
        assert!((dst.x - 50.0).abs() < 0.1);
        assert!((dst.y - 50.0).abs() < 0.1);
    }
}
