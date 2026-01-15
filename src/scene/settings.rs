use macroquad::prelude::*;

use super::{Scene, SceneTransition};
use crate::config::GameSettings;
use crate::game::{GaugeType, JudgeSystemType, RandomOption};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuItem {
    JudgeSystem,
    GaugeType,
    RandomOption,
    ScrollSpeed,
    Sudden,
    Hidden,
    Lift,
    KeyScratch,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Save,
    Back,
}

impl MenuItem {
    fn all() -> &'static [MenuItem] {
        &[
            MenuItem::JudgeSystem,
            MenuItem::GaugeType,
            MenuItem::RandomOption,
            MenuItem::ScrollSpeed,
            MenuItem::Sudden,
            MenuItem::Hidden,
            MenuItem::Lift,
            MenuItem::KeyScratch,
            MenuItem::Key1,
            MenuItem::Key2,
            MenuItem::Key3,
            MenuItem::Key4,
            MenuItem::Key5,
            MenuItem::Key6,
            MenuItem::Key7,
            MenuItem::Save,
            MenuItem::Back,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MenuItem::JudgeSystem => "Judge System",
            MenuItem::GaugeType => "Gauge Type",
            MenuItem::RandomOption => "Random",
            MenuItem::ScrollSpeed => "Scroll Speed",
            MenuItem::Sudden => "SUDDEN+",
            MenuItem::Hidden => "HIDDEN+",
            MenuItem::Lift => "LIFT",
            MenuItem::KeyScratch => "Key: Scratch",
            MenuItem::Key1 => "Key: 1",
            MenuItem::Key2 => "Key: 2",
            MenuItem::Key3 => "Key: 3",
            MenuItem::Key4 => "Key: 4",
            MenuItem::Key5 => "Key: 5",
            MenuItem::Key6 => "Key: 6",
            MenuItem::Key7 => "Key: 7",
            MenuItem::Save => "Save Settings",
            MenuItem::Back => "Back",
        }
    }

    fn is_key_binding(&self) -> bool {
        matches!(
            self,
            MenuItem::KeyScratch
                | MenuItem::Key1
                | MenuItem::Key2
                | MenuItem::Key3
                | MenuItem::Key4
                | MenuItem::Key5
                | MenuItem::Key6
                | MenuItem::Key7
        )
    }

    fn key_lane(&self) -> Option<usize> {
        match self {
            MenuItem::KeyScratch => Some(0),
            MenuItem::Key1 => Some(1),
            MenuItem::Key2 => Some(2),
            MenuItem::Key3 => Some(3),
            MenuItem::Key4 => Some(4),
            MenuItem::Key5 => Some(5),
            MenuItem::Key6 => Some(6),
            MenuItem::Key7 => Some(7),
            _ => None,
        }
    }
}

pub struct SettingsScene {
    settings: GameSettings,
    selected_index: usize,
    modified: bool,
    waiting_for_key: Option<usize>,
}

impl SettingsScene {
    pub fn new() -> Self {
        Self {
            settings: GameSettings::load(),
            selected_index: 0,
            modified: false,
            waiting_for_key: None,
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
            MenuItem::RandomOption => {
                self.settings.random_option = match (self.settings.random_option, delta > 0) {
                    (RandomOption::Off, true) => RandomOption::Mirror,
                    (RandomOption::Mirror, true) => RandomOption::Random,
                    (RandomOption::Random, true) => RandomOption::RRandom,
                    (RandomOption::RRandom, true) => RandomOption::SRandom,
                    (RandomOption::SRandom, true) => RandomOption::Off,

                    (RandomOption::Off, false) => RandomOption::SRandom,
                    (RandomOption::Mirror, false) => RandomOption::Off,
                    (RandomOption::Random, false) => RandomOption::Mirror,
                    (RandomOption::RRandom, false) => RandomOption::Random,
                    (RandomOption::SRandom, false) => RandomOption::RRandom,
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
            MenuItem::Hidden => {
                let step = if delta > 0 { 50 } else { -50 };
                self.settings.hidden = (self.settings.hidden as i32 + step).clamp(0, 500) as u16;
            }
            MenuItem::Lift => {
                let step = if delta > 0 { 50 } else { -50 };
                self.settings.lift = (self.settings.lift as i32 + step).clamp(0, 500) as u16;
            }
            MenuItem::KeyScratch
            | MenuItem::Key1
            | MenuItem::Key2
            | MenuItem::Key3
            | MenuItem::Key4
            | MenuItem::Key5
            | MenuItem::Key6
            | MenuItem::Key7
            | MenuItem::Save
            | MenuItem::Back => {}
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
            MenuItem::RandomOption => self.settings.random_option.display_name().to_string(),
            MenuItem::ScrollSpeed => format!("{:.2}x", self.settings.scroll_speed),
            MenuItem::Sudden => format!("{}", self.settings.sudden),
            MenuItem::Hidden => format!("{}", self.settings.hidden),
            MenuItem::Lift => format!("{}", self.settings.lift),
            MenuItem::KeyScratch => self.settings.key_bindings.scratch.clone(),
            MenuItem::Key1 => self.settings.key_bindings.key1.clone(),
            MenuItem::Key2 => self.settings.key_bindings.key2.clone(),
            MenuItem::Key3 => self.settings.key_bindings.key3.clone(),
            MenuItem::Key4 => self.settings.key_bindings.key4.clone(),
            MenuItem::Key5 => self.settings.key_bindings.key5.clone(),
            MenuItem::Key6 => self.settings.key_bindings.key6.clone(),
            MenuItem::Key7 => self.settings.key_bindings.key7.clone(),
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
        // Handle key binding input
        if let Some(lane) = self.waiting_for_key {
            if is_key_pressed(KeyCode::Escape) {
                self.waiting_for_key = None;
                return SceneTransition::None;
            }

            if let Some(key) = get_last_key_pressed() {
                self.settings.key_bindings.set(lane, key);
                self.modified = true;
                self.waiting_for_key = None;
            }
            return SceneTransition::None;
        }

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
            let current = self.current_item();
            if current.is_key_binding() {
                if let Some(lane) = current.key_lane() {
                    self.waiting_for_key = Some(lane);
                }
            } else {
                match current {
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

        let start_y = 80.0;
        let item_height = 35.0;
        let items = MenuItem::all();
        let visible_items = ((screen_height() - 150.0) / item_height) as usize;

        // Calculate scroll offset to keep selected item visible
        let scroll_offset = if self.selected_index >= visible_items {
            self.selected_index - visible_items + 1
        } else {
            0
        };

        for (i, &item) in items.iter().enumerate().skip(scroll_offset) {
            let display_index = i - scroll_offset;
            let y = start_y + display_index as f32 * item_height;

            if y > screen_height() - 80.0 {
                break;
            }

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
            let value = if self.waiting_for_key.is_some() && item.is_key_binding() && is_selected {
                "Press any key...".to_string()
            } else {
                self.format_value(item)
            };

            draw_text(label, 30.0, y + 18.0, 20.0, color);

            if !value.is_empty() {
                let value_color = if is_selected {
                    if self.waiting_for_key.is_some() {
                        ORANGE
                    } else {
                        SKYBLUE
                    }
                } else {
                    GRAY
                };
                draw_text(&value, 250.0, y + 18.0, 20.0, value_color);

                if is_selected && !item.is_key_binding() && self.waiting_for_key.is_none() {
                    draw_text("<", 220.0, y + 18.0, 20.0, GRAY);
                    let value_width = value.len() as f32 * 10.0;
                    draw_text(">", 260.0 + value_width, y + 18.0, 20.0, GRAY);
                }
            }
        }

        let help_text = if self.waiting_for_key.is_some() {
            "[Any Key] Set binding | [Esc] Cancel"
        } else {
            "[Up/Down] Select | [Left/Right] Adjust | [Enter] Confirm/Set Key | [Esc] Back"
        };

        draw_text(help_text, 20.0, screen_height() - 20.0, 14.0, GRAY);
    }
}
