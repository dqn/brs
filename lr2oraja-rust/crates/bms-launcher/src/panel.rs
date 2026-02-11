use bms_config::{Config, PlayerConfig};

use crate::tab::Tab;

/// A launcher settings panel.
pub trait LauncherPanel {
    fn tab(&self) -> Tab;
    fn load(&mut self, config: &Config, player_config: &PlayerConfig);
    fn ui(&mut self, ui: &mut egui::Ui);
    fn apply(&self, config: &mut Config, player_config: &mut PlayerConfig);
    fn has_changes(&self) -> bool;
}
