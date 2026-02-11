use bms_config::{Config, IRConfig, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;

const IR_SEND_LABELS: &[&str] = &["Always", "Complete Song", "Update Score"];

#[derive(Default)]
pub struct IrPanel {
    irconfigs: Vec<IRConfig>,
    selected: usize,
    dirty: bool,
}

impl LauncherPanel for IrPanel {
    fn tab(&self) -> Tab {
        Tab::Ir
    }

    fn load(&mut self, _config: &Config, player_config: &PlayerConfig) {
        self.irconfigs = player_config.irconfig.clone().unwrap_or_default();
        self.selected = 0;
        self.dirty = false;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Internet Ranking");
        ui.separator();

        if self.irconfigs.is_empty() {
            ui.label("No IR configurations.");
            if ui.button("Add IR").clicked() {
                self.irconfigs.push(IRConfig {
                    irname: "LR2IR".to_string(),
                    ..Default::default()
                });
                self.dirty = true;
            }
            return;
        }

        // IR selector
        let names: Vec<String> = self.irconfigs.iter().map(|c| c.irname.clone()).collect();
        egui::ComboBox::from_label("IR")
            .selected_text(names.get(self.selected).unwrap_or(&String::new()).as_str())
            .show_ui(ui, |ui| {
                for (i, name) in names.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, i, name.as_str());
                }
            });

        if self.selected < self.irconfigs.len() {
            let ir = &mut self.irconfigs[self.selected];

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("IR Name");
                if ui.text_edit_singleline(&mut ir.irname).changed() {
                    self.dirty = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("User ID");
                if ui.text_edit_singleline(&mut ir.userid).changed() {
                    self.dirty = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Password");
                if ui
                    .add(egui::TextEdit::singleline(&mut ir.password).password(true))
                    .changed()
                {
                    self.dirty = true;
                }
            });

            let prev = ir.irsend;
            let selected_label = IR_SEND_LABELS.get(ir.irsend as usize).unwrap_or(&"Unknown");
            egui::ComboBox::from_label("Send Mode")
                .selected_text(*selected_label)
                .show_ui(ui, |ui| {
                    for (i, &label) in IR_SEND_LABELS.iter().enumerate() {
                        ui.selectable_value(&mut ir.irsend, i as i32, label);
                    }
                });
            if ir.irsend != prev {
                self.dirty = true;
            }

            if ui.checkbox(&mut ir.importscore, "Import Score").changed() {
                self.dirty = true;
            }
            if ui.checkbox(&mut ir.importrival, "Import Rival").changed() {
                self.dirty = true;
            }
        }

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Add IR").clicked() {
                self.irconfigs.push(IRConfig {
                    irname: "NewIR".to_string(),
                    ..Default::default()
                });
                self.dirty = true;
            }
            if ui.button("Remove Selected").clicked() && self.selected < self.irconfigs.len() {
                self.irconfigs.remove(self.selected);
                if self.selected > 0 {
                    self.selected -= 1;
                }
                self.dirty = true;
            }
        });
    }

    fn apply(&self, _config: &mut Config, player_config: &mut PlayerConfig) {
        player_config.irconfig = Some(self.irconfigs.clone());
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
