use macroquad::prelude::*;

use crate::bms::NoteChannel;

pub struct InputHandler {
    key_bindings: [KeyCode; 8],
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
        }
    }

    pub fn is_lane_pressed(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index();
        is_key_pressed(self.key_bindings[lane])
    }

    // Public API for checking if a lane key is held down
    #[allow(dead_code)]
    pub fn is_lane_down(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index();
        is_key_down(self.key_bindings[lane])
    }

    pub fn is_lane_released(&self, channel: NoteChannel) -> bool {
        let lane = channel.lane_index();
        is_key_released(self.key_bindings[lane])
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
