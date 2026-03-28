// long_note_modifier::Mode

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Off,
    Add,
    Remove,
}

impl Mode {
    pub fn values() -> &'static [Mode] {
        &[Mode::Off, Mode::Add, Mode::Remove]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_note_modifier_mode_values() {
        let values = Mode::values();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_long_note_modifier_mode_clone_debug() {
        let m = Mode::Add;
        let copied = m;
        assert_eq!(format!("{:?}", copied), "Add");
    }

    #[test]
    fn test_long_note_modifier_mode_off_debug() {
        let m = Mode::Off;
        assert_eq!(format!("{:?}", m), "Off");
    }

    #[test]
    fn test_long_note_modifier_mode_remove_debug() {
        let m = Mode::Remove;
        assert_eq!(format!("{:?}", m), "Remove");
    }

    #[test]
    fn test_long_note_modifier_mode_values_order() {
        let values = Mode::values();
        assert!(matches!(values[0], Mode::Off));
        assert!(matches!(values[1], Mode::Add));
        assert!(matches!(values[2], Mode::Remove));
    }
}
