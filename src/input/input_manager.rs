use crate::input::gamepad::GamepadInput;
use crate::input::key_config::KeyConfig;
use crate::input::key_input_log::InputLogger;
use crate::input::key_state::KeyState;
use crate::input::keyboard::KeyboardInput;
use crate::model::note::{LANE_COUNT, Lane};
use anyhow::Result;
use gilrs::{EventType, Gilrs};
use std::time::Instant;
use tracing::{debug, warn};

/// Unified input manager for keyboard and gamepad.
pub struct InputManager {
    gilrs: Option<Gilrs>,
    keyboard: KeyboardInput,
    gamepads: Vec<GamepadInput>,
    key_config: KeyConfig,
    /// Key states for all supported lanes.
    lane_states: [KeyState; LANE_COUNT],
    /// Reference time for microsecond timestamps.
    start_time: Instant,
    /// Input logger for replay recording.
    logger: Option<InputLogger>,
}

impl InputManager {
    /// Create a new InputManager with the given configuration.
    pub fn new(key_config: KeyConfig) -> Result<Self> {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                warn!("Failed to initialize gamepad support: {}", e);
                None
            }
        };

        Ok(Self {
            gilrs,
            keyboard: KeyboardInput::new(),
            gamepads: Vec::new(),
            key_config,
            lane_states: [KeyState::default(); LANE_COUNT],
            start_time: Instant::now(),
            logger: None,
        })
    }

    /// Enable input logging for replay.
    pub fn enable_logging(&mut self) {
        self.logger = Some(InputLogger::new());
    }

    /// Disable input logging and return the logger.
    pub fn take_logger(&mut self) -> Option<InputLogger> {
        self.logger.take()
    }

    /// Get the current time in microseconds since start.
    pub fn current_time_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    /// Reset the start time (call at game start).
    pub fn reset_time(&mut self) {
        self.start_time = Instant::now();
    }

    /// Update input states. Call once per frame.
    pub fn update(&mut self) {
        let time_us = self.current_time_us();

        // Reset frame-specific state
        for state in &mut self.lane_states {
            state.reset_frame_state();
        }

        // Update keyboard
        let keyboard_changes =
            self.keyboard
                .update(&self.key_config.keyboard, &mut self.lane_states, time_us);

        // Log keyboard changes
        if let Some(ref mut logger) = self.logger {
            for (lane_idx, pressed) in &keyboard_changes {
                logger.record(time_us, *lane_idx as u8, *pressed);
            }
        }

        // Update gamepads
        if let Some(ref mut gilrs) = self.gilrs {
            while let Some(event) = gilrs.next_event() {
                // Handle connection/disconnection
                match event.event {
                    EventType::Connected => {
                        if !self.gamepads.iter().any(|g| g.gamepad_id() == event.id) {
                            self.gamepads.push(GamepadInput::new(event.id));
                            debug!("Gamepad connected: {:?}", event.id);
                        }
                    }
                    EventType::Disconnected => {
                        self.gamepads.retain(|g| g.gamepad_id() != event.id);
                        debug!("Gamepad disconnected: {:?}", event.id);
                    }
                    _ => {
                        // Process input events
                        if let Some(ref gamepad_config) = self.key_config.gamepad {
                            for gamepad in &mut self.gamepads {
                                let changes = gamepad.process_event(
                                    &event,
                                    gamepad_config,
                                    &mut self.lane_states,
                                    time_us,
                                );

                                // Log gamepad changes
                                if let Some(ref mut logger) = self.logger {
                                    for (lane_idx, pressed) in changes {
                                        logger.record(time_us, lane_idx as u8, pressed);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if a lane is currently pressed.
    pub fn is_pressed(&self, lane: Lane) -> bool {
        self.lane_states[lane.index()].pressed
    }

    /// Check if a lane was just pressed this frame.
    pub fn just_pressed(&self, lane: Lane) -> bool {
        self.lane_states[lane.index()].just_pressed
    }

    /// Check if a lane was just released this frame.
    pub fn just_released(&self, lane: Lane) -> bool {
        self.lane_states[lane.index()].just_released
    }

    /// Get the press timestamp for a lane (microseconds).
    pub fn press_time_us(&self, lane: Lane) -> u64 {
        self.lane_states[lane.index()].press_time_us
    }

    /// Get the release timestamp for a lane (microseconds).
    pub fn release_time_us(&self, lane: Lane) -> u64 {
        self.lane_states[lane.index()].release_time_us
    }

    /// Get the key state for a lane.
    pub fn lane_state(&self, lane: Lane) -> &KeyState {
        &self.lane_states[lane.index()]
    }

    /// Get all lane states.
    pub fn all_lane_states(&self) -> &[KeyState; LANE_COUNT] {
        &self.lane_states
    }

    /// Get the number of connected gamepads.
    pub fn connected_gamepad_count(&self) -> usize {
        self.gamepads.len()
    }

    /// Get the key configuration.
    pub fn key_config(&self) -> &KeyConfig {
        &self.key_config
    }

    /// Update key configuration.
    pub fn set_key_config(&mut self, config: KeyConfig) {
        self.key_config = config;
    }

    /// Check if start key is pressed (keyboard).
    pub fn is_start_pressed(&self) -> bool {
        self.keyboard.is_start_pressed(&self.key_config.keyboard)
    }

    /// Check if select key is pressed (keyboard).
    pub fn is_select_pressed(&self) -> bool {
        self.keyboard.is_select_pressed(&self.key_config.keyboard)
    }
}
