mod controller_config;
mod keyboard_config;
mod midi_config;
mod mouse_scratch_config;

pub use controller_config::*;
pub use keyboard_config::*;
pub use midi_config::*;
pub use mouse_scratch_config::*;

use crate::skin::bm_keys::BMKeys;
use bms::model::mode::Mode;

use crate::skin::play_config::PlayConfig;

// libGDX Keys constants (from com.badlogic.gdx.Input.Keys)
pub(crate) mod gdx_keys {
    pub const UNKNOWN: i32 = 0;
    pub const SHIFT_LEFT: i32 = 59;
    pub const SHIFT_RIGHT: i32 = 60;
    pub const CONTROL_LEFT: i32 = 129;
    pub const CONTROL_RIGHT: i32 = 130;
    pub const Z: i32 = 54;
    pub const S: i32 = 47;
    pub const X: i32 = 52;
    pub const D: i32 = 32;
    pub const C: i32 = 31;
    pub const F: i32 = 34;
    pub const V: i32 = 50;
    pub const G: i32 = 35;
    pub const B: i32 = 30;
    pub const Q: i32 = 45;
    pub const W: i32 = 51;
    pub const COMMA: i32 = 55;
    pub const L: i32 = 40;
    pub const PERIOD: i32 = 56;
    pub const SEMICOLON: i32 = 74;
    pub const SLASH: i32 = 76;
    pub const APOSTROPHE: i32 = 75;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PlayModeConfig {
    pub playconfig: PlayConfig,
    pub keyboard: KeyboardConfig,
    pub controller: Vec<ControllerConfig>,
    pub midi: MidiConfig,
    pub version: i32,
}

impl Default for PlayModeConfig {
    fn default() -> Self {
        PlayModeConfig::new(Mode::BEAT_7K)
    }
}

impl PlayModeConfig {
    pub fn new(mode: Mode) -> Self {
        let is_midi = mode == Mode::KEYBOARD_24K || mode == Mode::KEYBOARD_24K_DOUBLE;
        let keyboard = KeyboardConfig::new(mode, !is_midi);
        let player_count = mode.player() as usize;
        let controller: Vec<ControllerConfig> = (0..player_count)
            .map(|i| ControllerConfig::new_with_mode(mode, i as i32, false))
            .collect();
        let midi = MidiConfig::new(mode, is_midi);
        PlayModeConfig {
            playconfig: PlayConfig::default(),
            keyboard,
            controller,
            midi,
            version: 0,
        }
    }

    pub fn new_with_configs(
        keyboard: KeyboardConfig,
        controllers: Vec<ControllerConfig>,
        midi: MidiConfig,
    ) -> Self {
        PlayModeConfig {
            playconfig: PlayConfig::default(),
            keyboard,
            controller: controllers,
            midi,
            version: 0,
        }
    }

    pub fn validate(&mut self, keys: usize) {
        self.playconfig.validate();

        if self.keyboard.keys.is_empty() {
            self.keyboard.keys = vec![
                gdx_keys::Z,
                gdx_keys::S,
                gdx_keys::X,
                gdx_keys::D,
                gdx_keys::C,
                gdx_keys::F,
                gdx_keys::V,
                gdx_keys::SHIFT_LEFT,
                gdx_keys::CONTROL_LEFT,
            ];
        }
        if self.keyboard.keys.len() != keys {
            self.keyboard.keys.resize(keys, 0);
        }
        self.keyboard.duration = self.keyboard.duration.clamp(0, 100);

        let mousescratch = &mut self.keyboard.mouse_scratch_config;
        if mousescratch.keys.len() != keys {
            mousescratch.keys = vec![-1; keys];
        }
        mousescratch.mouse_scratch_distance = mousescratch.mouse_scratch_distance.clamp(1, 10000);
        mousescratch.mouse_scratch_time_threshold =
            mousescratch.mouse_scratch_time_threshold.clamp(1, 10000);

        let mut index = 0usize;
        for c in &mut self.controller {
            if c.keys.is_empty() {
                c.keys = vec![
                    BMKeys::BUTTON_4,
                    BMKeys::BUTTON_7,
                    BMKeys::BUTTON_3,
                    BMKeys::BUTTON_8,
                    BMKeys::BUTTON_2,
                    BMKeys::BUTTON_5,
                    BMKeys::AXIS2_PLUS,
                    BMKeys::AXIS1_PLUS,
                    BMKeys::AXIS1_MINUS,
                ];
            }
            if c.keys.len() != keys {
                let mut newkeys = vec![-1i32; keys];
                for &key in &c.keys {
                    if index < newkeys.len() {
                        newkeys[index] = key;
                        index += 1;
                    }
                }
                c.keys = newkeys;
            }
            c.duration = c.duration.clamp(0, 100);
        }

        // Button count extension (16->32) conversion (0.8.1 -> 0.8.2)
        if self.version == 0 {
            for c in &mut self.controller {
                for key in &mut c.keys {
                    if *key >= BMKeys::BUTTON_17 && *key <= BMKeys::BUTTON_20 {
                        *key += BMKeys::AXIS1_PLUS - BMKeys::BUTTON_17;
                    }
                }
            }
            self.version = 1;
        }

        if self.midi.keys.is_empty() {
            self.midi = MidiConfig::new(Mode::BEAT_7K, true);
        }
        if self.midi.keys.len() != keys {
            self.midi.keys.resize(keys, None);
        }

        // Exclusive processing for KB, controller, Midi buttons
        let mut exclusive = vec![false; self.keyboard.keys.len()];
        validate_exclusive(&mut self.keyboard.keys, &mut exclusive);
        for c in &mut self.controller {
            validate_exclusive(&mut c.keys, &mut exclusive);
        }

        for (i, key) in self.midi.keys.iter_mut().enumerate() {
            if exclusive[i] {
                *key = None;
            }
        }
    }
}

fn validate_exclusive(keys: &mut [i32], exclusive: &mut [bool]) {
    for (key, excl) in keys.iter_mut().zip(exclusive.iter_mut()) {
        if *excl {
            *key = -1;
        } else if *key != -1 {
            *excl = true;
        }
    }
}
