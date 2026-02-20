use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;

/// Trainer configuration panel.
///
/// Provides enable/disable toggles and configuration fields for
/// the frequency, random, and judge trainers.
pub struct TrainerPanel {
    freq_enabled: bool,
    freq_value: i32,
    random_enabled: bool,
    random_lane_order: [u8; 7],
    judge_enabled: bool,
    judge_rank: i32,
    dirty: bool,
}

const JUDGE_RANK_LABELS: &[&str] = &["EASY", "NORMAL", "HARD", "VERY HARD"];

impl Default for TrainerPanel {
    fn default() -> Self {
        Self {
            freq_enabled: false,
            freq_value: 100,
            random_enabled: false,
            random_lane_order: [1, 2, 3, 4, 5, 6, 7],
            judge_enabled: false,
            judge_rank: 0,
            dirty: false,
        }
    }
}

impl LauncherPanel for TrainerPanel {
    fn tab(&self) -> Tab {
        Tab::Trainer
    }

    fn load(&mut self, _config: &Config, _player_config: &PlayerConfig) {
        // Trainer state lives in PlayerResource at runtime,
        // not in saved config. Load defaults.
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Trainer Settings");
        ui.separator();

        // Frequency Trainer
        ui.group(|ui| {
            ui.strong("Frequency Trainer");
            let prev = (self.freq_enabled, self.freq_value);
            ui.checkbox(&mut self.freq_enabled, "Enable");
            if self.freq_enabled {
                ui.horizontal(|ui| {
                    ui.label("Speed %:");
                    ui.add(egui::Slider::new(&mut self.freq_value, 25..=200).suffix("%"));
                });
            }
            if (self.freq_enabled, self.freq_value) != prev {
                self.dirty = true;
            }
        });

        ui.separator();

        // Random Trainer
        ui.group(|ui| {
            ui.strong("Random Trainer");
            let prev = self.random_enabled;
            ui.checkbox(&mut self.random_enabled, "Enable fixed lane order");
            if self.random_enabled {
                ui.horizontal(|ui| {
                    ui.label("Lane order:");
                    for lane in &mut self.random_lane_order {
                        let mut v = *lane as i32;
                        ui.add(egui::DragValue::new(&mut v).range(1..=7));
                        *lane = v.clamp(1, 7) as u8;
                    }
                });
            }
            if self.random_enabled != prev {
                self.dirty = true;
            }
        });

        ui.separator();

        // Judge Trainer
        ui.group(|ui| {
            ui.strong("Judge Trainer");
            let prev = (self.judge_enabled, self.judge_rank);
            ui.checkbox(&mut self.judge_enabled, "Override judge rank");
            if self.judge_enabled {
                ui.horizontal(|ui| {
                    ui.label("Judge rank:");
                    egui::ComboBox::from_id_salt("judge_rank")
                        .selected_text(
                            *JUDGE_RANK_LABELS
                                .get(self.judge_rank as usize)
                                .unwrap_or(&"?"),
                        )
                        .show_ui(ui, |ui| {
                            for (i, label) in JUDGE_RANK_LABELS.iter().enumerate() {
                                ui.selectable_value(&mut self.judge_rank, i as i32, *label);
                            }
                        });
                });
            }
            if (self.judge_enabled, self.judge_rank) != prev {
                self.dirty = true;
            }
        });
    }

    fn apply(&self, _config: &mut Config, _player_config: &mut PlayerConfig) {
        // Trainer settings are applied to PlayerResource at runtime,
        // not persisted to config files.
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
