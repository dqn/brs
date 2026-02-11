use bms_config::{Config, PlayerConfig, SkinType};

use crate::panel::LauncherPanel;
use crate::tab::Tab;

#[derive(Default)]
pub struct SkinPanel {
    selected_skin_type: i32,
    skin_paths: Vec<String>,
    dirty: bool,
}

impl LauncherPanel for SkinPanel {
    fn tab(&self) -> Tab {
        Tab::Skin
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        self.skin_paths = player_config
            .skin
            .iter()
            .map(|s| s.path.clone().unwrap_or_default())
            .collect();
        self.selected_skin_type = 0;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Skin Settings");
        ui.separator();

        // Skin type selector
        let prev = self.selected_skin_type;
        egui::ComboBox::from_label("Skin Type")
            .selected_text(
                SkinType::from_id(self.selected_skin_type)
                    .map(|st| st.name())
                    .unwrap_or("Unknown"),
            )
            .show_ui(ui, |ui| {
                for id in 0..=SkinType::max_id() {
                    if let Some(st) = SkinType::from_id(id) {
                        ui.selectable_value(&mut self.selected_skin_type, id, st.name());
                    }
                }
            });
        if self.selected_skin_type != prev {
            self.dirty = true;
        }

        ui.separator();

        // Edit skin path for selected type
        let idx = self.selected_skin_type as usize;
        if idx < self.skin_paths.len() {
            let prev_path = self.skin_paths[idx].clone();
            ui.horizontal(|ui| {
                ui.label("Skin Path");
                ui.text_edit_singleline(&mut self.skin_paths[idx]);
            });
            if self.skin_paths[idx] != prev_path {
                self.dirty = true;
            }

            if ui.button("Browse...").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                self.skin_paths[idx] = path.to_string_lossy().to_string();
                self.dirty = true;
            }
        }
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        for (i, path) in self.skin_paths.iter().enumerate() {
            if i < player_config.skin.len() {
                player_config.skin[i].path = if path.is_empty() {
                    None
                } else {
                    Some(path.clone())
                };
            }
        }
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
