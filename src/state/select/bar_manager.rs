use super::bar::{Bar, FolderBar, SongBar};
use crate::database::song_db::SongDatabase;

/// Sort mode for bar lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Title,
    Artist,
    Level,
    Clear,
    AddDate,
}

/// Manages the bar list, cursor position, and folder navigation for the select screen.
#[derive(Default)]
pub struct BarManager {
    /// Current list of bars being displayed.
    bars: Vec<Bar>,
    /// Cursor position in the current bar list.
    cursor: usize,
    /// Stack of (bars, cursor) for folder navigation.
    stack: Vec<(Vec<Bar>, usize)>,
    /// Current sort mode.
    sort_mode: SortMode,
}

impl BarManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize with root folders from the song database.
    pub fn load_root(&mut self, song_db: &SongDatabase, bms_roots: &[String]) {
        let mut bars = Vec::new();
        for root in bms_roots {
            let crc = crc32_folder(root);
            let children = load_folder_children(song_db, &crc);
            bars.extend(children);
        }
        self.bars = bars;
        self.cursor = 0;
    }

    /// Set bars directly (for search results, etc.).
    pub fn set_bars(&mut self, bars: Vec<Bar>) {
        self.bars = bars;
        self.cursor = 0;
    }

    /// Get the current bar list.
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the currently selected bar, if any.
    pub fn selected(&self) -> Option<&Bar> {
        self.bars.get(self.cursor)
    }

    /// Move cursor up by `n` positions (wrapping).
    pub fn cursor_up(&mut self, n: usize) {
        if self.bars.is_empty() {
            return;
        }
        let len = self.bars.len();
        self.cursor = (self.cursor + len - (n % len)) % len;
    }

    /// Move cursor down by `n` positions (wrapping).
    pub fn cursor_down(&mut self, n: usize) {
        if self.bars.is_empty() {
            return;
        }
        let len = self.bars.len();
        self.cursor = (self.cursor + n) % len;
    }

    /// Enter a folder: push current state and load children.
    pub fn enter_folder(&mut self, song_db: &SongDatabase) -> bool {
        let bar = match self.bars.get(self.cursor) {
            Some(bar) if bar.is_directory() => bar.clone(),
            _ => return false,
        };

        let children = match &bar {
            Bar::Folder(fb) => load_folder_children(song_db, &fb.crc),
            Bar::Hash(hb) => {
                let hash_refs: Vec<&str> = hb.hashes.iter().map(|s| s.as_str()).collect();
                let songs = song_db.get_songs_by_hashes(&hash_refs).unwrap_or_default();
                SongBar::from_songs(songs)
            }
            _ => Vec::new(),
        };

        if children.is_empty() {
            return false;
        }

        let old_bars = std::mem::take(&mut self.bars);
        let old_cursor = self.cursor;
        self.stack.push((old_bars, old_cursor));
        self.bars = children;
        self.cursor = 0;
        self.sort_bars();
        true
    }

    /// Go back to the parent folder.
    pub fn leave_folder(&mut self) -> bool {
        if let Some((bars, cursor)) = self.stack.pop() {
            self.bars = bars;
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    /// Get the folder navigation depth.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Set the sort mode and re-sort.
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
        self.sort_bars();
    }

    /// Get the current sort mode.
    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    /// Sort the current bar list according to sort_mode.
    fn sort_bars(&mut self) {
        let mode = self.sort_mode;
        self.bars.sort_by(|a, b| match mode {
            SortMode::Title => a.title().to_lowercase().cmp(&b.title().to_lowercase()),
            SortMode::Artist => {
                let artist_a = match a {
                    Bar::Song(s) => s.song.artist.to_lowercase(),
                    _ => a.title().to_lowercase(),
                };
                let artist_b = match b {
                    Bar::Song(s) => s.song.artist.to_lowercase(),
                    _ => b.title().to_lowercase(),
                };
                artist_a.cmp(&artist_b)
            }
            SortMode::Level => {
                let level_a = match a {
                    Bar::Song(s) => s.song.level,
                    _ => 0,
                };
                let level_b = match b {
                    Bar::Song(s) => s.song.level,
                    _ => 0,
                };
                level_a.cmp(&level_b)
            }
            SortMode::Clear => {
                let clear_a = a.lamp();
                let clear_b = b.lamp();
                clear_a.cmp(&clear_b)
            }
            SortMode::AddDate => {
                let date_a = match a {
                    Bar::Song(s) => s.song.adddate,
                    _ => 0,
                };
                let date_b = match b {
                    Bar::Song(s) => s.song.adddate,
                    _ => 0,
                };
                date_b.cmp(&date_a) // newest first
            }
        });
        // Reset cursor to stay in bounds.
        if self.cursor >= self.bars.len() && !self.bars.is_empty() {
            self.cursor = 0;
        }
    }
}

/// Load children of a folder identified by its CRC.
fn load_folder_children(song_db: &SongDatabase, crc: &str) -> Vec<Bar> {
    // First try songs (leaf folder).
    let songs = song_db.get_songs_by("parent", crc).unwrap_or_default();
    if !songs.is_empty() {
        return SongBar::from_songs(songs);
    }

    // Otherwise load subfolders.
    let folders = song_db.get_folders_by("parent", crc).unwrap_or_default();
    folders
        .into_iter()
        .map(|f| {
            let folder_crc = crc32_folder(f.path.trim_end_matches(['/', '\\']));
            Bar::Folder(FolderBar::new(f, folder_crc))
        })
        .collect()
}

/// Compute a CRC32 string for folder identification.
/// Matches beatoraja's SongUtils.crc32() for folder paths.
pub fn crc32_folder(path: &str) -> String {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(path.as_bytes());
    format!("{:08x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::SongData;

    fn make_bars() -> Vec<Bar> {
        vec![
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Bravo".to_string(),
                sha256: "s1".to_string(),
                ..Default::default()
            }))),
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Alpha".to_string(),
                sha256: "s2".to_string(),
                ..Default::default()
            }))),
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Charlie".to_string(),
                sha256: "s3".to_string(),
                ..Default::default()
            }))),
        ]
    }

    #[test]
    fn cursor_movement_wraps() {
        let mut mgr = BarManager::new();
        mgr.set_bars(make_bars());
        assert_eq!(mgr.cursor(), 0);

        mgr.cursor_down(1);
        assert_eq!(mgr.cursor(), 1);

        mgr.cursor_up(2);
        assert_eq!(mgr.cursor(), 2); // wraps around: 1 - 2 + 3 = 2

        mgr.cursor_down(1);
        assert_eq!(mgr.cursor(), 0); // wraps: 2 + 1 = 3 % 3 = 0
    }

    #[test]
    fn sort_by_title() {
        let mut mgr = BarManager::new();
        mgr.set_bars(make_bars());
        mgr.set_sort_mode(SortMode::Title);

        assert_eq!(mgr.bars()[0].title(), "Alpha");
        assert_eq!(mgr.bars()[1].title(), "Bravo");
        assert_eq!(mgr.bars()[2].title(), "Charlie");
    }

    #[test]
    fn sort_by_level() {
        let mut mgr = BarManager::new();
        mgr.set_bars(vec![
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Easy".to_string(),
                level: 3,
                sha256: "s1".to_string(),
                ..Default::default()
            }))),
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Hard".to_string(),
                level: 12,
                sha256: "s2".to_string(),
                ..Default::default()
            }))),
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Normal".to_string(),
                level: 7,
                sha256: "s3".to_string(),
                ..Default::default()
            }))),
        ]);
        mgr.set_sort_mode(SortMode::Level);

        assert_eq!(mgr.bars()[0].title(), "Easy");
        assert_eq!(mgr.bars()[1].title(), "Normal");
        assert_eq!(mgr.bars()[2].title(), "Hard");
    }

    #[test]
    fn empty_bars_cursor_safe() {
        let mut mgr = BarManager::new();
        mgr.cursor_up(5);
        mgr.cursor_down(3);
        assert_eq!(mgr.cursor(), 0);
        assert!(mgr.selected().is_none());
    }

    #[test]
    fn folder_navigation() {
        let db = SongDatabase::open_in_memory().unwrap();
        // Insert a folder and a song.
        db.upsert_folder(&crate::database::models::FolderData {
            title: "SubFolder".to_string(),
            path: "bms/sub/".to_string(),
            parent: "root_crc".to_string(),
            ..Default::default()
        })
        .unwrap();

        let mut mgr = BarManager::new();
        mgr.set_bars(vec![Bar::Folder(FolderBar::new(
            crate::database::models::FolderData {
                title: "Root".to_string(),
                path: "bms/".to_string(),
                parent: "".to_string(),
                ..Default::default()
            },
            "root_crc".to_string(),
        ))]);

        assert_eq!(mgr.depth(), 0);
        let entered = mgr.enter_folder(&db);
        if entered {
            assert_eq!(mgr.depth(), 1);
            mgr.leave_folder();
            assert_eq!(mgr.depth(), 0);
        }
    }
}
