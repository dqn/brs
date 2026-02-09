/// Keyboard input processing.
///
/// Ported from Java `KeyBoardInputProcesseor.java`.
/// Polls keyboard state via a `KeyboardBackend` trait, performs debounce filtering,
/// and emits `InputEvent::KeyChanged` events with lane-index keycodes.
use crate::control_keys::ControlKeys;
use crate::device::InputEvent;
use bms_config::play_mode_config::KeyboardConfig;

/// Special keycode for start key events.
pub const KEYCODE_START: i32 = -1;
/// Special keycode for select key events.
pub const KEYCODE_SELECT: i32 = -2;

/// Platform abstraction for keyboard state queries.
pub trait KeyboardBackend {
    /// Returns `true` if the given raw keycode is currently pressed.
    fn is_key_pressed(&self, keycode: i32) -> bool;
}

/// Virtual keyboard backend for testing.
pub struct VirtualKeyboardBackend {
    pressed: std::collections::HashSet<i32>,
}

impl VirtualKeyboardBackend {
    pub fn new() -> Self {
        Self {
            pressed: std::collections::HashSet::new(),
        }
    }

    pub fn press(&mut self, keycode: i32) {
        self.pressed.insert(keycode);
    }

    pub fn release(&mut self, keycode: i32) {
        self.pressed.remove(&keycode);
    }
}

impl Default for VirtualKeyboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardBackend for VirtualKeyboardBackend {
    fn is_key_pressed(&self, keycode: i32) -> bool {
        self.pressed.contains(&keycode)
    }
}

/// Keyboard input processor with debounce support.
///
/// Tracks per-key state and emits lane-index-based `InputEvent::KeyChanged` events.
/// Start and select keys emit events with `KEYCODE_START` and `KEYCODE_SELECT`.
pub struct KeyboardInput {
    /// Lane key codes from config (up to 18 keys).
    keys: Vec<i32>,
    /// Start key code.
    start_key: i32,
    /// Select key code.
    select_key: i32,
    /// Internal key states for debounce tracking (indexed by raw keycode).
    keystate: [bool; 256],
    /// Last change time per key for debounce (indexed by raw keycode).
    /// Initialized to `i64::MIN` so the first press always passes debounce.
    keytime: [i64; 256],
    /// Debounce duration in microseconds.
    duration_us: i64,
    /// Last pressed raw key code (for config UI).
    last_pressed_key: i32,
}

impl KeyboardInput {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            start_key: -1,
            select_key: -1,
            keystate: [false; 256],
            keytime: [i64::MIN; 256],
            duration_us: 0,
            last_pressed_key: -1,
        }
    }

    /// Apply keyboard configuration (key assignments, start/select, debounce duration).
    pub fn set_config(&mut self, config: &KeyboardConfig) {
        self.keys = config.keys.clone();
        self.start_key = config.start;
        self.select_key = config.select;
        // Java stores duration in ms; convert to microseconds.
        self.duration_us = config.duration as i64 * 1000;
    }

    /// Poll the keyboard backend and return input events for changed keys.
    ///
    /// For lane keys, debounce is applied: a key change is only emitted if
    /// `now_us >= keytime[key] + duration_us`.
    /// Start/select keys do not use debounce (matching Java behavior).
    pub fn poll(&mut self, now_us: i64, backend: &dyn KeyboardBackend) -> Vec<InputEvent> {
        let mut events = Vec::new();

        // Poll lane keys with debounce.
        for i in 0..self.keys.len() {
            let key = self.keys[i];
            if key < 0 {
                continue;
            }
            let key_idx = key as usize;
            if key_idx >= 256 {
                continue;
            }
            let pressed = backend.is_key_pressed(key);
            if pressed != self.keystate[key_idx]
                && now_us >= self.keytime[key_idx].saturating_add(self.duration_us)
            {
                self.keystate[key_idx] = pressed;
                self.keytime[key_idx] = now_us;
                events.push(InputEvent::KeyChanged {
                    keycode: i as i32,
                    pressed,
                    time_us: now_us,
                });
            }
        }

        // Poll start key (no debounce, matching Java).
        if self.start_key >= 0 {
            let key_idx = self.start_key as usize;
            if key_idx < 256 {
                let pressed = backend.is_key_pressed(self.start_key);
                if pressed != self.keystate[key_idx] {
                    self.keystate[key_idx] = pressed;
                    events.push(InputEvent::KeyChanged {
                        keycode: KEYCODE_START,
                        pressed,
                        time_us: now_us,
                    });
                }
            }
        }

        // Poll select key (no debounce, matching Java).
        if self.select_key >= 0 {
            let key_idx = self.select_key as usize;
            if key_idx < 256 {
                let pressed = backend.is_key_pressed(self.select_key);
                if pressed != self.keystate[key_idx] {
                    self.keystate[key_idx] = pressed;
                    events.push(InputEvent::KeyChanged {
                        keycode: KEYCODE_SELECT,
                        pressed,
                        time_us: now_us,
                    });
                }
            }
        }

        events
    }

    /// Check if a control key is currently pressed on the backend.
    pub fn get_control_key_state(&self, key: ControlKeys, backend: &dyn KeyboardBackend) -> bool {
        backend.is_key_pressed(key.keycode())
    }

    /// Reset all internal key states and timestamps.
    pub fn clear(&mut self) {
        self.keystate.fill(false);
        self.keytime.fill(i64::MIN);
        self.last_pressed_key = -1;
    }

    /// Get the last pressed raw key code (-1 if none).
    pub fn last_pressed_key(&self) -> i32 {
        self.last_pressed_key
    }

    /// Set the last pressed raw key code (called from keyDown callback).
    pub fn set_last_pressed_key(&mut self, keycode: i32) {
        self.last_pressed_key = keycode;
    }
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(keys: Vec<i32>, start: i32, select: i32, duration_ms: i32) -> KeyboardConfig {
        KeyboardConfig {
            keys,
            start,
            select,
            duration: duration_ms,
            ..KeyboardConfig::default()
        }
    }

    #[test]
    fn basic_key_press() {
        let mut kb = KeyboardInput::new();
        let config = make_config(vec![10, 20, 30], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();
        backend.press(10);

        let events = kb.poll(1000, &backend);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0, // lane index
                pressed: true,
                time_us: 1000,
            }
        );
    }

    #[test]
    fn key_release() {
        let mut kb = KeyboardInput::new();
        let config = make_config(vec![10], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press first
        backend.press(10);
        kb.poll(1000, &backend);

        // Release
        backend.release(10);
        let events = kb.poll(2000, &backend);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0,
                pressed: false,
                time_us: 2000,
            }
        );
    }

    #[test]
    fn debounce_blocks_rapid_change() {
        let mut kb = KeyboardInput::new();
        // 16ms debounce = 16000us
        let config = make_config(vec![10], 40, 50, 16);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press at t=0
        backend.press(10);
        let events = kb.poll(0, &backend);
        assert_eq!(events.len(), 1);

        // Release at t=5000us (within 16000us debounce) -> blocked
        backend.release(10);
        let events = kb.poll(5000, &backend);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn debounce_allows_after_duration() {
        let mut kb = KeyboardInput::new();
        // 16ms debounce = 16000us
        let config = make_config(vec![10], 40, 50, 16);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press at t=0
        backend.press(10);
        let events = kb.poll(0, &backend);
        assert_eq!(events.len(), 1);

        // Release at t=16000us (exactly at debounce boundary) -> allowed
        backend.release(10);
        let events = kb.poll(16000, &backend);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0,
                pressed: false,
                time_us: 16000,
            }
        );
    }

    #[test]
    fn multiple_keys_simultaneously() {
        let mut kb = KeyboardInput::new();
        let config = make_config(vec![10, 20, 30], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();
        backend.press(10);
        backend.press(30);

        let events = kb.poll(1000, &backend);
        assert_eq!(events.len(), 2);

        // Lane 0 (key 10)
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0,
                pressed: true,
                time_us: 1000,
            }
        );
        // Lane 2 (key 30)
        assert_eq!(
            events[1],
            InputEvent::KeyChanged {
                keycode: 2,
                pressed: true,
                time_us: 1000,
            }
        );
    }

    #[test]
    fn config_change() {
        let mut kb = KeyboardInput::new();
        let config1 = make_config(vec![10], 40, 50, 0);
        kb.set_config(&config1);

        let mut backend = VirtualKeyboardBackend::new();
        backend.press(10);
        let events = kb.poll(1000, &backend);
        assert_eq!(events.len(), 1);

        // Change config to use different key
        let config2 = make_config(vec![20], 40, 50, 0);
        kb.set_config(&config2);

        // Key 10 is still pressed but no longer configured -> no event for it
        // Key 20 is not pressed -> no event
        let events = kb.poll(2000, &backend);
        assert_eq!(events.len(), 0);

        // Press key 20 (the new config key)
        backend.press(20);
        let events = kb.poll(3000, &backend);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0,
                pressed: true,
                time_us: 3000,
            }
        );
    }

    #[test]
    fn control_key_state() {
        let kb = KeyboardInput::new();
        let mut backend = VirtualKeyboardBackend::new();

        // F1 not pressed
        assert!(!kb.get_control_key_state(ControlKeys::F1, &backend));

        // Press F1 (LibGDX keycode 244)
        backend.press(ControlKeys::F1.keycode());
        assert!(kb.get_control_key_state(ControlKeys::F1, &backend));

        // Release
        backend.release(ControlKeys::F1.keycode());
        assert!(!kb.get_control_key_state(ControlKeys::F1, &backend));
    }

    #[test]
    fn start_select_key_events() {
        let mut kb = KeyboardInput::new();
        let config = make_config(vec![10], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press start
        backend.press(40);
        let events = kb.poll(1000, &backend);
        assert!(events.contains(&InputEvent::KeyChanged {
            keycode: KEYCODE_START,
            pressed: true,
            time_us: 1000,
        }));

        // Press select
        backend.press(50);
        let events = kb.poll(2000, &backend);
        assert!(events.contains(&InputEvent::KeyChanged {
            keycode: KEYCODE_SELECT,
            pressed: true,
            time_us: 2000,
        }));

        // Release start
        backend.release(40);
        let events = kb.poll(3000, &backend);
        assert!(events.contains(&InputEvent::KeyChanged {
            keycode: KEYCODE_START,
            pressed: false,
            time_us: 3000,
        }));
    }

    #[test]
    fn start_select_no_debounce() {
        let mut kb = KeyboardInput::new();
        // 100ms debounce
        let config = make_config(vec![10], 40, 50, 100);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press start at t=0
        backend.press(40);
        let events = kb.poll(0, &backend);
        assert!(events.contains(&InputEvent::KeyChanged {
            keycode: KEYCODE_START,
            pressed: true,
            time_us: 0,
        }));

        // Release start at t=1000us (well within debounce) -> should still emit
        backend.release(40);
        let events = kb.poll(1000, &backend);
        assert!(events.contains(&InputEvent::KeyChanged {
            keycode: KEYCODE_START,
            pressed: false,
            time_us: 1000,
        }));
    }

    #[test]
    fn clear_resets_state() {
        let mut kb = KeyboardInput::new();
        let config = make_config(vec![10], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press key
        backend.press(10);
        kb.poll(1000, &backend);

        // Clear state
        kb.clear();

        // Key is still physically pressed -> poll should emit event again
        let events = kb.poll(2000, &backend);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0,
                pressed: true,
                time_us: 2000,
            }
        );
    }

    #[test]
    fn no_event_when_state_unchanged() {
        let mut kb = KeyboardInput::new();
        let config = make_config(vec![10], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();

        // Press key
        backend.press(10);
        let events = kb.poll(1000, &backend);
        assert_eq!(events.len(), 1);

        // Poll again without state change -> no event
        let events = kb.poll(2000, &backend);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn negative_key_code_skipped() {
        let mut kb = KeyboardInput::new();
        // keys[1] = -1 should be skipped
        let config = make_config(vec![10, -1, 30], 40, 50, 0);
        kb.set_config(&config);

        let mut backend = VirtualKeyboardBackend::new();
        backend.press(10);
        backend.press(30);

        let events = kb.poll(1000, &backend);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            InputEvent::KeyChanged {
                keycode: 0,
                pressed: true,
                time_us: 1000,
            }
        );
        assert_eq!(
            events[1],
            InputEvent::KeyChanged {
                keycode: 2,
                pressed: true,
                time_us: 1000,
            }
        );
    }

    #[test]
    fn last_pressed_key() {
        let mut kb = KeyboardInput::new();
        assert_eq!(kb.last_pressed_key(), -1);

        kb.set_last_pressed_key(42);
        assert_eq!(kb.last_pressed_key(), 42);

        kb.clear();
        assert_eq!(kb.last_pressed_key(), -1);
    }

    #[test]
    fn default_trait() {
        let kb = KeyboardInput::default();
        assert_eq!(kb.last_pressed_key(), -1);
        assert_eq!(kb.keys.len(), 0);
    }
}
