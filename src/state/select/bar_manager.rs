use crate::database::{Database, ScoreDatabaseAccessor, SongDatabaseAccessor};
use crate::state::select::bar::{Bar, SongBar};
use anyhow::Result;

/// Manages the list of bars in the song selection screen.
pub struct BarManager {
    /// All bars in the current view.
    bars: Vec<Bar>,
    /// Current cursor position.
    cursor: usize,
}

impl BarManager {
    /// Create a new empty bar manager.
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            cursor: 0,
        }
    }

    /// Load all songs from the database.
    pub fn load_songs(&mut self, song_db: &Database, score_db: &Database) -> Result<()> {
        let song_accessor = SongDatabaseAccessor::new(song_db);
        let score_accessor = ScoreDatabaseAccessor::new(score_db);

        let songs = song_accessor.get_all_songs()?;

        self.bars = songs
            .into_iter()
            .map(|song| {
                let score = score_accessor.get_score(&song.sha256, 0).ok().flatten();
                Bar::Song(Box::new(SongBar::new(song, score)))
            })
            .collect();

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
}

impl Default for BarManager {
    fn default() -> Self {
        Self::new()
    }
}
