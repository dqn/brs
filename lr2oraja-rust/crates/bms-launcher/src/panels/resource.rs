use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::path_list::PathListWidget;
use crate::widgets::url_list::UrlListWidget;

pub struct ResourcePanel {
    bmsroot: Vec<String>,
    table_url: Vec<String>,
    bgmpath: String,
    soundpath: String,
    skinpath: String,
    systemfontpath: String,
    messagefontpath: String,
    dirty: bool,
}

impl Default for ResourcePanel {
    fn default() -> Self {
        let config = Config::default();
        Self {
            bmsroot: config.bmsroot,
            table_url: config.table_url,
            bgmpath: config.bgmpath,
            soundpath: config.soundpath,
            skinpath: config.skinpath,
            systemfontpath: config.systemfontpath,
            messagefontpath: config.messagefontpath,
            dirty: false,
        }
    }
}

impl LauncherPanel for ResourcePanel {
    fn tab(&self) -> Tab {
        Tab::Resource
    }

    fn load(&mut self, config: &Config, _player_config: &PlayerConfig) {
        self.bmsroot = config.bmsroot.clone();
        self.table_url = config.table_url.clone();
        self.bgmpath = config.bgmpath.clone();
        self.soundpath = config.soundpath.clone();
        self.skinpath = config.skinpath.clone();
        self.systemfontpath = config.systemfontpath.clone();
        self.messagefontpath = config.messagefontpath.clone();
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Resource Settings");
        ui.separator();

        PathListWidget::new("BMS Root Paths", &mut self.bmsroot).show(ui);
        self.dirty = true; // Simplified: always mark dirty after UI interaction

        ui.separator();

        UrlListWidget::new("Difficulty Table URLs", &mut self.table_url).show(ui);

        ui.separator();
        ui.label("Paths");

        let prev_bgm = self.bgmpath.clone();
        ui.horizontal(|ui| {
            ui.label("BGM Path");
            ui.text_edit_singleline(&mut self.bgmpath);
        });
        if self.bgmpath != prev_bgm {
            self.dirty = true;
        }

        let prev = self.soundpath.clone();
        ui.horizontal(|ui| {
            ui.label("Sound Path");
            ui.text_edit_singleline(&mut self.soundpath);
        });
        if self.soundpath != prev {
            self.dirty = true;
        }

        let prev = self.skinpath.clone();
        ui.horizontal(|ui| {
            ui.label("Skin Path");
            ui.text_edit_singleline(&mut self.skinpath);
        });
        if self.skinpath != prev {
            self.dirty = true;
        }

        let prev = self.systemfontpath.clone();
        ui.horizontal(|ui| {
            ui.label("System Font");
            ui.text_edit_singleline(&mut self.systemfontpath);
        });
        if self.systemfontpath != prev {
            self.dirty = true;
        }

        let prev = self.messagefontpath.clone();
        ui.horizontal(|ui| {
            ui.label("Message Font");
            ui.text_edit_singleline(&mut self.messagefontpath);
        });
        if self.messagefontpath != prev {
            self.dirty = true;
        }
    }

    fn apply(&self, config: &mut Config, _player_config: &mut PlayerConfig) {
        config.bmsroot = self.bmsroot.clone();
        config.table_url = self.table_url.clone();
        config.bgmpath = self.bgmpath.clone();
        config.soundpath = self.soundpath.clone();
        config.skinpath = self.skinpath.clone();
        config.systemfontpath = self.systemfontpath.clone();
        config.messagefontpath = self.messagefontpath.clone();
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
