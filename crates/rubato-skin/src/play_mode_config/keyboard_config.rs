use bms::model::mode::Mode;

use super::MouseScratchConfig;
use super::gdx_keys;

// -- KeyboardConfig --

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct KeyboardConfig {
    #[serde(rename = "mouseScratchConfig")]
    pub mouse_scratch_config: MouseScratchConfig,
    pub keys: Vec<i32>,
    pub start: i32,
    pub select: i32,
    pub duration: i32,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        KeyboardConfig::new(Mode::BEAT_14K, true)
    }
}

impl KeyboardConfig {
    pub fn new(mode: Mode, enable: bool) -> Self {
        let mut config = KeyboardConfig {
            mouse_scratch_config: MouseScratchConfig::new(mode),
            keys: Vec::new(),
            start: 0,
            select: 0,
            duration: 16,
        };
        config.set_key_assign(mode, enable);
        config
    }

    pub fn set_key_assign(&mut self, mode: Mode, enable: bool) {
        self.keys = match mode {
            Mode::BEAT_5K => vec![
                gdx_keys::Z,
                gdx_keys::S,
                gdx_keys::X,
                gdx_keys::D,
                gdx_keys::C,
                gdx_keys::SHIFT_LEFT,
                gdx_keys::CONTROL_LEFT,
            ],
            Mode::BEAT_7K => vec![
                gdx_keys::Z,
                gdx_keys::S,
                gdx_keys::X,
                gdx_keys::D,
                gdx_keys::C,
                gdx_keys::F,
                gdx_keys::V,
                gdx_keys::SHIFT_LEFT,
                gdx_keys::CONTROL_LEFT,
            ],
            Mode::BEAT_10K => vec![
                gdx_keys::Z,
                gdx_keys::S,
                gdx_keys::X,
                gdx_keys::D,
                gdx_keys::C,
                gdx_keys::SHIFT_LEFT,
                gdx_keys::CONTROL_LEFT,
                gdx_keys::COMMA,
                gdx_keys::L,
                gdx_keys::PERIOD,
                gdx_keys::SEMICOLON,
                gdx_keys::SLASH,
                gdx_keys::SHIFT_RIGHT,
                gdx_keys::CONTROL_RIGHT,
            ],
            Mode::POPN_5K | Mode::POPN_9K => vec![
                gdx_keys::Z,
                gdx_keys::S,
                gdx_keys::X,
                gdx_keys::D,
                gdx_keys::C,
                gdx_keys::F,
                gdx_keys::V,
                gdx_keys::G,
                gdx_keys::B,
            ],
            Mode::KEYBOARD_24K => {
                let mut keys = vec![
                    gdx_keys::Z,
                    gdx_keys::S,
                    gdx_keys::X,
                    gdx_keys::D,
                    gdx_keys::C,
                    gdx_keys::F,
                    gdx_keys::V,
                    gdx_keys::SHIFT_LEFT,
                    gdx_keys::CONTROL_LEFT,
                    gdx_keys::COMMA,
                    gdx_keys::L,
                    gdx_keys::PERIOD,
                    gdx_keys::SEMICOLON,
                    gdx_keys::SLASH,
                    gdx_keys::APOSTROPHE,
                    gdx_keys::UNKNOWN,
                    gdx_keys::SHIFT_RIGHT,
                    gdx_keys::CONTROL_RIGHT,
                ];
                keys.resize(26, 0);
                keys
            }
            Mode::KEYBOARD_24K_DOUBLE => {
                let mut keys = vec![
                    gdx_keys::Z,
                    gdx_keys::S,
                    gdx_keys::X,
                    gdx_keys::D,
                    gdx_keys::C,
                    gdx_keys::F,
                    gdx_keys::V,
                    gdx_keys::SHIFT_LEFT,
                    gdx_keys::CONTROL_LEFT,
                    gdx_keys::COMMA,
                    gdx_keys::L,
                    gdx_keys::PERIOD,
                    gdx_keys::SEMICOLON,
                    gdx_keys::SLASH,
                    gdx_keys::APOSTROPHE,
                    gdx_keys::UNKNOWN,
                    gdx_keys::SHIFT_RIGHT,
                    gdx_keys::CONTROL_RIGHT,
                ];
                keys.resize(52, 0);
                keys
            }
            // BEAT_14K and default
            _ => vec![
                gdx_keys::Z,
                gdx_keys::S,
                gdx_keys::X,
                gdx_keys::D,
                gdx_keys::C,
                gdx_keys::F,
                gdx_keys::V,
                gdx_keys::SHIFT_LEFT,
                gdx_keys::CONTROL_LEFT,
                gdx_keys::COMMA,
                gdx_keys::L,
                gdx_keys::PERIOD,
                gdx_keys::SEMICOLON,
                gdx_keys::SLASH,
                gdx_keys::APOSTROPHE,
                gdx_keys::UNKNOWN,
                gdx_keys::SHIFT_RIGHT,
                gdx_keys::CONTROL_RIGHT,
            ],
        };
        if !enable {
            for k in &mut self.keys {
                *k = -1;
            }
        }
        self.start = gdx_keys::Q;
        self.select = gdx_keys::W;
    }
}
