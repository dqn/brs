/// Mouse scratch input for BMS turntable lanes.
///
/// Ported from Java `MouseScratchInput.java`.
/// Supports two algorithms: V1 (time-based) and V2 (distance-based).
/// Each algorithm handles both X and Y axes independently.
use bms_config::play_mode_config::{KeyboardConfig, MouseScratchConfig};

use crate::device::InputEvent;

// Mouse scratch direction indices (matches Java constants).
#[cfg(test)]
const MOUSESCRATCH_UP: usize = 2;
#[cfg(test)]
const MOUSESCRATCH_DOWN: usize = 3;
const MOUSESCRATCH_STATE_COUNT: usize = 4;

/// Abstraction over mouse position access for testability.
pub trait MouseBackend {
    fn get_position(&self) -> (i32, i32);
    fn get_screen_size(&self) -> (i32, i32);
    fn set_cursor_position(&mut self, x: i32, y: i32);
}

/// Virtual mouse backend for testing.
pub struct VirtualMouseBackend {
    position: (i32, i32),
    screen_size: (i32, i32),
}

impl VirtualMouseBackend {
    pub fn new(screen_w: i32, screen_h: i32) -> Self {
        Self {
            position: (screen_w / 2, screen_h / 2),
            screen_size: (screen_w, screen_h),
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
    }
}

impl MouseBackend for VirtualMouseBackend {
    fn get_position(&self) -> (i32, i32) {
        self.position
    }

    fn get_screen_size(&self) -> (i32, i32) {
        self.screen_size
    }

    fn set_cursor_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
    }
}

/// Converts mouse pixel movement to analog-like distance tracking.
///
/// Ported from Java `MouseScratchInput.MouseToAnalog`.
struct MouseToAnalog {
    scratch_distance: i32,
    domain: i32,
    total_x_distance_moved: i32,
    total_y_distance_moved: i32,
}

impl MouseToAnalog {
    const TICKS_FOR_SCRATCH: i32 = 2;

    fn new(scratch_distance: i32) -> Self {
        let tick_length = (scratch_distance / Self::TICKS_FOR_SCRATCH).max(1);
        let domain = 256 * tick_length;
        Self {
            scratch_distance,
            domain,
            total_x_distance_moved: 0,
            total_y_distance_moved: 0,
        }
    }

    fn update(&mut self, backend: &mut dyn MouseBackend) {
        let (mx, my) = backend.get_position();
        let (sw, sh) = backend.get_screen_size();
        let x_distance_moved = mx - sw / 2;
        let y_distance_moved = my - sh / 2;
        backend.set_cursor_position(sw / 2, sh / 2);

        self.total_x_distance_moved =
            ((self.total_x_distance_moved + x_distance_moved) % self.domain + self.domain)
                % self.domain;
        self.total_y_distance_moved =
            ((self.total_y_distance_moved + y_distance_moved) % self.domain + self.domain)
                % self.domain;
    }

    fn get_scratch_distance(&self) -> i32 {
        self.scratch_distance
    }

    fn compute_distance_diff(&self, v1: i32, v2: i32) -> i32 {
        let v = v2 - v1;
        if v >= self.domain / 2 {
            return v - self.domain;
        }
        if v < -self.domain / 2 {
            return v + self.domain;
        }
        v
    }

    fn get_distance_moved(&self, x_axis: bool) -> i32 {
        if x_axis {
            self.total_x_distance_moved
        } else {
            self.total_y_distance_moved
        }
    }

    fn get_analog_value(&self, x_axis: bool) -> f32 {
        (self.get_distance_moved(x_axis) % 256) as f32 / 128.0 - 1.0
    }
}

/// V1 algorithm: time-based scratch activation.
///
/// Ported from Java `MouseScratchAlgorithmVersion1`.
/// When the mouse moves in a direction, the scratch becomes active in that direction.
/// It stays active for `scratch_duration` ms after the last movement.
struct MouseScratchAlgorithmV1 {
    x_axis: bool,
    scratch_duration: i32,
    prev_position: i32,
    remaining_time: i32,
    current_scratch: i32,
    lastpresstime: i64,
}

impl MouseScratchAlgorithmV1 {
    fn new(scratch_duration: i32, mouse_to_analog: &MouseToAnalog, x_axis: bool) -> Self {
        Self {
            x_axis,
            scratch_duration,
            prev_position: mouse_to_analog.get_distance_moved(x_axis),
            remaining_time: 0,
            current_scratch: 0,
            lastpresstime: -1,
        }
    }

    fn is_scratch_active(&self, positive: bool) -> bool {
        if positive {
            self.current_scratch > 0
        } else {
            self.current_scratch < 0
        }
    }

    fn get_time_diff(&mut self, presstime: i64) -> i64 {
        if self.lastpresstime < 0 {
            self.lastpresstime = presstime;
            return 0;
        }
        let dtime = presstime - self.lastpresstime;
        self.lastpresstime = presstime;
        dtime
    }

    fn update(&mut self, presstime: i64, mouse_to_analog: &MouseToAnalog) {
        let dtime = self.get_time_diff(presstime);

        let curr_position = mouse_to_analog.get_distance_moved(self.x_axis);
        let d_ticks = mouse_to_analog.compute_distance_diff(self.prev_position, curr_position);
        self.prev_position = curr_position;

        if d_ticks > 0 {
            self.remaining_time = self.scratch_duration;
            self.current_scratch = 1;
        } else if d_ticks < 0 {
            self.remaining_time = self.scratch_duration;
            self.current_scratch = -1;
        } else if self.remaining_time > 0 {
            self.remaining_time -= dtime as i32;
        } else {
            self.current_scratch = 0;
        }
    }

    fn reset(&mut self) {
        self.lastpresstime = -1;
        self.remaining_time = 0;
        self.current_scratch = 0;
    }
}

/// V2 algorithm: distance-based scratch activation.
///
/// Ported from Java `MouseScratchAlgorithmVersion2`.
/// Accumulates mouse movement distance in each direction.
/// Activates scratch when accumulated distance exceeds scratch_distance.
/// Uses a reverse distance threshold (scratch_distance / 3) for direction changes.
struct MouseScratchAlgorithmV2 {
    x_axis: bool,
    scratch_duration: i32,
    scratch_distance: i32,
    scratch_reverse_distance: i32,
    prev_position: i32,
    current_scratch: i32,
    positive_no_movement_time: i32,
    negative_no_movement_time: i32,
    positive_distance: i32,
    negative_distance: i32,
    lastpresstime: i64,
}

impl MouseScratchAlgorithmV2 {
    fn new(scratch_duration: i32, mouse_to_analog: &MouseToAnalog, x_axis: bool) -> Self {
        let scratch_distance = mouse_to_analog.get_scratch_distance();
        Self {
            x_axis,
            scratch_duration,
            scratch_distance,
            scratch_reverse_distance: scratch_distance / 3,
            prev_position: mouse_to_analog.get_distance_moved(x_axis),
            current_scratch: 0,
            positive_no_movement_time: 0,
            negative_no_movement_time: 0,
            positive_distance: 0,
            negative_distance: 0,
            lastpresstime: -1,
        }
    }

    fn is_scratch_active(&self, positive: bool) -> bool {
        if positive {
            self.current_scratch > 0
        } else {
            self.current_scratch < 0
        }
    }

    fn get_time_diff(&mut self, presstime: i64) -> i64 {
        if self.lastpresstime < 0 {
            self.lastpresstime = presstime;
            return 0;
        }
        let dtime = presstime - self.lastpresstime;
        self.lastpresstime = presstime;
        dtime
    }

    fn update(&mut self, presstime: i64, mouse_to_analog: &MouseToAnalog) {
        let dtime = self.get_time_diff(presstime);
        let curr_position = mouse_to_analog.get_distance_moved(self.x_axis);
        let distance_diff =
            mouse_to_analog.compute_distance_diff(self.prev_position, curr_position);
        self.prev_position = curr_position;

        if self.positive_distance == 0 {
            self.positive_no_movement_time = 0;
        }
        if self.negative_distance == 0 {
            self.negative_no_movement_time = 0;
        }

        self.positive_distance = (self.positive_distance + distance_diff).max(0);
        self.negative_distance = (self.negative_distance - distance_diff).max(0);
        self.positive_no_movement_time += dtime as i32;
        self.negative_no_movement_time += dtime as i32;

        if self.positive_distance > 0 {
            if self.current_scratch == -1 && self.positive_distance >= self.scratch_reverse_distance
            {
                self.current_scratch = 0;
                self.negative_distance = 0;
                self.negative_no_movement_time = 0;
            }
            if self.positive_distance > self.scratch_distance {
                self.current_scratch = 1;
                self.positive_no_movement_time = 0;
                self.positive_distance = self.scratch_distance;
            }
        }
        if self.negative_distance > 0 {
            if self.current_scratch == 1 && self.negative_distance >= self.scratch_reverse_distance
            {
                self.current_scratch = 0;
                self.positive_distance = 0;
                self.positive_no_movement_time = 0;
            }
            if self.negative_distance > self.scratch_distance {
                self.current_scratch = -1;
                self.negative_no_movement_time = 0;
                self.negative_distance = self.scratch_distance;
            }
        }

        if self.positive_no_movement_time >= self.scratch_duration {
            self.positive_distance = 0;
            if self.current_scratch == 1 {
                self.current_scratch = 0;
            }
        }
        if self.negative_no_movement_time >= self.scratch_duration {
            self.negative_distance = 0;
            if self.current_scratch == -1 {
                self.current_scratch = 0;
            }
        }
    }

    fn reset(&mut self) {
        self.lastpresstime = -1;
        self.current_scratch = 0;
        self.positive_no_movement_time = 0;
        self.negative_no_movement_time = 0;
        self.positive_distance = 0;
        self.negative_distance = 0;
    }
}

/// Dispatches between V1 and V2 scratch algorithms.
enum ScratchAlgorithm {
    V1(MouseScratchAlgorithmV1),
    V2(MouseScratchAlgorithmV2),
}

impl ScratchAlgorithm {
    fn is_scratch_active(&self, positive: bool) -> bool {
        match self {
            ScratchAlgorithm::V1(v) => v.is_scratch_active(positive),
            ScratchAlgorithm::V2(v) => v.is_scratch_active(positive),
        }
    }

    fn update(&mut self, presstime: i64, mouse_to_analog: &MouseToAnalog) {
        match self {
            ScratchAlgorithm::V1(v) => v.update(presstime, mouse_to_analog),
            ScratchAlgorithm::V2(v) => v.update(presstime, mouse_to_analog),
        }
    }

    fn reset(&mut self) {
        match self {
            ScratchAlgorithm::V1(v) => v.reset(),
            ScratchAlgorithm::V2(v) => v.reset(),
        }
    }
}

/// Mouse scratch input processor.
///
/// Ported from Java `MouseScratchInput`.
/// Manages mouse-based scratch input for BMS turntable lanes,
/// supporting both X-axis (left/right) and Y-axis (up/down) scratch.
pub struct MouseScratchInput {
    /// Lane key codes from mouse scratch config.
    /// Each element maps a lane index to a mouse scratch direction (0-3), or -1 for unmapped.
    keys: Vec<i32>,
    /// Start/select key codes (direction indices or -1).
    control: [i32; 2],
    /// Whether mouse scratch is enabled.
    enabled: bool,
    /// Scratch state for each of the 4 directions.
    scratch_state: [bool; MOUSESCRATCH_STATE_COUNT],
    /// Whether each scratch direction changed since last consumed.
    scratch_changed: [bool; MOUSESCRATCH_STATE_COUNT],
    /// Mouse-to-analog converter (only active when enabled).
    mouse_to_analog: Option<MouseToAnalog>,
    /// Scratch algorithms: [0] = X axis, [1] = Y axis.
    algorithms: Option<[ScratchAlgorithm; 2]>,
    /// Last activated mouse scratch direction.
    last_mouse_scratch: i32,
}

impl MouseScratchInput {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            control: [-1, -1],
            enabled: false,
            scratch_state: [false; MOUSESCRATCH_STATE_COUNT],
            scratch_changed: [false; MOUSESCRATCH_STATE_COUNT],
            mouse_to_analog: None,
            algorithms: None,
            last_mouse_scratch: -1,
        }
    }

    /// Apply configuration from a KeyboardConfig.
    pub fn set_config(&mut self, config: &KeyboardConfig) {
        let ms: &MouseScratchConfig = &config.mouse_scratch_config;
        self.keys = ms.keys.clone();
        self.control = [ms.start, ms.select];
        self.enabled = ms.mouse_scratch_enabled;

        if self.enabled {
            let mta = MouseToAnalog::new(ms.mouse_scratch_distance);
            let threshold = ms.mouse_scratch_time_threshold;
            let mode = ms.mouse_scratch_mode;

            let make_algo = |x_axis: bool, mta: &MouseToAnalog| -> ScratchAlgorithm {
                match mode {
                    1 => ScratchAlgorithm::V2(MouseScratchAlgorithmV2::new(threshold, mta, x_axis)),
                    _ => ScratchAlgorithm::V1(MouseScratchAlgorithmV1::new(threshold, mta, x_axis)),
                }
            };

            let algo_x = make_algo(true, &mta);
            let algo_y = make_algo(false, &mta);
            self.algorithms = Some([algo_x, algo_y]);
            self.mouse_to_analog = Some(mta);
        } else {
            self.mouse_to_analog = None;
            self.algorithms = None;
        }
    }

    /// Poll mouse input and return generated events.
    ///
    /// `microtime` is the current time in microseconds.
    /// Java uses `presstime = microtime / 1000` (milliseconds) for algorithm updates.
    pub fn poll(&mut self, microtime: i64, backend: &mut dyn MouseBackend) -> Vec<InputEvent> {
        let mut events = Vec::new();

        if !self.enabled {
            return events;
        }

        let mta = match self.mouse_to_analog.as_mut() {
            Some(mta) => mta,
            None => return events,
        };

        // Update mouse position tracking.
        mta.update(backend);

        // Java: presstime = microtime / 1000
        let presstime = microtime / 1000;

        // We need to borrow mouse_to_analog immutably for algorithm updates,
        // but it was already updated above. Use a two-phase approach.
        let algorithms = match self.algorithms.as_mut() {
            Some(algos) => algos,
            None => return events,
        };
        let mta = self.mouse_to_analog.as_ref().unwrap();

        // Update algorithms.
        for algo in algorithms.iter_mut() {
            algo.update(presstime, mta);
        }

        // Update scratch state per direction.
        for mouse_input in 0..MOUSESCRATCH_STATE_COUNT {
            let prev = self.scratch_state[mouse_input];
            // algorithm index: 0 for X axis (right/left), 1 for Y axis (up/down)
            let algo_idx = mouse_input / 2;
            let positive = mouse_input % 2 == 0;
            self.scratch_state[mouse_input] = algorithms[algo_idx].is_scratch_active(positive);
            if prev != self.scratch_state[mouse_input] {
                self.scratch_changed[mouse_input] = true;
                if !prev {
                    self.last_mouse_scratch = mouse_input as i32;
                }
            }
        }

        // Emit key events for mapped lanes.
        for (i, &axis) in self.keys.iter().enumerate() {
            if axis >= 0 {
                let axis_idx = axis as usize;
                if axis_idx < MOUSESCRATCH_STATE_COUNT && self.scratch_changed[axis_idx] {
                    events.push(InputEvent::KeyChanged {
                        keycode: i as i32,
                        pressed: self.scratch_state[axis_idx],
                        time_us: microtime,
                    });
                    self.scratch_changed[axis_idx] = false;
                }
            }
        }

        // Start key.
        if self.control[0] >= 0 {
            let idx = self.control[0] as usize;
            if idx < MOUSESCRATCH_STATE_COUNT && self.scratch_changed[idx] {
                events.push(InputEvent::KeyChanged {
                    keycode: -1, // start key sentinel
                    pressed: self.scratch_state[idx],
                    time_us: microtime,
                });
                self.scratch_changed[idx] = false;
            }
        }

        // Select key.
        if self.control[1] >= 0 {
            let idx = self.control[1] as usize;
            if idx < MOUSESCRATCH_STATE_COUNT && self.scratch_changed[idx] {
                events.push(InputEvent::KeyChanged {
                    keycode: -2, // select key sentinel
                    pressed: self.scratch_state[idx],
                    time_us: microtime,
                });
                self.scratch_changed[idx] = false;
            }
        }

        // Emit analog state for mapped lanes.
        let mta = self.mouse_to_analog.as_ref().unwrap();
        for (i, &axis) in self.keys.iter().enumerate() {
            if axis >= 0 {
                let plus = axis % 2 == 0;
                let x_axis = axis < 2;
                let value = mta.get_analog_value(x_axis);
                let analog_value = if plus { value } else { -value };
                events.push(InputEvent::AnalogState {
                    keycode: i as i32,
                    is_analog: true,
                    value: analog_value,
                });
            }
        }

        events
    }

    /// Reset all scratch state.
    pub fn clear(&mut self) {
        if let Some(algos) = self.algorithms.as_mut() {
            for algo in algos.iter_mut() {
                algo.reset();
            }
        }
        self.last_mouse_scratch = -1;
        self.scratch_state = [false; MOUSESCRATCH_STATE_COUNT];
        self.scratch_changed = [false; MOUSESCRATCH_STATE_COUNT];
    }

    pub fn last_mouse_scratch(&self) -> i32 {
        self.last_mouse_scratch
    }

    pub fn set_last_mouse_scratch(&mut self, value: i32) {
        self.last_mouse_scratch = value;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for MouseScratchInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(mode: i32, enabled: bool, threshold: i32, distance: i32) -> KeyboardConfig {
        KeyboardConfig {
            mouse_scratch_config: MouseScratchConfig {
                keys: vec![
                    MOUSESCRATCH_UP as i32,
                    -1,
                    -1,
                    -1,
                    -1,
                    -1,
                    -1,
                    MOUSESCRATCH_DOWN as i32,
                    -1,
                ],
                start: -1,
                select: -1,
                mouse_scratch_enabled: enabled,
                mouse_scratch_time_threshold: threshold,
                mouse_scratch_distance: distance,
                mouse_scratch_mode: mode,
            },
            ..KeyboardConfig::default()
        }
    }

    // ── V1 tests ──

    #[test]
    fn v1_mouse_movement_activates_scratch() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);

        // Move mouse up (negative Y delta in screen coordinates).
        backend.set_position(400, 280); // 20px above center (300)
        let events = input.poll(1_000_000, &mut backend);

        // Should have key events for mapped lanes.
        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { .. }))
            .collect();
        assert!(
            !key_events.is_empty(),
            "Should emit key events on mouse movement"
        );
    }

    #[test]
    fn v1_timeout_deactivates_scratch() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);

        // Move mouse to activate scratch.
        backend.set_position(400, 280);
        input.poll(1_000_000, &mut backend);

        // Wait beyond threshold (150ms). No movement.
        // V1 algorithm: on this poll, d_ticks=0 and remainingTime>0, so remainingTime is reduced.
        input.poll(200_000_000, &mut backend);

        // Third poll: remainingTime is now < 0, so currentScratch = 0.
        let events = input.poll(201_000_000, &mut backend);

        // After timeout, scratch should deactivate. Check that we get a release event.
        let release_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: false, .. }))
            .collect();
        assert!(
            !release_events.is_empty(),
            "Should emit release event after timeout"
        );
    }

    #[test]
    fn v1_no_movement_no_key_events() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);

        // No movement (cursor already at center).
        let events = input.poll(1_000_000, &mut backend);

        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { .. }))
            .collect();
        assert!(
            key_events.is_empty(),
            "No key events when mouse hasn't moved"
        );
    }

    // ── V2 tests ──

    #[test]
    fn v2_distance_accumulation_activates_scratch() {
        let mut input = MouseScratchInput::new();
        // distance=12, so we need to accumulate > 12 pixels to activate.
        let config = make_config(1, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);

        // Move mouse down (positive Y delta) by 15 pixels - exceeds distance threshold.
        backend.set_position(400, 315);
        let events = input.poll(1_000_000, &mut backend);

        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: true, .. }))
            .collect();
        assert!(
            !key_events.is_empty(),
            "V2: Should activate after exceeding distance threshold"
        );
    }

    #[test]
    fn v2_direction_reversal_deactivates() {
        let mut input = MouseScratchInput::new();
        let config = make_config(1, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);

        // Move down to activate.
        backend.set_position(400, 315);
        input.poll(1_000_000, &mut backend);

        // Move up (reverse direction) beyond reverse threshold (12/3=4).
        backend.set_position(400, 295);
        let events = input.poll(2_000_000, &mut backend);

        // Should see state changes from direction reversal.
        let any_key_event = events
            .iter()
            .any(|e| matches!(e, InputEvent::KeyChanged { .. }));
        assert!(
            any_key_event,
            "V2: Direction reversal should produce state change events"
        );
    }

    #[test]
    fn v2_timeout_deactivates() {
        let mut input = MouseScratchInput::new();
        let config = make_config(1, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);

        // Move to activate.
        backend.set_position(400, 315);
        input.poll(1_000_000, &mut backend);

        // Wait beyond threshold with no movement.
        let events = input.poll(200_000_000, &mut backend);

        let release_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: false, .. }))
            .collect();
        assert!(
            !release_events.is_empty(),
            "V2: Should deactivate after timeout with no movement"
        );
    }

    // ── Common tests ──

    #[test]
    fn disabled_returns_no_events() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, false, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);
        backend.set_position(400, 200);
        let events = input.poll(1_000_000, &mut backend);
        assert!(events.is_empty(), "Disabled: should return no events");
    }

    #[test]
    fn config_application() {
        let mut input = MouseScratchInput::new();
        assert!(!input.is_enabled());
        assert_eq!(input.keys.len(), 0);

        let config = make_config(0, true, 200, 20);
        input.set_config(&config);

        assert!(input.is_enabled());
        assert_eq!(input.keys.len(), 9);
        assert_eq!(input.keys[0], MOUSESCRATCH_UP as i32);
        assert_eq!(input.keys[7], MOUSESCRATCH_DOWN as i32);
        assert!(input.algorithms.is_some());
        assert!(input.mouse_to_analog.is_some());
    }

    #[test]
    fn clear_resets_state() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);
        backend.set_position(400, 280);
        input.poll(1_000_000, &mut backend);

        assert_ne!(input.last_mouse_scratch(), -1);

        input.clear();
        assert_eq!(input.last_mouse_scratch(), -1);
        assert!(input.scratch_state.iter().all(|&s| !s));
        assert!(input.scratch_changed.iter().all(|&c| !c));
    }

    #[test]
    fn last_mouse_scratch_tracking() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        assert_eq!(input.last_mouse_scratch(), -1);

        input.set_last_mouse_scratch(MOUSESCRATCH_UP as i32);
        assert_eq!(input.last_mouse_scratch(), MOUSESCRATCH_UP as i32);
    }

    #[test]
    fn analog_events_emitted_for_mapped_lanes() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);
        backend.set_position(400, 280);
        let events = input.poll(1_000_000, &mut backend);

        let analog_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::AnalogState { .. }))
            .collect();
        // Should emit analog events for each mapped lane (lanes 0 and 7).
        assert_eq!(
            analog_events.len(),
            2,
            "Should emit analog events for both mapped lanes"
        );
    }

    #[test]
    fn cursor_recentered_after_poll() {
        let mut input = MouseScratchInput::new();
        let config = make_config(0, true, 150, 12);
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);
        backend.set_position(400, 200);
        input.poll(1_000_000, &mut backend);

        // After poll, cursor should be at screen center.
        assert_eq!(backend.get_position(), (400, 300));
    }

    // ── MouseToAnalog unit tests ──

    #[test]
    fn mouse_to_analog_domain_wrapping() {
        let mta = MouseToAnalog::new(12);
        // tick_length = 12 / 2 = 6, domain = 256 * 6 = 1536
        assert_eq!(mta.domain, 256 * 6);
    }

    #[test]
    fn mouse_to_analog_distance_diff_wrapping() {
        let mta = MouseToAnalog::new(12);
        // Normal diff.
        assert_eq!(mta.compute_distance_diff(100, 150), 50);
        assert_eq!(mta.compute_distance_diff(150, 100), -50);
        // Wrapping around domain.
        assert_eq!(mta.compute_distance_diff(0, mta.domain - 1), -1);
        assert_eq!(mta.compute_distance_diff(mta.domain - 1, 0), 1);
    }

    #[test]
    fn virtual_backend_recentering() {
        let mut backend = VirtualMouseBackend::new(800, 600);
        backend.set_position(100, 200);
        assert_eq!(backend.get_position(), (100, 200));
        assert_eq!(backend.get_screen_size(), (800, 600));
        backend.set_cursor_position(400, 300);
        assert_eq!(backend.get_position(), (400, 300));
    }

    #[test]
    fn start_select_key_events() {
        let mut input = MouseScratchInput::new();
        let config = KeyboardConfig {
            mouse_scratch_config: MouseScratchConfig {
                keys: vec![-1; 9],
                start: MOUSESCRATCH_UP as i32,
                select: MOUSESCRATCH_DOWN as i32,
                mouse_scratch_enabled: true,
                mouse_scratch_time_threshold: 150,
                mouse_scratch_distance: 12,
                mouse_scratch_mode: 0,
            },
            ..KeyboardConfig::default()
        };
        input.set_config(&config);

        let mut backend = VirtualMouseBackend::new(800, 600);
        // Move mouse down (positive Y delta) to trigger Y-axis positive direction = MOUSESCRATCH_UP.
        backend.set_position(400, 320);
        let events = input.poll(1_000_000, &mut backend);

        let start_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { keycode: -1, .. }))
            .collect();
        assert!(
            !start_events.is_empty(),
            "Should emit start key event for MOUSESCRATCH_UP"
        );
    }

    #[test]
    fn default_impl() {
        let input = MouseScratchInput::default();
        assert!(!input.is_enabled());
        assert_eq!(input.last_mouse_scratch(), -1);
    }
}
