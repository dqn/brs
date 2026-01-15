use macroquad::prelude::*;

use super::{Scene, SceneTransition};
use crate::config::GameSettings;
use crate::game::{GaugeType, JudgeSystemType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuItem {
    JudgeSystem,
    GaugeType,
    ScrollSpeed,
    Sudden,
    Lift,
    Save,
    Back,
}

impl MenuItem {
    fn all() -> &'static [MenuItem] {
        &[
            MenuItem::JudgeSystem,
            MenuItem::GaugeType,
            MenuItem::ScrollSpeed,
            MenuItem::Sudden,
            MenuItem::Lift,
            MenuItem::Save,
            MenuItem::Back,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MenuItem::JudgeSystem => "Judge System",
            MenuItem::GaugeType => "Gauge Type",
            MenuItem::ScrollSpeed => "Scroll Speed",
            MenuItem::Sudden => "SUDDEN+",
            MenuItem::Lift => "LIFT",
            MenuItem::Save => "Save Settings",
            MenuItem::Back => "Back",
        }
    }
}

pub struct SettingsScene {
    settings: GameSettings,
    selected_index: usize,
    modified: bool,
}

impl SettingsScene {
    pub fn new() -> Self {
        Self {
            settings: GameSettings::load(),
            selected_index: 0,
            modified: false,
        }
    }

    fn current_item(&self) -> MenuItem {
        MenuItem::all()[self.selected_index]
    }

    fn adjust_value(&mut self, delta: i32) {
        self.modified = true;
        match self.current_item() {
            MenuItem::JudgeSystem => {
                self.settings.judge_system = match self.settings.judge_system {
                    JudgeSystemType::Beatoraja => JudgeSystemType::Lr2,
                    JudgeSystemType::Lr2 => JudgeSystemType::Beatoraja,
                };
            }
            MenuItem::GaugeType => {
                self.settings.gauge_type = match (self.settings.gauge_type, delta > 0) {
                    (GaugeType::AssistEasy, true) => GaugeType::Easy,
                    (GaugeType::Easy, true) => GaugeType::Normal,
                    (GaugeType::Normal, true) => GaugeType::Hard,
                    (GaugeType::Hard, true) => GaugeType::ExHard,
                    (GaugeType::ExHard, true) => GaugeType::Hazard,
                    (GaugeType::Hazard, true) => GaugeType::AssistEasy,

                    (GaugeType::AssistEasy, false) => GaugeType::Hazard,
                    (GaugeType::Easy, false) => GaugeType::AssistEasy,
                    (GaugeType::Normal, false) => GaugeType::Easy,
                    (GaugeType::Hard, false) => GaugeType::Normal,
                    (GaugeType::ExHard, false) => GaugeType::Hard,
                    (GaugeType::Hazard, false) => GaugeType::ExHard,
                };
            }
            MenuItem::ScrollSpeed => {
                let step = if delta > 0 { 0.25 } else { -0.25 };
                self.settings.scroll_speed = (self.settings.scroll_speed + step).clamp(0.25, 10.0);
            }
            MenuItem::Sudden => {
                let step = if delta > 0 { 50 } else { -50 };
                self.settings.sudden = (self.settings.sudden as i32 + step).clamp(0, 1000) as u16;
            }
            MenuItem::Lift => {
                let step = if delta > 0 { 50 } else { -50 };
                self.settings.lift = (self.settings.lift as i32 + step).clamp(0, 500) as u16;
            }
            MenuItem::Save | MenuItem::Back => {}
        }
    }

    fn format_value(&self, item: MenuItem) -> String {
        match item {
            MenuItem::JudgeSystem => match self.settings.judge_system {
                JudgeSystemType::Beatoraja => "beatoraja".to_string(),
                JudgeSystemType::Lr2 => "LR2".to_string(),
            },
            MenuItem::GaugeType => match self.settings.gauge_type {
                GaugeType::AssistEasy => "ASSIST EASY".to_string(),
                GaugeType::Easy => "EASY".to_string(),
                GaugeType::Normal => "NORMAL".to_string(),
                GaugeType::Hard => "HARD".to_string(),
                GaugeType::ExHard => "EX-HARD".to_string(),
                GaugeType::Hazard => "HAZARD".to_string(),
            },
            MenuItem::ScrollSpeed => format!("{:.2}x", self.settings.scroll_speed),
            MenuItem::Sudden => format!("{}", self.settings.sudden),
            MenuItem::Lift => format!("{}", self.settings.lift),
            MenuItem::Save => "".to_string(),
            MenuItem::Back => "".to_string(),
        }
    }

    fn save_settings(&mut self) {
        if let Err(e) = self.settings.save() {
            eprintln!("Failed to save settings: {}", e);
        } else {
            self.modified = false;
        }
    }
}

impl Scene for SettingsScene {
    fn update(&mut self) -> SceneTransition {
        let items = MenuItem::all();

        if is_key_pressed(KeyCode::Up) && self.selected_index > 0 {
            self.selected_index -= 1;
        }

        if is_key_pressed(KeyCode::Down) && self.selected_index < items.len() - 1 {
            self.selected_index += 1;
        }

        if is_key_pressed(KeyCode::Left) {
            self.adjust_value(-1);
        }

        if is_key_pressed(KeyCode::Right) {
            self.adjust_value(1);
        }

        if is_key_pressed(KeyCode::Enter) {
            match self.current_item() {
                MenuItem::Save => {
                    self.save_settings();
                }
                MenuItem::Back => {
                    if self.modified {
                        self.save_settings();
                    }
                    return SceneTransition::Pop;
                }
                _ => {
                    self.adjust_value(1);
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            if self.modified {
                self.save_settings();
            }
            return SceneTransition::Pop;
        }

        SceneTransition::None
    }

    fn draw(&self) {
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));

        draw_text("SETTINGS", 20.0, 40.0, 32.0, WHITE);

        if self.modified {
            draw_text("(modified)", 150.0, 40.0, 18.0, YELLOW);
        }

        let start_y = 100.0;
        let item_height = 45.0;
        let items = MenuItem::all();

        for (i, &item) in items.iter().enumerate() {
            let y = start_y + i as f32 * item_height;
            let is_selected = i == self.selected_index;

            if is_selected {
                draw_rectangle(
                    10.0,
                    y - 5.0,
                    screen_width() - 20.0,
                    item_height - 5.0,
                    Color::new(0.2, 0.3, 0.5, 1.0),
                );
            }

            let color = if is_selected { YELLOW } else { WHITE };
            let label = item.label();
            let value = self.format_value(item);

            draw_text(label, 30.0, y + 22.0, 24.0, color);

            if !value.is_empty() {
                let value_color = if is_selected { SKYBLUE } else { GRAY };
                draw_text(&value, 300.0, y + 22.0, 24.0, value_color);

                if is_selected {
                    draw_text("<", 260.0, y + 22.0, 24.0, GRAY);
                    let value_width = value.len() as f32 * 12.0;
                    draw_text(">", 310.0 + value_width, y + 22.0, 24.0, GRAY);
                }
            }
        }

        draw_text(
            "[Up/Down] Select | [Left/Right] Adjust | [Enter] Confirm | [Esc] Back",
            20.0,
            screen_height() - 20.0,
            16.0,
            GRAY,
        );
    }
}
