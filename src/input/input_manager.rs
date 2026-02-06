use winit::keyboard::KeyCode;

use crate::state::config::key_config::KeyConfig;
use crate::traits::input::{InputProvider, KeyEvent};

use super::gamepad::GamepadHandler;
use super::key_state::KeyState;
use super::keyboard::KeyboardHandler;

/// Unified input manager that merges keyboard and gamepad events.
/// Implements InputProvider so it can be used by PlayState.
pub struct InputManager {
    keyboard: KeyboardHandler,
    gamepad: GamepadHandler,
    state: KeyState,
    /// Buffered events from the current frame.
    pending_events: Vec<KeyEvent>,
}

impl InputManager {
    /// Create a new InputManager with the given key configuration.
    pub fn new(key_config: KeyConfig, key_count: usize) -> Self {
        Self {
            keyboard: KeyboardHandler::new(key_config.clone()),
            gamepad: GamepadHandler::new(key_config),
            state: KeyState::new(key_count),
            pending_events: Vec::new(),
        }
    }

    /// Feed a keyboard event from winit.
    pub fn handle_keyboard(&mut self, keycode: KeyCode, pressed: bool, time_us: i64) {
        let events = self.keyboard.handle_key_event(keycode, pressed, time_us);
        for event in &events {
            self.state.set(event.key, event.pressed, event.time_us);
        }
        self.pending_events.extend(events);
    }

    /// Poll gamepad events from gilrs.
    pub fn poll_gamepad(&mut self, time_us: i64) {
        let events = self.gamepad.poll(time_us);
        for event in &events {
            self.state.set(event.key, event.pressed, event.time_us);
        }
        self.pending_events.extend(events);
    }

    /// Update key configuration for all input handlers.
    pub fn set_key_config(&mut self, key_config: KeyConfig) {
        self.keyboard.set_key_config(key_config.clone());
        self.gamepad.set_key_config(key_config);
    }

    /// Reset all input state.
    pub fn reset(&mut self) {
        self.keyboard.reset();
        self.gamepad.reset();
        self.state.reset();
        self.pending_events.clear();
    }
}

impl InputProvider for InputManager {
    fn poll_events(&mut self) -> Vec<KeyEvent> {
        std::mem::take(&mut self.pending_events)
    }

    fn is_pressed(&self, key: usize) -> bool {
        self.state.is_pressed(key)
    }

    fn key_count(&self) -> usize {
        self.state.key_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: GamepadHandler requires gilrs which needs OS support.
    // These tests focus on keyboard integration.

    #[test]
    fn keyboard_events_flow_through() {
        let config = KeyConfig::default_7k();
        let mut mgr = InputManager::new(config, 8);

        mgr.handle_keyboard(KeyCode::KeyZ, true, 1000);
        let events = mgr.poll_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, 1); // Z -> KEY1 (lane 1)
        assert!(mgr.is_pressed(1));
    }

    #[test]
    fn poll_clears_buffer() {
        let config = KeyConfig::default_7k();
        let mut mgr = InputManager::new(config, 8);

        mgr.handle_keyboard(KeyCode::KeyZ, true, 1000);
        let events = mgr.poll_events();
        assert_eq!(events.len(), 1);

        let events = mgr.poll_events();
        assert!(events.is_empty());
    }

    #[test]
    fn key_count() {
        let config = KeyConfig::default_7k();
        let mgr = InputManager::new(config, 8);
        assert_eq!(mgr.key_count(), 8);
    }

    #[test]
    fn reset_clears_everything() {
        let config = KeyConfig::default_7k();
        let mut mgr = InputManager::new(config, 8);

        mgr.handle_keyboard(KeyCode::KeyZ, true, 1000);
        assert!(mgr.is_pressed(1));

        mgr.reset();
        assert!(!mgr.is_pressed(1));
        assert!(mgr.poll_events().is_empty());
    }
}
