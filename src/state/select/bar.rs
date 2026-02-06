use crate::database::models::{FolderData, ScoreData, SongData};

/// Bar types used in the song select screen.
/// Corresponds to beatoraja's bar hierarchy.
#[derive(Debug, Clone)]
pub enum Bar {
    Song(Box<SongBar>),
    Folder(FolderBar),
    Grade(GradeBar),
    Table(TableBar),
    Hash(HashBar),
}

impl Bar {
    /// Get the display title of this bar.
    pub fn title(&self) -> &str {
        match self {
            Self::Song(b) => &b.title,
            Self::Folder(b) => &b.title,
            Self::Grade(b) => &b.title,
            Self::Table(b) => &b.title,
            Self::Hash(b) => &b.title,
        }
    }

    /// Get the score associated with this bar, if any.
    pub fn score(&self) -> Option<&ScoreData> {
        match self {
            Self::Song(b) => b.score.as_ref(),
            Self::Grade(b) => b.score.as_ref(),
            _ => None,
        }
    }

    /// Get the clear lamp value (0 = no play).
    pub fn lamp(&self) -> i32 {
        self.score().map_or(0, |s| s.clear)
    }

    /// Return true if this bar is a directory (can be opened to show children).
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Folder(_) | Self::Table(_) | Self::Hash(_))
    }
}

/// A song bar representing a single BMS chart.
#[derive(Debug, Clone)]
pub struct SongBar {
    pub song: SongData,
    pub title: String,
    pub score: Option<ScoreData>,
}

impl SongBar {
    pub fn new(song: SongData) -> Self {
        let title = if song.subtitle.is_empty() {
            song.title.clone()
        } else {
            format!("{} {}", song.title, song.subtitle)
        };
        Self {
            title,
            song,
            score: None,
        }
    }

    /// Create SongBars from a list of SongData, removing duplicates by SHA256.
    pub fn from_songs(songs: Vec<SongData>) -> Vec<Bar> {
        let mut seen = std::collections::HashSet::new();
        songs
            .into_iter()
            .filter(|s| seen.insert(s.sha256.clone()))
            .map(|s| Bar::Song(Box::new(SongBar::new(s))))
            .collect()
    }
}

/// A folder bar representing a filesystem directory.
#[derive(Debug, Clone)]
pub struct FolderBar {
    pub folder: FolderData,
    pub title: String,
    /// CRC32 hash identifying this folder for DB lookups.
    pub crc: String,
    /// Aggregated lamp counts for contained songs.
    pub lamps: [i32; 11],
}

impl FolderBar {
    pub fn new(folder: FolderData, crc: String) -> Self {
        let title = folder.title.clone();
        Self {
            folder,
            title,
            crc,
            lamps: [0; 11],
        }
    }
}

/// A grade/course bar for dan-i ninteisystems etc.
#[derive(Debug, Clone)]
pub struct GradeBar {
    pub title: String,
    pub song_hashes: Vec<String>,
    pub score: Option<ScoreData>,
}

impl GradeBar {
    pub fn new(title: String, song_hashes: Vec<String>) -> Self {
        Self {
            title,
            song_hashes,
            score: None,
        }
    }
}

/// A difficulty table bar (e.g., insane BMS table).
#[derive(Debug, Clone)]
pub struct TableBar {
    pub title: String,
    pub url: String,
    pub levels: Vec<HashBar>,
    pub grades: Vec<GradeBar>,
}

impl TableBar {
    pub fn new(title: String, url: String) -> Self {
        Self {
            title,
            url,
            levels: Vec::new(),
            grades: Vec::new(),
        }
    }
}

/// A hash-based folder bar containing songs identified by hash.
#[derive(Debug, Clone)]
pub struct HashBar {
    pub title: String,
    pub hashes: Vec<String>,
    /// Aggregated lamp counts.
    pub lamps: [i32; 11],
}

impl HashBar {
    pub fn new(title: String, hashes: Vec<String>) -> Self {
        Self {
            title,
            hashes,
            lamps: [0; 11],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::SongData;

    #[test]
    fn song_bar_title() {
        let song = SongData {
            title: "Test".to_string(),
            subtitle: "Hard".to_string(),
            ..Default::default()
        };
        let bar = SongBar::new(song);
        assert_eq!(bar.title, "Test Hard");
    }

    #[test]
    fn song_bar_title_no_subtitle() {
        let song = SongData {
            title: "Test".to_string(),
            ..Default::default()
        };
        let bar = SongBar::new(song);
        assert_eq!(bar.title, "Test");
    }

    #[test]
    fn from_songs_deduplicates() {
        let songs = vec![
            SongData {
                title: "A".to_string(),
                sha256: "same".to_string(),
                path: "p1".to_string(),
                ..Default::default()
            },
            SongData {
                title: "A".to_string(),
                sha256: "same".to_string(),
                path: "p2".to_string(),
                ..Default::default()
            },
        ];
        let bars = SongBar::from_songs(songs);
        assert_eq!(bars.len(), 1);
    }

    #[test]
    fn bar_is_directory() {
        let song_bar = Bar::Song(Box::new(SongBar::new(SongData::default())));
        assert!(!song_bar.is_directory());

        let folder_bar = Bar::Folder(FolderBar::new(FolderData::default(), String::new()));
        assert!(folder_bar.is_directory());

        let hash_bar = Bar::Hash(HashBar::new(String::new(), Vec::new()));
        assert!(hash_bar.is_directory());
    }

    #[test]
    fn bar_lamp_no_score() {
        let bar = Bar::Song(Box::new(SongBar::new(SongData::default())));
        assert_eq!(bar.lamp(), 0);
    }

    #[test]
    fn bar_lamp_with_score() {
        let mut sb = SongBar::new(SongData::default());
        sb.score = Some(ScoreData {
            clear: 7,
            ..Default::default()
        });
        let bar = Bar::Song(Box::new(sb));
        assert_eq!(bar.lamp(), 7);
    }
}
