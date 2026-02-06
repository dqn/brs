use crate::state::config::key_config::{AxisDirection, KeyConfig};
use crate::traits::input::KeyEvent;

/// Axis threshold for digital conversion.
const AXIS_THRESHOLD: f32 = 0.5;

/// Handles gamepad input via gilrs and maps buttons/axes to game keys.
pub struct GamepadHandler {
    /// gilrs instance for polling gamepad events.
    gilrs: gilrs::Gilrs,
    /// Key config mapping controller inputs to lanes.
    key_config: KeyConfig,
    /// Axis state for each binding (used for edge detection).
    axis_active: Vec<bool>,
}

impl GamepadHandler {
    /// Create a new gamepad handler.
    pub fn new(key_config: KeyConfig) -> Self {
        let gilrs = gilrs::Gilrs::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to initialize gilrs: {e}");
            match e {
                gilrs::Error::NotImplemented(gilrs) => gilrs,
                other => panic!("gilrs initialization failed: {other}"),
            }
        });
        let axis_active = vec![false; key_config.bindings.len()];
        Self {
            gilrs,
            key_config,
            axis_active,
        }
    }

    /// Poll gamepad events and return game key events.
    pub fn poll(&mut self, time_us: i64) -> Vec<KeyEvent> {
        let mut events = Vec::new();

        while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
            match event {
                gilrs::ev::EventType::ButtonPressed(button, ..) => {
                    let code = button_to_u32(button);
                    self.map_button(code, true, time_us, &mut events);
                }
                gilrs::ev::EventType::ButtonReleased(button, ..) => {
                    let code = button_to_u32(button);
                    self.map_button(code, false, time_us, &mut events);
                }
                gilrs::ev::EventType::AxisChanged(axis, value, ..) => {
                    let axis_code = axis_to_u32(axis);
                    self.map_axis(axis_code, value, time_us, &mut events);
                }
                _ => {}
            }
        }

        events
    }

    fn map_button(&self, code: u32, pressed: bool, time_us: i64, events: &mut Vec<KeyEvent>) {
        for (lane, binding) in self.key_config.bindings.iter().enumerate() {
            if binding.controller_button == Some(code) {
                events.push(KeyEvent {
                    key: lane,
                    pressed,
                    time_us,
                });
            }
        }
    }

    fn map_axis(&mut self, axis_code: u32, value: f32, time_us: i64, events: &mut Vec<KeyEvent>) {
        for (lane, binding) in self.key_config.bindings.iter().enumerate() {
            if let Some((bound_axis, direction)) = binding.controller_axis
                && bound_axis == axis_code
            {
                let active = match direction {
                    AxisDirection::Positive => value > AXIS_THRESHOLD,
                    AxisDirection::Negative => value < -AXIS_THRESHOLD,
                };
                if lane < self.axis_active.len() && active != self.axis_active[lane] {
                    self.axis_active[lane] = active;
                    events.push(KeyEvent {
                        key: lane,
                        pressed: active,
                        time_us,
                    });
                }
            }
        }
    }

    /// Update the key configuration.
    pub fn set_key_config(&mut self, key_config: KeyConfig) {
        self.axis_active = vec![false; key_config.bindings.len()];
        self.key_config = key_config;
    }

    /// Reset axis tracking state.
    pub fn reset(&mut self) {
        self.axis_active.fill(false);
    }
}

/// Convert gilrs button to a u32 code.
fn button_to_u32(button: gilrs::Button) -> u32 {
    match button {
        gilrs::Button::South => 0,
        gilrs::Button::East => 1,
        gilrs::Button::North => 2,
        gilrs::Button::West => 3,
        gilrs::Button::LeftTrigger => 4,
        gilrs::Button::LeftTrigger2 => 5,
        gilrs::Button::RightTrigger => 6,
        gilrs::Button::RightTrigger2 => 7,
        gilrs::Button::Select => 8,
        gilrs::Button::Start => 9,
        gilrs::Button::LeftThumb => 10,
        gilrs::Button::RightThumb => 11,
        gilrs::Button::DPadUp => 12,
        gilrs::Button::DPadDown => 13,
        gilrs::Button::DPadLeft => 14,
        gilrs::Button::DPadRight => 15,
        _ => 255,
    }
}

/// Convert gilrs axis to a u32 code.
fn axis_to_u32(axis: gilrs::Axis) -> u32 {
    match axis {
        gilrs::Axis::LeftStickX => 0,
        gilrs::Axis::LeftStickY => 1,
        gilrs::Axis::RightStickX => 2,
        gilrs::Axis::RightStickY => 3,
        _ => 255,
    }
}
