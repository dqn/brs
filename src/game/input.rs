use macroquad::prelude::*;

use super::gamepad::GamepadHandler;
use crate::bms::NoteChannel;

pub struct InputHandler {
    key_bindings: [KeyCode; 8],
    gamepad: Option<GamepadHandler>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            key_bindings: [
                KeyCode::LeftShift, // Scratch
                KeyCode::Z,         // Key1
                KeyCode::S,         // Key2
                KeyCode::X,         // Key3
                KeyCode::D,         // Key4
                KeyCode::C,         // Key5
                KeyCode::F,         // Key6
                KeyCode::V,         // Key7
            ],
            gamepad: GamepadHandler::new(),
        }
    }

    /// Create InputHandler with custom key bindings
    pub fn with_bindings(bindings: [KeyCode; 8]) -> Self {
        Self {
            key_bindings: bindings,
            gamepad: GamepadHandler::new(),
        }
    }

    /// Set binding for a specific lane
    #[allow(dead_code)]
    pub fn set_binding(&mut self, lane: usize, key: KeyCode) {
        if lane < 8 {
            self.key_bindings[lane] = key;
        }
    }

    /// Get current key bindings
    #[allow(dead_code)]
    pub fn bindings(&self) -> &[KeyCode; 8] {
        &self.key_bindings
    }

    /// Update gamepad state (must be called each frame)
    pub fn update(&mut self) {
        if let Some(gamepad) = &mut self.gamepad {
            gamepad.update();
        }
    }

    pub fn is_lane_pressed(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index();
        let keyboard = is_key_pressed(self.key_bindings[lane]);
        let gamepad = self
            .gamepad
            .as_ref()
            .is_some_and(|g| g.is_button_pressed(lane));
        keyboard || gamepad
    }

    // Public API for checking if a lane key is held down
    #[allow(dead_code)]
    pub fn is_lane_down(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index();
        let keyboard = is_key_down(self.key_bindings[lane]);
        let gamepad = self
            .gamepad
            .as_ref()
            .is_some_and(|g| g.is_button_down(lane));
        keyboard || gamepad
    }

    pub fn is_lane_released(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index();
        let keyboard = is_key_released(self.key_bindings[lane]);
        let gamepad = self
            .gamepad
            .as_ref()
            .is_some_and(|g| g.is_button_released(lane));
        keyboard || gamepad
    }

    pub fn get_pressed_lanes(&self) -> Vec<NoteChannel> {
        let channels = [
            NoteChannel::Scratch,
            NoteChannel::Key1,
            NoteChannel::Key2,
            NoteChannel::Key3,
            NoteChannel::Key4,
            NoteChannel::Key5,
            NoteChannel::Key6,
            NoteChannel::Key7,
        ];

        channels
            .into_iter()
            .filter(|&ch| self.is_lane_pressed(ch))
            .collect()
    }

    pub fn get_released_lanes(&self) -> Vec<NoteChannel> {
        let channels = [
            NoteChannel::Scratch,
            NoteChannel::Key1,
            NoteChannel::Key2,
            NoteChannel::Key3,
            NoteChannel::Key4,
            NoteChannel::Key5,
            NoteChannel::Key6,
            NoteChannel::Key7,
        ];

        channels
            .into_iter()
            .filter(|&ch| self.is_lane_released(ch))
            .collect()
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
