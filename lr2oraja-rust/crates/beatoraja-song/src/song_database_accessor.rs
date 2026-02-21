use crate::folder_data::FolderData;
use crate::song_data::SongData;
use crate::song_database_update_listener::SongDatabaseUpdateListener;
use crate::song_information_accessor::SongInformationAccessor;

/// Song database accessor interface
pub trait SongDatabaseAccessor {
    /// Get song data by key-value pair
    fn get_song_datas(&self, key: &str, value: &str) -> Vec<SongData>;

    /// Get song data by MD5/SHA256 hashes
    fn get_song_datas_by_hashes(&self, hashes: &[String]) -> Vec<SongData>;

    /// Query song data using SQL across score, scorelog, and info databases
    fn get_song_datas_by_sql(
        &self,
        sql: &str,
        score: &str,
        scorelog: &str,
        info: Option<&str>,
    ) -> Vec<SongData>;

    /// Set song data
    fn set_song_datas(&self, songs: &[SongData]);

    /// Search song data by text
    fn get_song_datas_by_text(&self, text: &str) -> Vec<SongData>;

    /// Get folder data by key-value pair
    fn get_folder_datas(&self, key: &str, value: &str) -> Vec<FolderData>;

    /// Update song database
    fn update_song_datas(
        &self,
        update_path: Option<&str>,
        bmsroot: &[String],
        update_all: bool,
        update_parent_when_missing: bool,
        info: Option<&SongInformationAccessor>,
    );

    /// Update song database with listener
    fn update_song_datas_with_listener(
        &self,
        update_path: Option<&str>,
        bmsroot: &[String],
        update_all: bool,
        update_parent_when_missing: bool,
        info: Option<&SongInformationAccessor>,
        listener: &SongDatabaseUpdateListener,
    );
}
