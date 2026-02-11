use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::clamped_i32;

#[derive(Default)]
pub struct InputPanel {
    musicselectinput: i32,
    dirty: bool,
}

impl LauncherPanel for InputPanel {
    fn tab(&self) -> Tab {
        Tab::Input
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        self.musicselectinput = player_config.musicselectinput;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Input Settings");
        ui.separator();

        ui.label("Keyboard key assignment is configured via the in-game Key Config screen.");

        ui.separator();

        let prev = self.musicselectinput;
        egui::ComboBox::from_label("Music Select Input")
            .selected_text(match self.musicselectinput {
                0 => "Keyboard",
                _ => "Controller",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.musicselectinput, 0, "Keyboard");
                ui.selectable_value(&mut self.musicselectinput, 1, "Controller");
            });
        if self.musicselectinput != prev {
            self.dirty = true;
        }

        ui.separator();

        let prev = self.musicselectinput;
        clamped_i32(
            ui,
            "Music Select Input Mode",
            &mut self.musicselectinput,
            0,
            1,
        );
        if self.musicselectinput != prev {
            self.dirty = true;
        }
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        player_config.musicselectinput = self.musicselectinput;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
