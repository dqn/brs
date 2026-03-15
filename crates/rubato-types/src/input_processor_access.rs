/// ControlKeys enum — keyboard control keys for BMS player input.
///
/// Translated from: KeyBoardInputProcesseor.ControlKeys
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKeys {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Insert,
    Del,
    Escape,
    KeyC,
}

/// KeyCommand enum — high-level keyboard commands.
///
/// Translated from: bms.player.beatoraja.input.KeyCommand
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    ShowFps,
    UpdateFolder,
    OpenExplorer,
    CopySongMd5Hash,
    CopySongSha256Hash,
    SwitchScreenMode,
    SaveScreenshot,
    AddFavoriteSong,
    AddFavoriteChart,
    AutoplayFolder,
    OpenIr,
    OpenSkinConfiguration,
    ToggleModMenu,
    CopyHighlightedMenuText,
}

/// Trait interface for input processor access.
///
/// Downstream crates use `&dyn InputProcessorAccess` instead of concrete
/// BMSPlayerInputProcessor references.
pub trait InputProcessorAccess {
    /// Get the state of a control key (true = pressed).
    fn control_key_state(&self, key: ControlKeys) -> bool;

    /// Check if a key command has been activated this frame.
    fn is_activated(&self, cmd: KeyCommand) -> bool;

    /// Get the start time of the input processor.
    fn start_time(&self) -> i64 {
        0
    }

    /// Get accumulated scroll value.
    fn scroll(&self) -> i32 {
        0
    }

    /// Reset scroll accumulator to zero.
    fn reset_scroll(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestInput;
    impl InputProcessorAccess for TestInput {
        fn control_key_state(&self, _key: ControlKeys) -> bool {
            false
        }
        fn is_activated(&self, _cmd: KeyCommand) -> bool {
            false
        }
    }

    #[test]
    fn test_input_processor_access_trait() {
        let input = TestInput;
        assert!(!input.control_key_state(ControlKeys::Num1));
        assert!(!input.is_activated(KeyCommand::ShowFps));
        assert_eq!(input.start_time(), 0);
    }

    #[test]
    fn test_control_keys_all_variants_distinct() {
        let keys = [
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
        // Verify all are unique (no duplicates)
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(
                    keys[i], keys[j],
                    "Keys at index {} and {} should be distinct",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_key_command_all_variants_distinct() {
        let cmds = [
            KeyCommand::ShowFps,
            KeyCommand::UpdateFolder,
            KeyCommand::OpenExplorer,
            KeyCommand::CopySongMd5Hash,
            KeyCommand::CopySongSha256Hash,
            KeyCommand::SwitchScreenMode,
            KeyCommand::SaveScreenshot,
            KeyCommand::AddFavoriteSong,
            KeyCommand::AddFavoriteChart,
            KeyCommand::AutoplayFolder,
            KeyCommand::OpenIr,
            KeyCommand::OpenSkinConfiguration,
            KeyCommand::ToggleModMenu,
            KeyCommand::CopyHighlightedMenuText,
        ];
        for i in 0..cmds.len() {
            for j in (i + 1)..cmds.len() {
                assert_ne!(cmds[i], cmds[j]);
            }
        }
    }

    #[test]
    fn test_default_scroll_returns_zero() {
        let input = TestInput;
        assert_eq!(input.scroll(), 0);
    }

    #[test]
    fn test_control_keys_clone_copy() {
        let k = ControlKeys::Enter;
        let k2 = k;
        assert_eq!(k, k2);
    }

    #[test]
    fn test_key_command_clone_copy() {
        let c = KeyCommand::SaveScreenshot;
        let c2 = c;
        assert_eq!(c, c2);
    }

    #[test]
    fn test_control_keys_debug() {
        let k = ControlKeys::F12;
        assert_eq!(format!("{:?}", k), "F12");
    }

    #[test]
    fn test_key_command_debug() {
        let c = KeyCommand::ToggleModMenu;
        assert_eq!(format!("{:?}", c), "ToggleModMenu");
    }
}
