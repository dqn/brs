// BarManager — manages the song/folder bar list and cursor navigation.
//
// Provides a hierarchical browser with folder push/pop navigation.

use bms_database::{CourseData, SongData, SongDatabase};

/// Sort modes for the bar list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Default,
    Title,
    Artist,
    Level,
}

impl SortMode {
    /// Cycle to the next sort mode.
    pub fn next(self) -> Self {
        match self {
            Self::Default => Self::Title,
            Self::Title => Self::Artist,
            Self::Artist => Self::Level,
            Self::Level => Self::Default,
        }
    }
}

/// A single bar entry in the song list.
#[derive(Debug, Clone)]
pub enum Bar {
    Song(Box<SongData>),
    #[allow(dead_code)] // Used in tests and folder navigation
    Folder {
        name: String,
        path: String,
    },
    #[allow(dead_code)] // Used in tests and course selection
    Course(Box<CourseData>),
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

    /// Load course data and add them as bars.
    #[allow(dead_code)] // Used in tests and course mode
    pub fn add_courses(&mut self, courses: &[CourseData]) {
        for course in courses {
            self.bars.push(Bar::Course(Box::new(course.clone())));
        }
    }

    /// Sort bars by the given mode.
    ///
    /// Sort order for non-Song bars: Folders first, then Courses (by name).
    pub fn sort(&mut self, mode: SortMode) {
        match mode {
            SortMode::Default => {} // Keep original order
            SortMode::Title => {
                self.bars.sort_by(|a, b| {
                    let title_a = match a {
                        Bar::Song(s) => &s.title,
                        Bar::Folder { name, .. } => name,
                        Bar::Course(c) => &c.name,
                    };
                    let title_b = match b {
                        Bar::Song(s) => &s.title,
                        Bar::Folder { name, .. } => name,
                        Bar::Course(c) => &c.name,
                    };
                    title_a.to_lowercase().cmp(&title_b.to_lowercase())
                });
            }
            SortMode::Artist => {
                self.bars.sort_by(|a, b| {
                    let artist_a = match a {
                        Bar::Song(s) => s.artist.as_str(),
                        Bar::Folder { .. } | Bar::Course(_) => "",
                    };
                    let artist_b = match b {
                        Bar::Song(s) => s.artist.as_str(),
                        Bar::Folder { .. } | Bar::Course(_) => "",
                    };
                    artist_a.to_lowercase().cmp(&artist_b.to_lowercase())
                });
            }
            SortMode::Level => {
                self.bars.sort_by(|a, b| {
                    let level_a = match a {
                        Bar::Song(s) => s.level,
                        Bar::Folder { .. } | Bar::Course(_) => 0,
                    };
                    let level_b = match b {
                        Bar::Song(s) => s.level,
                        Bar::Folder { .. } | Bar::Course(_) => 0,
                    };
                    level_a.cmp(&level_b)
                });
            }
        }
        self.cursor = 0;
    }

    /// Filter bars to retain only songs matching the given mode ID.
    /// Folder and Course bars are always retained.
    pub fn filter_by_mode(&mut self, mode: Option<i32>) {
        if let Some(mode_id) = mode {
            self.bars.retain(|bar| match bar {
                Bar::Song(s) => s.mode == mode_id,
                Bar::Folder { .. } | Bar::Course(_) => true,
            });
            self.cursor = 0;
        }
    }

    /// Search for songs matching the query text, pushing the current bar list onto the folder stack.
    pub fn search(&mut self, song_db: &SongDatabase, query: &str) {
        let songs = song_db.get_song_datas_by_text(query).unwrap_or_default();
        // Save current state to folder stack
        let old_bars = std::mem::take(&mut self.bars);
        let old_cursor = self.cursor;
        self.folder_stack.push((old_bars, old_cursor));
        self.bars = songs.into_iter().map(|s| Bar::Song(Box::new(s))).collect();
        self.cursor = 0;
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

    #[test]
    fn sort_by_title_alphabetical() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "Charlie".to_string(),
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "Alpha".to_string(),
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "Bravo".to_string(),
                ..Default::default()
            })),
        ];
        bm.cursor = 2;
        bm.sort(SortMode::Title);
        assert_eq!(bm.cursor_pos(), 0);
        match &bm.bars[0] {
            Bar::Song(s) => assert_eq!(s.title, "Alpha"),
            _ => panic!("expected Song"),
        }
        match &bm.bars[1] {
            Bar::Song(s) => assert_eq!(s.title, "Bravo"),
            _ => panic!("expected Song"),
        }
        match &bm.bars[2] {
            Bar::Song(s) => assert_eq!(s.title, "Charlie"),
            _ => panic!("expected Song"),
        }
    }

    #[test]
    fn sort_by_level_ascending() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "Hard".to_string(),
                level: 12,
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "Easy".to_string(),
                level: 3,
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "Medium".to_string(),
                level: 7,
                ..Default::default()
            })),
        ];
        bm.sort(SortMode::Level);
        assert_eq!(bm.cursor_pos(), 0);
        match &bm.bars[0] {
            Bar::Song(s) => assert_eq!(s.level, 3),
            _ => panic!("expected Song"),
        }
        match &bm.bars[1] {
            Bar::Song(s) => assert_eq!(s.level, 7),
            _ => panic!("expected Song"),
        }
        match &bm.bars[2] {
            Bar::Song(s) => assert_eq!(s.level, 12),
            _ => panic!("expected Song"),
        }
    }

    #[test]
    fn filter_by_mode_retains_matching() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "7K Song".to_string(),
                mode: 7,
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "14K Song".to_string(),
                mode: 14,
                ..Default::default()
            })),
            Bar::Folder {
                name: "Folder".to_string(),
                path: "f".to_string(),
            },
            Bar::Song(Box::new(SongData {
                title: "Another 7K".to_string(),
                mode: 7,
                ..Default::default()
            })),
        ];
        bm.cursor = 2;
        bm.filter_by_mode(Some(7));
        assert_eq!(bm.cursor_pos(), 0);
        // Should retain: 7K Song, Folder, Another 7K (3 bars)
        assert_eq!(bm.bar_count(), 3);
        // 14K Song should be removed
        for bar in &bm.bars {
            if let Bar::Song(s) = bar {
                assert_eq!(s.mode, 7);
            }
        }
    }

    #[test]
    fn filter_by_mode_none_is_noop() {
        let mut bm = BarManager::new();
        bm.bars = vec![Bar::Song(Box::new(SongData {
            title: "Song".to_string(),
            mode: 7,
            ..Default::default()
        }))];
        bm.filter_by_mode(None);
        assert_eq!(bm.bar_count(), 1);
    }

    #[test]
    fn search_pushes_to_folder_stack() {
        let db = SongDatabase::open_in_memory().unwrap();
        let songs = vec![
            SongData {
                md5: "aaa".to_string(),
                sha256: "aaa_sha".to_string(),
                title: "Find Me".to_string(),
                path: "a.bms".to_string(),
                ..Default::default()
            },
            SongData {
                md5: "bbb".to_string(),
                sha256: "bbb_sha".to_string(),
                title: "Other Song".to_string(),
                path: "b.bms".to_string(),
                ..Default::default()
            },
        ];
        db.set_song_datas(&songs).unwrap();

        let mut bm = BarManager::new();
        bm.bars = vec![Bar::Song(Box::new(SongData {
            title: "Root".to_string(),
            ..Default::default()
        }))];
        bm.cursor = 0;

        bm.search(&db, "Find");
        assert!(bm.is_in_folder());
        assert_eq!(bm.bar_count(), 1);
        assert_eq!(bm.cursor_pos(), 0);
        match bm.current() {
            Some(Bar::Song(s)) => assert_eq!(s.title, "Find Me"),
            _ => panic!("expected Song with title 'Find Me'"),
        }

        // Leave search results should restore original
        bm.leave_folder();
        assert_eq!(bm.bar_count(), 1);
        assert!(!bm.is_in_folder());
    }

    #[test]
    fn sort_mode_cycles() {
        assert_eq!(SortMode::Default.next(), SortMode::Title);
        assert_eq!(SortMode::Title.next(), SortMode::Artist);
        assert_eq!(SortMode::Artist.next(), SortMode::Level);
        assert_eq!(SortMode::Level.next(), SortMode::Default);
    }

    fn sample_course(name: &str) -> CourseData {
        use bms_database::CourseSongData;
        CourseData {
            name: name.to_string(),
            hash: vec![CourseSongData {
                sha256: "abc".to_string(),
                md5: String::new(),
                title: "Stage 1".to_string(),
            }],
            constraint: Vec::new(),
            trophy: Vec::new(),
            release: true,
        }
    }

    #[test]
    fn add_courses_appends_bars() {
        let mut bm = BarManager::new();
        bm.bars = vec![Bar::Song(Box::new(SongData {
            title: "Song".to_string(),
            ..Default::default()
        }))];

        let courses = vec![sample_course("Course A"), sample_course("Course B")];
        bm.add_courses(&courses);
        assert_eq!(bm.bar_count(), 3);
        match &bm.bars[1] {
            Bar::Course(c) => assert_eq!(c.name, "Course A"),
            _ => panic!("expected Course bar"),
        }
        match &bm.bars[2] {
            Bar::Course(c) => assert_eq!(c.name, "Course B"),
            _ => panic!("expected Course bar"),
        }
    }

    #[test]
    fn sort_by_title_includes_courses() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "Zebra".to_string(),
                ..Default::default()
            })),
            Bar::Course(Box::new(sample_course("Alpha Course"))),
            Bar::Folder {
                name: "Middle Folder".to_string(),
                path: "f".to_string(),
            },
        ];
        bm.sort(SortMode::Title);

        // Expected order: "Alpha Course", "Middle Folder", "Zebra"
        match &bm.bars[0] {
            Bar::Course(c) => assert_eq!(c.name, "Alpha Course"),
            _ => panic!("expected Course bar at index 0"),
        }
        match &bm.bars[1] {
            Bar::Folder { name, .. } => assert_eq!(name, "Middle Folder"),
            _ => panic!("expected Folder bar at index 1"),
        }
        match &bm.bars[2] {
            Bar::Song(s) => assert_eq!(s.title, "Zebra"),
            _ => panic!("expected Song bar at index 2"),
        }
    }

    #[test]
    fn filter_by_mode_retains_courses() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "7K Song".to_string(),
                mode: 7,
                ..Default::default()
            })),
            Bar::Song(Box::new(SongData {
                title: "14K Song".to_string(),
                mode: 14,
                ..Default::default()
            })),
            Bar::Course(Box::new(sample_course("My Course"))),
        ];
        bm.filter_by_mode(Some(7));
        // Should retain: 7K Song + Course (2 bars), 14K removed
        assert_eq!(bm.bar_count(), 2);
        match &bm.bars[0] {
            Bar::Song(s) => assert_eq!(s.mode, 7),
            _ => panic!("expected 7K Song"),
        }
        match &bm.bars[1] {
            Bar::Course(c) => assert_eq!(c.name, "My Course"),
            _ => panic!("expected Course bar"),
        }
    }

    #[test]
    fn sort_by_artist_courses_sort_as_empty() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "Song".to_string(),
                artist: "Beta Artist".to_string(),
                ..Default::default()
            })),
            Bar::Course(Box::new(sample_course("Course"))),
        ];
        bm.sort(SortMode::Artist);
        // Course has empty artist, so it sorts before "Beta Artist"
        match &bm.bars[0] {
            Bar::Course(c) => assert_eq!(c.name, "Course"),
            _ => panic!("expected Course bar first"),
        }
    }

    #[test]
    fn sort_by_level_courses_sort_as_zero() {
        let mut bm = BarManager::new();
        bm.bars = vec![
            Bar::Song(Box::new(SongData {
                title: "Hard".to_string(),
                level: 12,
                ..Default::default()
            })),
            Bar::Course(Box::new(sample_course("Course"))),
        ];
        bm.sort(SortMode::Level);
        // Course has level 0, so it sorts before level 12
        match &bm.bars[0] {
            Bar::Course(c) => assert_eq!(c.name, "Course"),
            _ => panic!("expected Course bar first"),
        }
    }
}
