use bms_config::{Config, PlayerConfig};

use crate::panel::LauncherPanel;
use crate::tab::Tab;

/// Song database metadata viewer and editor panel.
///
/// Displays song entries from the database with title, artist, level,
/// and path information. Supports search/filter and inline editing.
#[derive(Default)]
pub struct SongDataPanel {
    search_query: String,
    entries: Vec<SongEntry>,
    dirty: bool,
}

/// A single song entry for display/editing.
struct SongEntry {
    #[allow(dead_code)] // retained as primary key for future DB write-back in apply()
    md5: String,
    title: String,
    artist: String,
    level: i32,
    path: String,
    modified: bool,
}

impl LauncherPanel for SongDataPanel {
    fn tab(&self) -> Tab {
        Tab::SongData
    }

    fn load(&mut self, _config: &Config, _player_config: &PlayerConfig) {
        self.dirty = false;
    }

    fn load_with_db(
        &mut self,
        config: &Config,
        player_config: &PlayerConfig,
        db_path: Option<&str>,
    ) {
        self.load(config, player_config);

        if let Some(path) = db_path
            && let Ok(db) = bms_database::SongDatabase::open(path)
            && let Ok(songs) = db.get_all_song_datas()
        {
            self.entries = songs
                .into_iter()
                .map(|s| SongEntry {
                    md5: s.md5.clone(),
                    title: s.title.clone(),
                    artist: s.artist.clone(),
                    level: s.level,
                    path: s.path.clone(),
                    modified: false,
                })
                .collect();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Song Data");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_query);
        });

        ui.separator();
        ui.label(format!("{} songs loaded", self.entries.len()));

        let query = self.search_query.to_lowercase();

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                egui::Grid::new("song_data_grid")
                    .striped(true)
                    .min_col_width(60.0)
                    .show(ui, |ui| {
                        ui.strong("Title");
                        ui.strong("Artist");
                        ui.strong("Level");
                        ui.strong("Path");
                        ui.end_row();

                        for entry in &mut self.entries {
                            if !query.is_empty()
                                && !entry.title.to_lowercase().contains(&query)
                                && !entry.artist.to_lowercase().contains(&query)
                            {
                                continue;
                            }

                            let prev_title = entry.title.clone();
                            let prev_artist = entry.artist.clone();

                            ui.text_edit_singleline(&mut entry.title);
                            ui.text_edit_singleline(&mut entry.artist);
                            ui.add(egui::DragValue::new(&mut entry.level));
                            ui.label(&entry.path);
                            ui.end_row();

                            if entry.title != prev_title || entry.artist != prev_artist {
                                entry.modified = true;
                                self.dirty = true;
                            }
                        }
                    });
            });
    }

    fn apply(&self, _config: &mut Config, _player_config: &mut PlayerConfig) {
        // Song data changes are written directly to the database,
        // not to Config/PlayerConfig. A future enhancement can wire
        // modified entries back to the song DB here.
    }

    fn has_changes(&self) -> bool {
        self.dirty
    }
}
