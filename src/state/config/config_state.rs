use anyhow::Result;

use crate::state::game_state::{GameState, StateTransition};

/// Section of the config screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSection {
    /// Key binding configuration.
    KeyConfig,
    /// Skin selection/configuration.
    SkinConfig,
    /// Display settings.
    Display,
    /// Audio settings.
    Audio,
    /// General settings.
    General,
}

/// Input actions for the config screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigInput {
    Up,
    Down,
    Left,
    Right,
    Decide,
    Back,
}

/// State for the configuration screen.
///
/// Corresponds to beatoraja's KeyConfiguration and SkinConfiguration.
/// Manages settings UI for key bindings, skins, and other options.
pub struct ConfigState {
    /// Current config section.
    section: ConfigSection,
    /// Cursor position within current section.
    cursor: usize,
    /// Whether exit was requested.
    exit_requested: bool,
    /// Pending input actions.
    pending_inputs: Vec<ConfigInput>,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            section: ConfigSection::KeyConfig,
            cursor: 0,
            exit_requested: false,
            pending_inputs: Vec::new(),
        }
    }

    /// Get the current section.
    pub fn section(&self) -> ConfigSection {
        self.section
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Set the config section.
    pub fn set_section(&mut self, section: ConfigSection) {
        self.section = section;
        self.cursor = 0;
    }

    /// Queue an input.
    pub fn push_input(&mut self, input: ConfigInput) {
        self.pending_inputs.push(input);
    }

    fn process_inputs(&mut self) -> StateTransition {
        let inputs: Vec<ConfigInput> = self.pending_inputs.drain(..).collect();
        for input in inputs {
            match input {
                ConfigInput::Up => {
                    self.cursor = self.cursor.saturating_sub(1);
                }
                ConfigInput::Down => {
                    self.cursor += 1;
                }
                ConfigInput::Left | ConfigInput::Right | ConfigInput::Decide => {
                    // Value adjustment handled by specific config sections
                }
                ConfigInput::Back => {
                    self.exit_requested = true;
                    return StateTransition::Back;
                }
            }
        }
        StateTransition::None
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState for ConfigState {
    fn create(&mut self) -> Result<()> {
        self.cursor = 0;
        self.exit_requested = false;
        self.pending_inputs.clear();
        Ok(())
    }

    fn update(&mut self, _dt_us: i64) -> Result<StateTransition> {
        let transition = self.process_inputs();
        Ok(transition)
    }

    fn dispose(&mut self) {
        self.pending_inputs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let state = ConfigState::new();
        assert_eq!(state.section(), ConfigSection::KeyConfig);
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn navigation() {
        let mut state = ConfigState::new();
        state.create().unwrap();

        state.push_input(ConfigInput::Down);
        state.update(16_667).unwrap();
        assert_eq!(state.cursor(), 1);

        state.push_input(ConfigInput::Down);
        state.update(16_667).unwrap();
        assert_eq!(state.cursor(), 2);

        state.push_input(ConfigInput::Up);
        state.update(16_667).unwrap();
        assert_eq!(state.cursor(), 1);
    }

    #[test]
    fn cursor_does_not_go_below_zero() {
        let mut state = ConfigState::new();
        state.create().unwrap();

        state.push_input(ConfigInput::Up);
        state.update(16_667).unwrap();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn back_transitions_back() {
        let mut state = ConfigState::new();
        state.create().unwrap();

        state.push_input(ConfigInput::Back);
        let transition = state.update(16_667).unwrap();
        assert_eq!(transition, StateTransition::Back);
    }

    #[test]
    fn set_section() {
        let mut state = ConfigState::new();
        state.create().unwrap();

        state.push_input(ConfigInput::Down);
        state.update(16_667).unwrap();
        assert_eq!(state.cursor(), 1);

        state.set_section(ConfigSection::SkinConfig);
        assert_eq!(state.section(), ConfigSection::SkinConfig);
        assert_eq!(state.cursor(), 0); // Reset on section change
    }
}
