/// Game controller (gamepad) input handling.
///
/// Ported from Java `BMControllerInputProcessor`.
use bms_config::play_mode_config::ControllerConfig;

use crate::analog_scratch::{AnalogScratchAlgorithm, AnalogScratchV1, AnalogScratchV2};
use crate::controller_keys::bm_keys;
use crate::device::InputEvent;

const AXIS_LENGTH: usize = 8;
/// Axis threshold for digital conversion when analog scratch is disabled.
const AXIS_DIGITAL_THRESHOLD: f32 = 0.9;

/// A single game controller input handler.
pub struct ControllerInput {
    /// Whether this controller is enabled.
    enabled: bool,
    /// Lane key assignments (BmKeys constants).
    buttons: Vec<i32>,
    /// Start button assignment.
    start: i32,
    /// Select button assignment.
    select: i32,
    /// Current axis values (-1.0 to 1.0).
    axis: [f32; AXIS_LENGTH],
    /// Button on/off states.
    buttonstate: [bool; bm_keys::MAXID],
    /// Whether button state changed since last poll.
    buttonchanged: [bool; bm_keys::MAXID],
    /// Debounce timestamps (microseconds).
    buttontime: [i64; bm_keys::MAXID],
    /// Debounce duration in microseconds.
    duration_us: i64,
    /// JKOC hack enabled (UP/DOWN false positive prevention).
    jkoc: bool,
    /// Analog scratch algorithms per axis (None = use digital threshold).
    analog_scratch: Option<Vec<Box<dyn AnalogScratchAlgorithm>>>,
    /// Last pressed button code (for config UI).
    last_pressed_button: i32,
}

impl ControllerInput {
    pub fn new() -> Self {
        Self {
            enabled: false,
            buttons: Vec::new(),
            start: bm_keys::BUTTON_9,
            select: bm_keys::BUTTON_10,
            axis: [0.0; AXIS_LENGTH],
            buttonstate: [false; bm_keys::MAXID],
            buttonchanged: [false; bm_keys::MAXID],
            buttontime: [i64::MIN; bm_keys::MAXID],
            duration_us: 16_000,
            jkoc: false,
            analog_scratch: None,
            last_pressed_button: -1,
        }
    }

    /// Apply controller configuration.
    pub fn set_config(&mut self, config: &ControllerConfig) {
        self.buttons = config.keys.clone();
        self.start = config.start;
        self.select = config.select;
        self.duration_us = config.duration as i64 * 1000;
        self.jkoc = config.jkoc_hack;

        if config.analog_scratch {
            let mut algorithms: Vec<Box<dyn AnalogScratchAlgorithm>> =
                Vec::with_capacity(AXIS_LENGTH);
            for _ in 0..AXIS_LENGTH {
                let algo: Box<dyn AnalogScratchAlgorithm> = match config.analog_scratch_mode {
                    1 => Box::new(AnalogScratchV2::new(config.analog_scratch_threshold)),
                    _ => Box::new(AnalogScratchV1::new(config.analog_scratch_threshold)),
                };
                algorithms.push(algo);
            }
            self.analog_scratch = Some(algorithms);
        } else {
            self.analog_scratch = None;
        }

        self.enabled = true;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Clear state (called when play starts).
    pub fn clear(&mut self) {
        self.buttonchanged.fill(false);
        self.buttontime.fill(i64::MIN);
        self.last_pressed_button = -1;
    }

    /// Poll controller state via gilrs and emit input events.
    ///
    /// `button_reader` and `axis_reader` are callbacks for reading hardware state.
    /// This abstraction allows testing without a real gilrs instance.
    pub fn poll_with<B, A>(
        &mut self,
        now_us: i64,
        button_reader: &B,
        axis_reader: &A,
    ) -> Vec<InputEvent>
    where
        B: Fn(i32) -> bool,
        A: Fn(usize) -> f32,
    {
        if !self.enabled {
            return Vec::new();
        }

        let mut events = Vec::new();

        // Update axis values
        for i in 0..AXIS_LENGTH {
            self.axis[i] = axis_reader(i);
        }

        // Update button states with debounce
        for button in 0..bm_keys::MAXID {
            if now_us >= self.buttontime[button] + self.duration_us {
                let prev = self.buttonstate[button];

                if button <= bm_keys::BUTTON_32 as usize {
                    // Digital button
                    self.buttonstate[button] = button_reader(button as i32);
                } else if self.jkoc {
                    // JKOC hack: only AXIS1 from axis[0] and axis[3]
                    self.buttonstate[button] = if button == bm_keys::AXIS1_PLUS as usize {
                        self.axis[0] > AXIS_DIGITAL_THRESHOLD
                            || self.axis[3] > AXIS_DIGITAL_THRESHOLD
                    } else if button == bm_keys::AXIS1_MINUS as usize {
                        self.axis[0] < -AXIS_DIGITAL_THRESHOLD
                            || self.axis[3] < -AXIS_DIGITAL_THRESHOLD
                    } else {
                        false
                    };
                } else {
                    // Axis to digital conversion
                    let axis_index = (button - bm_keys::AXIS1_PLUS as usize) / 2;
                    let is_plus = (button - bm_keys::AXIS1_PLUS as usize).is_multiple_of(2);
                    self.buttonstate[button] = self.scratch_input(axis_index, is_plus);
                }

                self.buttonchanged[button] = prev != self.buttonstate[button];
                if self.buttonchanged[button] {
                    self.buttontime[button] = now_us;
                }

                if !prev && self.buttonstate[button] {
                    self.last_pressed_button = button as i32;
                }
            }
        }

        // Emit events for configured lane buttons
        for (i, &button) in self.buttons.iter().enumerate() {
            if button >= 0
                && (button as usize) < bm_keys::MAXID
                && self.buttonchanged[button as usize]
            {
                events.push(InputEvent::KeyChanged {
                    keycode: i as i32,
                    pressed: self.buttonstate[button as usize],
                    time_us: now_us,
                });
                self.buttonchanged[button as usize] = false;
            }
        }

        // Start button
        if self.start >= 0
            && (self.start as usize) < bm_keys::MAXID
            && self.buttonchanged[self.start as usize]
        {
            events.push(InputEvent::KeyChanged {
                keycode: -1, // special: start
                pressed: self.buttonstate[self.start as usize],
                time_us: now_us,
            });
            self.buttonchanged[self.start as usize] = false;
        }

        // Select button
        if self.select >= 0
            && (self.select as usize) < bm_keys::MAXID
            && self.buttonchanged[self.select as usize]
        {
            events.push(InputEvent::KeyChanged {
                keycode: -2, // special: select
                pressed: self.buttonstate[self.select as usize],
                time_us: now_us,
            });
            self.buttonchanged[self.select as usize] = false;
        }

        // Emit analog state for axis-mapped buttons
        let is_analog = !self.jkoc && self.analog_scratch.is_some();
        for (i, &button) in self.buttons.iter().enumerate() {
            if button < 0 || (button as usize) >= bm_keys::MAXID {
                continue;
            }
            if is_analog && button >= bm_keys::AXIS1_PLUS {
                let value = self.get_analog_value(button);
                events.push(InputEvent::AnalogState {
                    keycode: i as i32,
                    is_analog: true,
                    value,
                });
            } else {
                events.push(InputEvent::AnalogState {
                    keycode: i as i32,
                    is_analog: false,
                    value: 0.0,
                });
            }
        }

        events
    }

    /// Get the analog axis value for a button code.
    fn get_analog_value(&self, button: i32) -> f32 {
        let axis_index = (button - bm_keys::AXIS1_PLUS) as usize / 2;
        let is_plus = (button - bm_keys::AXIS1_PLUS) % 2 == 0;
        if axis_index < AXIS_LENGTH {
            if is_plus {
                self.axis[axis_index]
            } else {
                -self.axis[axis_index]
            }
        } else {
            0.0
        }
    }

    /// Determine scratch/axis state for a given axis and direction.
    fn scratch_input(&mut self, axis_index: usize, plus: bool) -> bool {
        if let Some(ref mut algorithms) = self.analog_scratch {
            if axis_index < algorithms.len() {
                algorithms[axis_index].input(self.axis[axis_index], plus)
            } else {
                false
            }
        } else {
            // No analog scratch: use digital threshold
            if axis_index < AXIS_LENGTH {
                if plus {
                    self.axis[axis_index] > AXIS_DIGITAL_THRESHOLD
                } else {
                    self.axis[axis_index] < -AXIS_DIGITAL_THRESHOLD
                }
            } else {
                false
            }
        }
    }

    pub fn get_last_pressed_button(&self) -> i32 {
        self.last_pressed_button
    }
}

impl Default for ControllerInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_buttons(_button: i32) -> bool {
        false
    }

    fn no_axes(_axis: usize) -> f32 {
        0.0
    }

    #[test]
    fn test_disabled_returns_no_events() {
        let mut controller = ControllerInput::new();
        let events = controller.poll_with(0, &no_buttons, &no_axes);
        assert!(events.is_empty());
    }

    #[test]
    fn test_button_press_event() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![0, 1, 2]; // BUTTON_1, BUTTON_2, BUTTON_3
        config.duration = 0; // no debounce
        controller.set_config(&config);

        // Press BUTTON_1
        let button_reader = |b: i32| b == 0;
        let events = controller.poll_with(1000, &button_reader, &no_axes);

        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { .. }))
            .collect();
        assert!(
            key_events.iter().any(|e| matches!(
                e,
                InputEvent::KeyChanged {
                    keycode: 0,
                    pressed: true,
                    ..
                }
            )),
            "Expected KeyChanged event for lane 0"
        );
    }

    #[test]
    fn test_axis_digital_conversion_no_analog_scratch() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![bm_keys::AXIS1_PLUS]; // Axis 1 positive
        config.analog_scratch = false;
        config.duration = 0;
        controller.set_config(&config);

        // Axis 0 at 0.95 (> 0.9 threshold)
        let axis_reader = |axis: usize| if axis == 0 { 0.95 } else { 0.0 };
        let events = controller.poll_with(1000, &no_buttons, &axis_reader);

        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: true, .. }))
            .collect();
        assert!(!key_events.is_empty(), "Expected axis digital press event");
    }

    #[test]
    fn test_axis_below_threshold_no_event() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![bm_keys::AXIS1_PLUS];
        config.analog_scratch = false;
        config.duration = 0;
        controller.set_config(&config);

        // Axis 0 at 0.5 (< 0.9 threshold)
        let axis_reader = |axis: usize| if axis == 0 { 0.5 } else { 0.0 };
        let events = controller.poll_with(1000, &no_buttons, &axis_reader);

        let key_changed: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { .. }))
            .collect();
        assert!(
            key_changed.is_empty(),
            "No key change expected below threshold"
        );
    }

    #[test]
    fn test_debounce() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![0];
        config.duration = 16; // 16ms = 16000us
        controller.set_config(&config);

        // First press at time 0
        let pressed = |_: i32| true;
        controller.poll_with(0, &pressed, &no_axes);

        // Release at time 1000 (within debounce)
        let released = |_: i32| false;
        let events = controller.poll_with(1000, &released, &no_axes);
        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { .. }))
            .collect();
        assert!(
            key_events.is_empty(),
            "Should not register change within debounce period"
        );

        // Release at time 20000 (after debounce)
        let events = controller.poll_with(20000, &released, &no_axes);
        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: false, .. }))
            .collect();
        assert!(
            !key_events.is_empty(),
            "Should register change after debounce period"
        );
    }

    #[test]
    fn test_jkoc_hack() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![bm_keys::AXIS1_PLUS];
        config.jkoc_hack = true;
        config.duration = 0;
        controller.set_config(&config);

        // axis[0] > 0.9 → AXIS1_PLUS pressed
        let axis_reader = |axis: usize| if axis == 0 { 0.95 } else { 0.0 };
        let events = controller.poll_with(1000, &no_buttons, &axis_reader);

        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: true, .. }))
            .collect();
        assert!(
            !key_events.is_empty(),
            "JKOC hack should detect AXIS1_PLUS from axis[0]"
        );
    }

    #[test]
    fn test_jkoc_hack_axis3_redundant() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![bm_keys::AXIS1_MINUS];
        config.jkoc_hack = true;
        config.duration = 0;
        controller.set_config(&config);

        // axis[3] < -0.9 → AXIS1_MINUS pressed (redundant read)
        let axis_reader = |axis: usize| if axis == 3 { -0.95 } else { 0.0 };
        let events = controller.poll_with(1000, &no_buttons, &axis_reader);

        let key_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, InputEvent::KeyChanged { pressed: true, .. }))
            .collect();
        assert!(
            !key_events.is_empty(),
            "JKOC hack should detect AXIS1_MINUS from axis[3]"
        );
    }

    #[test]
    fn test_start_select_events() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![0];
        config.start = bm_keys::BUTTON_9;
        config.select = bm_keys::BUTTON_10;
        config.duration = 0;
        controller.set_config(&config);

        // Press start and select buttons
        let button_reader = |b: i32| b == bm_keys::BUTTON_9 || b == bm_keys::BUTTON_10;
        let events = controller.poll_with(1000, &button_reader, &no_axes);

        let start_events: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    InputEvent::KeyChanged {
                        keycode: -1,
                        pressed: true,
                        ..
                    }
                )
            })
            .collect();
        let select_events: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    InputEvent::KeyChanged {
                        keycode: -2,
                        pressed: true,
                        ..
                    }
                )
            })
            .collect();

        assert_eq!(start_events.len(), 1, "Expected one start press event");
        assert_eq!(select_events.len(), 1, "Expected one select press event");
    }

    #[test]
    fn test_clear_resets_state() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![0];
        config.duration = 0;
        controller.set_config(&config);

        // Press button
        let pressed = |_: i32| true;
        controller.poll_with(0, &pressed, &no_axes);

        controller.clear();
        assert_eq!(controller.last_pressed_button, -1);
    }

    #[test]
    fn test_last_pressed_button() {
        let mut controller = ControllerInput::new();
        let mut config = ControllerConfig::default();
        config.keys = vec![3, 5]; // BUTTON_4, BUTTON_6
        config.duration = 0;
        controller.set_config(&config);

        // Press BUTTON_6 (index 5)
        let button_reader = |b: i32| b == 5;
        controller.poll_with(1000, &button_reader, &no_axes);

        assert_eq!(controller.last_pressed_button, 5);
    }
}
