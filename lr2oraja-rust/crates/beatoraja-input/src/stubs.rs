// Config types re-exported from beatoraja-types
pub use beatoraja_types::config::Config;
pub use beatoraja_types::play_mode_config::{
    ControllerConfig, KeyboardConfig, MidiConfig, MidiInput, MidiInputType, MouseScratchConfig,
    PlayModeConfig,
};
pub use beatoraja_types::player_config::PlayerConfig;
pub use beatoraja_types::resolution::Resolution;

// Real implementations moved to dedicated modules (Phase 25a)
pub use crate::gdx_compat::{GdxGraphics, GdxInput, get_shared_key_state, set_shared_key_state};
pub use crate::keys::Keys;

/// Stub for SkinWidgetManager
pub struct SkinWidgetManager;

impl SkinWidgetManager {
    pub fn get_focus() -> bool {
        false
    }
}

/// Controller state wrapper (com.badlogic.gdx.controllers.Controller)
///
/// Holds button/axis state that is updated by the controller manager each frame.
pub struct Controller {
    name: String,
    pub button_state: Vec<bool>,
    pub axis_state: Vec<f32>,
}

impl Controller {
    pub fn new(name: String) -> Self {
        Self {
            name,
            button_state: Vec::new(),
            axis_state: Vec::new(),
        }
    }

    pub fn with_state(name: String, num_buttons: usize, num_axes: usize) -> Self {
        Self {
            name,
            button_state: vec![false; num_buttons],
            axis_state: vec![0.0; num_axes],
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_button(&self, button: i32) -> bool {
        if button >= 0 && (button as usize) < self.button_state.len() {
            self.button_state[button as usize]
        } else {
            false
        }
    }

    pub fn get_axis(&self, axis: i32) -> f32 {
        if axis >= 0 && (axis as usize) < self.axis_state.len() {
            self.axis_state[axis as usize]
        } else {
            0.0
        }
    }
}
