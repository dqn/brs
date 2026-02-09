//! Analog scratch algorithms for controller turntable input.
//!
//! Ported from Java `BMControllerInputProcessor` inner classes
//! `AnalogScratchAlgorithmVersion1` and `AnalogScratchAlgorithmVersion2`.

/// Tick size for analog scratch quantization.
/// arcin board -> 0.00784, so 0.009 is used as the maximum tick size.
const TICK_MAX_SIZE: f32 = 0.009;

/// Analog scratch algorithm trait.
pub trait AnalogScratchAlgorithm {
    /// Process an analog input value. Returns true if scratch should activate
    /// for the given direction (`plus` = right/clockwise).
    fn input(&mut self, current: f32, plus: bool) -> bool;

    /// Reset internal state.
    fn reset(&mut self);
}

/// Compute the quantized tick difference between two analog positions.
///
/// Handles wrapping around the -1.0/1.0 boundary. Returns positive for
/// rightward motion, negative for leftward.
pub fn compute_analog_diff(old_value: f32, new_value: f32) -> i32 {
    let mut diff = new_value - old_value;
    if diff > 1.0 {
        diff -= 2.0 + TICK_MAX_SIZE / 2.0;
    } else if diff < -1.0 {
        diff += 2.0 + TICK_MAX_SIZE / 2.0;
    }
    diff /= TICK_MAX_SIZE;
    if diff > 0.0 {
        diff.ceil() as i32
    } else {
        diff.floor() as i32
    }
}

/// Version 1: Direction detection with counter-based stop.
///
/// Detects scratch direction by comparing old and new positions (handling
/// wrapping), then keeps the scratch active until the counter exceeds the
/// threshold (i.e., no movement for `threshold` calls).
pub struct AnalogScratchV1 {
    threshold: i64,
    counter: i64,
    old_scratch_x: f32,
    scratch_active: bool,
    right_move_scratching: bool,
}

impl AnalogScratchV1 {
    pub fn new(threshold: i32) -> Self {
        Self {
            threshold: threshold as i64,
            counter: 1,
            old_scratch_x: 10.0, // sentinel: > 1.0 means uninitialized
            scratch_active: false,
            right_move_scratching: false,
        }
    }
}

impl AnalogScratchAlgorithm for AnalogScratchV1 {
    fn input(&mut self, current: f32, plus: bool) -> bool {
        // First call: initialize position
        if self.old_scratch_x > 1.0 {
            self.old_scratch_x = current;
            self.scratch_active = false;
            return false;
        }

        if self.old_scratch_x != current {
            // Determine direction with wrapping
            let now_right = if self.old_scratch_x < current {
                // Naive: rightward, but check wrapping
                if (current - self.old_scratch_x) > (1.0 - current + self.old_scratch_x) {
                    false // wrapping: actually leftward
                } else {
                    true
                }
            } else {
                // Naive: leftward, but check wrapping
                if (self.old_scratch_x - current) > ((current + 1.0) - self.old_scratch_x) {
                    true // wrapping: actually rightward
                } else {
                    false
                }
            };

            if self.scratch_active && self.right_move_scratching != now_right {
                // Direction change while active
                self.right_move_scratching = now_right;
            } else if !self.scratch_active {
                // Start scratching
                self.scratch_active = true;
                self.right_move_scratching = now_right;
            }

            self.counter = 0;
            self.old_scratch_x = current;
        }

        // counter > threshold -> stop scratching
        if self.counter > self.threshold && self.scratch_active {
            self.scratch_active = false;
            self.counter = 0;
        }

        if self.counter == i64::MAX {
            self.counter = 0;
        }

        self.counter += 1;

        if plus {
            self.scratch_active && self.right_move_scratching
        } else {
            self.scratch_active && !self.right_move_scratching
        }
    }

    fn reset(&mut self) {
        self.counter = 1;
        self.old_scratch_x = 10.0;
        self.scratch_active = false;
        self.right_move_scratching = false;
    }
}

/// Version 2: Tick accumulation with direction change reset.
///
/// Uses `compute_analog_diff` to count ticks. Accumulates ticks until 2 are
/// reached to activate scratching. Direction changes reset the tick counter.
/// Scratching stops after `threshold * 2` calls without movement.
pub struct AnalogScratchV2 {
    threshold: i64,
    counter: i64,
    tick_counter: i32,
    old_scratch_x: f32,
    scratch_active: bool,
    right_move_scratching: bool,
}

impl AnalogScratchV2 {
    pub fn new(threshold: i32) -> Self {
        Self {
            threshold: threshold as i64,
            counter: 1,
            tick_counter: 0,
            old_scratch_x: 10.0, // sentinel: > 1.0 means uninitialized
            scratch_active: false,
            right_move_scratching: false,
        }
    }
}

impl AnalogScratchAlgorithm for AnalogScratchV2 {
    fn input(&mut self, current: f32, plus: bool) -> bool {
        // First call: initialize position
        if self.old_scratch_x > 1.0 {
            self.old_scratch_x = current;
            self.scratch_active = false;
            return false;
        }

        if self.old_scratch_x != current {
            let ticks = compute_analog_diff(self.old_scratch_x, current);
            let now_right = ticks >= 0;

            if self.scratch_active && self.right_move_scratching != now_right {
                // Direction change while active -> deactivate and reset
                self.right_move_scratching = now_right;
                self.scratch_active = false;
                self.tick_counter = 0;
            } else if !self.scratch_active {
                // Accumulate ticks if counter is fresh or within threshold
                if self.tick_counter == 0 || self.counter <= self.threshold {
                    self.tick_counter += ticks.unsigned_abs() as i32;
                }
                // Activate once 2 ticks accumulated
                if self.tick_counter >= 2 {
                    self.scratch_active = true;
                    self.right_move_scratching = now_right;
                }
            }

            self.counter = 0;
            self.old_scratch_x = current;
        }

        // counter > 2*threshold -> stop scratching
        if self.counter > self.threshold * 2 {
            self.scratch_active = false;
            self.tick_counter = 0;
            self.counter = 0;
        }

        self.counter += 1;

        if plus {
            self.scratch_active && self.right_move_scratching
        } else {
            self.scratch_active && !self.right_move_scratching
        }
    }

    fn reset(&mut self) {
        self.counter = 1;
        self.tick_counter = 0;
        self.old_scratch_x = 10.0;
        self.scratch_active = false;
        self.right_move_scratching = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- compute_analog_diff tests ---

    #[test]
    fn test_compute_analog_diff_zero() {
        assert_eq!(compute_analog_diff(0.5, 0.5), 0);
        assert_eq!(compute_analog_diff(0.0, 0.0), 0);
        assert_eq!(compute_analog_diff(-0.5, -0.5), 0);
    }

    #[test]
    fn test_compute_analog_diff_positive() {
        // Small rightward movement: 0.018 / 0.009 = 2.0 -> ceil(2.0) = 2
        assert_eq!(compute_analog_diff(0.0, 0.018), 2);
        // 0.01 / 0.009 = 1.111... -> ceil = 2
        assert_eq!(compute_analog_diff(0.0, 0.01), 2);
        // Exactly one tick
        assert_eq!(compute_analog_diff(0.0, 0.009), 1);
    }

    #[test]
    fn test_compute_analog_diff_negative() {
        // Small leftward movement: -0.018 / 0.009 = -2.0 -> floor(-2.0) = -2
        assert_eq!(compute_analog_diff(0.0, -0.018), -2);
        // -0.01 / 0.009 = -1.111... -> floor = -2
        assert_eq!(compute_analog_diff(0.0, -0.01), -2);
        // Exactly one tick
        assert_eq!(compute_analog_diff(0.0, -0.009), -1);
    }

    #[test]
    fn test_compute_analog_diff_wrapping_positive() {
        // Wrapping around: old=0.99, new=-0.99
        // raw = -0.99 - 0.99 = -1.98 -> < -1.0, so add 2.0045 -> 0.0245
        // 0.0245 / 0.009 = 2.722... -> ceil = 3
        let diff = compute_analog_diff(0.99, -0.99);
        assert!(
            diff > 0,
            "Wrapping rightward should be positive, got {diff}"
        );
    }

    #[test]
    fn test_compute_analog_diff_wrapping_negative() {
        // Wrapping around: old=-0.99, new=0.99
        // raw = 0.99 - (-0.99) = 1.98 -> > 1.0, so sub 2.0045 -> -0.0245
        // -0.0245 / 0.009 = -2.722... -> floor = -3
        let diff = compute_analog_diff(-0.99, 0.99);
        assert!(diff < 0, "Wrapping leftward should be negative, got {diff}");
    }

    // --- AnalogScratchV1 tests ---

    #[test]
    fn test_v1_first_call_initializes() {
        let mut v1 = AnalogScratchV1::new(10);
        // First call initializes position, returns false
        assert!(!v1.input(0.5, true));
        assert!(!v1.input(0.5, false));
    }

    #[test]
    fn test_v1_rightward_activation() {
        let mut v1 = AnalogScratchV1::new(10);
        v1.input(0.0, true); // init
        // Move right
        assert!(v1.input(0.1, true)); // plus=true, rightward -> true
        assert!(!v1.input(0.1, false)); // plus=false, rightward -> false (still active)
    }

    #[test]
    fn test_v1_leftward_activation() {
        let mut v1 = AnalogScratchV1::new(10);
        v1.input(0.5, true); // init
        // Move left
        assert!(!v1.input(0.3, true)); // plus=true, leftward -> false
        assert!(v1.input(0.3, false)); // plus=false, leftward -> true (still active)
    }

    #[test]
    fn test_v1_counter_timeout() {
        let mut v1 = AnalogScratchV1::new(3);
        v1.input(0.0, true); // init
        v1.input(0.1, true); // activate rightward
        assert!(v1.input(0.1, true)); // still active

        // No movement for threshold+1 calls -> deactivate
        for _ in 0..3 {
            v1.input(0.1, true);
        }
        // After threshold exceeded, should be deactivated
        assert!(!v1.input(0.1, true));
    }

    #[test]
    fn test_v1_direction_change() {
        let mut v1 = AnalogScratchV1::new(100);
        v1.input(0.5, true); // init
        v1.input(0.6, true); // activate rightward
        assert!(v1.input(0.6, true)); // active + right -> true for plus

        // Now move left -> direction changes
        v1.input(0.4, false);
        assert!(v1.input(0.4, false)); // active + left -> true for minus
        assert!(!v1.input(0.4, true)); // active + left -> false for plus
    }

    #[test]
    fn test_v1_reset() {
        let mut v1 = AnalogScratchV1::new(10);
        v1.input(0.0, true); // init
        v1.input(0.1, true); // activate
        assert!(v1.input(0.1, true));

        v1.reset();
        // After reset, first call should re-initialize
        assert!(!v1.input(0.5, true)); // re-init
        assert!(!v1.input(0.5, true)); // no movement
    }

    // --- AnalogScratchV2 tests ---

    #[test]
    fn test_v2_first_call_initializes() {
        let mut v2 = AnalogScratchV2::new(10);
        assert!(!v2.input(0.5, true));
        assert!(!v2.input(0.5, false));
    }

    #[test]
    fn test_v2_needs_two_ticks_to_activate() {
        let mut v2 = AnalogScratchV2::new(100);
        v2.input(0.0, true); // init

        // Move by 1 tick -> not yet active (tick_counter == 1 < 2)
        let one_tick = TICK_MAX_SIZE;
        assert!(!v2.input(one_tick, true));

        // Move by another tick -> should activate (tick_counter >= 2)
        assert!(v2.input(one_tick * 2.0, true));
    }

    #[test]
    fn test_v2_large_movement_activates_immediately() {
        let mut v2 = AnalogScratchV2::new(100);
        v2.input(0.0, true); // init

        // Move by multiple ticks at once (>= 2 ticks)
        assert!(v2.input(0.02, true)); // 0.02 / 0.009 = 2.22 -> ceil = 3 ticks
    }

    #[test]
    fn test_v2_direction_change_resets() {
        let mut v2 = AnalogScratchV2::new(100);
        v2.input(0.0, true); // init
        v2.input(0.02, true); // activate rightward (3+ ticks)
        assert!(v2.input(0.02, true)); // active + right -> true

        // Move left -> direction change resets scratch
        v2.input(-0.02, false);
        // After direction change, scratch_active becomes false, tick_counter = 0
        // The movement itself doesn't reactivate because tick_counter was just reset
        // Need to accumulate 2 more ticks in the new direction
    }

    #[test]
    fn test_v2_counter_timeout() {
        let mut v2 = AnalogScratchV2::new(3);
        v2.input(0.0, true); // init
        v2.input(0.02, true); // activate (>= 2 ticks)
        assert!(v2.input(0.02, true)); // still active

        // No movement for 2*threshold+1 calls -> deactivate
        for _ in 0..7 {
            v2.input(0.02, true);
        }
        assert!(!v2.input(0.02, true));
    }

    #[test]
    fn test_v2_reset() {
        let mut v2 = AnalogScratchV2::new(10);
        v2.input(0.0, true); // init
        v2.input(0.02, true); // activate
        assert!(v2.input(0.02, true));

        v2.reset();
        assert!(!v2.input(0.5, true)); // re-init
        assert!(!v2.input(0.5, true)); // no movement
    }

    #[test]
    fn test_v2_tick_accumulation_within_threshold() {
        let mut v2 = AnalogScratchV2::new(100);
        v2.input(0.0, true); // init

        // Move by 1 tick
        v2.input(TICK_MAX_SIZE, true);
        assert!(!v2.input(TICK_MAX_SIZE, true)); // no further movement, not active

        // Move by 1 more tick within threshold -> should accumulate and activate
        assert!(v2.input(TICK_MAX_SIZE * 2.0, true));
    }
}
