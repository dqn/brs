/// Key state at a specific point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// Key index (lane-based, 0-indexed).
    pub key: usize,
    /// true = pressed, false = released.
    pub pressed: bool,
    /// Timestamp in microseconds from play start.
    pub time_us: i64,
}

/// Abstraction over input sources.
/// Implementations: InputManager (keyboard/gamepad/MIDI), ScriptedInput (testing).
pub trait InputProvider {
    /// Poll current key events since last call.
    fn poll_events(&mut self) -> Vec<KeyEvent>;

    /// Check if a specific key is currently held down.
    fn is_pressed(&self, key: usize) -> bool;

    /// Number of available keys for the current play mode.
    fn key_count(&self) -> usize;
}
