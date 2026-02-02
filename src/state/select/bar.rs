use crate::database::{ScoreData, SongData};

/// A bar in the song selection list.
#[derive(Debug, Clone)]
pub enum Bar {
    /// A song bar representing a playable BMS chart.
    Song(Box<SongBar>),
    /// A folder bar for navigation (not implemented in MVP).
    Folder(FolderBar),
}

impl Bar {
    /// Get the display title for this bar.
    pub fn title(&self) -> &str {
        match self {
            Bar::Song(song_bar) => &song_bar.song.title,
            Bar::Folder(folder_bar) => &folder_bar.name,
        }
    }

    /// Check if this bar is a song.
    pub fn is_song(&self) -> bool {
        matches!(self, Bar::Song(_))
    }

    /// Get the song bar if this is a song.
    pub fn as_song(&self) -> Option<&SongBar> {
        match self {
            Bar::Song(song_bar) => Some(song_bar.as_ref()),
            Bar::Folder(_) => None,
        }
    }
}

/// A song bar representing a playable BMS chart.
#[derive(Debug, Clone)]
pub struct SongBar {
    /// Song metadata from the database.
    pub song: SongData,
    /// Score data if the player has played this song before.
    pub score: Option<ScoreData>,
}

impl SongBar {
    /// Create a new song bar.
    pub fn new(song: SongData, score: Option<ScoreData>) -> Self {
        Self { song, score }
    }
}

/// A folder bar for navigation.
#[derive(Debug, Clone)]
pub struct FolderBar {
    /// Display name of the folder.
    pub name: String,
    /// Path to the folder.
    pub path: String,
    /// Number of songs in this folder.
    pub song_count: usize,
}

impl FolderBar {
    /// Create a new folder bar.
    pub fn new(name: String, path: String, song_count: usize) -> Self {
        Self {
            name,
            path,
            song_count,
        }
    }
}
