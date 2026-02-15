use bms_config::{Config, PlayerConfig};

use crate::tab::Tab;

/// A launcher settings panel.
pub trait LauncherPanel {
    fn tab(&self) -> Tab;
    fn load(&mut self, config: &Config, player_config: &PlayerConfig);
    /// Load with optional song database path. Default delegates to `load()`.
    fn load_with_db(
        &mut self,
        config: &Config,
        player_config: &PlayerConfig,
        _song_db_path: Option<&str>,
    ) {
        self.load(config, player_config);
    }
    fn ui(&mut self, ui: &mut egui::Ui);
    fn apply(&self, config: &mut Config, player_config: &mut PlayerConfig);
    fn has_changes(&self) -> bool;
}
