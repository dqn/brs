/// Editable list of URLs with add, remove, and edit-in-place.
pub struct UrlListWidget<'a> {
    label: &'a str,
    urls: &'a mut Vec<String>,
}

impl<'a> UrlListWidget<'a> {
    pub fn new(label: &'a str, urls: &'a mut Vec<String>) -> Self {
        Self { label, urls }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label(self.label);
        ui.indent(self.label, |ui| {
            let mut remove_idx = None;

            for (i, url) in self.urls.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(url);
                    if ui.small_button("\u{2717}").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                self.urls.remove(idx);
            }

            if ui.button("Add URL").clicked() {
                self.urls.push(String::new());
            }
        });
    }
}
