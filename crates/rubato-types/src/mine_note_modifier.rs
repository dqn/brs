// mine_note_modifier::Mode

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Off,
    Remove,
}

impl Mode {
    pub fn values() -> &'static [Mode] {
        &[Mode::Off, Mode::Remove]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_note_modifier_mode_values() {
        let values = Mode::values();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_mine_note_modifier_mode_clone_debug() {
        let m = Mode::Remove;
        let copied = m;
        assert_eq!(format!("{:?}", copied), "Remove");
    }

    #[test]
    fn test_mine_note_modifier_mode_off_debug() {
        let m = Mode::Off;
        assert_eq!(format!("{:?}", m), "Off");
    }

    #[test]
    fn test_mine_note_modifier_mode_values_order() {
        let values = Mode::values();
        assert!(matches!(values[0], Mode::Off));
        assert!(matches!(values[1], Mode::Remove));
    }
}
