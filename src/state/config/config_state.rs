use anyhow::Result;
use macroquad::prelude::*;

use crate::input::{InputManager, KeyConfig};
use crate::state::config::key_config_screen::KeyConfigScreen;

/// Available configuration screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigScreen {
    #[default]
    Main,
    KeyConfig,
}

/// Transition from the config screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigTransition {
    #[default]
    None,
    Back,
}

/// Configuration screen state.
pub struct ConfigState {
    current_screen: ConfigScreen,
    selected_item: usize,
    key_config_screen: KeyConfigScreen,
    input_manager: InputManager,
    transition: ConfigTransition,
}

impl ConfigState {
    /// Create a new config state.
    pub fn new(input_manager: InputManager) -> Self {
        Self {
            current_screen: ConfigScreen::Main,
            selected_item: 0,
            key_config_screen: KeyConfigScreen::new(input_manager.key_config().clone()),
            input_manager,
            transition: ConfigTransition::None,
        }
    }

    /// Update the config state.
    pub fn update(&mut self) -> Result<()> {
        self.input_manager.update();

        match self.current_screen {
            ConfigScreen::Main => self.update_main_screen(),
            ConfigScreen::KeyConfig => self.update_key_config_screen(),
        }

        Ok(())
    }

    fn update_main_screen(&mut self) {
        // Navigation
        if is_key_pressed(KeyCode::Up) {
            self.selected_item = self.selected_item.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_item = (self.selected_item + 1).min(1); // 2 items
        }

        // Selection
        if is_key_pressed(KeyCode::Enter) {
            match self.selected_item {
                0 => self.current_screen = ConfigScreen::KeyConfig,
                1 => self.transition = ConfigTransition::Back,
                _ => {}
            }
        }

        // Back
        if is_key_pressed(KeyCode::Escape) {
            self.transition = ConfigTransition::Back;
        }
    }

    fn update_key_config_screen(&mut self) {
        if self.key_config_screen.update() {
            // Config was saved or cancelled, return to main
            self.current_screen = ConfigScreen::Main;
            // Update input manager with new config
            self.input_manager
                .set_key_config(self.key_config_screen.key_config().clone());
        }
    }

    /// Draw the config screen.
    pub fn draw(&self) {
        match self.current_screen {
            ConfigScreen::Main => self.draw_main_screen(),
            ConfigScreen::KeyConfig => self.key_config_screen.draw(),
        }
    }

    fn draw_main_screen(&self) {
        let x = 100.0;
        let mut y = 100.0;

        draw_text("=== CONFIGURATION ===", x, y, 48.0, WHITE);
        y += 80.0;

        let items = ["Key Config", "Back to Select"];

        for (i, item) in items.iter().enumerate() {
            let is_selected = i == self.selected_item;
            let color = if is_selected { YELLOW } else { WHITE };
            let prefix = if is_selected { "> " } else { "  " };

            draw_text(&format!("{}{}", prefix, item), x, y, 24.0, color);
            y += 35.0;
        }

        y += 30.0;
        draw_text("Up/Down: Navigate", x, y, 16.0, DARKGRAY);
        y += 20.0;
        draw_text("Enter: Select", x, y, 16.0, DARKGRAY);
        y += 20.0;
        draw_text("Escape: Back", x, y, 16.0, DARKGRAY);
    }

    /// Take the current transition.
    pub fn take_transition(&mut self) -> ConfigTransition {
        std::mem::take(&mut self.transition)
    }

    /// Take the input manager for reuse.
    pub fn take_input_manager(&mut self) -> InputManager {
        let key_config = self.input_manager.key_config().clone();
        let dummy = InputManager::new(key_config).unwrap();
        std::mem::replace(&mut self.input_manager, dummy)
    }

    /// Get the current key config.
    pub fn key_config(&self) -> &KeyConfig {
        self.input_manager.key_config()
    }
}
