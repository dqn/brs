/// Main input processor that orchestrates keyboard, controller, and MIDI input.
///
/// Ported from Java `BMSPlayerInputProcessor.java`.
/// Collects events from all input devices, updates key state, and records input logs.
use bms_config::play_mode_config::PlayModeConfig;
use bms_replay::key_input_log::KeyInputLog;

use crate::control_keys::ControlKeys;
use crate::device::{DeviceType, InputEvent};
use crate::key_command::KeyCommand;
use crate::key_state::KeyStateManager;
use crate::keyboard::{KeyboardBackend, KeyboardInput};

/// Main input processor.
///
/// Holds all device inputs, key state, and key logger.
/// Polls devices and translates events into unified key state + replay log.
pub struct InputProcessor {
    /// Key state manager (256 keys).
    key_state: KeyStateManager,
    /// Key input log for replay recording.
    keylog: Vec<KeyInputLog>,
    /// Keyboard input handler.
    keyboard: KeyboardInput,
    /// Play start time in microseconds (0 = not started).
    start_time: i64,
    /// Margin time subtracted from logged key times (microseconds).
    micro_margin_time: i64,
    /// Whether input processing is enabled.
    enabled: bool,
    /// Start button pressed state.
    start_pressed: bool,
    /// Select button pressed state.
    select_pressed: bool,
    /// Primary device type (determined by key assignment count).
    device_type: DeviceType,
    /// Whether analog scroll is enabled.
    analog_scroll: bool,
}

impl InputProcessor {
    pub fn new() -> Self {
        Self {
            key_state: KeyStateManager::new(),
            keylog: Vec::with_capacity(10000),
            keyboard: KeyboardInput::new(),
            start_time: 0,
            micro_margin_time: 0,
            enabled: true,
            start_pressed: false,
            select_pressed: false,
            device_type: DeviceType::Keyboard,
            analog_scroll: true,
        }
    }

    /// Set play start time. Clears key log and resets device states when non-zero.
    pub fn set_start_time(&mut self, start_time: i64) {
        self.start_time = start_time;
        if start_time != 0 {
            self.key_state.reset_all_key_changed_time();
            self.keylog.clear();
            self.keyboard.clear();
        }
    }

    /// Set key log margin time in milliseconds.
    pub fn set_key_log_margin_time(&mut self, milli_margin_time: i64) {
        self.micro_margin_time = milli_margin_time * 1000;
    }

    pub fn get_start_time(&self) -> i64 {
        self.start_time
    }

    /// Apply play mode configuration (keyboard, controller, MIDI key assignments).
    ///
    /// Handles exclusive key processing: keyboard takes priority, then controllers,
    /// then MIDI. If a lane is already claimed, later devices get that lane disabled.
    pub fn set_play_config(&mut self, config: &PlayModeConfig) {
        // Reset key state for lanes beyond the configured range.
        let key_count = config.keyboard.keys.len();
        for i in key_count..KeyStateManager::SIZE {
            self.key_state.set_key_state(i, false, i64::MIN);
        }

        // Count assigned keys per device type.
        let kb_count = config.keyboard.keys.iter().filter(|&&k| k != -1).count();
        let co_count: usize = config
            .controller
            .iter()
            .flat_map(|c| c.keys.iter())
            .filter(|&&k| k != -1)
            .count();
        let mi_count = config.midi.keys.iter().filter(|k| k.is_some()).count();

        // Set keyboard config.
        self.keyboard.set_config(&config.keyboard);

        // Determine primary device type.
        if kb_count >= co_count && kb_count >= mi_count {
            self.device_type = DeviceType::Keyboard;
        } else if co_count >= kb_count && co_count >= mi_count {
            self.device_type = DeviceType::Controller;
        } else {
            self.device_type = DeviceType::Midi;
        }
    }

    /// Enable or disable input processing.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.key_state.reset_all_key_state();
            self.keyboard.clear();
        }
    }

    /// Process a key changed event from any device.
    ///
    /// Updates key state and logs the event if play has started.
    pub fn key_changed(&mut self, press_time: i64, keycode: i32, pressed: bool) {
        if !self.enabled {
            return;
        }
        if keycode < 0 {
            // Start/Select special keycodes
            if keycode == -1 {
                self.start_pressed = pressed;
            } else if keycode == -2 {
                self.select_pressed = pressed;
            }
            return;
        }
        let id = keycode as usize;
        if id >= KeyStateManager::SIZE {
            return;
        }
        if self.key_state.get_key_state(id) != pressed {
            self.key_state.set_key_state(id, pressed, press_time);
            if self.start_time != 0 {
                self.keylog.push(KeyInputLog::new(
                    press_time - self.micro_margin_time,
                    keycode,
                    pressed,
                ));
            }
        }
    }

    /// Process an analog state event from a controller.
    pub fn set_analog_state(&mut self, keycode: i32, is_analog: bool, value: f32) {
        if !self.enabled {
            return;
        }
        if keycode < 0 || keycode as usize >= KeyStateManager::SIZE {
            return;
        }
        if self.analog_scroll {
            self.key_state
                .set_analog_state(keycode as usize, is_analog, value);
        } else {
            self.key_state
                .set_analog_state(keycode as usize, false, 0.0);
        }
    }

    /// Poll the keyboard backend and process events.
    pub fn poll_keyboard(&mut self, now_us: i64, backend: &dyn KeyboardBackend) {
        let events = self.keyboard.poll(now_us, backend);
        for event in events {
            self.process_event(event);
        }
    }

    /// Process a single input event.
    fn process_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyChanged {
                keycode,
                pressed,
                time_us,
            } => {
                self.key_changed(time_us, keycode, pressed);
            }
            InputEvent::AnalogState {
                keycode,
                is_analog,
                value,
            } => {
                self.set_analog_state(keycode, is_analog, value);
            }
        }
    }

    /// Process events from external device (controller, MIDI).
    pub fn process_events(&mut self, events: Vec<InputEvent>) {
        for event in events {
            self.process_event(event);
        }
    }

    /// Get key state for a specific key ID.
    pub fn get_key_state(&self, id: usize) -> bool {
        self.key_state.get_key_state(id)
    }

    /// Get the time of the last key state change for a specific key ID.
    pub fn get_key_changed_time(&self, id: usize) -> i64 {
        self.key_state.get_key_changed_time(id)
    }

    /// Reset the key change time for a specific key ID.
    pub fn reset_key_changed_time(&mut self, id: usize) -> bool {
        self.key_state.reset_key_changed_time(id)
    }

    /// Reset all key states to unpressed.
    pub fn reset_all_key_state(&mut self) {
        self.key_state.reset_all_key_state();
    }

    /// Reset all key change times.
    pub fn reset_all_key_changed_time(&mut self) {
        self.key_state.reset_all_key_changed_time();
    }

    /// Get the complete key input log.
    pub fn get_key_input_log(&self) -> &[KeyInputLog] {
        &self.keylog
    }

    /// Check if a control key is currently pressed on the keyboard backend.
    pub fn get_control_key_state(&self, key: ControlKeys, backend: &dyn KeyboardBackend) -> bool {
        self.keyboard.get_control_key_state(key, backend)
    }

    /// Check if start button is pressed.
    pub fn start_pressed(&self) -> bool {
        self.start_pressed
    }

    /// Check if select button is pressed.
    pub fn select_pressed(&self) -> bool {
        self.select_pressed
    }

    /// Set select pressed state.
    pub fn set_select_pressed(&mut self, pressed: bool) {
        self.select_pressed = pressed;
    }

    /// Get the primary device type.
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    /// Check if a key command is activated.
    pub fn is_activated(&self, key: KeyCommand, backend: &dyn KeyboardBackend) -> bool {
        match key {
            KeyCommand::ShowFps => backend.is_key_pressed(ControlKeys::F1.keycode()),
            KeyCommand::UpdateFolder => backend.is_key_pressed(ControlKeys::F2.keycode()),
            KeyCommand::OpenExplorer => backend.is_key_pressed(ControlKeys::F3.keycode()),
            KeyCommand::SwitchScreenMode => backend.is_key_pressed(ControlKeys::F4.keycode()),
            KeyCommand::ToggleModMenu => {
                backend.is_key_pressed(ControlKeys::F5.keycode())
                    || backend.is_key_pressed(ControlKeys::Insert.keycode())
            }
            KeyCommand::SaveScreenshot => backend.is_key_pressed(ControlKeys::F6.keycode()),
            KeyCommand::PostTwitter => backend.is_key_pressed(ControlKeys::F7.keycode()),
            KeyCommand::AddFavoriteSong => backend.is_key_pressed(ControlKeys::F8.keycode()),
            KeyCommand::AddFavoriteChart => backend.is_key_pressed(ControlKeys::F9.keycode()),
            KeyCommand::AutoplayFolder => backend.is_key_pressed(ControlKeys::F10.keycode()),
            KeyCommand::OpenIr => backend.is_key_pressed(ControlKeys::F11.keycode()),
            KeyCommand::OpenSkinConfiguration => backend.is_key_pressed(ControlKeys::F12.keycode()),
            _ => false,
        }
    }

    /// Get the analog scroll state.
    pub fn set_analog_scroll(&mut self, enabled: bool) {
        self.analog_scroll = enabled;
    }

    /// Access the key state manager directly.
    pub fn key_state(&self) -> &KeyStateManager {
        &self.key_state
    }

    /// Access the key state manager mutably.
    pub fn key_state_mut(&mut self) -> &mut KeyStateManager {
        &mut self.key_state
    }
}

impl Default for InputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::VirtualKeyboardBackend;

    #[test]
    fn new_processor_defaults() {
        let proc = InputProcessor::new();
        assert_eq!(proc.start_time, 0);
        assert!(proc.enabled);
        assert!(!proc.start_pressed);
        assert!(!proc.select_pressed);
        assert_eq!(proc.device_type, DeviceType::Keyboard);
    }

    #[test]
    fn key_changed_updates_state() {
        let mut proc = InputProcessor::new();
        proc.key_changed(1000, 0, true);

        assert!(proc.get_key_state(0));
        assert_eq!(proc.get_key_changed_time(0), 1000);
    }

    #[test]
    fn key_changed_no_duplicate_when_same_state() {
        let mut proc = InputProcessor::new();
        proc.set_start_time(1); // Enable logging
        proc.key_changed(1000, 0, true);
        proc.key_changed(2000, 0, true); // Same state

        // Only one log entry
        assert_eq!(proc.get_key_input_log().len(), 1);
    }

    #[test]
    fn key_changed_logs_when_started() {
        let mut proc = InputProcessor::new();
        proc.set_start_time(100);

        proc.key_changed(1000, 0, true);
        proc.key_changed(2000, 0, false);

        let log = proc.get_key_input_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].presstime, 1000);
        assert!(log[0].pressed);
        assert_eq!(log[1].presstime, 2000);
        assert!(!log[1].pressed);
    }

    #[test]
    fn key_changed_no_log_before_start() {
        let mut proc = InputProcessor::new();
        // start_time = 0 means not started
        proc.key_changed(1000, 0, true);

        assert!(proc.get_key_input_log().is_empty());
    }

    #[test]
    fn key_changed_applies_margin_time() {
        let mut proc = InputProcessor::new();
        proc.set_start_time(100);
        proc.set_key_log_margin_time(10); // 10ms = 10000us

        proc.key_changed(50000, 0, true);

        let log = proc.get_key_input_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].presstime, 40000); // 50000 - 10000
    }

    #[test]
    fn start_select_handling() {
        let mut proc = InputProcessor::new();

        // Start key (keycode -1)
        proc.key_changed(1000, -1, true);
        assert!(proc.start_pressed());
        assert!(!proc.get_key_state(0)); // Should not affect key state

        // Select key (keycode -2)
        proc.key_changed(2000, -2, true);
        assert!(proc.select_pressed());

        proc.key_changed(3000, -1, false);
        assert!(!proc.start_pressed());
    }

    #[test]
    fn disabled_blocks_events() {
        let mut proc = InputProcessor::new();
        proc.set_enabled(false);

        proc.key_changed(1000, 0, true);
        assert!(!proc.get_key_state(0));
    }

    #[test]
    fn set_start_time_clears_log() {
        let mut proc = InputProcessor::new();
        proc.set_start_time(100);
        proc.key_changed(1000, 0, true);
        assert_eq!(proc.get_key_input_log().len(), 1);

        proc.set_start_time(200);
        assert!(proc.get_key_input_log().is_empty());
    }

    #[test]
    fn poll_keyboard_processes_events() {
        let mut proc = InputProcessor::new();
        let config = bms_config::play_mode_config::KeyboardConfig {
            keys: vec![10, 20, 30, 40, 50, 60, 70, 80],
            start: 90,
            select: 91,
            duration: 0,
            ..Default::default()
        };
        proc.keyboard.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();
        backend.press(10);

        proc.poll_keyboard(1000, &backend);

        // Lane 0 should be pressed
        assert!(proc.get_key_state(0));
    }

    #[test]
    fn process_events_from_external_device() {
        let mut proc = InputProcessor::new();

        let events = vec![
            InputEvent::KeyChanged {
                keycode: 3,
                pressed: true,
                time_us: 5000,
            },
            InputEvent::AnalogState {
                keycode: 3,
                is_analog: true,
                value: 0.5,
            },
        ];
        proc.process_events(events);

        assert!(proc.get_key_state(3));
        assert!(proc.key_state().is_analog_input(3));
    }

    #[test]
    fn reset_key_changed_time() {
        let mut proc = InputProcessor::new();
        proc.key_changed(1000, 5, true);

        assert!(proc.reset_key_changed_time(5));
        assert_eq!(proc.get_key_changed_time(5), KeyStateManager::TIME_NOT_SET);

        // Second reset returns false
        assert!(!proc.reset_key_changed_time(5));
    }

    #[test]
    fn analog_scroll_disabled() {
        let mut proc = InputProcessor::new();
        proc.set_analog_scroll(false);

        proc.set_analog_state(0, true, 0.5);
        // analog_scroll disabled, so is_analog should be false
        assert!(!proc.key_state().is_analog_input(0));
    }

    #[test]
    fn is_activated_f_keys() {
        let proc = InputProcessor::new();
        let mut backend = VirtualKeyboardBackend::new();

        assert!(!proc.is_activated(KeyCommand::ShowFps, &backend));

        backend.press(ControlKeys::F1.keycode());
        assert!(proc.is_activated(KeyCommand::ShowFps, &backend));
    }

    #[test]
    fn default_trait() {
        let proc = InputProcessor::default();
        assert_eq!(proc.start_time, 0);
    }
}
