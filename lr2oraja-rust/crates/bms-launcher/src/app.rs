use std::path::PathBuf;

use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::panels::audio::AudioPanel;
use crate::panels::discord::DiscordPanel;
use crate::panels::input::InputPanel;
use crate::panels::ir::IrPanel;
use crate::panels::music_select::MusicSelectPanel;
use crate::panels::obs::ObsPanel;
use crate::panels::play_option::PlayOptionPanel;
use crate::panels::resource::ResourcePanel;
use crate::panels::skin::SkinPanel;
use crate::panels::stream::StreamPanel;
use crate::panels::video::VideoPanel;
use crate::tab::Tab;

pub struct LauncherApp {
    pub config: Config,
    pub player_config: PlayerConfig,
    config_path: PathBuf,
    player_config_path: PathBuf,
    active_tab: Tab,
    panels: Vec<Box<dyn LauncherPanel>>,
    pub should_start_game: bool,
}

impl LauncherApp {
    pub fn new(
        config: Config,
        player_config: PlayerConfig,
        config_path: PathBuf,
        player_config_path: PathBuf,
    ) -> Self {
        let mut panels: Vec<Box<dyn LauncherPanel>> = vec![
            Box::new(VideoPanel::default()),
            Box::new(AudioPanel::default()),
            Box::new(InputPanel::default()),
            Box::new(ResourcePanel::default()),
            Box::new(MusicSelectPanel::default()),
            Box::new(PlayOptionPanel::default()),
            Box::new(SkinPanel::default()),
            Box::new(IrPanel::default()),
            Box::new(DiscordPanel::default()),
            Box::new(ObsPanel::default()),
            Box::new(StreamPanel::default()),
        ];

        for panel in &mut panels {
            panel.load(&config, &player_config);
        }

        Self {
            config,
            player_config,
            config_path,
            player_config_path,
            active_tab: Tab::Video,
            panels,
            should_start_game: false,
        }
    }

    /// Apply all panel changes to config and save to files.
    pub fn apply_and_save(&mut self) {
        for panel in &self.panels {
            panel.apply(&mut self.config, &mut self.player_config);
        }
        self.config.validate();
        self.player_config.validate();

        if let Err(e) = self.config.write(&self.config_path) {
            tracing::error!("Failed to save system config: {e}");
        }
        if let Err(e) = self.player_config.write(&self.player_config_path) {
            tracing::error!("Failed to save player config: {e}");
        }
        tracing::info!("Configuration saved");
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel: player name
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("brs Launcher");
                ui.separator();
                ui.label(format!("Player: {}", self.player_config.name));
            });
        });

        // Bottom panel: action buttons
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Start Game").clicked() {
                    self.apply_and_save();
                    self.should_start_game = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("Save").clicked() {
                    self.apply_and_save();
                }
                if ui.button("Cancel").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                // Show unsaved indicator
                let has_changes = self.panels.iter().any(|p| p.has_changes());
                if has_changes {
                    ui.label("(unsaved changes)");
                }
            });
        });

        // Left panel: tab bar
        egui::SidePanel::left("tab_panel")
            .resizable(false)
            .default_width(120.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for &tab in Tab::ALL {
                        let selected = self.active_tab == tab;
                        if ui.selectable_label(selected, tab.label()).clicked() {
                            self.active_tab = tab;
                        }
                    }
                });
            });

        // Central panel: active panel content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(panel) = self.panels.iter_mut().find(|p| p.tab() == self.active_tab) {
                    panel.ui(ui);
                }
            });
        });
    }
}
