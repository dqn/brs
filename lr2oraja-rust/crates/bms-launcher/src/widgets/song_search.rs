use bms_database::{CourseSongData, SongData, SongDatabase};

/// Song search widget state.
#[derive(Default)]
pub struct SongSearchState {
    query: String,
    results: Vec<SongData>,
}

impl SongSearchState {
    /// Show the song search widget. Returns songs added by the user.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        song_db: Option<&SongDatabase>,
    ) -> Vec<CourseSongData> {
        let mut added = Vec::new();

        ui.separator();
        ui.label("Song Search");

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.query);
            let can_search = song_db.is_some() && self.query.len() >= 2;
            if ui
                .add_enabled(can_search, egui::Button::new("Search"))
                .clicked()
                && let Some(db) = song_db
            {
                self.results = self.do_search(db);
            }
        });

        if !self.results.is_empty() {
            ui.label(format!("{} results", self.results.len()));

            egui::ScrollArea::vertical()
                .id_salt("song_search_results")
                .max_height(150.0)
                .show(ui, |ui| {
                    // Show up to 50 results to avoid UI lag
                    for song in self.results.iter().take(50) {
                        ui.horizontal(|ui| {
                            let label = format!(
                                "{} - {} [{}]",
                                song.title,
                                song.artist,
                                &song.sha256[..song.sha256.len().min(8)]
                            );
                            ui.label(label);
                            if ui.small_button("Add").clicked() {
                                added.push(CourseSongData {
                                    sha256: song.sha256.clone(),
                                    md5: song.md5.clone(),
                                    title: if song.title.is_empty() {
                                        String::new()
                                    } else {
                                        format!("{} {}", song.title, song.subtitle)
                                            .trim()
                                            .to_string()
                                    },
                                });
                            }
                        });
                    }
                });
        }

        added
    }

    fn do_search(&self, db: &SongDatabase) -> Vec<SongData> {
        let query = self.query.trim();
        if query.is_empty() {
            return Vec::new();
        }

        // Auto-detect: hash (32-char MD5 or 64-char SHA256) vs text search
        let is_hash = query.len() >= 32 && query.chars().all(|c| c.is_ascii_hexdigit());

        if is_hash {
            db.get_song_datas_by_hashes(&[query]).unwrap_or_default()
        } else {
            db.get_song_datas_by_text(query).unwrap_or_default()
        }
    }
}
