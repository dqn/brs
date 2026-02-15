use bms_database::{SongDatabase, TableFolder};

use crate::widgets::item_list::show_item_list;
use crate::widgets::song_list::show_song_list;
use crate::widgets::song_search::SongSearchState;

/// State for the Folder sub-tab of the Table Editor.
#[derive(Default)]
pub struct FolderEditorState {
    pub folders: Vec<TableFolder>,
    selected_idx: Option<usize>,
    folder_name: String,
    song_search: SongSearchState,
    pub dirty: bool,
}

impl FolderEditorState {
    pub fn set_folders(&mut self, folders: Vec<TableFolder>) {
        self.folders = folders;
        self.selected_idx = if self.folders.is_empty() {
            None
        } else {
            Some(0)
        };
        self.sync_from_selected();
        self.dirty = false;
    }

    pub fn get_folders(&self) -> &[TableFolder] {
        &self.folders
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, song_db: Option<&SongDatabase>) {
        ui.columns(2, |cols| {
            // Left: folder list
            cols[0].label("Folders");
            let list_changed = show_item_list(
                &mut cols[0],
                "folder_list",
                &mut self.folders,
                &mut self.selected_idx,
                |f| {
                    if f.name.is_empty() {
                        "(unnamed)".to_string()
                    } else {
                        f.name.clone()
                    }
                },
                || TableFolder {
                    name: "New Folder".to_string(),
                    songs: Vec::new(),
                },
                "Add Folder",
            );
            if list_changed {
                self.dirty = true;
                self.sync_from_selected();
            }

            // Right: folder details
            if let Some(idx) = self.selected_idx {
                cols[1].label("Folder Name");
                let prev_name = self.folder_name.clone();
                cols[1].text_edit_singleline(&mut self.folder_name);
                if self.folder_name != prev_name {
                    if let Some(folder) = self.folders.get_mut(idx) {
                        folder.name = self.folder_name.clone();
                    }
                    self.dirty = true;
                }

                cols[1].separator();
                cols[1].label("Songs");

                if let Some(folder) = self.folders.get_mut(idx)
                    && show_song_list(&mut cols[1], "folder_songs", &mut folder.songs)
                {
                    self.dirty = true;
                }

                // Song search
                let added = self.song_search.show(&mut cols[1], song_db);
                if !added.is_empty()
                    && let Some(folder) = self.folders.get_mut(idx)
                {
                    folder.songs.extend(added);
                    self.dirty = true;
                }
            } else {
                cols[1].label("Select or add a folder to edit.");
            }
        });
    }

    fn sync_from_selected(&mut self) {
        if let Some(idx) = self.selected_idx {
            if let Some(folder) = self.folders.get(idx) {
                self.folder_name = folder.name.clone();
            }
        } else {
            self.folder_name.clear();
        }
        self.song_search = SongSearchState::default();
    }
}
