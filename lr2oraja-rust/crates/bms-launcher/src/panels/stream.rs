use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::clamped::clamped_i32;

pub struct StreamPanel {
    enable_request: bool,
    notify_request: bool,
    max_request_count: i32,
    dirty: bool,
}

impl Default for StreamPanel {
    fn default() -> Self {
        let pc = PlayerConfig::default();
        Self {
            enable_request: pc.enable_request,
            notify_request: pc.notify_request,
            max_request_count: pc.max_request_count,
            dirty: false,
        }
    }
}

impl LauncherPanel for StreamPanel {
    fn tab(&self) -> Tab {
        Tab::Stream
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        self.enable_request = player_config.enable_request;
        self.notify_request = player_config.notify_request;
        self.max_request_count = player_config.max_request_count;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Stream Settings");
        ui.separator();

        if ui
            .checkbox(&mut self.enable_request, "Enable Song Requests")
            .changed()
        {
            self.dirty = true;
        }

        if ui
            .checkbox(&mut self.notify_request, "Notify on Request")
            .changed()
        {
            self.dirty = true;
        }

        let prev = self.max_request_count;
        clamped_i32(ui, "Max Request Count", &mut self.max_request_count, 0, 100);
        if self.max_request_count != prev {
            self.dirty = true;
        }
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        player_config.enable_request = self.enable_request;
        player_config.notify_request = self.notify_request;
        player_config.max_request_count = self.max_request_count;
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
