use std::collections::HashMap;

use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};

use crate::config::ControllerBindings;

/// Controller input type: either a button or an axis direction
#[derive(Debug, Clone)]
pub enum ControllerInput {
    Button(Button),
    /// Axis input with direction (positive = true for +, false for -)
    #[allow(dead_code)] // For future IIDX turntable support
    Axis {
        axis: Axis,
        positive: bool,
    },
}

/// Gamepad handler for controller input (supports IIDX-style controllers)
pub struct GamepadHandler {
    gilrs: Gilrs,
    active_gamepad: Option<GamepadId>,
    button_states: [bool; 8],
    prev_button_states: [bool; 8],
    input_mappings: [ControllerInput; 8],
    prev_axis_values: HashMap<Axis, f32>,
    axis_threshold: f32,
}

impl GamepadHandler {
    /// Create with default button mappings (for generic gamepads)
    pub fn new() -> Option<Self> {
        let gilrs = Gilrs::new().ok()?;
        let active_gamepad = gilrs.gamepads().next().map(|(id, _)| id);

        Some(Self {
            gilrs,
            active_gamepad,
            button_states: [false; 8],
            prev_button_states: [false; 8],
            input_mappings: [
                ControllerInput::Button(Button::LeftTrigger),  // Scratch
                ControllerInput::Button(Button::South),        // Key1
                ControllerInput::Button(Button::East),         // Key2
                ControllerInput::Button(Button::West),         // Key3
                ControllerInput::Button(Button::North),        // Key4
                ControllerInput::Button(Button::LeftTrigger2), // Key5
                ControllerInput::Button(Button::RightTrigger), // Key6
                ControllerInput::Button(Button::RightTrigger2), // Key7
            ],
            prev_axis_values: HashMap::new(),
            axis_threshold: 0.3,
        })
    }

    /// Create with custom bindings (for IIDX controllers with turntable)
    #[allow(dead_code)] // For future custom controller configuration
    pub fn with_bindings(bindings: &ControllerBindings) -> Option<Self> {
        let gilrs = Gilrs::new().ok()?;
        let active_gamepad = gilrs.gamepads().next().map(|(id, _)| id);

        let input_mappings = [
            parse_controller_binding(&bindings.scratch)?,
            parse_controller_binding(&bindings.key1)?,
            parse_controller_binding(&bindings.key2)?,
            parse_controller_binding(&bindings.key3)?,
            parse_controller_binding(&bindings.key4)?,
            parse_controller_binding(&bindings.key5)?,
            parse_controller_binding(&bindings.key6)?,
            parse_controller_binding(&bindings.key7)?,
        ];

        Some(Self {
            gilrs,
            active_gamepad,
            button_states: [false; 8],
            prev_button_states: [false; 8],
            input_mappings,
            prev_axis_values: HashMap::new(),
            axis_threshold: bindings.axis_threshold,
        })
    }

    /// Poll and process gamepad events
    pub fn update(&mut self) {
        // Save previous states
        self.prev_button_states = self.button_states;

        // Process events to track active gamepad
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::Connected => {
                    if self.active_gamepad.is_none() {
                        self.active_gamepad = Some(id);
                    }
                }
                EventType::Disconnected => {
                    if Some(id) == self.active_gamepad {
                        self.active_gamepad = None;
                        self.button_states = [false; 8];
                        self.prev_axis_values.clear();
                    }
                }
                _ => {
                    if self.active_gamepad.is_none() {
                        self.active_gamepad = Some(id);
                    }
                }
            }
        }

        // Update button states based on current input mappings
        let gamepad = self
            .active_gamepad
            .and_then(|id| self.gilrs.connected_gamepad(id));

        for (lane, input) in self.input_mappings.iter().enumerate() {
            self.button_states[lane] = match input {
                ControllerInput::Button(btn) => gamepad.is_some_and(|gp| gp.is_pressed(*btn)),
                ControllerInput::Axis { axis, positive } => {
                    if let Some(gp) = gamepad {
                        let value = gp.axis_data(*axis).map(|d| d.value()).unwrap_or(0.0);
                        let prev = self.prev_axis_values.get(axis).copied().unwrap_or(0.0);
                        let delta = value - prev;
                        self.prev_axis_values.insert(*axis, value);

                        // Detect axis movement in the specified direction
                        if *positive {
                            delta > self.axis_threshold
                        } else {
                            delta < -self.axis_threshold
                        }
                    } else {
                        false
                    }
                }
            };
        }
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_pressed(&self, lane: usize) -> bool {
        lane < 8 && self.button_states[lane] && !self.prev_button_states[lane]
    }

    /// Check if a button was just released this frame
    pub fn is_button_released(&self, lane: usize) -> bool {
        lane < 8 && !self.button_states[lane] && self.prev_button_states[lane]
    }

    /// Check if a button is currently held
    #[allow(dead_code)]
    pub fn is_button_down(&self, lane: usize) -> bool {
        lane < 8 && self.button_states[lane]
    }

    /// Check if a gamepad is connected
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.active_gamepad.is_some()
    }
}

/// Parse a controller binding string into a ControllerInput
///
/// Format:
/// - "Button:South" - Button input
/// - "Axis:LeftStickX:+" - Axis input (positive direction)
/// - "Axis:LeftStickX:-" - Axis input (negative direction)
#[allow(dead_code)] // For future custom controller configuration
pub fn parse_controller_binding(s: &str) -> Option<ControllerInput> {
    if let Some(rest) = s.strip_prefix("Button:") {
        let button = match rest {
            "South" => Button::South,
            "East" => Button::East,
            "North" => Button::North,
            "West" => Button::West,
            "LeftTrigger" => Button::LeftTrigger,
            "RightTrigger" => Button::RightTrigger,
            "LeftTrigger2" => Button::LeftTrigger2,
            "RightTrigger2" => Button::RightTrigger2,
            "LeftThumb" => Button::LeftThumb,
            "RightThumb" => Button::RightThumb,
            "Start" => Button::Start,
            "Select" => Button::Select,
            "DPadUp" => Button::DPadUp,
            "DPadDown" => Button::DPadDown,
            "DPadLeft" => Button::DPadLeft,
            "DPadRight" => Button::DPadRight,
            _ => return None,
        };
        Some(ControllerInput::Button(button))
    } else if let Some(rest) = s.strip_prefix("Axis:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let axis = match parts[0] {
            "LeftStickX" => Axis::LeftStickX,
            "LeftStickY" => Axis::LeftStickY,
            "RightStickX" => Axis::RightStickX,
            "RightStickY" => Axis::RightStickY,
            "LeftZ" => Axis::LeftZ,
            "RightZ" => Axis::RightZ,
            "DPadX" => Axis::DPadX,
            "DPadY" => Axis::DPadY,
            _ => return None,
        };
        let positive = parts[1] == "+";
        Some(ControllerInput::Axis { axis, positive })
    } else {
        None
    }
}
