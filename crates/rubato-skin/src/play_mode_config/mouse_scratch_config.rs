use bms::model::mode::Mode;

// -- MouseScratchConfig --

pub const MOUSE_SCRATCH_VER_2: i32 = 0;
pub const MOUSE_SCRATCH_VER_1: i32 = 1;

const MOUSESCRATCH_STRING: [&str; 4] = ["MOUSE RIGHT", "MOUSE LEFT", "MOUSE DOWN", "MOUSE UP"];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct MouseScratchConfig {
    pub keys: Vec<i32>,
    pub start: i32,
    pub select: i32,
    #[serde(rename = "mouseScratchEnabled")]
    pub mouse_scratch_enabled: bool,
    #[serde(rename = "mouseScratchTimeThreshold")]
    pub mouse_scratch_time_threshold: i32,
    #[serde(rename = "mouseScratchDistance")]
    pub mouse_scratch_distance: i32,
    #[serde(rename = "mouseScratchMode")]
    pub mouse_scratch_mode: i32,
}

impl Default for MouseScratchConfig {
    fn default() -> Self {
        MouseScratchConfig::new(Mode::BEAT_7K)
    }
}

impl MouseScratchConfig {
    pub fn new(mode: Mode) -> Self {
        let mut config = MouseScratchConfig {
            keys: Vec::new(),
            start: -1,
            select: -1,
            mouse_scratch_enabled: false,
            mouse_scratch_time_threshold: 150,
            mouse_scratch_distance: 12,
            mouse_scratch_mode: 0,
        };
        config.set_key_assign(mode);
        config
    }

    pub fn set_key_assign(&mut self, mode: Mode) {
        let len = match mode {
            Mode::BEAT_5K => 7,
            Mode::BEAT_7K => 9,
            Mode::BEAT_10K => 14,
            Mode::POPN_5K | Mode::POPN_9K => 9,
            Mode::KEYBOARD_24K => 26,
            Mode::KEYBOARD_24K_DOUBLE => 52,
            // BEAT_14K and default
            _ => 18,
        };
        self.keys = vec![-1; len];
        self.start = -1;
        self.select = -1;
    }

    pub fn key_string(&self, index: usize) -> Option<&'static str> {
        let key = *self.keys.get(index)?;
        if key < 0 || (key as usize) >= MOUSESCRATCH_STRING.len() {
            return None;
        }
        Some(MOUSESCRATCH_STRING[key as usize])
    }

    pub fn start_string(&self) -> Option<&'static str> {
        if self.start < 0 || (self.start as usize) >= MOUSESCRATCH_STRING.len() {
            return None;
        }
        Some(MOUSESCRATCH_STRING[self.start as usize])
    }

    pub fn select_string(&self) -> Option<&'static str> {
        if self.select < 0 || (self.select as usize) >= MOUSESCRATCH_STRING.len() {
            return None;
        }
        Some(MOUSESCRATCH_STRING[self.select as usize])
    }

    pub fn set_mouse_scratch_time_threshold(&mut self, value: i32) {
        self.mouse_scratch_time_threshold = if value > 0 { value } else { 1 };
    }

    pub fn set_mouse_scratch_distance(&mut self, value: i32) {
        self.mouse_scratch_distance = if value > 0 { value } else { 1 };
    }
}
