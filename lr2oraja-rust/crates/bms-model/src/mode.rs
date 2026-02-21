use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Mode {
    BEAT_5K,
    BEAT_7K,
    BEAT_10K,
    BEAT_14K,
    POPN_5K,
    POPN_9K,
    KEYBOARD_24K,
    KEYBOARD_24K_DOUBLE,
}

impl Mode {
    pub fn id(&self) -> i32 {
        match self {
            Mode::BEAT_5K => 5,
            Mode::BEAT_7K => 7,
            Mode::BEAT_10K => 10,
            Mode::BEAT_14K => 14,
            Mode::POPN_5K => 9,
            Mode::POPN_9K => 9,
            Mode::KEYBOARD_24K => 25,
            Mode::KEYBOARD_24K_DOUBLE => 50,
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Mode::BEAT_5K => "beat-5k",
            Mode::BEAT_7K => "beat-7k",
            Mode::BEAT_10K => "beat-10k",
            Mode::BEAT_14K => "beat-14k",
            Mode::POPN_5K => "popn-5k",
            Mode::POPN_9K => "popn-9k",
            Mode::KEYBOARD_24K => "keyboard-24k",
            Mode::KEYBOARD_24K_DOUBLE => "keyboard-24k-double",
        }
    }

    pub fn player(&self) -> i32 {
        match self {
            Mode::BEAT_5K => 1,
            Mode::BEAT_7K => 1,
            Mode::BEAT_10K => 2,
            Mode::BEAT_14K => 2,
            Mode::POPN_5K => 1,
            Mode::POPN_9K => 1,
            Mode::KEYBOARD_24K => 1,
            Mode::KEYBOARD_24K_DOUBLE => 2,
        }
    }

    pub fn key(&self) -> i32 {
        match self {
            Mode::BEAT_5K => 6,
            Mode::BEAT_7K => 8,
            Mode::BEAT_10K => 12,
            Mode::BEAT_14K => 16,
            Mode::POPN_5K => 5,
            Mode::POPN_9K => 9,
            Mode::KEYBOARD_24K => 26,
            Mode::KEYBOARD_24K_DOUBLE => 52,
        }
    }

    pub fn scratch_key(&self) -> &'static [i32] {
        match self {
            Mode::BEAT_5K => &[5],
            Mode::BEAT_7K => &[7],
            Mode::BEAT_10K => &[5, 11],
            Mode::BEAT_14K => &[7, 15],
            Mode::POPN_5K => &[],
            Mode::POPN_9K => &[],
            Mode::KEYBOARD_24K => &[24, 25],
            Mode::KEYBOARD_24K_DOUBLE => &[24, 25, 50, 51],
        }
    }

    pub fn is_scratch_key(&self, key: i32) -> bool {
        for sc in self.scratch_key() {
            if key == *sc {
                return true;
            }
        }
        false
    }

    pub fn get_mode(hint: &str) -> Option<Mode> {
        let modes = [
            Mode::BEAT_5K,
            Mode::BEAT_7K,
            Mode::BEAT_10K,
            Mode::BEAT_14K,
            Mode::POPN_5K,
            Mode::POPN_9K,
            Mode::KEYBOARD_24K,
            Mode::KEYBOARD_24K_DOUBLE,
        ];
        for mode in modes {
            if mode.hint() == hint {
                return Some(mode);
            }
        }
        None
    }
}
