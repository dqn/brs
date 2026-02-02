use macroquad::prelude::*;

use crate::input::KeyConfig;

/// Key configuration screen for remapping keys.
pub struct KeyConfigScreen {
    key_config: KeyConfig,
    selected_lane: usize,
    waiting_for_key: bool,
}

impl KeyConfigScreen {
    /// Create a new key config screen.
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            key_config,
            selected_lane: 0,
            waiting_for_key: false,
        }
    }

    /// Update the screen. Returns true if should return to previous screen.
    pub fn update(&mut self) -> bool {
        if self.waiting_for_key {
            // Check for any key press
            if let Some(key) = get_last_key_pressed() {
                if key == KeyCode::Escape {
                    self.waiting_for_key = false;
                } else {
                    self.assign_key(key);
                    self.waiting_for_key = false;
                }
            }
        } else {
            // Navigation
            if is_key_pressed(KeyCode::Up) {
                self.selected_lane = self.selected_lane.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) {
                self.selected_lane = (self.selected_lane + 1).min(8); // 8 lanes + save
            }

            // Selection
            if is_key_pressed(KeyCode::Enter) {
                if self.selected_lane < 8 {
                    self.waiting_for_key = true;
                } else {
                    // Save and return
                    self.save_config();
                    return true;
                }
            }

            // Back without saving
            if is_key_pressed(KeyCode::Escape) {
                return true;
            }
        }

        false
    }

    fn assign_key(&mut self, key: KeyCode) {
        let lanes = &mut self.key_config.keyboard.lanes;
        if self.selected_lane < lanes.len() {
            lanes[self.selected_lane] = key.into();
        }
    }

    fn save_config(&self) {
        // Save to keyconfig.json
        if let Ok(json) = serde_json::to_string_pretty(&self.key_config) {
            let _ = std::fs::write("keyconfig.json", json);
        }
    }

    /// Draw the screen.
    pub fn draw(&self) {
        let x = 100.0;
        let mut y = 100.0;

        draw_text("=== KEY CONFIG ===", x, y, 48.0, WHITE);
        y += 60.0;

        let lane_names = [
            "Scratch",
            "Key 1 (S)",
            "Key 2 (D)",
            "Key 3 (F)",
            "Key 4 (Space)",
            "Key 5 (J)",
            "Key 6 (K)",
            "Key 7 (L)",
        ];

        for (i, name) in lane_names.iter().enumerate() {
            let is_selected = i == self.selected_lane;
            let color = if is_selected {
                if self.waiting_for_key {
                    Color::new(1.0, 0.5, 0.5, 1.0)
                } else {
                    YELLOW
                }
            } else {
                WHITE
            };

            let prefix = if is_selected { "> " } else { "  " };
            let key_name = self.get_key_name(i);

            let text = if is_selected && self.waiting_for_key {
                format!("{}{}  [Press a key...]", prefix, name)
            } else {
                format!("{}{}  [{}]", prefix, name, key_name)
            };

            draw_text(&text, x, y, 20.0, color);
            y += 30.0;
        }

        y += 20.0;

        // Save option
        let is_save_selected = self.selected_lane == 8;
        let save_color = if is_save_selected { YELLOW } else { WHITE };
        let save_prefix = if is_save_selected { "> " } else { "  " };
        draw_text(
            &format!("{}Save & Return", save_prefix),
            x,
            y,
            20.0,
            save_color,
        );

        y += 50.0;
        draw_text("Up/Down: Navigate", x, y, 16.0, DARKGRAY);
        y += 20.0;
        draw_text("Enter: Change key / Save", x, y, 16.0, DARKGRAY);
        y += 20.0;
        draw_text("Escape: Cancel", x, y, 16.0, DARKGRAY);
    }

    fn get_key_name(&self, lane: usize) -> String {
        let keys = &self.key_config.keyboard.lanes;
        if lane < keys.len() {
            format!("{:?}", keys[lane])
        } else {
            "N/A".to_string()
        }
    }

    /// Get the current key config.
    pub fn key_config(&self) -> &KeyConfig {
        &self.key_config
    }
}
