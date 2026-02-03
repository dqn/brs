use crate::database::{Database, ScoreDatabaseAccessor, SongDatabaseAccessor};
use crate::state::select::bar::{Bar, SongBar};
use anyhow::Result;
use std::collections::HashSet;

/// Manages the list of bars in the song selection screen.
pub struct BarManager {
    /// All bars in the current view.
    bars: Vec<Bar>,
    /// Full list of bars before filtering.
    all_bars: Vec<Bar>,
    /// Current cursor position.
    cursor: usize,
    /// Current sort mode.
    sort_mode: SortMode,
    /// Current filter mode.
    filter_mode: FilterMode,
    /// Favorite song hashes.
    favorites: HashSet<String>,
}

/// Sorting options for the select list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Title,
    Artist,
    Level,
    Bpm,
    Notes,
    Date,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Title => SortMode::Artist,
            SortMode::Artist => SortMode::Level,
            SortMode::Level => SortMode::Bpm,
            SortMode::Bpm => SortMode::Notes,
            SortMode::Notes => SortMode::Date,
            SortMode::Date => SortMode::Title,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SortMode::Title => SortMode::Date,
            SortMode::Artist => SortMode::Title,
            SortMode::Level => SortMode::Artist,
            SortMode::Bpm => SortMode::Level,
            SortMode::Notes => SortMode::Bpm,
            SortMode::Date => SortMode::Notes,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::Title => "TITLE",
            SortMode::Artist => "ARTIST",
            SortMode::Level => "LEVEL",
            SortMode::Bpm => "BPM",
            SortMode::Notes => "NOTES",
            SortMode::Date => "DATE",
        }
    }

    pub fn label_ja(self) -> &'static str {
        match self {
            SortMode::Title => "タイトル",
            SortMode::Artist => "アーティスト",
            SortMode::Level => "レベル",
            SortMode::Bpm => "BPM",
            SortMode::Notes => "ノーツ",
            SortMode::Date => "更新日",
        }
    }
}

/// Filtering options for the select list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    Favorites,
    Cleared,
    Unplayed,
}

impl FilterMode {
    pub fn next(self) -> Self {
        match self {
            FilterMode::All => FilterMode::Favorites,
            FilterMode::Favorites => FilterMode::Cleared,
            FilterMode::Cleared => FilterMode::Unplayed,
            FilterMode::Unplayed => FilterMode::All,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FilterMode::All => FilterMode::Unplayed,
            FilterMode::Favorites => FilterMode::All,
            FilterMode::Cleared => FilterMode::Favorites,
            FilterMode::Unplayed => FilterMode::Cleared,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FilterMode::All => "ALL",
            FilterMode::Favorites => "FAVORITES",
            FilterMode::Cleared => "CLEARED",
            FilterMode::Unplayed => "UNPLAYED",
        }
    }

    pub fn label_ja(self) -> &'static str {
        match self {
            FilterMode::All => "すべて",
            FilterMode::Favorites => "お気に入り",
            FilterMode::Cleared => "クリア済み",
            FilterMode::Unplayed => "未プレイ",
        }
    }
}

impl BarManager {
    /// Create a new empty bar manager.
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            all_bars: Vec::new(),
            cursor: 0,
            sort_mode: SortMode::Title,
            filter_mode: FilterMode::All,
            favorites: HashSet::new(),
        }
    }

    /// Load all songs from the database.
    pub fn load_songs(&mut self, song_db: &Database, score_db: &Database) -> Result<()> {
        let song_accessor = SongDatabaseAccessor::new(song_db);
        let score_accessor = ScoreDatabaseAccessor::new(score_db);

        let songs = song_accessor.get_all_songs()?;

        self.all_bars = songs
            .into_iter()
            .map(|song| {
                let score = score_accessor.get_score(&song.sha256, 0).ok().flatten();
                Bar::Song(Box::new(SongBar::new(song, score)))
            })
            .collect();

        self.apply_filters();

        // Reset cursor if out of bounds
        if self.cursor >= self.bars.len() && !self.bars.is_empty() {
            self.cursor = 0;
        }

        Ok(())
    }

    /// Get the number of bars.
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Check if there are no bars.
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, index: usize) {
        if self.bars.is_empty() {
            self.cursor = 0;
            return;
        }
        self.cursor = index.min(self.bars.len() - 1);
    }

    /// Move the cursor up.
    pub fn move_up(&mut self) {
        if self.bars.is_empty() {
            return;
        }

        if self.cursor == 0 {
            self.cursor = self.bars.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }

    /// Move the cursor down.
    pub fn move_down(&mut self) {
        if self.bars.is_empty() {
            return;
        }

        self.cursor = (self.cursor + 1) % self.bars.len();
    }

    /// Get the bar at the current cursor position.
    pub fn current_bar(&self) -> Option<&Bar> {
        self.bars.get(self.cursor)
    }

    /// Get the bar at a specific index.
    pub fn get(&self, index: usize) -> Option<&Bar> {
        self.bars.get(index)
    }

    /// Get all bars.
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub fn filter_mode(&self) -> FilterMode {
        self.filter_mode
    }

    pub fn next_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.apply_filters();
    }

    pub fn prev_sort(&mut self) {
        self.sort_mode = self.sort_mode.prev();
        self.apply_filters();
    }

    pub fn next_filter(&mut self) {
        self.filter_mode = self.filter_mode.next();
        self.apply_filters();
    }

    pub fn prev_filter(&mut self) {
        self.filter_mode = self.filter_mode.prev();
        self.apply_filters();
    }

    pub fn set_favorites(&mut self, favorites: HashSet<String>) {
        self.favorites = favorites;
        self.apply_filters();
    }

    pub fn is_favorite(&self, sha256: &str) -> bool {
        self.favorites.contains(sha256)
    }

    /// Get visible bars around the cursor for display.
    /// Returns (start_index, bars) where bars are the visible bars.
    pub fn visible_bars(&self, visible_count: usize) -> (usize, &[Bar]) {
        if self.bars.is_empty() {
            return (0, &[]);
        }

        let half = visible_count / 2;
        let total = self.bars.len();

        if total <= visible_count {
            return (0, &self.bars);
        }

        let start = if self.cursor < half {
            0
        } else if self.cursor >= total - half {
            total - visible_count
        } else {
            self.cursor - half
        };

        let end = (start + visible_count).min(total);

        (start, &self.bars[start..end])
    }

    fn apply_filters(&mut self) {
        self.bars = self
            .all_bars
            .iter()
            .filter(|bar| match (&self.filter_mode, bar) {
                (FilterMode::All, _) => true,
                (FilterMode::Favorites, Bar::Song(song_bar)) => {
                    self.favorites.contains(&song_bar.song.sha256)
                }
                (FilterMode::Cleared, Bar::Song(song_bar)) => song_bar
                    .score
                    .as_ref()
                    .map(|s| s.clear.is_cleared())
                    .unwrap_or(false),
                (FilterMode::Unplayed, Bar::Song(song_bar)) => song_bar.score.is_none(),
                (_, Bar::Folder(_)) => true,
            })
            .cloned()
            .collect();

        self.sort_bars();

        if self.cursor >= self.bars.len() && !self.bars.is_empty() {
            self.cursor = 0;
        }
    }

    fn sort_bars(&mut self) {
        let sort_mode = self.sort_mode;
        self.bars.sort_by(|a, b| match (a, b) {
            (Bar::Song(song_a), Bar::Song(song_b)) => match sort_mode {
                SortMode::Title => song_a.song.title.cmp(&song_b.song.title),
                SortMode::Artist => song_a.song.artist.cmp(&song_b.song.artist),
                SortMode::Level => song_a.song.level.cmp(&song_b.song.level),
                SortMode::Bpm => song_a.song.max_bpm.cmp(&song_b.song.max_bpm),
                SortMode::Notes => song_a.song.notes.cmp(&song_b.song.notes),
                SortMode::Date => song_a.song.date.cmp(&song_b.song.date),
            },
            (Bar::Folder(folder_a), Bar::Folder(folder_b)) => folder_a.name.cmp(&folder_b.name),
            (Bar::Folder(_), Bar::Song(_)) => std::cmp::Ordering::Less,
            (Bar::Song(_), Bar::Folder(_)) => std::cmp::Ordering::Greater,
        });

        if matches!(sort_mode, SortMode::Date) {
            self.bars.reverse();
        }
    }
}

impl Default for BarManager {
    fn default() -> Self {
        Self::new()
    }
}
