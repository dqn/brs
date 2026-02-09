/// Control keys used for UI commands (F-keys, numpad, arrows, etc.).
///
/// Ported from Java `KeyBoardInputProcesseor.ControlKeys`.
/// Key codes use LibGDX constants for now; will be mapped to winit in Phase 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// LibGDX Keys constants for ControlKeys mapping
mod libgdx {
    pub const NUM_0: i32 = 7;
    pub const NUM_1: i32 = 8;
    pub const NUM_2: i32 = 9;
    pub const NUM_3: i32 = 10;
    pub const NUM_4: i32 = 11;
    pub const NUM_5: i32 = 12;
    pub const NUM_6: i32 = 13;
    pub const NUM_7: i32 = 14;
    pub const NUM_8: i32 = 15;
    pub const NUM_9: i32 = 16;
    pub const F1: i32 = 244;
    pub const F2: i32 = 245;
    pub const F3: i32 = 246;
    pub const F4: i32 = 247;
    pub const F5: i32 = 248;
    pub const F6: i32 = 249;
    pub const F7: i32 = 250;
    pub const F8: i32 = 251;
    pub const F9: i32 = 252;
    pub const F10: i32 = 253;
    pub const F11: i32 = 254;
    pub const F12: i32 = 255;
    pub const UP: i32 = 19;
    pub const DOWN: i32 = 20;
    pub const LEFT: i32 = 21;
    pub const RIGHT: i32 = 22;
    pub const ENTER: i32 = 66;
    pub const INSERT: i32 = 124;
    pub const FORWARD_DEL: i32 = 112;
    pub const ESCAPE: i32 = 111;
    pub const C: i32 = 31;
}

impl ControlKeys {
    /// All control key variants.
    pub const ALL: [ControlKeys; 31] = [
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

    /// Numeric ID (matches Java ControlKeys.id).
    pub fn id(self) -> i32 {
        match self {
            ControlKeys::Num0 => 0,
            ControlKeys::Num1 => 1,
            ControlKeys::Num2 => 2,
            ControlKeys::Num3 => 3,
            ControlKeys::Num4 => 4,
            ControlKeys::Num5 => 5,
            ControlKeys::Num6 => 6,
            ControlKeys::Num7 => 7,
            ControlKeys::Num8 => 8,
            ControlKeys::Num9 => 9,
            ControlKeys::F1 => 10,
            ControlKeys::F2 => 11,
            ControlKeys::F3 => 12,
            ControlKeys::F4 => 13,
            ControlKeys::F5 => 14,
            ControlKeys::F6 => 15,
            ControlKeys::F7 => 16,
            ControlKeys::F8 => 17,
            ControlKeys::F9 => 18,
            ControlKeys::F10 => 19,
            ControlKeys::F11 => 20,
            ControlKeys::F12 => 21,
            ControlKeys::Up => 22,
            ControlKeys::Down => 23,
            ControlKeys::Left => 24,
            ControlKeys::Right => 25,
            ControlKeys::Enter => 26,
            ControlKeys::Insert => 27,
            ControlKeys::Del => 28,
            ControlKeys::Escape => 29,
            ControlKeys::KeyC => 30,
        }
    }

    /// LibGDX key code (matches Java ControlKeys.keycode).
    pub fn keycode(self) -> i32 {
        match self {
            ControlKeys::Num0 => libgdx::NUM_0,
            ControlKeys::Num1 => libgdx::NUM_1,
            ControlKeys::Num2 => libgdx::NUM_2,
            ControlKeys::Num3 => libgdx::NUM_3,
            ControlKeys::Num4 => libgdx::NUM_4,
            ControlKeys::Num5 => libgdx::NUM_5,
            ControlKeys::Num6 => libgdx::NUM_6,
            ControlKeys::Num7 => libgdx::NUM_7,
            ControlKeys::Num8 => libgdx::NUM_8,
            ControlKeys::Num9 => libgdx::NUM_9,
            ControlKeys::F1 => libgdx::F1,
            ControlKeys::F2 => libgdx::F2,
            ControlKeys::F3 => libgdx::F3,
            ControlKeys::F4 => libgdx::F4,
            ControlKeys::F5 => libgdx::F5,
            ControlKeys::F6 => libgdx::F6,
            ControlKeys::F7 => libgdx::F7,
            ControlKeys::F8 => libgdx::F8,
            ControlKeys::F9 => libgdx::F9,
            ControlKeys::F10 => libgdx::F10,
            ControlKeys::F11 => libgdx::F11,
            ControlKeys::F12 => libgdx::F12,
            ControlKeys::Up => libgdx::UP,
            ControlKeys::Down => libgdx::DOWN,
            ControlKeys::Left => libgdx::LEFT,
            ControlKeys::Right => libgdx::RIGHT,
            ControlKeys::Enter => libgdx::ENTER,
            ControlKeys::Insert => libgdx::INSERT,
            ControlKeys::Del => libgdx::FORWARD_DEL,
            ControlKeys::Escape => libgdx::ESCAPE,
            ControlKeys::KeyC => libgdx::C,
        }
    }

    /// Whether this key should work in text input mode.
    pub fn is_text_key(self) -> bool {
        matches!(
            self,
            ControlKeys::Num0
                | ControlKeys::Num1
                | ControlKeys::Num2
                | ControlKeys::Num3
                | ControlKeys::Num4
                | ControlKeys::Num5
                | ControlKeys::Num6
                | ControlKeys::Num7
                | ControlKeys::Num8
                | ControlKeys::Num9
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_keys_ids() {
        assert_eq!(ControlKeys::Num0.id(), 0);
        assert_eq!(ControlKeys::Num9.id(), 9);
        assert_eq!(ControlKeys::F1.id(), 10);
        assert_eq!(ControlKeys::F12.id(), 21);
        assert_eq!(ControlKeys::Up.id(), 22);
        assert_eq!(ControlKeys::KeyC.id(), 30);
    }

    #[test]
    fn test_control_keys_text() {
        assert!(ControlKeys::Num0.is_text_key());
        assert!(ControlKeys::Num9.is_text_key());
        assert!(!ControlKeys::F1.is_text_key());
        assert!(!ControlKeys::Up.is_text_key());
        assert!(!ControlKeys::Enter.is_text_key());
    }

    #[test]
    fn test_all_variants_count() {
        assert_eq!(ControlKeys::ALL.len(), 31);
    }
}
