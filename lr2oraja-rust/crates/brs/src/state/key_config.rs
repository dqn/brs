// KeyConfig state — key configuration screen.
//
// Allows users to configure keyboard/controller/MIDI key assignments
// for each play mode.

use tracing::info;

use bms_input::control_keys::ControlKeys;

use crate::app_state::AppStateType;
use crate::state::{GameStateHandler, StateContext};

/// Play mode configurations available for key binding.
const MODE_CONFIGS: &[(i32, &str, usize)] = &[
    (5, "5KEYS", 7),
    (7, "7KEYS", 9),
    (9, "PMS 9KEYS", 9),
    (10, "10KEYS", 14),
    (14, "14KEYS", 18),
    (25, "24KEYS", 26),
    (50, "24KEYS DP", 52),
];

/// Input device types for key configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputDevice {
    Keyboard,
    Controller,
    Midi,
}

/// Key configuration state — configures key assignments per play mode.
pub struct KeyConfigState {
    mode_index: usize,
    cursor: usize,
    key_input_mode: bool,
    input_device: InputDevice,
    /// Tracks which keycodes were already pressed when entering key_input_mode,
    /// so we only register newly pressed keys.
    baseline_pressed: [bool; 256],
}

impl KeyConfigState {
    pub fn new() -> Self {
        Self {
            mode_index: 1, // Default to 7KEYS
            cursor: 0,
            key_input_mode: false,
            input_device: InputDevice::Keyboard,
            baseline_pressed: [false; 256],
        }
    }

    fn current_mode_id(&self) -> i32 {
        MODE_CONFIGS[self.mode_index].0
    }

    fn current_key_count(&self) -> usize {
        MODE_CONFIGS[self.mode_index].2
    }

    fn current_mode_name(&self) -> &str {
        MODE_CONFIGS[self.mode_index].1
    }

    /// Scan the keyboard backend for a newly pressed key (not in baseline).
    fn scan_new_key_press(
        &self,
        backend: &dyn bms_input::keyboard::KeyboardBackend,
    ) -> Option<i32> {
        (0..256i32).find(|&keycode| {
            backend.is_key_pressed(keycode) && !self.baseline_pressed[keycode as usize]
        })
    }

    /// Capture baseline of currently pressed keys from backend.
    fn capture_baseline(&mut self, backend: &dyn bms_input::keyboard::KeyboardBackend) {
        for keycode in 0..256i32 {
            self.baseline_pressed[keycode as usize] = backend.is_key_pressed(keycode);
        }
    }
}

impl Default for KeyConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStateHandler for KeyConfigState {
    fn create(&mut self, _ctx: &mut StateContext) {
        self.cursor = 0;
        self.key_input_mode = false;
        info!(mode = self.current_mode_name(), "KeyConfig: create");
    }

    fn render(&mut self, _ctx: &mut StateContext) {
        // KeyConfig is input-driven, no timer-based transitions
    }

    fn input(&mut self, ctx: &mut StateContext) {
        if self.key_input_mode {
            // In key input mode: scan keyboard backend for any newly pressed key
            if let Some(backend) = ctx.keyboard_backend
                && let Some(key_code) = self.scan_new_key_press(backend)
            {
                let mode_id = self.current_mode_id();
                let config = ctx.player_config.play_config_mut(mode_id);

                match self.input_device {
                    InputDevice::Keyboard => {
                        // Remove duplicate assignment
                        for k in &mut config.keyboard.keys {
                            if *k == key_code {
                                *k = -1;
                            }
                        }
                        if self.cursor < config.keyboard.keys.len() {
                            config.keyboard.keys[self.cursor] = key_code;
                        }
                    }
                    InputDevice::Controller => {
                        if let Some(ctrl) = config.controller.first_mut() {
                            for k in &mut ctrl.keys {
                                if *k == key_code {
                                    *k = -1;
                                }
                            }
                            if self.cursor < ctrl.keys.len() {
                                ctrl.keys[self.cursor] = key_code;
                            }
                        }
                    }
                    InputDevice::Midi => {
                        // MIDI key assignment requires MIDI backend; skip for now
                    }
                }

                self.key_input_mode = false;
                info!(
                    cursor = self.cursor,
                    key = key_code,
                    "KeyConfig: key assigned"
                );
            }
            return;
        }

        if let Some(input_state) = ctx.input_state {
            for key in &input_state.pressed_keys {
                match key {
                    ControlKeys::Left => {
                        // Switch to previous mode
                        if self.mode_index == 0 {
                            self.mode_index = MODE_CONFIGS.len() - 1;
                        } else {
                            self.mode_index -= 1;
                        }
                        self.cursor = 0;
                        info!(mode = self.current_mode_name(), "KeyConfig: mode changed");
                        return;
                    }
                    ControlKeys::Right => {
                        // Switch to next mode
                        self.mode_index = (self.mode_index + 1) % MODE_CONFIGS.len();
                        self.cursor = 0;
                        info!(mode = self.current_mode_name(), "KeyConfig: mode changed");
                        return;
                    }
                    ControlKeys::Up => {
                        // Move cursor up
                        let key_count = self.current_key_count();
                        if key_count > 0 {
                            if self.cursor == 0 {
                                self.cursor = key_count - 1;
                            } else {
                                self.cursor -= 1;
                            }
                        }
                        return;
                    }
                    ControlKeys::Down => {
                        // Move cursor down
                        let key_count = self.current_key_count();
                        if key_count > 0 {
                            self.cursor = (self.cursor + 1) % key_count;
                        }
                        return;
                    }
                    ControlKeys::Enter => {
                        // Enter key input mode
                        self.key_input_mode = true;
                        // Capture baseline so we only detect new presses
                        if let Some(backend) = ctx.keyboard_backend {
                            self.capture_baseline(backend);
                        }
                        info!(cursor = self.cursor, "KeyConfig: waiting for key input");
                        return;
                    }
                    ControlKeys::Del => {
                        // Delete current key assignment
                        let mode_id = self.current_mode_id();
                        let config = ctx.player_config.play_config_mut(mode_id);
                        match self.input_device {
                            InputDevice::Keyboard => {
                                if self.cursor < config.keyboard.keys.len() {
                                    config.keyboard.keys[self.cursor] = -1;
                                }
                            }
                            InputDevice::Controller => {
                                if let Some(ctrl) = config.controller.first_mut()
                                    && self.cursor < ctrl.keys.len()
                                {
                                    ctrl.keys[self.cursor] = -1;
                                }
                            }
                            InputDevice::Midi => {
                                let midi = &mut config.midi;
                                if self.cursor < midi.keys.len() {
                                    midi.keys[self.cursor] = None;
                                }
                            }
                        }
                        info!(cursor = self.cursor, "KeyConfig: key deleted");
                        return;
                    }
                    ControlKeys::Num7 => {
                        // Reset keyboard defaults
                        let mode_id = self.current_mode_id();
                        let key_count = self.current_key_count();
                        let config = ctx.player_config.play_config_mut(mode_id);
                        config.keyboard = bms_config::KeyboardConfig::default();
                        config.keyboard.keys.resize(key_count, 0);
                        info!("KeyConfig: keyboard reset to defaults");
                        return;
                    }
                    ControlKeys::Num8 => {
                        // Reset controller defaults
                        let mode_id = self.current_mode_id();
                        let key_count = self.current_key_count();
                        let config = ctx.player_config.play_config_mut(mode_id);
                        config.controller = vec![bms_config::ControllerConfig::default()];
                        config.controller[0].keys.resize(key_count, -1);
                        info!("KeyConfig: controller reset to defaults");
                        return;
                    }
                    ControlKeys::Num9 => {
                        // Reset MIDI defaults
                        let mode_id = self.current_mode_id();
                        let key_count = self.current_key_count();
                        let config = ctx.player_config.play_config_mut(mode_id);
                        config.midi = bms_config::MidiConfig::default();
                        config.midi.keys.resize(key_count, None);
                        info!("KeyConfig: MIDI reset to defaults");
                        return;
                    }
                    ControlKeys::Num1 => {
                        self.input_device = InputDevice::Keyboard;
                        info!("KeyConfig: input device = Keyboard");
                        return;
                    }
                    ControlKeys::Num2 => {
                        self.input_device = InputDevice::Controller;
                        info!("KeyConfig: input device = Controller");
                        return;
                    }
                    ControlKeys::Num3 => {
                        self.input_device = InputDevice::Midi;
                        info!("KeyConfig: input device = MIDI");
                        return;
                    }
                    ControlKeys::Escape => {
                        // Save and exit
                        ctx.resource.config_save_requested = true;
                        *ctx.transition = Some(AppStateType::MusicSelect);
                        info!("KeyConfig: save and exit");
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    fn shutdown(&mut self, _ctx: &mut StateContext) {
        info!("KeyConfig: shutdown");
    }
}

#[cfg(test)]
impl KeyConfigState {
    pub(crate) fn mode_index(&self) -> usize {
        self.mode_index
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn is_key_input_mode(&self) -> bool {
        self.key_input_mode
    }

    pub(crate) fn input_device(&self) -> InputDevice {
        self.input_device
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mapper::InputState;
    use crate::player_resource::PlayerResource;
    use crate::timer_manager::TimerManager;
    use bms_config::{Config, PlayerConfig};

    fn make_ctx<'a>(
        timer: &'a mut TimerManager,
        resource: &'a mut PlayerResource,
        config: &'a Config,
        player_config: &'a mut PlayerConfig,
        transition: &'a mut Option<AppStateType>,
    ) -> StateContext<'a> {
        StateContext {
            timer,
            resource,
            config,
            player_config,
            transition,
            keyboard_backend: None,
            database: None,
            input_state: None,
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        }
    }

    fn make_input_ctx<'a>(
        timer: &'a mut TimerManager,
        resource: &'a mut PlayerResource,
        config: &'a Config,
        player_config: &'a mut PlayerConfig,
        transition: &'a mut Option<AppStateType>,
        input_state: &'a InputState,
    ) -> StateContext<'a> {
        StateContext {
            timer,
            resource,
            config,
            player_config,
            transition,
            keyboard_backend: None,
            database: None,
            input_state: Some(input_state),
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        }
    }

    #[test]
    fn create_resets_cursor() {
        let mut state = KeyConfigState::new();
        state.cursor = 5;
        state.key_input_mode = true;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let mut ctx = make_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
        );
        state.create(&mut ctx);

        assert_eq!(state.cursor(), 0);
        assert!(!state.is_key_input_mode());
    }

    #[test]
    fn mode_switch_right_wraps() {
        let mut state = KeyConfigState::new();
        state.mode_index = MODE_CONFIGS.len() - 1;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Right],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(state.mode_index(), 0);
    }

    #[test]
    fn mode_switch_left_wraps() {
        let mut state = KeyConfigState::new();
        state.mode_index = 0;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Left],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(state.mode_index(), MODE_CONFIGS.len() - 1);
    }

    #[test]
    fn cursor_wraps_down() {
        let mut state = KeyConfigState::new();
        // 7KEYS mode has 9 keys
        state.cursor = 8;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Down],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn cursor_wraps_up() {
        let mut state = KeyConfigState::new();
        state.cursor = 0;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Up],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(state.cursor(), 8); // 7KEYS mode has 9 keys, so last = 8
    }

    #[test]
    fn delete_clears_key() {
        let mut state = KeyConfigState::new();
        state.cursor = 0;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate(); // Ensure key arrays are populated
        let mut transition = None;

        // Verify key is set
        assert_ne!(player_config.play_config(7).keyboard.keys[0], -1);

        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Del],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(player_config.play_config(7).keyboard.keys[0], -1);
    }

    #[test]
    fn escape_transitions_to_music_select() {
        let mut state = KeyConfigState::new();

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Escape],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(transition, Some(AppStateType::MusicSelect));
        assert!(resource.config_save_requested);
    }

    #[test]
    fn enter_enables_key_input_mode() {
        let mut state = KeyConfigState::new();

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Enter],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert!(state.is_key_input_mode());
    }

    #[test]
    fn num7_resets_keyboard_defaults() {
        let mut state = KeyConfigState::new();

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        // Modify a key
        player_config.play_config_mut(7).keyboard.keys[0] = 999;
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num7],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        // Should be reset (not 999)
        assert_ne!(player_config.play_config(7).keyboard.keys[0], 999);
    }

    #[test]
    fn mode_switch_resets_cursor() {
        let mut state = KeyConfigState::new();
        state.cursor = 5;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Right],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);

        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn num1_num2_num3_switch_input_device() {
        let mut state = KeyConfigState::new();
        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        let mut transition = None;

        // Num2 -> Controller
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num2],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.input_device(), InputDevice::Controller);

        // Num3 -> Midi
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num3],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.input_device(), InputDevice::Midi);

        // Num1 -> Keyboard
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Num1],
        };
        let mut ctx = make_input_ctx(
            &mut timer,
            &mut resource,
            &config,
            &mut player_config,
            &mut transition,
            &input,
        );
        state.input(&mut ctx);
        assert_eq!(state.input_device(), InputDevice::Keyboard);
    }

    #[test]
    fn key_input_mode_assigns_key_via_backend() {
        use bms_input::keyboard::VirtualKeyboardBackend;

        let mut state = KeyConfigState::new();
        state.key_input_mode = true;
        state.cursor = 0;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        // Press keycode 42 on the virtual backend
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(42);

        let mut ctx = StateContext {
            timer: &mut timer,
            resource: &mut resource,
            config: &config,
            player_config: &mut player_config,
            transition: &mut transition,
            keyboard_backend: Some(&backend),
            database: None,
            input_state: None,
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        };
        state.input(&mut ctx);

        // Should have assigned key 42 and exited key_input_mode
        assert!(!state.is_key_input_mode());
        assert_eq!(player_config.play_config(7).keyboard.keys[0], 42);
    }

    #[test]
    fn key_input_mode_removes_duplicate() {
        use bms_input::keyboard::VirtualKeyboardBackend;

        let mut state = KeyConfigState::new();
        state.key_input_mode = true;
        state.cursor = 1; // Assign to slot 1

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        // Set key 42 in slot 0
        player_config.play_config_mut(7).keyboard.keys[0] = 42;
        let mut transition = None;

        // Press keycode 42 (already assigned to slot 0)
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(42);

        let mut ctx = StateContext {
            timer: &mut timer,
            resource: &mut resource,
            config: &config,
            player_config: &mut player_config,
            transition: &mut transition,
            keyboard_backend: Some(&backend),
            database: None,
            input_state: None,
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        };
        state.input(&mut ctx);

        // Slot 0 should be cleared, slot 1 should have 42
        assert_eq!(player_config.play_config(7).keyboard.keys[0], -1);
        assert_eq!(player_config.play_config(7).keyboard.keys[1], 42);
    }

    #[test]
    fn key_input_mode_ignores_baseline_keys() {
        use bms_input::keyboard::VirtualKeyboardBackend;

        let mut state = KeyConfigState::new();
        state.cursor = 0;

        let mut timer = TimerManager::new();
        let mut resource = PlayerResource::default();
        let config = Config::default();
        let mut player_config = PlayerConfig::default();
        player_config.validate();
        let mut transition = None;

        // Backend has key 42 already pressed when Enter is pressed
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(42);

        // Enter key_input_mode with Enter pressed (captures baseline)
        let input = InputState {
            commands: vec![],
            pressed_keys: vec![ControlKeys::Enter],
        };
        let mut ctx = StateContext {
            timer: &mut timer,
            resource: &mut resource,
            config: &config,
            player_config: &mut player_config,
            transition: &mut transition,
            keyboard_backend: Some(&backend),
            database: None,
            input_state: Some(&input),
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        };
        state.input(&mut ctx);
        assert!(state.is_key_input_mode());

        // Now poll again — key 42 is in baseline, should not be picked up
        let mut ctx2 = StateContext {
            timer: &mut timer,
            resource: &mut resource,
            config: &config,
            player_config: &mut player_config,
            transition: &mut transition,
            keyboard_backend: Some(&backend),
            database: None,
            input_state: None,
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        };
        state.input(&mut ctx2);
        // Still in key_input_mode because baseline key was ignored
        assert!(state.is_key_input_mode());

        // Now press a new key (99) that is not in baseline
        backend.press(99);
        let mut ctx3 = StateContext {
            timer: &mut timer,
            resource: &mut resource,
            config: &config,
            player_config: &mut player_config,
            transition: &mut transition,
            keyboard_backend: Some(&backend),
            database: None,
            input_state: None,
            skin_manager: None,
            sound_manager: None,
            received_chars: &[],
        };
        state.input(&mut ctx3);
        assert!(!state.is_key_input_mode());
        assert_eq!(player_config.play_config(7).keyboard.keys[0], 99);
    }
}
