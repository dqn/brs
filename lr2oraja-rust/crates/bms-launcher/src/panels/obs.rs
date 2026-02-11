use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::clamped_i32;

pub struct ObsPanel {
    use_obs_ws: bool,
    obs_ws_host: String,
    obs_ws_port: i32,
    obs_ws_pass: String,
    obs_ws_rec_mode: i32,
    obs_ws_rec_stop_wait: i32,
    dirty: bool,
}

impl Default for ObsPanel {
    fn default() -> Self {
        let config = Config::default();
        Self {
            use_obs_ws: config.use_obs_ws,
            obs_ws_host: config.obs_ws_host,
            obs_ws_port: config.obs_ws_port,
            obs_ws_pass: config.obs_ws_pass,
            obs_ws_rec_mode: config.obs_ws_rec_mode,
            obs_ws_rec_stop_wait: config.obs_ws_rec_stop_wait,
            dirty: false,
        }
    }
}

impl LauncherPanel for ObsPanel {
    fn tab(&self) -> Tab {
        Tab::Obs
    }

    fn load(&mut self, config: &Config, _player_config: &PlayerConfig) {
        self.use_obs_ws = config.use_obs_ws;
        self.obs_ws_host = config.obs_ws_host.clone();
        self.obs_ws_port = config.obs_ws_port;
        self.obs_ws_pass = config.obs_ws_pass.clone();
        self.obs_ws_rec_mode = config.obs_ws_rec_mode;
        self.obs_ws_rec_stop_wait = config.obs_ws_rec_stop_wait;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("OBS WebSocket Settings");
        ui.separator();

        if ui
            .checkbox(&mut self.use_obs_ws, "Enable OBS WebSocket")
            .changed()
        {
            self.dirty = true;
        }

        ui.separator();

        let prev = self.obs_ws_host.clone();
        ui.horizontal(|ui| {
            ui.label("Host");
            ui.text_edit_singleline(&mut self.obs_ws_host);
        });
        if self.obs_ws_host != prev {
            self.dirty = true;
        }

        let prev = self.obs_ws_port;
        clamped_i32(ui, "Port", &mut self.obs_ws_port, 1, 65535);
        if self.obs_ws_port != prev {
            self.dirty = true;
        }

        let prev = self.obs_ws_pass.clone();
        ui.horizontal(|ui| {
            ui.label("Password");
            ui.add(egui::TextEdit::singleline(&mut self.obs_ws_pass).password(true));
        });
        if self.obs_ws_pass != prev {
            self.dirty = true;
        }

        ui.separator();

        let prev = self.obs_ws_rec_mode;
        egui::ComboBox::from_label("Recording Mode")
            .selected_text(match self.obs_ws_rec_mode {
                0 => "Off",
                1 => "Per Song",
                _ => "Session",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.obs_ws_rec_mode, 0, "Off");
                ui.selectable_value(&mut self.obs_ws_rec_mode, 1, "Per Song");
                ui.selectable_value(&mut self.obs_ws_rec_mode, 2, "Session");
            });
        if self.obs_ws_rec_mode != prev {
            self.dirty = true;
        }

        let prev = self.obs_ws_rec_stop_wait;
        clamped_i32(
            ui,
            "Recording Stop Wait (ms)",
            &mut self.obs_ws_rec_stop_wait,
            0,
            60000,
        );
        if self.obs_ws_rec_stop_wait != prev {
            self.dirty = true;
        }
    }

    fn apply(&self, config: &mut Config, _player_config: &mut PlayerConfig) {
        config.use_obs_ws = self.use_obs_ws;
        config.obs_ws_host = self.obs_ws_host.clone();
        config.obs_ws_port = self.obs_ws_port;
        config.obs_ws_pass = self.obs_ws_pass.clone();
        config.obs_ws_rec_mode = self.obs_ws_rec_mode;
        config.obs_ws_rec_stop_wait = self.obs_ws_rec_stop_wait;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
