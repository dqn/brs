use crate::input::key_config::GamepadConfig;
use crate::input::key_state::KeyState;
use crate::model::note::LANE_COUNT;
use gilrs::{Axis, Button, Event, EventType, GamepadId};

/// Tracks axis position for scratch detection.
#[derive(Debug, Default)]
struct AxisState {
    is_active: bool,
}

/// Gamepad input handler using gilrs.
pub struct GamepadInput {
    gamepad_id: GamepadId,
    axis_state: AxisState,
}

impl GamepadInput {
    /// Create a new gamepad input handler.
    pub fn new(gamepad_id: GamepadId) -> Self {
        Self {
            gamepad_id,
            axis_state: AxisState::default(),
        }
    }

    /// Get the gamepad ID.
    pub fn gamepad_id(&self) -> GamepadId {
        self.gamepad_id
    }

    /// Process a gamepad event and update key states.
    /// Returns indices of lanes that had state changes (lane_idx, pressed).
    pub fn process_event(
        &mut self,
        event: &Event,
        config: &GamepadConfig,
        states: &mut [KeyState; LANE_COUNT],
        time_us: u64,
    ) -> Vec<(usize, bool)> {
        if event.id != self.gamepad_id {
            return Vec::new();
        }

        let mut changes = Vec::new();

        match event.event {
            EventType::ButtonPressed(button, _) => {
                if let Some(lane_idx) = self.button_to_lane(button, config) {
                    states[lane_idx].on_press(time_us);
                    changes.push((lane_idx, true));
                }
            }
            EventType::ButtonReleased(button, _) => {
                if let Some(lane_idx) = self.button_to_lane(button, config) {
                    states[lane_idx].on_release(time_us);
                    changes.push((lane_idx, false));
                }
            }
            EventType::AxisChanged(axis, value, _) => {
                if let Some(change) =
                    self.process_scratch_axis(axis, value, config, states, time_us)
                {
                    changes.push(change);
                }
            }
            _ => {}
        }

        changes
    }

    fn button_to_lane(&self, button: Button, config: &GamepadConfig) -> Option<usize> {
        let button_name = button_to_string(button);
        config
            .lanes
            .iter()
            .position(|b| b.as_deref() == Some(&button_name))
    }

    fn process_scratch_axis(
        &mut self,
        axis: Axis,
        value: f32,
        config: &GamepadConfig,
        states: &mut [KeyState; LANE_COUNT],
        time_us: u64,
    ) -> Option<(usize, bool)> {
        let scratch_axis_name = config.scratch_axis.as_deref()?;
        let axis_name = axis_to_string(axis);

        if axis_name != scratch_axis_name {
            return None;
        }

        let was_active = self.axis_state.is_active;
        let is_active = value.abs() > config.axis_threshold;
        self.axis_state.is_active = is_active;

        // Scratch is lane 0
        if is_active && !was_active {
            states[0].on_press(time_us);
            Some((0, true))
        } else if !is_active && was_active {
            states[0].on_release(time_us);
            Some((0, false))
        } else {
            None
        }
    }
}

/// Convert Button to string for comparison with config.
fn button_to_string(button: Button) -> String {
    match button {
        Button::South => "South",
        Button::East => "East",
        Button::North => "North",
        Button::West => "West",
        Button::C => "C",
        Button::Z => "Z",
        Button::LeftTrigger => "LeftTrigger",
        Button::LeftTrigger2 => "LeftTrigger2",
        Button::RightTrigger => "RightTrigger",
        Button::RightTrigger2 => "RightTrigger2",
        Button::Select => "Select",
        Button::Start => "Start",
        Button::Mode => "Mode",
        Button::LeftThumb => "LeftThumb",
        Button::RightThumb => "RightThumb",
        Button::DPadUp => "DPadUp",
        Button::DPadDown => "DPadDown",
        Button::DPadLeft => "DPadLeft",
        Button::DPadRight => "DPadRight",
        Button::Unknown => "Unknown",
    }
    .to_string()
}

/// Convert Axis to string for comparison with config.
fn axis_to_string(axis: Axis) -> String {
    match axis {
        Axis::LeftStickX => "LeftStickX",
        Axis::LeftStickY => "LeftStickY",
        Axis::LeftZ => "LeftZ",
        Axis::RightStickX => "RightStickX",
        Axis::RightStickY => "RightStickY",
        Axis::RightZ => "RightZ",
        Axis::DPadX => "DPadX",
        Axis::DPadY => "DPadY",
        Axis::Unknown => "Unknown",
    }
    .to_string()
}
