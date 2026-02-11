// DatabaseManager — holds song/score/score-log database connections.
//
// Wraps the three database types from bms-database into a single manager.

use std::path::Path;

use anyhow::Result;
use bms_database::{ScoreDataLogDatabase, ScoreDatabase, SongDatabase};

/// Unified database manager holding all database connections.
pub struct DatabaseManager {
    pub song_db: SongDatabase,
    pub score_db: ScoreDatabase,
    pub score_log_db: ScoreDataLogDatabase,
}

impl DatabaseManager {
    /// Open all databases at the given directory path.
    ///
    /// Creates `song.db`, `score.db`, and `scorelog.db` in the directory.
    pub fn open(db_dir: impl AsRef<Path>) -> Result<Self> {
        let db_dir = db_dir.as_ref();
        std::fs::create_dir_all(db_dir)?;
        let song_db = SongDatabase::open(db_dir.join("song.db"))?;
        let score_db = ScoreDatabase::open(db_dir.join("score.db"))?;
        let score_log_db = ScoreDataLogDatabase::open(db_dir.join("scorelog.db"))?;
        Ok(Self {
            song_db,
            score_db,
            score_log_db,
        })
    }

    /// Open all databases in memory (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let song_db = SongDatabase::open_in_memory()?;
        let score_db = ScoreDatabase::open_in_memory()?;
        let score_log_db = ScoreDataLogDatabase::open_in_memory()?;
        Ok(Self {
            song_db,
            score_db,
            score_log_db,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn open_creates_databases() {
        let tmp = TempDir::new().unwrap();
        let mgr = DatabaseManager::open(tmp.path()).unwrap();
        // Verify we can query without error
        let songs = mgr.song_db.get_song_datas("md5", "nonexistent").unwrap();
        assert!(songs.is_empty());
    }

    #[test]
    fn open_in_memory_works() {
        let mgr = DatabaseManager::open_in_memory().unwrap();
        let songs = mgr.song_db.get_song_datas("md5", "nonexistent").unwrap();
        assert!(songs.is_empty());
    }

    #[test]
    fn open_creates_dir_if_missing() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("sub").join("dir");
        let mgr = DatabaseManager::open(&nested).unwrap();
        let songs = mgr.song_db.get_song_datas("md5", "nonexistent").unwrap();
        assert!(songs.is_empty());
        assert!(nested.exists());
    }
}
