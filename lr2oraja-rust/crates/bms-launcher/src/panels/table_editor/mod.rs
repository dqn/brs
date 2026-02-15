mod course_editor;
mod folder_editor;

use std::fs;
use std::path::{Path, PathBuf};

use bms_config::{Config, PlayerConfig};
use bms_database::{SongDatabase, TableData};

use crate::panel::LauncherPanel;
use crate::tab::Tab;

use course_editor::CourseEditorState;
use folder_editor::FolderEditorState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubTab {
    Folder,
    Course,
}

/// Table Editor panel: edit .bmt/.json table files (folders & courses).
pub struct TableEditorPanel {
    song_db: Option<SongDatabase>,
    table_dir: PathBuf,
    table_files: Vec<PathBuf>,
    selected_file_idx: Option<usize>,
    table_name: String,
    folder_editor: FolderEditorState,
    course_editor: CourseEditorState,
    active_sub_tab: SubTab,
    status_message: Option<String>,
}

impl Default for TableEditorPanel {
    fn default() -> Self {
        Self {
            song_db: None,
            table_dir: PathBuf::new(),
            table_files: Vec::new(),
            selected_file_idx: None,
            table_name: String::new(),
            folder_editor: FolderEditorState::default(),
            course_editor: CourseEditorState::default(),
            active_sub_tab: SubTab::Folder,
            status_message: None,
        }
    }
}

impl TableEditorPanel {
    fn scan_table_files(&mut self) {
        self.table_files.clear();
        if !self.table_dir.is_dir() {
            return;
        }
        if let Ok(entries) = fs::read_dir(&self.table_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && (ext == "bmt" || ext == "json")
                {
                    self.table_files.push(path);
                }
            }
        }
        self.table_files.sort();
    }

    fn load_selected_file(&mut self) {
        if let Some(idx) = self.selected_file_idx {
            if let Some(path) = self.table_files.get(idx) {
                match TableData::read(path) {
                    Ok(td) => {
                        self.table_name = td.name.clone();
                        self.folder_editor.set_folders(td.folder);
                        self.course_editor.set_courses(td.course);
                        self.status_message = None;
                    }
                    Err(e) => {
                        tracing::error!("Failed to read table file: {e}");
                        self.status_message = Some(format!("Error reading file: {e}"));
                        self.table_name.clear();
                        self.folder_editor.set_folders(Vec::new());
                        self.course_editor.set_courses(Vec::new());
                    }
                }
            }
        } else {
            self.table_name.clear();
            self.folder_editor.set_folders(Vec::new());
            self.course_editor.set_courses(Vec::new());
        }
    }

    fn save_current_file(&mut self) {
        let Some(idx) = self.selected_file_idx else {
            self.status_message = Some("No file selected.".to_string());
            return;
        };
        let Some(path) = self.table_files.get(idx).cloned() else {
            return;
        };

        let td = TableData {
            name: self.table_name.clone(),
            folder: self.folder_editor.get_folders().to_vec(),
            course: self.course_editor.get_courses().to_vec(),
            ..Default::default()
        };

        match TableData::write(&path, &td) {
            Ok(()) => {
                self.status_message = Some(format!("Saved: {}", path.display()));
                self.folder_editor.dirty = false;
                self.course_editor.dirty = false;
            }
            Err(e) => {
                tracing::error!("Failed to save table file: {e}");
                self.status_message = Some(format!("Error saving: {e}"));
            }
        }
    }

    fn create_new_file(&mut self) {
        let dialog = rfd::FileDialog::new()
            .add_filter("Table files", &["bmt", "json"])
            .set_directory(&self.table_dir);

        if let Some(path) = dialog.save_file() {
            let td = TableData {
                name: "New Table".to_string(),
                folder: Vec::new(),
                course: Vec::new(),
                ..Default::default()
            };
            if let Err(e) = TableData::write(&path, &td) {
                tracing::error!("Failed to create table file: {e}");
                self.status_message = Some(format!("Error creating file: {e}"));
                return;
            }
            self.scan_table_files();
            // Select the newly created file
            self.selected_file_idx = self.table_files.iter().position(|p| p == &path);
            self.load_selected_file();
        }
    }

    fn open_file(&mut self) {
        let dialog = rfd::FileDialog::new()
            .add_filter("Table files", &["bmt", "json"])
            .set_directory(&self.table_dir);

        if let Some(path) = dialog.pick_file() {
            // Copy to table_dir if not already there
            let dest = if path.parent() != Some(&self.table_dir) {
                let dest = self.table_dir.join(path.file_name().unwrap_or_default());
                if let Err(e) = fs::copy(&path, &dest) {
                    tracing::error!("Failed to copy table file: {e}");
                    self.status_message = Some(format!("Error: {e}"));
                    return;
                }
                dest
            } else {
                path
            };
            self.scan_table_files();
            self.selected_file_idx = self.table_files.iter().position(|p| p == &dest);
            self.load_selected_file();
        }
    }

    fn file_display_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string()
    }
}

impl LauncherPanel for TableEditorPanel {
    fn tab(&self) -> Tab {
        Tab::TableEditor
    }

    fn load(&mut self, config: &Config, _player_config: &PlayerConfig) {
        self.table_dir = PathBuf::from(&config.tablepath);
        self.scan_table_files();
        if !self.table_files.is_empty() {
            self.selected_file_idx = Some(0);
            self.load_selected_file();
        }
    }

    fn load_with_db(
        &mut self,
        config: &Config,
        player_config: &PlayerConfig,
        song_db_path: Option<&str>,
    ) {
        if let Some(db_path) = song_db_path {
            match SongDatabase::open(db_path) {
                Ok(db) => self.song_db = Some(db),
                Err(e) => tracing::warn!("Failed to open song DB for table editor: {e}"),
            }
        }
        self.load(config, player_config);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Table Editor");
        ui.separator();

        // File selection row
        ui.horizontal(|ui| {
            ui.label("Table File:");

            let current_label = self
                .selected_file_idx
                .and_then(|idx| self.table_files.get(idx))
                .map(|p| Self::file_display_name(p))
                .unwrap_or_else(|| "(none)".to_string());

            let prev_idx = self.selected_file_idx;
            egui::ComboBox::from_id_salt("table_file_selector")
                .selected_text(&current_label)
                .show_ui(ui, |ui| {
                    for (i, path) in self.table_files.iter().enumerate() {
                        let name = Self::file_display_name(path);
                        if ui
                            .selectable_label(self.selected_file_idx == Some(i), name)
                            .clicked()
                        {
                            self.selected_file_idx = Some(i);
                        }
                    }
                });

            if self.selected_file_idx != prev_idx {
                self.load_selected_file();
            }

            if ui.button("New...").clicked() {
                self.create_new_file();
            }
            if ui.button("Open File...").clicked() {
                self.open_file();
            }
        });

        // Table name
        if self.selected_file_idx.is_some() {
            ui.horizontal(|ui| {
                ui.label("Table Name:");
                ui.text_edit_singleline(&mut self.table_name);
            });

            ui.separator();

            // Sub-tab bar
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.active_sub_tab == SubTab::Folder, "Folder")
                    .clicked()
                {
                    self.active_sub_tab = SubTab::Folder;
                }
                if ui
                    .selectable_label(self.active_sub_tab == SubTab::Course, "Course")
                    .clicked()
                {
                    self.active_sub_tab = SubTab::Course;
                }
            });

            ui.separator();

            // Sub-tab content
            match self.active_sub_tab {
                SubTab::Folder => self.folder_editor.ui(ui, self.song_db.as_ref()),
                SubTab::Course => self.course_editor.ui(ui, self.song_db.as_ref()),
            }

            ui.separator();

            // Save button
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.save_current_file();
                }

                if self.folder_editor.dirty || self.course_editor.dirty {
                    ui.label("(unsaved changes)");
                }
            });
        }

        // Status message
        if let Some(msg) = &self.status_message {
            ui.separator();
            ui.label(msg);
        }
    }

    fn apply(&self, _config: &mut Config, _player_config: &mut PlayerConfig) {
        // Table editor saves to files independently, not to Config.
    }

    fn has_changes(&self) -> bool {
        self.folder_editor.dirty || self.course_editor.dirty
    }
}
