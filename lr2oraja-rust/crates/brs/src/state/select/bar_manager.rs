// BarManager — manages the song/folder bar list and cursor navigation.
//
// Provides a hierarchical browser with folder push/pop navigation.

use bms_database::{SongData, SongDatabase};

/// A single bar entry in the song list.
#[derive(Debug, Clone)]
pub enum Bar {
    Song(Box<SongData>),
    #[allow(dead_code)] // Reserved for folder navigation system
    Folder {
        name: String,
        path: String,
    },
}

/// Manages the bar list, cursor position, and folder navigation stack.
pub struct BarManager {
    bars: Vec<Bar>,
    cursor: usize,
    folder_stack: Vec<(Vec<Bar>, usize)>,
}

impl BarManager {
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            cursor: 0,
            folder_stack: Vec::new(),
        }
    }

    /// Load all songs from the database as a flat list.
    pub fn load_root(&mut self, song_db: &SongDatabase) {
        let songs = song_db.get_all_song_datas().unwrap_or_default();
        self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
        self.cursor = 0;
        self.folder_stack.clear();
    }

    /// Move cursor by delta with wrap-around.
    pub fn move_cursor(&mut self, delta: i32) {
        if self.bars.is_empty() {
            return;
        }
        let len = self.bars.len() as i32;
        let new_pos = ((self.cursor as i32 + delta) % len + len) % len;
        self.cursor = new_pos as usize;
    }

    /// Enter the currently selected folder.
    /// Pushes current bars and cursor onto the stack, loads folder contents.
    pub fn enter_folder(&mut self, song_db: &SongDatabase) {
        let folder_path = match self.bars.get(self.cursor) {
            Some(Bar::Folder { path, .. }) => path.clone(),
            _ => return,
        };

        let old_bars = std::mem::take(&mut self.bars);
        let old_cursor = self.cursor;
        self.folder_stack.push((old_bars, old_cursor));

        let songs = song_db
            .get_song_datas("folder", &folder_path)
            .unwrap_or_default();
        self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
        self.cursor = 0;
    }

    /// Leave the current folder, restoring the parent bar list and cursor.
    pub fn leave_folder(&mut self) {
        if let Some((bars, cursor)) = self.folder_stack.pop() {
            self.bars = bars;
            self.cursor = cursor;
        }
    }

    /// Returns the bar at the current cursor position.
    pub fn current(&self) -> Option<&Bar> {
        self.bars.get(self.cursor)
    }

    /// Returns the total number of bars.
    pub fn bar_count(&self) -> usize {
        self.bars.len()
    }

    /// Returns the current cursor position.
    #[allow(dead_code)] // Used in tests
    pub fn cursor_pos(&self) -> usize {
        self.cursor
    }

    /// Returns true if currently inside a folder (not at root).
    pub fn is_in_folder(&self) -> bool {
        !self.folder_stack.is_empty()
    }
}

impl Default for BarManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let bm = BarManager::new();
        assert_eq!(bm.bar_count(), 0);
        assert_eq!(bm.cursor_pos(), 0);
        assert!(bm.current().is_none());
    }

    #[test]
    fn load_root_populates_bars() {
        let db = SongDatabase::open_in_memory().unwrap();
        let songs = vec![
            SongData {
                md5: "aaa".to_string(),
                sha256: "aaa_sha".to_string(),
                title: "Song A".to_string(),
                path: "a.bms".to_string(),
                ..Default::default()
            },
            SongData {
                md5: "bbb".to_string(),
                sha256: "bbb_sha".to_string(),
                title: "Song B".to_string(),
                path: "b.bms".to_string(),
                ..Default::default()
            },
        ];
        db.set_song_datas(&songs).unwrap();

        let mut bm = BarManager::new();
        bm.load_root(&db);
        assert_eq!(bm.bar_count(), 2);
        assert_eq!(bm.cursor_pos(), 0);
    }

    #[test]
    fn load_root_empty_db() {
        let db = SongDatabase::open_in_memory().unwrap();
        let mut bm = BarManager::new();
        bm.load_root(&db);
        assert_eq!(bm.bar_count(), 0);
    }

    #[test]
    fn move_cursor_wraps_forward() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "A".to_string(),
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "B".to_string(),
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "C".to_string(),
                ..Default::default()
            })),
        ];
        bm.cursor = 2;
        bm.move_cursor(1);
        assert_eq!(bm.cursor_pos(), 0);
    }

    #[test]
    fn move_cursor_wraps_backward() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "A".to_string(),
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "B".to_string(),
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "C".to_string(),
                ..Default::default()
            })),
        ];
        bm.cursor = 0;
        bm.move_cursor(-1);
        assert_eq!(bm.cursor_pos(), 2);
    }

    #[test]
    fn move_cursor_empty_is_noop() {
        let mut bm = BarManager::new();
        bm.move_cursor(1);
        assert_eq!(bm.cursor_pos(), 0);
    }

    #[test]
    fn enter_and_leave_folder() {
        let db = SongDatabase::open_in_memory().unwrap();
        // Insert songs in a folder
        let songs = vec![SongData {
            md5: "ccc".to_string(),
            sha256: "ccc_sha".to_string(),
            title: "Folder Song".to_string(),
            path: "folder/c.bms".to_string(),
            folder: "my_folder".to_string(),
            ..Default::default()
        }];
        db.set_song_datas(&songs).unwrap();

        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Folder {
                name: "My Folder".to_string(),
                path: "my_folder".to_string(),
            },
            Bar::Song(Box::new(SongData {
                title: "Root Song".to_string(),
                ..Default::default()
            })),
        ];
        bm.cursor = 0;

        // Enter folder
        bm.enter_folder(&db);
        assert_eq!(bm.bar_count(), 1);
        assert_eq!(bm.cursor_pos(), 0);
        assert!(bm.is_in_folder());
        match bm.current() {
            Some(Bar::Song(sd)) => assert_eq!(sd.title, "Folder Song"),
            _ => panic!("expected Song bar"),
        }

        // Leave folder
        bm.leave_folder();
        assert_eq!(bm.bar_count(), 2);
        assert_eq!(bm.cursor_pos(), 0);
        assert!(!bm.is_in_folder());
    }

    #[test]
    fn enter_folder_on_song_is_noop() {
        let db = SongDatabase::open_in_memory().unwrap();
        let mut bm = BarManager::new();
        bm.bars = vec![Bar::Song(Box::new(SongData {
            title: "Song".to_string(),
            ..Default::default()
        }))];
        bm.cursor = 0;
        bm.enter_folder(&db);
        // Should not push to stack
        assert!(!bm.is_in_folder());
        assert_eq!(bm.bar_count(), 1);
    }

    #[test]
    fn leave_folder_at_root_is_noop() {
        let mut bm = BarManager::new();
        bm.bars = vec![Bar::Song(Box::new(SongData {
            title: "Song".to_string(),
            ..Default::default()
        }))];
        bm.leave_folder();
        assert_eq!(bm.bar_count(), 1);
    }
}
