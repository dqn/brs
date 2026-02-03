use macroquad::prelude::*;

use crate::input::KeyConfig;
use crate::model::note::LANE_COUNT;

/// Key configuration screen for remapping keys.
pub struct KeyConfigScreen {
    key_config: KeyConfig,
    selected_item: usize,
    waiting_for_key: bool,
}

impl KeyConfigScreen {
    const START_ITEM_INDEX: usize = LANE_COUNT;
    const SELECT_ITEM_INDEX: usize = LANE_COUNT + 1;
    const SAVE_ITEM_INDEX: usize = LANE_COUNT + 2;

    /// Create a new key config screen.
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            key_config,
            selected_item: 0,
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
                self.selected_item = self.selected_item.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Down) {
                self.selected_item = (self.selected_item + 1).min(Self::SAVE_ITEM_INDEX); // lanes + start/select + save
            }

            // Selection
            if is_key_pressed(KeyCode::Enter) {
                if self.selected_item < Self::SAVE_ITEM_INDEX {
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
        if self.selected_item < lanes.len() {
            lanes[self.selected_item] = key.into();
        } else if self.selected_item == Self::START_ITEM_INDEX {
            self.key_config.keyboard.start = key.into();
        } else if self.selected_item == Self::SELECT_ITEM_INDEX {
            self.key_config.keyboard.select = key.into();
        }
    }

    fn save_config(&self) {
        let _ = self.key_config.save();
    }

    /// Draw the screen.
    pub fn draw(&self) {
        let x = 100.0;
        let mut y = 100.0;

        draw_text("=== KEY CONFIG / キー設定 ===", x, y, 48.0, WHITE);
        y += 60.0;

        let lane_names = [
            "Scratch 1P / スクラッチ1P",
            "Key 1 / 1鍵",
            "Key 2 / 2鍵",
            "Key 3 / 3鍵",
            "Key 4 / 4鍵",
            "Key 5 / 5鍵",
            "Key 6 / 6鍵",
            "Key 7 / 7鍵",
            "Scratch 2P / スクラッチ2P",
            "Key 8 / 8鍵",
            "Key 9 / 9鍵",
            "Key 10 / 10鍵",
            "Key 11 / 11鍵",
            "Key 12 / 12鍵",
            "Key 13 / 13鍵",
            "Key 14 / 14鍵",
        ];

        for (i, name) in lane_names.iter().enumerate() {
            let is_selected = i == self.selected_item;
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
            let key_name = self.get_item_key_name(i);

            let text = if is_selected && self.waiting_for_key {
                format!(
                    "{}{}  [Press a key... / キーを押してください...]",
                    prefix, name
                )
            } else {
                format!("{}{}  [{}]", prefix, name, key_name)
            };

            draw_text(&text, x, y, 20.0, color);
            y += 30.0;
        }

        y += 10.0;

        let start_selected = self.selected_item == Self::START_ITEM_INDEX;
        let start_color = if start_selected { YELLOW } else { WHITE };
        let start_prefix = if start_selected { "> " } else { "  " };
        let start_key = self.get_item_key_name(Self::START_ITEM_INDEX);
        let start_text = if start_selected && self.waiting_for_key {
            format!(
                "{}Start / スタート  [Press a key... / キーを押してください...]",
                start_prefix
            )
        } else {
            format!("{}Start / スタート  [{}]", start_prefix, start_key)
        };
        draw_text(&start_text, x, y, 20.0, start_color);
        y += 30.0;

        let select_selected = self.selected_item == Self::SELECT_ITEM_INDEX;
        let select_color = if select_selected { YELLOW } else { WHITE };
        let select_prefix = if select_selected { "> " } else { "  " };
        let select_key = self.get_item_key_name(Self::SELECT_ITEM_INDEX);
        let select_text = if select_selected && self.waiting_for_key {
            format!(
                "{}Select / セレクト  [Press a key... / キーを押してください...]",
                select_prefix
            )
        } else {
            format!("{}Select / セレクト  [{}]", select_prefix, select_key)
        };
        draw_text(&select_text, x, y, 20.0, select_color);
        y += 30.0;

        // Save option
        let is_save_selected = self.selected_item == Self::SAVE_ITEM_INDEX;
        let save_color = if is_save_selected { YELLOW } else { WHITE };
        let save_prefix = if is_save_selected { "> " } else { "  " };
        draw_text(
            &format!("{}Save & Return / 保存して戻る", save_prefix),
            x,
            y,
            20.0,
            save_color,
        );

        y += 50.0;
        draw_text("Up/Down: Navigate / 移動", x, y, 16.0, DARKGRAY);
        y += 20.0;
        draw_text("Enter: Change key / 変更・保存", x, y, 16.0, DARKGRAY);
        y += 20.0;
        draw_text("Escape: Cancel / キャンセル", x, y, 16.0, DARKGRAY);
    }

    fn get_item_key_name(&self, index: usize) -> String {
        let keys = &self.key_config.keyboard.lanes;
        if index < keys.len() {
            format!("{:?}", keys[index])
        } else if index == Self::START_ITEM_INDEX {
            format!("{:?}", self.key_config.keyboard.start)
        } else if index == Self::SELECT_ITEM_INDEX {
            format!("{:?}", self.key_config.keyboard.select)
        } else {
            "N/A".to_string()
        }
    }

    /// Get the current key config.
    pub fn key_config(&self) -> &KeyConfig {
        &self.key_config
    }
}
