use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;
use crate::widgets::url_list::UrlListWidget;

pub struct DiscordPanel {
    use_discord_rpc: bool,
    webhook_option: i32,
    webhook_name: String,
    webhook_avatar: String,
    webhook_url: Vec<String>,
    dirty: bool,
}

impl Default for DiscordPanel {
    fn default() -> Self {
        let config = Config::default();
        Self {
            use_discord_rpc: config.use_discord_rpc,
            webhook_option: config.webhook_option,
            webhook_name: config.webhook_name,
            webhook_avatar: config.webhook_avatar,
            webhook_url: config.webhook_url,
            dirty: false,
        }
    }
}

impl LauncherPanel for DiscordPanel {
    fn tab(&self) -> Tab {
        Tab::Discord
    }

    fn load(&mut self, config: &Config, _player_config: &PlayerConfig) {
        self.use_discord_rpc = config.use_discord_rpc;
        self.webhook_option = config.webhook_option;
        self.webhook_name = config.webhook_name.clone();
        self.webhook_avatar = config.webhook_avatar.clone();
        self.webhook_url = config.webhook_url.clone();
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Discord Settings");
        ui.separator();

        if ui
            .checkbox(&mut self.use_discord_rpc, "Enable Discord RPC")
            .changed()
        {
            self.dirty = true;
        }

        ui.separator();
        ui.label("Webhook");

        let prev = self.webhook_option;
        egui::ComboBox::from_label("Webhook Mode")
            .selected_text(match self.webhook_option {
                0 => "Off",
                1 => "On Clear",
                _ => "Always",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.webhook_option, 0, "Off");
                ui.selectable_value(&mut self.webhook_option, 1, "On Clear");
                ui.selectable_value(&mut self.webhook_option, 2, "Always");
            });
        if self.webhook_option != prev {
            self.dirty = true;
        }

        let prev = self.webhook_name.clone();
        ui.horizontal(|ui| {
            ui.label("Webhook Name");
            ui.text_edit_singleline(&mut self.webhook_name);
        });
        if self.webhook_name != prev {
            self.dirty = true;
        }

        let prev = self.webhook_avatar.clone();
        ui.horizontal(|ui| {
            ui.label("Webhook Avatar URL");
            ui.text_edit_singleline(&mut self.webhook_avatar);
        });
        if self.webhook_avatar != prev {
            self.dirty = true;
        }

        ui.separator();
        UrlListWidget::new("Webhook URLs", &mut self.webhook_url).show(ui);
        self.dirty = true; // Simplified: always mark dirty after URL list interaction
    }

    fn apply(&self, config: &mut Config, _player_config: &mut PlayerConfig) {
        config.use_discord_rpc = self.use_discord_rpc;
        config.webhook_option = self.webhook_option;
        config.webhook_name = self.webhook_name.clone();
        config.webhook_avatar = self.webhook_avatar.clone();
        config.webhook_url = self.webhook_url.clone();
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
