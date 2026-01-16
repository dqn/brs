use macroquad::prelude::*;

use super::gamepad::GamepadHandler;
use crate::bms::{NoteChannel, PlayMode};

pub struct InputHandler {
    /// Key bindings for BMS 7-key mode
    key_bindings_bms: [KeyCode; 8],
    /// Key bindings for PMS 9-key mode
    key_bindings_pms: [KeyCode; 9],
    /// Current play mode
    play_mode: PlayMode,
    gamepad: Option<GamepadHandler>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            key_bindings_bms: [
                KeyCode::LeftShift, // Scratch
                KeyCode::Z,         // Key1
                KeyCode::S,         // Key2
                KeyCode::X,         // Key3
                KeyCode::D,         // Key4
                KeyCode::C,         // Key5
                KeyCode::F,         // Key6
                KeyCode::V,         // Key7
            ],
            key_bindings_pms: [
                KeyCode::A,         // Key1
                KeyCode::S,         // Key2
                KeyCode::D,         // Key3
                KeyCode::F,         // Key4
                KeyCode::Space,     // Key5
                KeyCode::J,         // Key6
                KeyCode::K,         // Key7
                KeyCode::L,         // Key8
                KeyCode::Semicolon, // Key9
            ],
            play_mode: PlayMode::Bms7Key,
            gamepad: GamepadHandler::new(),
        }
    }

    /// Create InputHandler for a specific play mode
    #[allow(dead_code)]
    pub fn for_mode(mode: PlayMode) -> Self {
        let mut handler = Self::new();
        handler.play_mode = mode;
        handler
    }

    /// Create InputHandler with custom BMS key bindings
    pub fn with_bindings(bindings: [KeyCode; 8]) -> Self {
        let mut handler = Self::new();
        handler.key_bindings_bms = bindings;
        handler
    }

    /// Create InputHandler with custom PMS key bindings
    #[allow(dead_code)]
    pub fn with_pms_bindings(bindings: [KeyCode; 9]) -> Self {
        let mut handler = Self::new();
        handler.key_bindings_pms = bindings;
        handler.play_mode = PlayMode::Pms9Key;
        handler
    }

    /// Set BMS key bindings
    #[allow(dead_code)]
    pub fn set_bms_bindings(&mut self, bindings: [KeyCode; 8]) {
        self.key_bindings_bms = bindings;
    }

    /// Set PMS key bindings
    #[allow(dead_code)]
    pub fn set_pms_bindings(&mut self, bindings: [KeyCode; 9]) {
        self.key_bindings_pms = bindings;
    }

    /// Set play mode
    #[allow(dead_code)]
    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.play_mode = mode;
    }

    /// Get current play mode
    #[allow(dead_code)]
    pub fn play_mode(&self) -> PlayMode {
        self.play_mode
    }

    /// Set binding for a specific lane (BMS mode)
    #[allow(dead_code)]
    pub fn set_binding(&mut self, lane: usize, key: KeyCode) {
        if lane < 8 {
            self.key_bindings_bms[lane] = key;
        }
    }

    /// Get current BMS key bindings
    #[allow(dead_code)]
    pub fn bindings(&self) -> &[KeyCode; 8] {
        &self.key_bindings_bms
    }

    /// Update gamepad state (must be called each frame)
    pub fn update(&mut self) {
        if let Some(gamepad) = &mut self.gamepad {
            gamepad.update();
        }
    }

    pub fn is_lane_pressed(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index_for_mode(self.play_mode);
        let keyboard = match self.play_mode {
            PlayMode::Bms7Key => {
                if lane < 8 {
                    is_key_pressed(self.key_bindings_bms[lane])
                } else {
                    false
                }
            }
            PlayMode::Pms9Key => {
                if lane < 9 {
                    is_key_pressed(self.key_bindings_pms[lane])
                } else {
                    false
                }
            }
        };
        let gamepad = self
            .gamepad
            .as_ref()
            .is_some_and(|g| g.is_button_pressed(lane));
        keyboard || gamepad
    }

    // Public API for checking if a lane key is held down
    #[allow(dead_code)]
    pub fn is_lane_down(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index_for_mode(self.play_mode);
        let keyboard = match self.play_mode {
            PlayMode::Bms7Key => {
                if lane < 8 {
                    is_key_down(self.key_bindings_bms[lane])
                } else {
                    false
                }
            }
            PlayMode::Pms9Key => {
                if lane < 9 {
                    is_key_down(self.key_bindings_pms[lane])
                } else {
                    false
                }
            }
        };
        let gamepad = self
            .gamepad
            .as_ref()
            .is_some_and(|g| g.is_button_down(lane));
        keyboard || gamepad
    }

    pub fn is_lane_released(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index_for_mode(self.play_mode);
        let keyboard = match self.play_mode {
            PlayMode::Bms7Key => {
                if lane < 8 {
                    is_key_released(self.key_bindings_bms[lane])
                } else {
                    false
                }
            }
            PlayMode::Pms9Key => {
                if lane < 9 {
                    is_key_released(self.key_bindings_pms[lane])
                } else {
                    false
                }
            }
        };
        let gamepad = self
            .gamepad
            .as_ref()
            .is_some_and(|g| g.is_button_released(lane));
        keyboard || gamepad
    }

    /// Get pressed lanes for BMS 7-key mode
    pub fn get_pressed_lanes(&self) -> Vec<NoteChannel> {
        match self.play_mode {
            PlayMode::Bms7Key => self.get_pressed_lanes_bms(),
            PlayMode::Pms9Key => self.get_pressed_lanes_pms(),
        }
    }

    /// Get released lanes for current mode
    pub fn get_released_lanes(&self) -> Vec<NoteChannel> {
        match self.play_mode {
            PlayMode::Bms7Key => self.get_released_lanes_bms(),
            PlayMode::Pms9Key => self.get_released_lanes_pms(),
        }
    }

    fn get_pressed_lanes_bms(&self) -> Vec<NoteChannel> {
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

    fn get_pressed_lanes_pms(&self) -> Vec<NoteChannel> {
        let channels = [
            NoteChannel::Key1,
            NoteChannel::Key2,
            NoteChannel::Key3,
            NoteChannel::Key4,
            NoteChannel::Key5,
            NoteChannel::Key6,
            NoteChannel::Key7,
            NoteChannel::Key8,
            NoteChannel::Key9,
        ];

        channels
            .into_iter()
            .filter(|&ch| self.is_lane_pressed(ch))
            .collect()
    }

    fn get_released_lanes_bms(&self) -> Vec<NoteChannel> {
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

    fn get_released_lanes_pms(&self) -> Vec<NoteChannel> {
        let channels = [
            NoteChannel::Key1,
            NoteChannel::Key2,
            NoteChannel::Key3,
            NoteChannel::Key4,
            NoteChannel::Key5,
            NoteChannel::Key6,
            NoteChannel::Key7,
            NoteChannel::Key8,
            NoteChannel::Key9,
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
