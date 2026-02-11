// InputMapper — maps keyboard state to KeyCommands for UI states.
//
// Converts the current keyboard backend state to a list of fired KeyCommands
// for use by Decide, Select, Result, and other non-Play states.

use bms_input::control_keys::ControlKeys;
use bms_input::key_command::KeyCommand;
use bms_input::keyboard::KeyboardBackend;

/// All control keys to poll each frame.
const ALL_CONTROL_KEYS: &[ControlKeys] = &[
    ControlKeys::Num0,
    ControlKeys::Num1,
    ControlKeys::Num2,
    ControlKeys::Num3,
    ControlKeys::Num4,
    ControlKeys::Num5,
    ControlKeys::Num6,
    ControlKeys::Num7,
    ControlKeys::Num8,
    ControlKeys::Num9,
    ControlKeys::F1,
    ControlKeys::F2,
    ControlKeys::F3,
    ControlKeys::F4,
    ControlKeys::F5,
    ControlKeys::F6,
    ControlKeys::F7,
    ControlKeys::F8,
    ControlKeys::F9,
    ControlKeys::F10,
    ControlKeys::F11,
    ControlKeys::F12,
    ControlKeys::Up,
    ControlKeys::Down,
    ControlKeys::Left,
    ControlKeys::Right,
    ControlKeys::Enter,
    ControlKeys::Insert,
    ControlKeys::Del,
    ControlKeys::Escape,
    ControlKeys::KeyC,
];

/// Maps control key to KeyCommand (Java KeyBoardInputProcesseor logic).
fn control_key_to_command(key: ControlKeys) -> Option<KeyCommand> {
    match key {
        ControlKeys::F1 => Some(KeyCommand::ShowFps),
        ControlKeys::F2 => Some(KeyCommand::UpdateFolder),
        ControlKeys::F3 => Some(KeyCommand::OpenExplorer),
        ControlKeys::F4 => Some(KeyCommand::CopySongMd5Hash),
        ControlKeys::F5 => Some(KeyCommand::CopySongSha256Hash),
        ControlKeys::F6 => Some(KeyCommand::SwitchScreenMode),
        ControlKeys::F7 => Some(KeyCommand::SaveScreenshot),
        ControlKeys::F8 => Some(KeyCommand::PostTwitter),
        ControlKeys::F9 => Some(KeyCommand::AddFavoriteSong),
        ControlKeys::F10 => Some(KeyCommand::AddFavoriteChart),
        ControlKeys::F11 => Some(KeyCommand::AutoplayFolder),
        ControlKeys::F12 => Some(KeyCommand::OpenIr),
        ControlKeys::Insert => Some(KeyCommand::OpenSkinConfiguration),
        ControlKeys::Del => Some(KeyCommand::ToggleModMenu),
        ControlKeys::KeyC => Some(KeyCommand::CopyHighlightedMenuText),
        _ => None,
    }
}

/// State tracker for detecting key-down edges (pressed this frame, not last frame).
#[derive(Default)]
pub struct InputMapper {
    prev_pressed: [bool; ALL_CONTROL_KEYS.len()],
}

impl InputMapper {
    pub fn new() -> Self {
        Self::default()
    }

    /// Poll the keyboard backend and return newly pressed KeyCommands this frame.
    /// Also returns raw control keys that transitioned to pressed for use by states.
    pub fn update(&mut self, backend: &dyn KeyboardBackend) -> InputState {
        let mut commands = Vec::new();
        let mut pressed_keys = Vec::new();

        for (i, &key) in ALL_CONTROL_KEYS.iter().enumerate() {
            let is_pressed = backend.is_key_pressed(key.keycode());
            let was_pressed = self.prev_pressed[i];
            self.prev_pressed[i] = is_pressed;

            // Edge detection: fire on key-down
            if is_pressed && !was_pressed {
                pressed_keys.push(key);
                if let Some(cmd) = control_key_to_command(key) {
                    commands.push(cmd);
                }
            }
        }

        InputState {
            commands,
            pressed_keys,
        }
    }
}

/// Result of a single frame's input mapping.
#[derive(Default)]
pub struct InputState {
    /// KeyCommands that fired this frame.
    pub commands: Vec<KeyCommand>,
    /// Raw control keys that transitioned to pressed this frame.
    pub pressed_keys: Vec<ControlKeys>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_input::keyboard::VirtualKeyboardBackend;

    #[test]
    fn no_input_produces_empty() {
        let mut mapper = InputMapper::new();
        let backend = VirtualKeyboardBackend::new();
        let state = mapper.update(&backend);
        assert!(state.commands.is_empty());
        assert!(state.pressed_keys.is_empty());
    }

    #[test]
    fn f1_produces_show_fps() {
        let mut mapper = InputMapper::new();
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(ControlKeys::F1.keycode());
        let state = mapper.update(&backend);
        assert_eq!(state.commands, vec![KeyCommand::ShowFps]);
        assert_eq!(state.pressed_keys, vec![ControlKeys::F1]);
    }

    #[test]
    fn held_key_fires_only_once() {
        let mut mapper = InputMapper::new();
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(ControlKeys::F1.keycode());
        let state = mapper.update(&backend);
        assert_eq!(state.commands.len(), 1);

        // Same key still held: should not fire again
        let state = mapper.update(&backend);
        assert!(state.commands.is_empty());
    }

    #[test]
    fn release_and_repress_fires_again() {
        let mut mapper = InputMapper::new();
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(ControlKeys::F1.keycode());
        mapper.update(&backend);

        backend.release(ControlKeys::F1.keycode());
        mapper.update(&backend);

        backend.press(ControlKeys::F1.keycode());
        let state = mapper.update(&backend);
        assert_eq!(state.commands, vec![KeyCommand::ShowFps]);
    }

    #[test]
    fn non_command_key_appears_in_pressed_only() {
        let mut mapper = InputMapper::new();
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(ControlKeys::Enter.keycode());
        let state = mapper.update(&backend);
        assert!(state.commands.is_empty()); // Enter has no KeyCommand
        assert_eq!(state.pressed_keys, vec![ControlKeys::Enter]);
    }

    #[test]
    fn multiple_keys_same_frame() {
        let mut mapper = InputMapper::new();
        let mut backend = VirtualKeyboardBackend::new();
        backend.press(ControlKeys::F1.keycode());
        backend.press(ControlKeys::F2.keycode());
        let state = mapper.update(&backend);
        assert_eq!(state.commands.len(), 2);
        assert!(state.commands.contains(&KeyCommand::ShowFps));
        assert!(state.commands.contains(&KeyCommand::UpdateFolder));
    }
}
