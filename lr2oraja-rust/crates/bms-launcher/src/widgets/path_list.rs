/// Editable list of directory paths with add (folder picker), remove, and reorder buttons.
pub struct PathListWidget<'a> {
    label: &'a str,
    paths: &'a mut Vec<String>,
}

impl<'a> PathListWidget<'a> {
    pub fn new(label: &'a str, paths: &'a mut Vec<String>) -> Self {
        Self { label, paths }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label(self.label);
        ui.indent(self.label, |ui| {
            let mut remove_idx = None;
            let mut swap = None;

            for (i, path) in self.paths.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(path.as_str());
                    if ui.small_button("\u{2191}").clicked() && i > 0 {
                        swap = Some((i - 1, i));
                    }
                    if ui.small_button("\u{2193}").clicked() && i + 1 < self.paths.len() {
                        swap = Some((i, i + 1));
                    }
                    if ui.small_button("\u{2717}").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                self.paths.remove(idx);
            }
            if let Some((a, b)) = swap {
                self.paths.swap(a, b);
            }

            if ui.button("Add Folder...").clicked()
                && let Some(dir) = rfd::FileDialog::new().pick_folder()
            {
                self.paths.push(dir.to_string_lossy().to_string());
            }
        });
    }
}
