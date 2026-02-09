/// Input device types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Keyboard,
    Controller,
    Midi,
}

/// Input events emitted by device backends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    /// A key was pressed or released.
    KeyChanged {
        keycode: i32,
        pressed: bool,
        time_us: i64,
    },
    /// Analog state update (e.g. controller axis for lane cover scroll).
    AnalogState {
        keycode: i32,
        is_analog: bool,
        value: f32,
    },
}
