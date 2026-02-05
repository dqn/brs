use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

const SONG_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS song (
    sha256 TEXT PRIMARY KEY NOT NULL,
    md5 TEXT NOT NULL DEFAULT '',
    path TEXT NOT NULL DEFAULT '',
    folder TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL DEFAULT '',
    subtitle TEXT NOT NULL DEFAULT '',
    artist TEXT NOT NULL DEFAULT '',
    subartist TEXT NOT NULL DEFAULT '',
    genre TEXT NOT NULL DEFAULT '',
    mode INTEGER NOT NULL DEFAULT 7,
    level INTEGER NOT NULL DEFAULT 0,
    difficulty INTEGER NOT NULL DEFAULT 0,
    max_bpm INTEGER NOT NULL DEFAULT 0,
    min_bpm INTEGER NOT NULL DEFAULT 0,
    notes INTEGER NOT NULL DEFAULT 0,
    date INTEGER NOT NULL DEFAULT 0,
    add_date INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_song_path ON song(path);
CREATE INDEX IF NOT EXISTS idx_song_md5 ON song(md5);
CREATE INDEX IF NOT EXISTS idx_song_folder ON song(folder);
"#;

const SCORE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS score (
    sha256 TEXT NOT NULL,
    mode INTEGER NOT NULL DEFAULT 0,
    clear INTEGER NOT NULL DEFAULT 0,
    ex_score INTEGER NOT NULL DEFAULT 0,
    max_combo INTEGER NOT NULL DEFAULT 0,
    min_bp INTEGER NOT NULL DEFAULT 2147483647,
    pg INTEGER NOT NULL DEFAULT 0,
    gr INTEGER NOT NULL DEFAULT 0,
    gd INTEGER NOT NULL DEFAULT 0,
    bd INTEGER NOT NULL DEFAULT 0,
    pr INTEGER NOT NULL DEFAULT 0,
    ms INTEGER NOT NULL DEFAULT 0,
    notes INTEGER NOT NULL DEFAULT 0,
    play_count INTEGER NOT NULL DEFAULT 0,
    clear_count INTEGER NOT NULL DEFAULT 0,
    date INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (sha256, mode)
);

CREATE INDEX IF NOT EXISTS idx_score_sha256 ON score(sha256);
"#;

/// Database type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Song,
    Score,
}

/// Database wrapper for SQLite connection.
pub struct Database {
    conn: Connection,
    db_type: DatabaseType,
}

impl Database {
    /// Open a song database at the given path.
    pub fn open_song_db(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open song database: {}", path.display()))?;

        conn.execute_batch(SONG_SCHEMA)
            .context("Failed to initialize song schema")?;

        Ok(Self {
            conn,
            db_type: DatabaseType::Song,
        })
    }

    /// Open a score database at the given path.
    pub fn open_score_db(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open score database: {}", path.display()))?;

        conn.execute_batch(SCORE_SCHEMA)
            .context("Failed to initialize score schema")?;

        Ok(Self {
            conn,
            db_type: DatabaseType::Score,
        })
    }

    /// Open an in-memory song database (for testing).
    pub fn open_song_db_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;

        conn.execute_batch(SONG_SCHEMA)
            .context("Failed to initialize song schema")?;

        Ok(Self {
            conn,
            db_type: DatabaseType::Song,
        })
    }

    /// Open an in-memory score database (for testing).
    pub fn open_score_db_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;

        conn.execute_batch(SCORE_SCHEMA)
            .context("Failed to initialize score schema")?;

        Ok(Self {
            conn,
            db_type: DatabaseType::Score,
        })
    }

    /// Get the underlying connection reference.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get the database type.
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_open_song_db() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("song.db");

        let db = Database::open_song_db(&path).expect("Failed to open song database");
        assert_eq!(db.db_type(), DatabaseType::Song);
        assert!(path.exists());
    }

    #[test]
    fn test_open_score_db() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("score.db");

        let db = Database::open_score_db(&path).expect("Failed to open score database");
        assert_eq!(db.db_type(), DatabaseType::Score);
        assert!(path.exists());
    }

    #[test]
    fn test_in_memory_song_db() {
        let db = Database::open_song_db_in_memory().expect("Failed to open in-memory database");
        assert_eq!(db.db_type(), DatabaseType::Song);
    }

    #[test]
    fn test_in_memory_score_db() {
        let db = Database::open_score_db_in_memory().expect("Failed to open in-memory database");
        assert_eq!(db.db_type(), DatabaseType::Score);
    }
}
