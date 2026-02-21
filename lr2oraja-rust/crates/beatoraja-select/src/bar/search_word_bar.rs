use super::bar::Bar;
use super::directory_bar::DirectoryBarData;
use super::song_bar::SongBar;

/// Search bar
/// Translates: bms.player.beatoraja.select.bar.SearchWordBar
pub struct SearchWordBar {
    pub directory: DirectoryBarData,
    pub text: String,
    pub title: String,
}

impl SearchWordBar {
    pub fn new(text: String) -> Self {
        let title = format!("Search : '{}'", text);
        Self {
            directory: DirectoryBarData::default(),
            text,
            title,
        }
    }

    pub fn get_children(&self) -> Vec<Bar> {
        // In Java: SongBar.toSongBarArray(selector.getSongDatabase().getSongDatasByText(text))
        todo!("SearchWordBar.getChildren requires SongDatabaseAccessor context")
    }

    pub fn update_folder_status(&mut self) {
        // In Java: updateFolderStatus(selector.getSongDatabase().getSongDatasByText(text))
        todo!("SearchWordBar.updateFolderStatus requires SongDatabaseAccessor context")
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }
}
