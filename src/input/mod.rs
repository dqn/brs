//! Input handling using gilrs and keyboard.
//!
//! This module provides:
//! - [`InputManager`]: Unified input handling for keyboard and gamepad
//! - [`KeyConfig`]: Key binding configuration with save/load
//! - [`InputLogger`]: Input recording for replay system
//! - [`KeyState`]: Individual key state with timestamps

mod gamepad;
mod input_manager;
mod key_config;
mod key_input_log;
mod key_state;
mod keyboard;

pub use input_manager::InputManager;
pub use key_config::{GamepadConfig, KeyConfig, KeyboardConfig, SerializableKeyCode};
pub use key_input_log::{InputLogger, KeyInputLog};
pub use key_state::KeyState;
